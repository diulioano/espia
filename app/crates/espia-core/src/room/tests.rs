//! Testes do redutor da sala (T021). Cada caso confere o comportamento do SITE
//! (`src/lib/stores/room.svelte.ts`, `mesh.ts`, `peer.ts`), não o que a implementação resolveu fazer.

use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::device::{GoLive, Surface};
use crate::devices::DeviceInfo;
use crate::media_policy::{BroadcastOrder, BroadcastStatus, TransmissionKey, DIRECT_ONLY};
use crate::protocol::{
    AudioKind, ClientMessage, IceServer, LiveInfo, LiveSource, Peer, RoomInfo, ServerMessage, Tone, Urls,
};
use crate::quality::{SendParams, ViewMode};
use crate::view_model::{
    CameraPermission, CaptureSource, LinkState, RoomAction, RoomStatus, ScreenSurface, StreamId, StreamWatchState,
    TileStatus, ToastKind,
};

const NOW: u64 = 1_700_000_000_000;
const ME: &str = "eu";

// ---------------------------------------------------------------------------
// Montagem
// ---------------------------------------------------------------------------

fn live_info(started_at: u64) -> LiveInfo {
    LiveInfo {
        resolution: 1080,
        fps: 60,
        audio: AudioKind::Pc,
        source: LiveSource::Screen,
        started_at,
        label: Some("Hollow Knight".into()),
    }
}

fn pessoa(id: &str, nick: &str, live: Option<u64>) -> Peer {
    Peer { id: id.into(), nickname: nick.into(), tone: Tone::Accent, live: live.map(live_info) }
}

fn init() -> RoomInit {
    RoomInit {
        code: "k7f-q2x".into(),
        go_live_prefs: GoLive::default(),
        owner_key: None,
        volumes: BTreeMap::new(),
        just_created: false,
    }
}

fn welcome_em(you: Peer, peers: Vec<Peer>, room: Option<RoomInfo>, order: Option<BroadcastOrder>, ice: Vec<IceServer>) -> Event {
    Event::Server(ServerMessage::Welcome {
        media_protocol: 2,
        media_policy: DIRECT_ONLY,
        broadcast_order: order,
        you,
        peers,
        ice_servers: ice,
        ice_expires_at: 0,
        resume_token: "x".into(),
        capacity: 10,
        room,
    })
}

fn welcome(you: Peer, peers: Vec<Peer>) -> Event {
    welcome_em(you, peers, None, None, vec![])
}

/// Aplica um evento e devolve os efeitos.
fn go(s: &mut RoomState, e: Event) -> Vec<Effect> {
    go_at(s, e, NOW)
}

fn go_at(s: &mut RoomState, e: Event, now: u64) -> Vec<Effect> {
    let (next, fx) = reduce(s.clone(), e, now);
    *s = next;
    fx
}

fn acao(s: &mut RoomState, a: RoomAction) -> Vec<Effect> {
    go(s, Event::Action(a))
}

/// Sala já aberta: eu sou "eu" e os outros chegam no welcome.
fn sala(outros: &[(&str, &str, Option<u64>)]) -> RoomState {
    let mut s = RoomState::new(init());
    let peers = outros.iter().map(|(id, n, l)| pessoa(id, n, *l)).collect();
    go(&mut s, welcome(pessoa(ME, "Ana", None), peers));
    s
}

/// A ligação com essa pessoa fica de pé e o canal `ctl` abre.
fn conectar(s: &mut RoomState, id: &str) -> Vec<Effect> {
    go(s, Event::Peer(PeerEvent::State { id: id.into(), conn: ConnState::Connected }));
    go(s, Event::Peer(PeerEvent::ControlOpen { id: id.into() }))
}

fn assistir(s: &mut RoomState, id: &str) -> Vec<Effect> {
    acao(s, RoomAction::Watch { id: id.into(), on: true })
}

fn janela() -> CaptureSource {
    CaptureSource {
        id: "w:1".into(),
        kind: ScreenSurface::Window,
        title: "Hollow Knight".into(),
        app: "hollow_knight.exe".into(),
        icon: None,
        thumb: None,
        playing: false,
        primary: false,
    }
}

fn tela_inteira(id: &str, title: &str) -> CaptureSource {
    CaptureSource { id: id.into(), kind: ScreenSurface::Monitor, title: title.into(), app: String::new(), ..janela() }
}

fn stream(id: &str) -> StreamId {
    StreamId(id.into())
}

fn minha_chave() -> TransmissionKey {
    TransmissionKey { peer_id: ME.into(), started_at: 500 }
}

/// Começa a transmitir a janela e recebe o `live-ack` (a minha chave passa a valer).
fn transmitir(s: &mut RoomState, prefs: GoLive) -> Vec<Effect> {
    let mut tudo = Vec::new();
    tudo.extend(acao(s, RoomAction::OpenGoLive));
    tudo.extend(go(s, Event::Capture(CaptureEvent::Sources(vec![janela()]))));
    tudo.extend(acao(s, RoomAction::SelectSource { id: "w:1".into() }));
    tudo.extend(acao(s, RoomAction::StartGoLive { prefs }));
    tudo.extend(go(
        s,
        Event::Capture(CaptureEvent::Started {
            request_id: "req-1".into(),
            stream: stream("tela"),
            audio_kind: AudioKind::Pc,
            audio_warn: None,
        }),
    ));
    tudo.extend(go(
        s,
        Event::Server(ServerMessage::LiveAck { request_id: "req-1".into(), broadcast_seq: 0, live: live_info(500) }),
    ));
    tudo
}

fn recebe_view(s: &mut RoomState, id: &str, revision: u64, mode: ViewMode) -> Vec<Effect> {
    go(
        s,
        Event::Peer(PeerEvent::View {
            id: id.into(),
            message: ViewMessage { transmission: minha_chave(), revision, mode },
        }),
    )
}

fn tem(fx: &[Effect], f: impl Fn(&Effect) -> bool) -> bool {
    fx.iter().any(f)
}

fn views_enviadas(fx: &[Effect]) -> Vec<(String, ViewMode, u64)> {
    fx.iter()
        .filter_map(|e| match e {
            Effect::SendView { to, message } => Some((to.clone(), message.mode, message.revision)),
            _ => None,
        })
        .collect()
}

fn parametros(fx: &[Effect]) -> Vec<(String, SendParams, bool)> {
    fx.iter()
        .filter_map(|e| match e {
            Effect::SetSendParams { to, params, sound } => Some((to.clone(), *params, *sound)),
            _ => None,
        })
        .collect()
}

/// Mensagens `live` mandadas ao servidor: (request_id, seq, resolução, fps).
fn anuncios(fx: &[Effect]) -> Vec<(String, u64, u16, u8)> {
    fx.iter()
        .filter_map(|e| match e {
            Effect::Send(ClientMessage::Live { request_id, broadcast_seq, resolution, fps, .. }) => {
                Some((request_id.clone(), *broadcast_seq, *resolution, *fps))
            }
            _ => None,
        })
        .collect()
}

fn qualidade_da_captura(fx: &[Effect]) -> Vec<(u16, u16)> {
    fx.iter()
        .filter_map(|e| match e {
            Effect::SetCaptureQuality { resolution, fps } => Some((*resolution, *fps)),
            _ => None,
        })
        .collect()
}

fn agendou(fx: &[Effect], timer: TimerId, after_ms: u64) -> bool {
    fx.contains(&Effect::Schedule { timer, after_ms })
}

fn cancelou(fx: &[Effect], timer: TimerId) -> bool {
    fx.contains(&Effect::Cancel { timer })
}

fn texto_dos_avisos(s: &RoomState) -> Vec<String> {
    s.toasts.iter().map(|t| t.text.clone()).collect()
}

fn dono() -> RoomState {
    let mut s = RoomState::new(RoomInit { owner_key: Some("k".repeat(64)), ..init() });
    go(
        &mut s,
        welcome_em(
            pessoa(ME, "Ana", None),
            vec![pessoa("bia", "Bia", None)],
            Some(RoomInfo::Fixed { name: "CS da galera".into() }),
            None,
            vec![],
        ),
    );
    s
}

// ---------------------------------------------------------------------------
// Constantes e textos do site
// ---------------------------------------------------------------------------

#[test]
fn numeros_e_textos_do_site() {
    assert_eq!(DEFAULT_VOLUME, 0.68);
    assert_eq!(TOAST_MS, 4_000);
    assert_eq!(FIRST_FRAME_TIMEOUT_MS, 10_000);
    assert_eq!(RECONNECT_FAIL_MS, 60_000);
    assert_eq!(COPIED_MS, 2_000);
    assert_eq!(OFFICIAL_ORIGIN, "https://espia.diuliano.com");
    assert_eq!(BLACK_WARN, "A tela tá saindo preta — tenta deixar o jogo em modo janela ou sem bordas.");
    assert_eq!(CONFIRM_LEAVE, "Você está transmitindo. Parar e sair da sala?");
    assert_eq!(AudioWarn::Unsupported.text(), "Seu Windows não deixa separar o som da janela — foi só a imagem.");
    assert_eq!(AudioWarn::ManyDiscords.text(), "Mais de um Discord aberto — tela inteira vai sem som.");
    assert_eq!(owner_error_text(OwnerFailure::Forbidden), "Só quem criou a sala pode fazer isso.");
    assert_eq!(owner_error_text(OwnerFailure::Gone), "Essa sala não existe mais.");
    assert_eq!(owner_error_text(OwnerFailure::Name), "Esse nome não dá. Use letras, números e espaços.");
    assert_eq!(owner_error_text(OwnerFailure::RateLimited), "Muitas mudanças daqui agora. Tenta de novo em um minuto.");
    assert_eq!(
        owner_error_text(OwnerFailure::Network),
        "Não deu pra falar com o servidor. Confere a internet e tenta de novo."
    );
}

// ---------------------------------------------------------------------------
// Welcome, entrada e saída de pessoas
// ---------------------------------------------------------------------------

#[test]
fn welcome_monta_o_estado_e_abre_ligacao_com_cada_pessoa() {
    let ice = vec![
        IceServer { urls: Urls::One("stun:stun.exemplo.com:3478".into()), username: None, credential: None },
        IceServer {
            urls: Urls::One("turn:turn.exemplo.com:3478".into()),
            username: Some("u".into()),
            credential: Some("c".into()),
        },
    ];
    let mut s = RoomState::new(init());
    assert!(!s.connected);
    assert_eq!(view(&s, NOW).status, RoomStatus::Connecting);
    let fx = go(
        &mut s,
        welcome_em(
            pessoa(ME, "Ana", None),
            vec![pessoa("bia", "Bia", None), pessoa("lu", "Lu", None)],
            None,
            None,
            ice,
        ),
    );
    assert!(s.connected);
    assert_eq!(s.me.as_ref().unwrap().id, ME);
    assert_eq!(s.peers.len(), 2);
    assert_eq!(s.capacity, 10);
    assert_eq!(s.room_info, RoomInfo::Quick);
    assert_eq!(view(&s, NOW).status, RoomStatus::Ready);
    // O app só usa STUN; nada de TURN, e nunca `turn-refresh`.
    let stun = IceServer { urls: Urls::Many(vec!["stun:stun.exemplo.com:3478".into()]), username: None, credential: None };
    assert_eq!(
        fx,
        vec![
            Effect::SetIceServers(vec![stun]),
            // "eu" < "lu" (educado); "eu" > "bia" (não educado). Quem chega inicia a ligação.
            Effect::OpenPeer { id: "bia".into(), initiator: true, polite: false },
            Effect::OpenPeer { id: "lu".into(), initiator: true, polite: true },
        ]
    );
    assert!(!tem(&fx, |e| matches!(e, Effect::Send(ClientMessage::TurnRefresh))));
    assert_eq!(s.links.len(), 2);
    // Ninguém escolhido para assistir: nada além de `off` para cada pessoa.
    assert!(s.watching.is_empty());
}

#[test]
fn welcome_de_sala_fixa_lembra_a_sala_neste_aparelho() {
    let mut s = RoomState::new(init());
    let fx = go_at(
        &mut s,
        welcome_em(pessoa(ME, "Ana", None), vec![], Some(RoomInfo::Fixed { name: "CS da galera".into() }), None, vec![]),
        42,
    );
    assert!(fx.contains(&Effect::Save(DeviceChange::RememberRoom { code: "k7f-q2x".into(), name: "CS da galera".into(), at_ms: 42 })));
    assert_eq!(view(&s, NOW).room, crate::view_model::RoomKind::Fixed { name: "CS da galera".into(), own: false });
    // Sala rápida não fica em Suas salas.
    let mut r = RoomState::new(init());
    let fx = go(&mut r, welcome(pessoa(ME, "Ana", None), vec![]));
    assert!(!tem(&fx, |e| matches!(e, Effect::Save(DeviceChange::RememberRoom { .. }))));
}

#[test]
fn sala_recem_criada_copia_o_convite_oficial_no_primeiro_welcome() {
    let mut s = RoomState::new(RoomInit { just_created: true, ..init() });
    let fx = go(&mut s, welcome(pessoa(ME, "Ana", None), vec![]));
    assert!(fx.contains(&Effect::CopyInvite("https://espia.diuliano.com/s/k7f-q2x".into())));
    assert_eq!(texto_dos_avisos(&s), vec!["Sala criada — fixe o link no grupo"]);
    assert!(s.copied);
    // Só no primeiro welcome.
    let fx = go(&mut s, welcome(pessoa(ME, "Ana", None), vec![]));
    assert!(!tem(&fx, |e| matches!(e, Effect::CopyInvite(_))));
}

#[test]
fn peer_joined_avisa_e_nao_abre_ligacao_quem_chega_inicia() {
    let mut s = sala(&[]);
    let fx = go(&mut s, Event::Server(ServerMessage::PeerJoined { peer: pessoa("bia", "Bia", None) }));
    assert_eq!(s.peers.len(), 1);
    assert_eq!(texto_dos_avisos(&s), vec!["Bia entrou na sala"]);
    assert_eq!(s.toasts[0].kind, ToastKind::Join);
    assert!(agendou(&fx, TimerId::Toast("t1".into()), 4_000));
    assert!(!tem(&fx, |e| matches!(e, Effect::OpenPeer { .. })));
    assert!(s.links.is_empty());
    // Já na lista (veio no welcome): só atualiza, sem aviso novo.
    let fx = go(&mut s, Event::Server(ServerMessage::PeerJoined { peer: pessoa("bia", "Bia", Some(7)) }));
    assert_eq!(s.peers.len(), 1);
    assert_eq!(s.toasts.len(), 1);
    assert!(s.peers[0].live.is_some());
    assert!(fx.is_empty());
}

#[test]
fn peer_left_fecha_a_ligacao_limpa_o_foco_e_avisa() {
    let mut s = sala(&[("bia", "Bia", Some(100)), ("lu", "Lu", Some(200))]);
    conectar(&mut s, "bia");
    assistir(&mut s, "bia");
    acao(&mut s, RoomAction::Focus { id: Some("bia".into()) });
    let fx = go(&mut s, Event::Server(ServerMessage::PeerLeft { id: "bia".into() }));
    assert_eq!(s.peers.len(), 1);
    assert_eq!(s.focused_id, None);
    assert!(s.watching.is_empty(), "a escolha de quem saiu não sobrevive");
    assert!(!s.links.contains_key("bia"));
    assert!(!s.media.contains_key("bia"));
    assert!(fx.contains(&Effect::ClosePeer { id: "bia".into() }));
    assert!(texto_dos_avisos(&s).contains(&"Bia saiu da sala".to_string()));
    let ultimo = s.toasts.last().unwrap();
    assert_eq!(ultimo.kind, ToastKind::Leave);
}

#[test]
fn peer_live_comeca_a_transmitir_com_aviso_e_assistir() {
    let mut s = sala(&[("bia", "Bia", None)]);
    conectar(&mut s, "bia");
    let fx = go(&mut s, Event::Server(ServerMessage::PeerLive { id: "bia".into(), live: Some(live_info(100)) }));
    let t = &s.toasts[0];
    assert_eq!((t.text.as_str(), t.kind, t.action.as_deref()), ("Bia começou a transmitir", ToastKind::Live, Some("Assistir")));
    assert!(agendou(&fx, TimerId::Toast("t1".into()), 4_000));
    assert_eq!(s.toast_peer.get("t1").map(String::as_str), Some("bia"));
    // Ainda não escolheu: nada de vídeo, e o tile só oferece.
    let v = view(&s, NOW);
    assert_eq!(v.streams[0].watch_state, Some(StreamWatchState::Available));
    assert_eq!(v.streams[0].watching, Some(false));
    assert!(s.watching.is_empty());
    assert_eq!(s.links["bia"].my_view, Some(ViewMode::Off));
    assert!(!tem(&fx, |e| matches!(e, Effect::Schedule { timer: TimerId::FirstFrame(_), .. })));
    // "Assistir" no aviso: escolhe, foca a tela e fecha o aviso.
    let fx = acao(&mut s, RoomAction::ToastAction { id: "t1".into() });
    assert!(s.watching.contains_key("bia"));
    assert_eq!(s.focused_id.as_deref(), Some("bia"));
    assert!(s.toasts.is_empty());
    assert!(!s.toast_peer.contains_key("t1"));
    assert_eq!(views_enviadas(&fx), vec![("bia".into(), ViewMode::Full, 2)]);
    assert!(agendou(&fx, TimerId::FirstFrame("bia".into()), 10_000));
}

#[test]
fn peer_live_nulo_limpa_foco_e_espera_do_primeiro_quadro() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    conectar(&mut s, "bia");
    assistir(&mut s, "bia");
    acao(&mut s, RoomAction::Focus { id: Some("bia".into()) });
    let fx = go(&mut s, Event::Server(ServerMessage::PeerLive { id: "bia".into(), live: None }));
    assert_eq!(s.focused_id, None);
    assert!(s.watching.is_empty());
    assert!(cancelou(&fx, TimerId::FirstFrame("bia".into())));
    assert!(s.frame_timers.is_empty());
    assert!(view(&s, NOW).streams.is_empty());
    // Acabar a transmissão não gera aviso de "começou".
    assert!(s.toasts.is_empty());
}

// ---------------------------------------------------------------------------
// A regra do Assistir (FR-021)
// ---------------------------------------------------------------------------

#[test]
fn sem_escolher_assistir_nada_chega_nem_com_destaque_ou_janela_flutuante() {
    let mut s = sala(&[("bia", "Bia", Some(100)), ("lu", "Lu", Some(200))]);
    let fx = conectar(&mut s, "bia");
    // Só diz `off` e nada mais.
    assert_eq!(views_enviadas(&fx), vec![("bia".into(), ViewMode::Off, 1)]);
    // O vídeo chega do motor mesmo assim: não é mostrado.
    go(
        &mut s,
        Event::Peer(PeerEvent::Remote { id: "bia".into(), stream: Some(stream("bia")), audio: Some(stream("bia-som")), flowing: true }),
    );
    assert!(!s.media["bia"].video);
    let v = view(&s, NOW);
    assert_eq!(v.streams[0].stream, None);
    assert_eq!(v.streams[0].status, TileStatus::Loading);
    // Abrir, destacar, aproximar, flutuar: nada disso muda o que chega.
    let mut tudo = Vec::new();
    tudo.extend(acao(&mut s, RoomAction::Focus { id: Some("bia".into()) }));
    tudo.extend(acao(&mut s, RoomAction::Pip { id: Some("bia".into()) }));
    tudo.extend(go(&mut s, Event::Window(WindowEvent::FloatingOpened("bia".into()))));
    tudo.extend(acao(&mut s, RoomAction::Focus { id: None }));
    tudo.extend(go(&mut s, Event::Window(WindowEvent::Visible(false))));
    tudo.extend(go(&mut s, Event::Window(WindowEvent::Visible(true))));
    assert!(s.watching.is_empty());
    assert!(s.stopped.is_empty());
    assert_eq!(s.links["bia"].my_view, Some(ViewMode::Off));
    assert_eq!(s.sent_modes.get("bia"), Some(&ViewMode::Off));
    assert!(views_enviadas(&tudo).is_empty(), "nenhuma mudança de modo sem ter escolhido: {tudo:?}");
    assert!(!tem(&tudo, |e| matches!(e, Effect::Schedule { timer: TimerId::FirstFrame(_), .. })));
}

#[test]
fn assistir_liga_o_video_e_o_primeiro_quadro_cancela_a_espera() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    conectar(&mut s, "bia");
    let fx = assistir(&mut s, "bia");
    assert_eq!(views_enviadas(&fx), vec![("bia".into(), ViewMode::Full, 2)]);
    assert_eq!(s.watching["bia"].transmission, TransmissionKey { peer_id: "bia".into(), started_at: 100 });
    assert!(agendou(&fx, TimerId::FirstFrame("bia".into()), 10_000));
    assert_eq!(view(&s, NOW).streams[0].watch_state, Some(StreamWatchState::Opening));
    let fx = go(
        &mut s,
        Event::Peer(PeerEvent::Remote { id: "bia".into(), stream: Some(stream("bia")), audio: None, flowing: true }),
    );
    assert!(cancelou(&fx, TimerId::FirstFrame("bia".into())));
    let v = view(&s, NOW);
    assert_eq!(v.streams[0].watch_state, Some(StreamWatchState::Playing));
    assert_eq!(v.streams[0].status, TileStatus::Ok);
    assert_eq!(v.streams[0].stream, Some(stream("bia")));
    assert_eq!(v.streams[0].watching, Some(true));
}

#[test]
fn parar_de_assistir_corta_video_e_som_e_so_vale_para_aquela_transmissao() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    conectar(&mut s, "bia");
    assistir(&mut s, "bia");
    go(&mut s, Event::Peer(PeerEvent::Remote { id: "bia".into(), stream: Some(stream("bia")), audio: Some(stream("som")), flowing: true }));
    assert!(s.media["bia"].video);
    let fx = acao(&mut s, RoomAction::Watch { id: "bia".into(), on: false });
    assert!(s.watching.is_empty());
    assert_eq!(s.stopped.get("bia"), Some(&100));
    assert!(!s.media["bia"].video);
    assert_eq!(views_enviadas(&fx), vec![("bia".into(), ViewMode::Off, 3)]);
    let v = view(&s, NOW);
    assert_eq!(v.streams[0].watch_state, Some(StreamWatchState::Stopped));
    assert_eq!(v.streams[0].stream, None);
    // Os quadros que ainda chegam não voltam a aparecer.
    go(&mut s, Event::Peer(PeerEvent::VideoFlow { id: "bia".into(), flowing: true }));
    assert!(!s.media["bia"].video);
    // Voltar a assistir tira a marca de parada.
    assistir(&mut s, "bia");
    assert!(s.stopped.is_empty());
    assert_eq!(view(&s, NOW).streams[0].watch_state, Some(StreamWatchState::Opening));
    // Parou e a pessoa recomeçou outra transmissão: volta a ser oferta.
    acao(&mut s, RoomAction::Watch { id: "bia".into(), on: false });
    go(&mut s, Event::Server(ServerMessage::PeerLive { id: "bia".into(), live: None }));
    go(&mut s, Event::Server(ServerMessage::PeerLive { id: "bia".into(), live: Some(live_info(200)) }));
    assert_eq!(view(&s, NOW).streams[0].watch_state, Some(StreamWatchState::Available));
}

#[test]
fn foco_muda_so_o_modo_entre_cheia_e_miniatura_e_nunca_assistir() {
    let mut s = sala(&[("bia", "Bia", Some(100)), ("lu", "Lu", Some(200))]);
    conectar(&mut s, "bia");
    conectar(&mut s, "lu");
    assistir(&mut s, "bia");
    assistir(&mut s, "lu");
    assert_eq!(s.sent_modes["bia"], ViewMode::Full);
    assert_eq!(s.sent_modes["lu"], ViewMode::Full);
    let fx = acao(&mut s, RoomAction::Focus { id: Some("bia".into()) });
    assert_eq!(s.sent_modes["bia"], ViewMode::Full);
    assert_eq!(s.sent_modes["lu"], ViewMode::Thumb);
    assert_eq!(views_enviadas(&fx).len(), 1);
    assert_eq!(views_enviadas(&fx)[0].0, "lu");
    assert_eq!(s.watching.len(), 2);
    acao(&mut s, RoomAction::Focus { id: None });
    assert_eq!(s.sent_modes["lu"], ViewMode::Full);
    assert_eq!(s.watching.len(), 2);
    // Parou de assistir a uma delas: `off` mesmo em destaque.
    acao(&mut s, RoomAction::Watch { id: "lu".into(), on: false });
    acao(&mut s, RoomAction::Focus { id: Some("lu".into()) });
    assert_eq!(s.sent_modes["lu"], ViewMode::Off);
    assert_eq!(s.sent_modes["bia"], ViewMode::Thumb);
}

#[test]
fn janela_escondida_corta_so_o_video_e_ao_voltar_espera_o_quadro_de_novo() {
    let mut s = sala(&[("bia", "Bia", Some(100)), ("lu", "Lu", Some(200))]);
    conectar(&mut s, "bia");
    conectar(&mut s, "lu");
    assistir(&mut s, "bia");
    go(&mut s, Event::Peer(PeerEvent::Remote { id: "bia".into(), stream: Some(stream("bia")), audio: None, flowing: true }));
    let fx = go(&mut s, Event::Window(WindowEvent::Visible(false)));
    // `hidden` para quem assisto, `off` para quem não (não assistida ganha de tudo).
    assert_eq!(s.sent_modes["bia"], ViewMode::Hidden);
    assert_eq!(s.sent_modes["lu"], ViewMode::Off);
    assert_eq!(views_enviadas(&fx), vec![("bia".into(), ViewMode::Hidden, 3)]);
    // Com a janela escondida o vídeo não vem e isso não é falha.
    assert!(!tem(&fx, |e| matches!(e, Effect::Schedule { timer: TimerId::FirstFrame(_), .. })));
    go(&mut s, Event::Peer(PeerEvent::VideoFlow { id: "bia".into(), flowing: false }));
    assert!(!s.timed_out.contains("bia"));
    // Janela flutuante continua recebendo.
    acao(&mut s, RoomAction::Pip { id: Some("bia".into()) });
    assert_eq!(s.sent_modes["bia"], ViewMode::Full);
    acao(&mut s, RoomAction::Pip { id: None });
    assert_eq!(s.sent_modes["bia"], ViewMode::Hidden);
    // De volta: modo cheio e a espera do primeiro quadro recomeça do zero.
    let fx = go(&mut s, Event::Window(WindowEvent::Visible(true)));
    assert_eq!(s.sent_modes["bia"], ViewMode::Full);
    assert!(agendou(&fx, TimerId::FirstFrame("bia".into()), 10_000));
    assert!(!s.timed_out.contains("bia"));
}

#[test]
fn primeiro_quadro_que_nao_chega_em_10_segundos_vira_falha_e_tentar_de_novo_recomeca() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    conectar(&mut s, "bia");
    assistir(&mut s, "bia");
    go(&mut s, Event::Timer(TimerId::FirstFrame("bia".into())));
    assert!(s.timed_out.contains("bia"));
    let v = view(&s, NOW);
    assert_eq!(v.streams[0].status, TileStatus::Failed);
    assert_eq!(v.streams[0].watch_state, Some(StreamWatchState::Unavailable));
    // Tentar de novo: limpa a falha, reinicia a espera e recomeça a ligação.
    let fx = acao(&mut s, RoomAction::Retry { id: "bia".into() });
    assert!(!s.timed_out.contains("bia"));
    assert!(agendou(&fx, TimerId::FirstFrame("bia".into()), 10_000));
    assert!(fx.contains(&Effect::RetryPeer { id: "bia".into() }));
    // Quem não assisto não tenta de novo.
    let mut t = sala(&[("bia", "Bia", Some(100))]);
    assert!(acao(&mut t, RoomAction::Retry { id: "bia".into() }).is_empty());
}

#[test]
fn tentar_de_novo_sem_ligacao_abre_uma_nova_iniciada_por_mim() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    assistir(&mut s, "bia");
    // A ligação sumiu (o motor a descartou): "Tentar de novo" cria outra, como `mesh.retry` do site.
    s.links.remove("bia");
    let fx = acao(&mut s, RoomAction::Retry { id: "bia".into() });
    assert!(fx.contains(&Effect::OpenPeer { id: "bia".into(), initiator: true, polite: false }));
    assert!(s.links.contains_key("bia"));
    assert!(!tem(&fx, |e| matches!(e, Effect::RetryPeer { .. })));
}

#[test]
fn quadro_que_chega_antes_dos_10_segundos_deixa_o_temporizador_sem_efeito() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    conectar(&mut s, "bia");
    assistir(&mut s, "bia");
    go(&mut s, Event::Peer(PeerEvent::VideoFlow { id: "bia".into(), flowing: true }));
    let antes = s.clone();
    let fx = go(&mut s, Event::Timer(TimerId::FirstFrame("bia".into())));
    assert_eq!((&s, fx), (&antes, vec![]));
    // Ligação que falhou: o tile mostra "falhou" assim que o vídeo para (`failed` ganha de `flowing` no `watchState`).
    go(&mut s, Event::Peer(PeerEvent::Remote { id: "bia".into(), stream: Some(stream("bia")), audio: None, flowing: true }));
    go(&mut s, Event::Peer(PeerEvent::State { id: "bia".into(), conn: ConnState::Failed }));
    assert_eq!(view(&s, NOW).streams[0].watch_state, Some(StreamWatchState::Unavailable));
    assert_eq!(view(&s, NOW).streams[0].status, TileStatus::Ok, "o quadro ainda chega: o status segue `flowing`");
    go(&mut s, Event::Peer(PeerEvent::VideoFlow { id: "bia".into(), flowing: false }));
    assert_eq!(view(&s, NOW).streams[0].status, TileStatus::Failed);
}

#[test]
fn nova_transmissao_da_pessoa_zera_o_modo_e_avisa_que_nao_assisto() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    conectar(&mut s, "bia");
    assistir(&mut s, "bia");
    assert_eq!(s.links["bia"].my_view, Some(ViewMode::Full));
    go(&mut s, Event::Server(ServerMessage::PeerLive { id: "bia".into(), live: None }));
    assert_eq!(s.links["bia"].remote_key, None);
    let fx = go(&mut s, Event::Server(ServerMessage::PeerLive { id: "bia".into(), live: Some(live_info(300)) }));
    // A escolha era da transmissão antiga: não vale para a nova, que começa em `off`.
    assert!(s.watching.is_empty());
    assert_eq!(s.links["bia"].remote_key, Some(TransmissionKey { peer_id: "bia".into(), started_at: 300 }));
    assert_eq!(s.links["bia"].my_view, Some(ViewMode::Off));
    assert!(views_enviadas(&fx).iter().any(|(to, m, _)| to == "bia" && *m == ViewMode::Off));
}

#[test]
fn welcome_de_pessoa_nova_esquece_o_que_eu_assistia() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    assistir(&mut s, "bia");
    acao(&mut s, RoomAction::Watch { id: "bia".into(), on: false });
    assert_eq!(s.stopped.len(), 1);
    // Mesmo id (retomada): mantém.
    assistir(&mut s, "bia");
    go(&mut s, welcome(pessoa(ME, "Ana", None), vec![pessoa("bia", "Bia", Some(100))]));
    assert_eq!(s.watching.len(), 1);
    // Id novo (passou da tolerância): recomeça do zero e refaz as ligações.
    acao(&mut s, RoomAction::Watch { id: "bia".into(), on: false });
    let fx = go(&mut s, welcome(pessoa("eu2", "Ana", None), vec![pessoa("bia", "Bia", Some(100))]));
    assert!(s.watching.is_empty());
    assert!(s.stopped.is_empty());
    assert!(fx.contains(&Effect::ClosePeer { id: "bia".into() }));
    assert!(fx.contains(&Effect::OpenPeer { id: "bia".into(), initiator: true, polite: false }));
    assert_eq!(s.mesh_id.as_deref(), Some("eu2"));
}

// ---------------------------------------------------------------------------
// Sinalização: queda e volta
// ---------------------------------------------------------------------------

#[test]
fn queda_e_volta_da_sinalizacao_com_falha_depois_de_60_segundos() {
    let mut s = sala(&[("bia", "Bia", None)]);
    let fx = go(&mut s, Event::Link(LinkEvent::Reconnecting));
    assert_eq!(s.link, LinkState::Reconnecting);
    assert_eq!(fx, vec![Effect::Schedule { timer: TimerId::LinkDown, after_ms: 60_000 }]);
    // A sala continua na tela e a ligação com as pessoas não cai junto.
    assert!(s.connected);
    assert_eq!(s.ended, None);
    assert!(s.links.contains_key("bia"));
    // Outra queda no meio da queda não reinicia a contagem.
    assert!(go(&mut s, Event::Link(LinkEvent::Reconnecting)).is_empty());
    go(&mut s, Event::Timer(TimerId::LinkDown));
    assert_eq!(s.link, LinkState::Failed);
    assert_eq!(view(&s, NOW).link, LinkState::Failed);
    // Voltou depois de "Não deu pra voltar".
    let fx = go(&mut s, Event::Link(LinkEvent::Back));
    assert_eq!(s.link, LinkState::Ok);
    assert!(cancelou(&fx, TimerId::LinkDown));
    let t = s.toasts.last().unwrap();
    assert_eq!((t.text.as_str(), t.kind), ("De volta!", ToastKind::Copied));
    // Voltar sem ter caído não faz nada.
    assert!(go(&mut s, Event::Link(LinkEvent::Back)).is_empty());
}

#[test]
fn queda_antes_do_welcome_nao_conta_e_voltar_antes_dos_60_segundos_cancela() {
    let mut s = RoomState::new(init());
    assert!(go(&mut s, Event::Link(LinkEvent::Reconnecting)).is_empty());
    assert_eq!(s.link, LinkState::Ok);
    let mut s = sala(&[]);
    go(&mut s, Event::Link(LinkEvent::Reconnecting));
    go(&mut s, Event::Link(LinkEvent::Back));
    assert_eq!(s.link, LinkState::Ok);
    // O temporizador que já estava no ar vence sem efeito.
    go(&mut s, Event::Timer(TimerId::LinkDown));
    assert_eq!(s.link, LinkState::Ok);
}

#[test]
fn tentar_de_novo_recomeca_a_contagem_e_pede_a_ligacao_na_hora() {
    let mut s = sala(&[]);
    go(&mut s, Event::Link(LinkEvent::Reconnecting));
    go(&mut s, Event::Timer(TimerId::LinkDown));
    let fx = acao(&mut s, RoomAction::RetryConnection);
    assert_eq!(s.link, LinkState::Reconnecting);
    assert_eq!(fx, vec![Effect::Schedule { timer: TimerId::LinkDown, after_ms: 60_000 }, Effect::RetryNow]);
}

#[test]
fn sem_servidor_nao_da_para_transmitir_nem_abrir_menus_do_dono() {
    let mut s = dono();
    go(&mut s, Event::Link(LinkEvent::Reconnecting));
    assert!(acao(&mut s, RoomAction::OpenGoLive).is_empty());
    assert!(!s.go_live_open);
    acao(&mut s, RoomAction::ToggleOwnerMenu { open: true });
    assert!(!s.owner_menu);
    acao(&mut s, RoomAction::OpenRename);
    assert!(s.name_dialog.is_none());
    acao(&mut s, RoomAction::OpenDelete);
    assert!(s.delete_dialog.is_none());
    acao(&mut s, RoomAction::AskRemove { peer_id: "bia".into() });
    assert!(s.remove_dialog.is_none());
}

#[test]
fn identidade_nova_da_sessao_vai_para_o_aparelho() {
    let mut s = sala(&[]);
    let sessao = RoomSession { peer_id: "eu".into(), resume_token: "tok".into() };
    let fx = go(&mut s, Event::Link(LinkEvent::Session(Some(sessao.clone()))));
    assert_eq!(fx, vec![Effect::Save(DeviceChange::Session { code: "k7f-q2x".into(), session: Some(sessao) })]);
    let fx = go(&mut s, Event::Link(LinkEvent::Session(None)));
    assert_eq!(fx, vec![Effect::Save(DeviceChange::Session { code: "k7f-q2x".into(), session: None })]);
}

// ---------------------------------------------------------------------------
// Fim da sala: cada motivo
// ---------------------------------------------------------------------------

#[test]
fn cada_motivo_de_fim_fecha_tudo_e_leva_a_tela_do_motivo() {
    for reason in [
        EndReason::NotFound,
        EndReason::Full,
        EndReason::Replaced,
        EndReason::Kicked,
        EndReason::Deleted,
        EndReason::Removed,
        EndReason::Update,
    ] {
        let mut s = sala(&[("bia", "Bia", Some(100))]);
        conectar(&mut s, "bia");
        transmitir(&mut s, GoLive::default());
        let fx = go(&mut s, Event::Link(LinkEvent::Ended(reason)));
        assert_eq!(s.ended, Some(reason), "{reason:?}");
        assert!(fx.contains(&Effect::Navigate(Destination::Ended(reason))), "{reason:?}");
        assert!(fx.contains(&Effect::ClosePeer { id: "bia".into() }), "{reason:?}");
        assert!(fx.contains(&Effect::StopCapture), "{reason:?}");
        // A sala acabou: ninguém é avisado e a transmissão não manda `unlive`.
        assert!(!fx.contains(&Effect::Leave), "{reason:?}");
        assert!(!tem(&fx, |e| matches!(e, Effect::Send(ClientMessage::Unlive { .. }))), "{reason:?}");
        assert!(s.my_broadcast.is_none() && s.links.is_empty() && s.mesh_id.is_none(), "{reason:?}");
        // Depois do fim, só dá para sair.
        let antes = s.clone();
        assert!(go(&mut s, Event::Server(ServerMessage::PeerJoined { peer: pessoa("lu", "Lu", None) })).is_empty());
        assert_eq!(s, antes);
    }
}

#[test]
fn apaguei_a_sala_eu_mesmo_volta_pro_inicio_uma_vez_so_em_qualquer_ordem() {
    // A resposta da API chega antes do fechamento 4410.
    let mut s = dono();
    acao(&mut s, RoomAction::OpenDelete);
    let fx = acao(&mut s, RoomAction::ConfirmDelete);
    assert_eq!(fx, vec![Effect::Api(ApiCall::Delete { code: "k7f-q2x".into(), key: "k".repeat(64) })]);
    assert!(s.deleting);
    let fx = go(&mut s, Event::Api(ApiEvent::Delete(Ok(()))));
    assert!(fx.contains(&Effect::Save(DeviceChange::ForgetRoom { code: "k7f-q2x".into() })));
    assert!(fx.contains(&Effect::Save(DeviceChange::ForgetOwnerKey { code: "k7f-q2x".into() })));
    assert!(fx.contains(&Effect::Leave));
    assert!(fx.contains(&Effect::Navigate(Destination::Home)));
    assert_eq!(s.ended, None, "sem a tela de 'apagaram a sala'");
    assert!(go(&mut s, Event::Link(LinkEvent::Ended(EndReason::Deleted))).is_empty());

    // O fechamento 4410 chega antes da resposta.
    let mut s = dono();
    acao(&mut s, RoomAction::OpenDelete);
    acao(&mut s, RoomAction::ConfirmDelete);
    let fx = go(&mut s, Event::Link(LinkEvent::Ended(EndReason::Deleted)));
    assert!(fx.contains(&Effect::Navigate(Destination::Home)));
    assert!(!tem(&fx, |e| matches!(e, Effect::Navigate(Destination::Ended(_)))));
    assert_eq!(s.ended, None);
    assert!(go(&mut s, Event::Api(ApiEvent::Delete(Ok(())))).is_empty());

    // Outra pessoa apagou: tela do motivo.
    let mut s = dono();
    let fx = go(&mut s, Event::Link(LinkEvent::Ended(EndReason::Deleted)));
    assert!(fx.contains(&Effect::Navigate(Destination::Ended(EndReason::Deleted))));
}

#[test]
fn sair_da_sala_avisa_o_servidor_e_solta_tudo() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    conectar(&mut s, "bia");
    let fx = acao(&mut s, RoomAction::Leave);
    assert!(fx.contains(&Effect::Leave));
    assert!(fx.contains(&Effect::ClosePeer { id: "bia".into() }));
    assert_eq!(fx.last(), Some(&Effect::Navigate(Destination::Home)));
    assert!(s.left);
    // Depois de sair, nada mais vale.
    let antes = s.clone();
    assert!(go(&mut s, Event::Server(ServerMessage::PeerJoined { peer: pessoa("lu", "Lu", None) })).is_empty());
    assert_eq!(s, antes);
}

#[test]
fn sair_de_proposito_esquece_a_identidade_como_o_leave_do_site() {
    // `signaling.leave()` chama `onSession(null)`: quem sai não retoma o lugar.
    let mut s = sala(&[]);
    go(&mut s, Event::Link(LinkEvent::Session(Some(RoomSession { peer_id: "eu".into(), resume_token: "t".into() }))));
    assert!(s.session.is_some());
    let fx = acao(&mut s, RoomAction::Leave);
    assert!(fx.contains(&Effect::Save(DeviceChange::Session { code: "k7f-q2x".into(), session: None })));
    assert!(s.session.is_none());
    // O mesmo ao apagar a sala eu mesmo; já com a sala acabada por outro motivo, não há o que esquecer
    // (a sinalização já mandou `Session(None)` antes do fim).
    let mut e = sala(&[]);
    go(&mut e, Event::Link(LinkEvent::Ended(EndReason::Kicked)));
    let fx = acao(&mut e, RoomAction::GoHome);
    assert!(!tem(&fx, |x| matches!(x, Effect::Save(DeviceChange::Session { .. }))));
}

#[test]
fn comecar_sem_fonte_escolhida_nao_derruba_a_lista_de_janelas() {
    let mut s = sala(&[]);
    acao(&mut s, RoomAction::OpenGoLive);
    go(&mut s, Event::Capture(CaptureEvent::Sources(vec![janela()])));
    let fx = acao(&mut s, RoomAction::StartGoLive { prefs: GoLive::default() });
    assert!(fx.is_empty());
    assert!(s.listing && s.go_live_open && s.pending_start.is_none());
}

#[test]
fn logo_transmitindo_pergunta_antes_de_sair() {
    let mut s = sala(&[]);
    assert_eq!(acao(&mut s, RoomAction::GoHome).last(), Some(&Effect::Navigate(Destination::Home)));
    let mut s = sala(&[]);
    transmitir(&mut s, GoLive::default());
    assert_eq!(acao(&mut s, RoomAction::GoHome), vec![Effect::ConfirmLeave]);
    assert!(!s.left && s.my_broadcast.is_some());
    // Confirmou: o laço devolve `Leave`; a transmissão para, sem avisar a sala de novo (o servidor já vê a saída).
    let fx = acao(&mut s, RoomAction::Leave);
    assert!(fx.contains(&Effect::StopCapture));
    assert!(fx.contains(&Effect::Leave));
    assert!(!tem(&fx, |e| matches!(e, Effect::Send(ClientMessage::Unlive { .. }))));
}

// ---------------------------------------------------------------------------
// Transmitir
// ---------------------------------------------------------------------------

#[test]
fn transmitir_pedido_ack_e_ordem_de_transmissao() {
    let order = BroadcastOrder {
        highest_broadcast_seq: 3,
        request_id: "00000000-0000-4000-8000-000000000001".into(),
        last_started_at: 9,
        status: BroadcastStatus::Ended,
    };
    let mut s = RoomState::new(init());
    go(&mut s, welcome_em(pessoa(ME, "Ana", None), vec![], None, Some(order), vec![]));
    assert_eq!(s.highest_broadcast_seq, 3);
    let fx = acao(&mut s, RoomAction::OpenGoLive);
    assert_eq!(fx, vec![Effect::StartListingSources]);
    assert!(s.go_live_open);
    go(&mut s, Event::Capture(CaptureEvent::Sources(vec![janela()])));
    acao(&mut s, RoomAction::SelectSource { id: "w:1".into() });
    let prefs = GoLive { fps: 60, ..GoLive::default() };
    let fx = acao(&mut s, RoomAction::StartGoLive { prefs: prefs.clone() });
    let esperado = GoLive { surface: Surface::Window, last_source_key: Some("hollow_knight.exe|Hollow Knight".into()), ..prefs };
    assert_eq!(
        fx,
        vec![
            Effect::StopListingSources,
            Effect::Save(DeviceChange::GoLivePrefs(esperado.clone())),
            Effect::StartCapture { prefs: esperado, target: CaptureTarget::Source(janela()) },
        ]
    );
    assert!(!s.go_live_open);
    assert!(s.my_broadcast.is_none(), "ainda abrindo a captura");
    let fx = go(
        &mut s,
        Event::Capture(CaptureEvent::Started {
            request_id: "00000000-0000-4000-8000-0000000000aa".into(),
            stream: stream("tela"),
            audio_kind: AudioKind::Pc,
            audio_warn: None,
        }),
    );
    // A ordem da sala manda: a próxima sequência é a seguinte à maior vista.
    assert_eq!(anuncios(&fx), vec![("00000000-0000-4000-8000-0000000000aa".to_string(), 4, 1080, 60)]);
    assert_eq!(qualidade_da_captura(&fx), vec![(1080, 60)]);
    assert!(fx.contains(&Effect::NewRequestId), "o redutor não sorteia: pede outro identificador");
    assert!(texto_dos_avisos(&s).contains(&"Você está ao vivo pra sala".to_string()));
    let anuncio = fx.iter().find_map(|e| match e {
        Effect::Send(ClientMessage::Live { audio, source, label, .. }) => Some((*audio, *source, label.clone())),
        _ => None,
    });
    assert_eq!(anuncio, Some((AudioKind::Pc, Some(LiveSource::Screen), Some("hollow_knight.exe".into()))));
    assert_eq!(s.capture_request, Some(LiveRequest { request_id: "00000000-0000-4000-8000-0000000000aa".into(), broadcast_seq: 4 }));
    assert_eq!(s.highest_broadcast_seq, 4);
    let v = view(&s, NOW);
    assert!(v.my_broadcast.is_some());
    assert_eq!(v.participants[0].live, true);
    assert_eq!(v.participants[0].live_label.as_deref(), Some("hollow_knight.exe"));

    // `live-ack` com outro pedido ou outra sequência não vale.
    let antes = s.clone();
    for (rid, seq) in [("outro", 4), ("00000000-0000-4000-8000-0000000000aa", 3)] {
        let fx = go(&mut s, Event::Server(ServerMessage::LiveAck { request_id: rid.into(), broadcast_seq: seq, live: live_info(500) }));
        assert!(fx.is_empty());
        assert_eq!(s, antes);
    }
    // O ack certo confirma a transmissão: a minha chave passa a valer.
    go(
        &mut s,
        Event::Server(ServerMessage::LiveAck {
            request_id: "00000000-0000-4000-8000-0000000000aa".into(),
            broadcast_seq: 4,
            live: live_info(500),
        }),
    );
    assert_eq!(s.local_key, Some(minha_chave()));
    assert_eq!(s.me.as_ref().unwrap().live.as_ref().unwrap().started_at, 500);
    assert_eq!(s.highest_broadcast_seq, 4);
}

#[test]
fn parar_de_transmitir_avisa_a_sala_e_solta_a_captura() {
    let mut s = sala(&[("bia", "Bia", None)]);
    conectar(&mut s, "bia");
    transmitir(&mut s, GoLive::default());
    assert!(s.local_key.is_some());
    let fx = acao(&mut s, RoomAction::StopGoLive);
    assert!(fx.contains(&Effect::StopCapture));
    assert!(fx.contains(&Effect::Send(ClientMessage::Unlive { request_id: "req-1".into(), broadcast_seq: 0 })));
    assert!(s.my_broadcast.is_none());
    assert_eq!(s.local_key, None);
    assert_eq!(s.capture_request, None);
    assert_eq!(s.me.as_ref().unwrap().live, None);
    assert!(fx.contains(&Effect::Snapshots { on: false }));
    assert!(view(&s, NOW).my_broadcast.is_none());
    // Parar de novo não faz nada.
    assert!(acao(&mut s, RoomAction::StopGoLive).is_empty());
}

#[test]
fn captura_que_acaba_sozinha_para_a_transmissao_como_o_botao() {
    let mut s = sala(&[]);
    transmitir(&mut s, GoLive::default());
    let fx = go(&mut s, Event::Capture(CaptureEvent::Ended));
    assert!(s.my_broadcast.is_none());
    assert!(fx.contains(&Effect::Send(ClientMessage::Unlive { request_id: "req-1".into(), broadcast_seq: 0 })));
    assert!(fx.contains(&Effect::StopCapture));
}

#[test]
fn captura_que_chega_depois_de_parar_e_solta_sem_transmitir() {
    let mut s = sala(&[]);
    let antes = s.clone();
    let fx = go(
        &mut s,
        Event::Capture(CaptureEvent::Started {
            request_id: "r".into(),
            stream: stream("x"),
            audio_kind: AudioKind::None,
            audio_warn: None,
        }),
    );
    assert_eq!(fx, vec![Effect::StopCapture]);
    assert_eq!(s, antes);
}

#[test]
fn falha_ao_abrir_a_captura_volta_ao_seletor_com_o_aviso_do_site() {
    let mut s = sala(&[]);
    acao(&mut s, RoomAction::OpenGoLive);
    go(&mut s, Event::Capture(CaptureEvent::Sources(vec![janela()])));
    acao(&mut s, RoomAction::SelectSource { id: "w:1".into() });
    acao(&mut s, RoomAction::StartGoLive { prefs: GoLive::default() });
    let fx = go(&mut s, Event::Capture(CaptureEvent::StartFailed(StartFailure::SourceGone)));
    assert_eq!(texto_dos_avisos(&s), vec!["Essa janela fechou — escolhe outra."]);
    assert_eq!(s.toasts[0].kind, ToastKind::Error);
    assert_eq!(s.selected_id, None);
    assert!(s.go_live_open);
    assert!(fx.contains(&Effect::StartListingSources));
    // Erro não some sozinho: nenhum temporizador.
    assert!(!tem(&fx, |e| matches!(e, Effect::Schedule { timer: TimerId::Toast(_), .. })));

    let mut s = sala(&[]);
    acao(&mut s, RoomAction::OpenGoLive);
    go(&mut s, Event::Capture(CaptureEvent::Sources(vec![janela()])));
    acao(&mut s, RoomAction::SelectSource { id: "w:1".into() });
    acao(&mut s, RoomAction::StartGoLive { prefs: GoLive::default() });
    go(&mut s, Event::Capture(CaptureEvent::StartFailed(StartFailure::Other)));
    assert_eq!(
        texto_dos_avisos(&s),
        vec!["Não deu pra capturar pelo app — tenta de novo, agora pelo seletor do Windows."]
    );
    assert_eq!(s.selected_id.as_deref(), Some("w:1"));
    assert!(s.go_live_open);
}

#[test]
fn transmitir_a_camera_usa_o_nome_dela_e_os_avisos_da_camera() {
    let mut s = sala(&[]);
    let prefs = GoLive { surface: Surface::Camera, ..GoLive::default() };
    acao(&mut s, RoomAction::OpenGoLive);
    acao(&mut s, RoomAction::SetSurface { surface: Surface::Camera });
    go(&mut s, Event::Capture(CaptureEvent::CameraPermission(CameraPermission::Granted)));
    go(
        &mut s,
        Event::Capture(CaptureEvent::CameraDevices {
            videos: vec![DeviceInfo { id: "v1".into(), group_id: "g".into(), label: "Logitech C920".into() }],
            audios: vec![],
        }),
    );
    assert_eq!(s.cam_video_id.as_deref(), Some("v1"));
    let fx = acao(&mut s, RoomAction::StartGoLive { prefs: prefs.clone() });
    assert!(tem(&fx, |e| matches!(
        e,
        Effect::StartCapture { target: CaptureTarget::Camera { video_id, audio_id: None, .. }, .. } if video_id == "v1"
    )));
    assert!(fx.contains(&Effect::LeaveCameraTab), "fecha as prévias antes de abrir a câmera");
    let fx = go(
        &mut s,
        Event::Capture(CaptureEvent::Started { request_id: "r1".into(), stream: stream("cam"), audio_kind: AudioKind::None, audio_warn: None }),
    );
    assert!(texto_dos_avisos(&s).contains(&"Sua câmera está ao vivo pra sala".to_string()));
    let anuncio = fx.iter().find_map(|e| match e {
        Effect::Send(ClientMessage::Live { source, label, .. }) => Some((*source, label.clone())),
        _ => None,
    });
    assert_eq!(anuncio, Some((Some(LiveSource::Camera), Some("Câmera · Logitech C920".into()))));
    assert_eq!(s.my_broadcast.as_ref().unwrap().prefs.last_video_label.as_deref(), Some("Logitech C920"));
    // Câmera em uso por outro programa.
    let mut t = sala(&[]);
    acao(&mut t, RoomAction::OpenGoLive);
    acao(&mut t, RoomAction::SetSurface { surface: Surface::Camera });
    go(&mut t, Event::Capture(CaptureEvent::CameraPermission(CameraPermission::Granted)));
    go(
        &mut t,
        Event::Capture(CaptureEvent::CameraDevices {
            videos: vec![DeviceInfo { id: "v1".into(), group_id: "g".into(), label: "Cam".into() }],
            audios: vec![],
        }),
    );
    acao(&mut t, RoomAction::StartGoLive { prefs });
    go(&mut t, Event::Capture(CaptureEvent::StartFailed(StartFailure::Busy)));
    assert_eq!(texto_dos_avisos(&t), vec!["Essa câmera está sendo usada por outro programa."]);
    assert!(t.go_live_open);
    assert_eq!(t.tab, Surface::Camera);
}

#[test]
fn avisos_de_som_e_de_tela_preta_durante_a_transmissao() {
    let mut s = sala(&[]);
    // Sem transmitir, não há o que avisar.
    go(&mut s, Event::Capture(CaptureEvent::BlackScreen));
    assert!(s.toasts.is_empty());
    transmitir(&mut s, GoLive::default());
    go(&mut s, Event::Capture(CaptureEvent::BlackScreen));
    assert_eq!(s.toasts.last().unwrap().text, BLACK_WARN);
    // O som mudou no meio: avisa e anuncia de novo, com o mesmo pedido.
    let fx = go(
        &mut s,
        Event::Capture(CaptureEvent::AudioChanged { kind: AudioKind::None, warn: Some(AudioWarn::ManyDiscords) }),
    );
    assert_eq!(s.toasts.last().unwrap().text, "Mais de um Discord aberto — tela inteira vai sem som.");
    assert_eq!(s.toasts.last().unwrap().kind, ToastKind::Error);
    assert_eq!(anuncios(&fx).len(), 1);
    assert_eq!(anuncios(&fx)[0].0, "req-1");
    // Mesmo tipo de som: não anuncia de novo.
    let fx = go(&mut s, Event::Capture(CaptureEvent::AudioChanged { kind: AudioKind::None, warn: None }));
    assert!(anuncios(&fx).is_empty());
}

#[test]
fn welcome_de_pessoa_perdida_anuncia_de_novo_com_identificador_novo() {
    let mut s = sala(&[]);
    transmitir(&mut s, GoLive::default());
    // O servidor não lembra da minha transmissão (voltei como pessoa nova, mesmo id).
    let fx = go(&mut s, welcome(pessoa(ME, "Ana", None), vec![]));
    assert_eq!(s.capture_request, None);
    assert!(anuncios(&fx).is_empty(), "o identificador de reserva já foi usado");
    assert!(s.live_pending);
    assert_eq!(s.local_key, None);
    let fx = go(&mut s, Event::Capture(CaptureEvent::RequestId("req-2".into())));
    assert_eq!(anuncios(&fx), vec![("req-2".to_string(), 0, 1080, 30)]);
    assert!(!s.live_pending);
}

#[test]
fn welcome_sem_transmitir_mas_servidor_acha_que_sim_manda_unlive() {
    let order = BroadcastOrder {
        highest_broadcast_seq: 7,
        request_id: "00000000-0000-4000-8000-000000000001".into(),
        last_started_at: 9,
        status: BroadcastStatus::Live,
    };
    let mut s = RoomState::new(init());
    let fx = go(&mut s, welcome_em(pessoa(ME, "Ana", Some(9)), vec![], None, Some(order), vec![]));
    assert!(fx.contains(&Effect::Send(ClientMessage::Unlive {
        request_id: "00000000-0000-4000-8000-000000000001".into(),
        broadcast_seq: 7
    })));
}

// ---------------------------------------------------------------------------
// ViewMessage recebida, qualidade e processador
// ---------------------------------------------------------------------------

#[test]
fn view_message_valida_muda_os_parametros_de_envio_e_so_ela() {
    let mut s = sala(&[("bia", "Bia", None)]);
    transmitir(&mut s, GoLive { fps: 60, ..GoLive::default() });
    // Sem nenhuma mensagem de view: nada sai para ninguém, nem o som.
    let fx = conectar(&mut s, "bia");
    assert!(fx.contains(&Effect::RequestView { to: "bia".into(), transmission: minha_chave() }));
    assert_eq!(s.links["bia"].their_view, ViewMode::Off);
    let fx = go(&mut s, Event::Peer(PeerEvent::ControlClosed { id: "bia".into() }));
    let (_, p, som) = parametros(&fx).pop().unwrap();
    assert_eq!((p.active, som), (false, false));
    go(&mut s, Event::Peer(PeerEvent::ControlOpen { id: "bia".into() }));

    // Cheia: vídeo e som, na qualidade escolhida.
    let fx = recebe_view(&mut s, "bia", 1, ViewMode::Full);
    assert_eq!(
        parametros(&fx),
        vec![("bia".into(), SendParams { active: true, scale_resolution_down_by: 1.0, max_framerate: 60, max_bitrate: 6_000_000 }, true)]
    );
    // Miniatura: 360p, 15 fps, 500 kbps.
    let fx = recebe_view(&mut s, "bia", 2, ViewMode::Thumb);
    let (_, p, som) = parametros(&fx).pop().unwrap();
    assert_eq!((p.active, p.max_framerate, p.max_bitrate, som), (true, 15, 500_000, true));
    assert!((p.scale_resolution_down_by - 3.0).abs() < 1e-9);
    // Segundo plano: sem vídeo, o som continua. Parou de assistir: sem os dois.
    let (_, p, som) = parametros(&recebe_view(&mut s, "bia", 3, ViewMode::Hidden)).pop().unwrap();
    assert_eq!((p.active, som), (false, true));
    let (_, p, som) = parametros(&recebe_view(&mut s, "bia", 4, ViewMode::Off)).pop().unwrap();
    assert_eq!((p.active, som), (false, false));
}

#[test]
fn view_message_invalida_nunca_autoriza_midia() {
    let mut s = sala(&[("bia", "Bia", None)]);
    transmitir(&mut s, GoLive::default());
    // Canal fechado: não vale.
    assert!(recebe_view(&mut s, "bia", 1, ViewMode::Full).is_empty());
    conectar(&mut s, "bia");
    // Outra transmissão (começo diferente): não vale.
    let errada = ViewMessage { transmission: TransmissionKey { peer_id: ME.into(), started_at: 499 }, revision: 1, mode: ViewMode::Full };
    assert!(go(&mut s, Event::Peer(PeerEvent::View { id: "bia".into(), message: errada })).is_empty());
    let de_outra_pessoa = ViewMessage { transmission: TransmissionKey { peer_id: "bia".into(), started_at: 500 }, revision: 1, mode: ViewMode::Full };
    assert!(go(&mut s, Event::Peer(PeerEvent::View { id: "bia".into(), message: de_outra_pessoa })).is_empty());
    // Revisão fora do que o JavaScript conta com exatidão: não vale.
    assert!(recebe_view(&mut s, "bia", u64::MAX, ViewMode::Full).is_empty());
    assert_eq!(s.links["bia"].their_view, ViewMode::Off);
    // Pessoa desconhecida: ignorada.
    assert!(recebe_view(&mut s, "ninguem", 1, ViewMode::Full).is_empty());
    // Revisão velha ou repetida: ignorada.
    recebe_view(&mut s, "bia", 5, ViewMode::Full);
    assert!(recebe_view(&mut s, "bia", 5, ViewMode::Off).is_empty());
    assert!(recebe_view(&mut s, "bia", 4, ViewMode::Off).is_empty());
    assert_eq!(s.links["bia"].their_view, ViewMode::Full);
}

#[test]
fn view_message_antes_do_live_ack_nao_vale_e_o_ack_pede_a_view_de_novo() {
    let mut s = sala(&[("bia", "Bia", None)]);
    acao(&mut s, RoomAction::OpenGoLive);
    go(&mut s, Event::Capture(CaptureEvent::Sources(vec![janela()])));
    acao(&mut s, RoomAction::SelectSource { id: "w:1".into() });
    acao(&mut s, RoomAction::StartGoLive { prefs: GoLive::default() });
    go(&mut s, Event::Capture(CaptureEvent::Started { request_id: "req-1".into(), stream: stream("t"), audio_kind: AudioKind::Pc, audio_warn: None }));
    conectar(&mut s, "bia");
    // Sem chave confirmada, não há transmissão para autorizar.
    assert!(recebe_view(&mut s, "bia", 1, ViewMode::Full).is_empty());
    let fx = go(&mut s, Event::Server(ServerMessage::LiveAck { request_id: "req-1".into(), broadcast_seq: 0, live: live_info(500) }));
    assert!(fx.contains(&Effect::RequestView { to: "bia".into(), transmission: minha_chave() }));
    // E antes do ack, mesmo com a view cheia, o vídeo não sai (`active: false`).
    let (_, p, _) = parametros(&recebe_view(&mut s, "bia", 1, ViewMode::Full)).pop().unwrap();
    assert!(p.active);
}

#[test]
fn canal_de_controle_fechado_volta_a_nao_enviar_nada() {
    let mut s = sala(&[("bia", "Bia", None)]);
    transmitir(&mut s, GoLive::default());
    conectar(&mut s, "bia");
    recebe_view(&mut s, "bia", 1, ViewMode::Full);
    let fx = go(&mut s, Event::Peer(PeerEvent::ControlClosed { id: "bia".into() }));
    assert_eq!(s.links["bia"].their_view, ViewMode::Off);
    assert_eq!(s.links["bia"].grant_revision, -1);
    let (_, p, som) = parametros(&fx).pop().unwrap();
    assert_eq!((p.active, som), (false, false));
    // Reaberto, a revisão recomeça: uma view de revisão 1 volta a valer.
    go(&mut s, Event::Peer(PeerEvent::ControlOpen { id: "bia".into() }));
    assert!(!recebe_view(&mut s, "bia", 1, ViewMode::Full).is_empty());
}

#[test]
fn control_open_e_view_sync_dizem_de_novo_como_eu_vejo_a_tela_dela() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    go(&mut s, Event::Peer(PeerEvent::State { id: "bia".into(), conn: ConnState::Connected }));
    assistir(&mut s, "bia");
    // Canal ainda fechado: nada vai.
    assert!(views_enviadas(&go(&mut s, Event::Peer(PeerEvent::ViewSync { id: "bia".into(), transmission: TransmissionKey { peer_id: "bia".into(), started_at: 100 } }))).is_empty());
    let fx = go(&mut s, Event::Peer(PeerEvent::ControlOpen { id: "bia".into() }));
    assert_eq!(views_enviadas(&fx), vec![("bia".into(), ViewMode::Full, 1)]);
    // Ela pede de novo, pela transmissão certa: revisão sobe.
    let fx = go(&mut s, Event::Peer(PeerEvent::ViewSync { id: "bia".into(), transmission: TransmissionKey { peer_id: "bia".into(), started_at: 100 } }));
    assert_eq!(views_enviadas(&fx), vec![("bia".into(), ViewMode::Full, 2)]);
    // Pedido por outra transmissão: ignorado.
    let fx = go(&mut s, Event::Peer(PeerEvent::ViewSync { id: "bia".into(), transmission: TransmissionKey { peer_id: "bia".into(), started_at: 99 } }));
    assert!(fx.is_empty());
}

fn com_n_espectadores(n: usize, prefs: GoLive) -> (RoomState, Vec<String>) {
    let ids: Vec<String> = (1..=n).map(|i| format!("p{i}")).collect();
    let mut s = RoomState::new(init());
    let peers = ids.iter().map(|id| pessoa(id, id, None)).collect();
    go(&mut s, welcome(pessoa(ME, "Ana", None), peers));
    transmitir(&mut s, prefs);
    for id in &ids {
        conectar(&mut s, id);
    }
    (s, ids)
}

#[test]
fn qualidade_cai_com_o_numero_de_espectadores_em_tela_cheia() {
    let (mut s, ids) = com_n_espectadores(7, GoLive { fps: 60, ..GoLive::default() });
    for id in &ids[..4] {
        recebe_view(&mut s, id, 1, ViewMode::Full);
    }
    assert_eq!(s.quality().map(|q| (q.resolution, q.fps)), Some((1080, 60)));
    // 5 espectadores cheios: 720p60 (escolheu 60) e teto de 4 Mbps.
    let fx = recebe_view(&mut s, &ids[4], 1, ViewMode::Full);
    assert_eq!(qualidade_da_captura(&fx), vec![(720, 60)]);
    let q = s.quality().unwrap();
    assert_eq!((q.resolution, q.fps, q.max_bitrate), (720, 60, 4_000_000));
    assert_eq!(anuncios(&fx).len(), 1);
    assert_eq!((anuncios(&fx)[0].2, anuncios(&fx)[0].3), (720, 60));
    // Os parâmetros de todas as ligações são reaplicados (a que mandou a view entra duas vezes, como no site).
    let alvos: BTreeSet<String> = parametros(&fx).into_iter().map(|(to, _, _)| to).collect();
    assert_eq!(alvos, ids.iter().cloned().collect::<BTreeSet<_>>());
    assert!(view(&s, NOW).my_broadcast.unwrap().adjusted == Some(true));
    // Miniatura não conta: a sexta em miniatura mantém 720p60.
    recebe_view(&mut s, &ids[5], 1, ViewMode::Thumb);
    assert_eq!(s.quality().map(|q| (q.resolution, q.fps)), Some((720, 60)));
    // 7 em tela cheia: 720p30.
    recebe_view(&mut s, &ids[5], 2, ViewMode::Full);
    let fx = recebe_view(&mut s, &ids[6], 1, ViewMode::Full);
    assert_eq!(qualidade_da_captura(&fx), vec![(720, 30)]);
    assert_eq!((anuncios(&fx)[0].2, anuncios(&fx)[0].3), (720, 30));
    // Quem para de ver em tela cheia devolve a qualidade.
    let fx = recebe_view(&mut s, &ids[6], 2, ViewMode::Off);
    let fx2 = recebe_view(&mut s, &ids[5], 3, ViewMode::Off);
    let tudo: Vec<Effect> = fx.into_iter().chain(fx2).collect();
    assert_eq!(qualidade_da_captura(&tudo), vec![(720, 60)]);
}

#[test]
fn qualidade_de_5_a_6_com_30_fps_so_baixa_o_teto_de_bitrate_e_nao_reanuncia() {
    let (mut s, ids) = com_n_espectadores(5, GoLive::default());
    for id in &ids[..4] {
        recebe_view(&mut s, id, 1, ViewMode::Full);
    }
    let fx = recebe_view(&mut s, &ids[4], 1, ViewMode::Full);
    let q = s.quality().unwrap();
    assert_eq!((q.resolution, q.fps, q.max_bitrate), (1080, 30, 4_000_000));
    assert_eq!(qualidade_da_captura(&fx), vec![(1080, 30)]);
    assert!(anuncios(&fx).is_empty(), "resolução e fps iguais: a sala não é avisada de novo");
    assert!(!view(&s, NOW).my_broadcast.unwrap().adjusted.unwrap());
}

#[test]
fn espectadores_que_veem_o_video_contam_no_painel_e_so_a_tela_cheia_baixa_a_qualidade() {
    let (mut s, ids) = com_n_espectadores(3, GoLive::default());
    recebe_view(&mut s, &ids[0], 1, ViewMode::Full);
    recebe_view(&mut s, &ids[1], 1, ViewMode::Thumb);
    recebe_view(&mut s, &ids[2], 1, ViewMode::Hidden);
    assert_eq!(view(&s, NOW).my_broadcast.unwrap().viewers, 2);
    // Ligação que não está de pé não é espectador.
    go(&mut s, Event::Peer(PeerEvent::State { id: ids[0].clone(), conn: ConnState::Disconnected }));
    assert_eq!(view(&s, NOW).my_broadcast.unwrap().viewers, 1);
}

#[test]
fn governador_de_processador_baixa_um_degrau_depois_de_10_segundos_e_sobe_depois_de_60() {
    let (mut s, ids) = com_n_espectadores(2, GoLive { fps: 60, ..GoLive::default() });
    recebe_view(&mut s, &ids[0], 1, ViewMode::Full);
    recebe_view(&mut s, &ids[1], 1, ViewMode::Thumb);
    let limitado = |id: &str, cpu: bool| Event::Peer(PeerEvent::SendStats(vec![SendReading { id: id.into(), cpu_limited: cpu }]));
    for _ in 0..9 {
        assert!(go(&mut s, limitado(&ids[0], true)).is_empty());
    }
    assert_eq!(s.cpu_steps, 0);
    let fx = go(&mut s, limitado(&ids[0], true));
    assert_eq!(s.cpu_steps, 1);
    assert_eq!(qualidade_da_captura(&fx), vec![(1080, 30)]);
    assert_eq!((anuncios(&fx)[0].2, anuncios(&fx)[0].3), (1080, 30));
    assert!(view(&s, NOW).my_broadcast.unwrap().adjusted == Some(true));
    // Quem só vê em miniatura não conta para o governador.
    for _ in 0..30 {
        go(&mut s, limitado(&ids[1], true));
    }
    assert_eq!(s.cpu_steps, 1);
    // Mais 10 s no limite: 1080p30 -> 720p30.
    for _ in 0..10 {
        go(&mut s, limitado(&ids[0], true));
    }
    assert_eq!(s.cpu_steps, 2);
    assert_eq!(s.quality().map(|q| (q.resolution, q.fps)), Some((720, 30)));
    // Já no mínimo: não baixa mais.
    for _ in 0..30 {
        go(&mut s, limitado(&ids[0], true));
    }
    assert_eq!(s.cpu_steps, 2);
    // 60 s de folga sobem um degrau.
    for _ in 0..59 {
        go(&mut s, limitado(&ids[0], false));
    }
    assert_eq!(s.cpu_steps, 2);
    let fx = go(&mut s, limitado(&ids[0], false));
    assert_eq!(s.cpu_steps, 1);
    assert_eq!(qualidade_da_captura(&fx), vec![(1080, 30)]);
    // Começar outra transmissão zera os degraus.
    acao(&mut s, RoomAction::StopGoLive);
    transmitir(&mut s, GoLive { fps: 60, ..GoLive::default() });
    assert_eq!(s.cpu_steps, 0);
}

#[test]
fn leitura_de_envio_sem_transmitir_ou_sem_tela_cheia_nao_mexe_no_processador() {
    let mut s = sala(&[("bia", "Bia", None)]);
    conectar(&mut s, "bia");
    let antes = s.clone();
    for _ in 0..20 {
        assert!(go(&mut s, Event::Peer(PeerEvent::SendStats(vec![SendReading { id: "bia".into(), cpu_limited: true }]))).is_empty());
    }
    assert_eq!(s, antes);
}

// ---------------------------------------------------------------------------
// Som, volume e convite
// ---------------------------------------------------------------------------

#[test]
fn mudo_e_volume_com_gravacao_no_aparelho() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    // Começa mudo (autoplay) e no volume padrão.
    let v = view(&s, NOW);
    assert_eq!((v.streams[0].muted, v.streams[0].volume), (true, 0.68));
    acao(&mut s, RoomAction::ToggleMute { id: "bia".into() });
    let v = view(&s, NOW);
    assert_eq!((v.streams[0].muted, v.streams[0].volume), (false, 0.68));
    // Volume: grava no aparelho com o apelido como veio.
    let fx = acao(&mut s, RoomAction::SetVolume { id: "bia".into(), volume: 0.3 });
    assert_eq!(fx, vec![Effect::Save(DeviceChange::Volume { nickname: "Bia".into(), volume: 0.3 })]);
    assert_eq!(view(&s, NOW).streams[0].volume, 0.3);
    // Volume zero muda; mudo com volume zero ao desmutar volta ao padrão.
    acao(&mut s, RoomAction::SetVolume { id: "bia".into(), volume: 0.0 });
    assert!(view(&s, NOW).streams[0].muted);
    acao(&mut s, RoomAction::ToggleMute { id: "bia".into() });
    let v = view(&s, NOW);
    assert_eq!((v.streams[0].muted, v.streams[0].volume), (false, 0.68));
    // Pessoa que não está na sala: ajusta, mas não grava.
    let fx = acao(&mut s, RoomAction::SetVolume { id: "fantasma".into(), volume: 0.5 });
    assert!(fx.is_empty());
}

#[test]
fn volume_guardado_vale_pelo_apelido_em_minusculas() {
    let mut volumes = BTreeMap::new();
    volumes.insert("bia".to_string(), 0.4);
    let mut s = RoomState::new(RoomInit { volumes, ..init() });
    go(&mut s, welcome(pessoa(ME, "Ana", None), vec![pessoa("x1", "BIA", Some(1)), pessoa("x2", "Lu", Some(2))]));
    let v = view(&s, NOW);
    assert_eq!(v.streams[0].volume, 0.4);
    assert_eq!(v.streams[1].volume, 0.68);
    // O que a pessoa ajustou fica valendo para o mesmo apelido numa sala nova.
    acao(&mut s, RoomAction::SetVolume { id: "x2".into(), volume: 0.9 });
    assert_eq!(s.saved_volume("LU"), Some(0.9));
}

#[test]
fn copiar_convite_usa_o_endereco_oficial_e_o_aviso_some_sozinho() {
    let mut s = sala(&[]);
    let fx = acao(&mut s, RoomAction::CopyInvite);
    assert_eq!(
        fx,
        vec![
            Effect::CopyInvite("https://espia.diuliano.com/s/k7f-q2x".into()),
            Effect::Schedule { timer: TimerId::Toast("t1".into()), after_ms: 4_000 },
            Effect::Schedule { timer: TimerId::Copied, after_ms: 2_000 },
        ]
    );
    assert!(s.copied);
    assert_eq!(view(&s, NOW).invite_url, "https://espia.diuliano.com/s/k7f-q2x");
    assert!(view(&s, NOW).copied);
    assert_eq!(s.toasts[0].text, "Link copiado — é só mandar pra galera");
    assert_eq!(s.toasts[0].kind, ToastKind::Copied);
    go(&mut s, Event::Timer(TimerId::Copied));
    assert!(!s.copied);
    go(&mut s, Event::Timer(TimerId::Toast("t1".into())));
    assert!(s.toasts.is_empty());
    assert_eq!(s.invite_url(), "https://espia.diuliano.com/s/k7f-q2x");
}

#[test]
fn avisos_de_erro_nao_somem_sozinhos_e_fechar_cancela_o_temporizador() {
    let mut s = sala(&[]);
    go(&mut s, Event::Capture(CaptureEvent::BlackScreen));
    transmitir(&mut s, GoLive::default());
    go(&mut s, Event::Capture(CaptureEvent::BlackScreen));
    let erro = s.toasts.last().unwrap().id.clone();
    // Erro: sem temporizador, fica até ser fechado.
    let fx = acao(&mut s, RoomAction::DismissToast { id: erro.clone() });
    assert!(!cancelou(&fx, TimerId::Toast(erro)));
    // Aviso comum: fechar na mão cancela o temporizador.
    let comum = s.toasts.iter().find(|t| t.kind != ToastKind::Error).unwrap().id.clone();
    let fx = acao(&mut s, RoomAction::DismissToast { id: comum.clone() });
    assert!(cancelou(&fx, TimerId::Toast(comum)));
}

// ---------------------------------------------------------------------------
// Dono: renomear, apagar e tirar da sala
// ---------------------------------------------------------------------------

#[test]
fn renomear_sucesso_e_cada_falha() {
    let mut s = dono();
    acao(&mut s, RoomAction::ToggleOwnerMenu { open: true });
    assert!(s.owner_menu);
    acao(&mut s, RoomAction::OpenRename);
    assert!(!s.owner_menu);
    let d = s.name_dialog.clone().unwrap();
    assert_eq!((d.value.as_str(), d.busy, d.error), ("CS da galera", false, None));
    let fx = acao(&mut s, RoomAction::SubmitRename { name: "Novo nome".into() });
    assert_eq!(fx, vec![Effect::Api(ApiCall::Rename { code: "k7f-q2x".into(), key: "k".repeat(64), name: "Novo nome".into() })]);
    assert!(s.name_dialog.as_ref().unwrap().busy);
    for (reason, texto) in [
        (OwnerFailure::Forbidden, "Só quem criou a sala pode fazer isso."),
        (OwnerFailure::Gone, "Essa sala não existe mais."),
        (OwnerFailure::Name, "Esse nome não dá. Use letras, números e espaços."),
        (OwnerFailure::RateLimited, "Muitas mudanças daqui agora. Tenta de novo em um minuto."),
        (OwnerFailure::Network, "Não deu pra falar com o servidor. Confere a internet e tenta de novo."),
    ] {
        acao(&mut s, RoomAction::SubmitRename { name: "Novo nome".into() });
        go(&mut s, Event::Api(ApiEvent::Rename(Err(reason))));
        let d = s.name_dialog.clone().unwrap();
        assert_eq!((d.busy, d.error.as_deref(), d.value.as_str()), (false, Some(texto), "Novo nome"), "{reason:?}");
        assert_eq!(view(&s, NOW).name_dialog.unwrap().error.as_deref(), Some(texto));
    }
    // Sucesso: o modal fecha; o nome novo chega pelo `room-renamed`.
    acao(&mut s, RoomAction::SubmitRename { name: "Novo nome".into() });
    go(&mut s, Event::Api(ApiEvent::Rename(Ok(()))));
    assert!(s.name_dialog.is_none());
    assert_eq!(s.room_info, RoomInfo::Fixed { name: "CS da galera".into() });
    let fx = go(&mut s, Event::Server(ServerMessage::RoomRenamed { name: "Novo nome".into() }));
    assert_eq!(s.room_info, RoomInfo::Fixed { name: "Novo nome".into() });
    assert_eq!(fx, vec![Effect::Save(DeviceChange::RenameRoom { code: "k7f-q2x".into(), name: "Novo nome".into() })]);
}

#[test]
fn renomear_sem_ser_dono_ou_sem_modal_nao_chama_a_api() {
    let mut s = sala(&[]);
    assert!(acao(&mut s, RoomAction::OpenRename).is_empty());
    assert!(s.name_dialog.is_none(), "sala rápida não tem nome");
    let mut s = RoomState::new(init());
    go(&mut s, welcome_em(pessoa(ME, "Ana", None), vec![], Some(RoomInfo::Fixed { name: "X".into() }), None, vec![]));
    acao(&mut s, RoomAction::OpenRename);
    assert!(acao(&mut s, RoomAction::SubmitRename { name: "Y".into() }).is_empty(), "sem a chave de dono");
    let mut d = dono();
    assert!(acao(&mut d, RoomAction::SubmitRename { name: "Y".into() }).is_empty(), "sem o modal aberto");
    // Sala rápida: `room-renamed` não a transforma em fixa.
    let mut q = sala(&[]);
    go(&mut q, Event::Server(ServerMessage::RoomRenamed { name: "Z".into() }));
    assert_eq!(q.room_info, RoomInfo::Quick);
}

#[test]
fn apagar_cada_falha_volta_a_poder_tentar() {
    let mut s = dono();
    acao(&mut s, RoomAction::OpenDelete);
    let d = s.delete_dialog.clone().unwrap();
    assert_eq!((d.name.as_str(), d.people, d.busy), ("CS da galera", 1, false));
    for (reason, texto) in [
        (OwnerFailure::Forbidden, "Só quem criou a sala pode fazer isso."),
        (OwnerFailure::Gone, "Essa sala não existe mais."),
        (OwnerFailure::Name, "Esse nome não dá. Use letras, números e espaços."),
        (OwnerFailure::RateLimited, "Muitas mudanças daqui agora. Tenta de novo em um minuto."),
        (OwnerFailure::Network, "Não deu pra falar com o servidor. Confere a internet e tenta de novo."),
    ] {
        acao(&mut s, RoomAction::ConfirmDelete);
        assert!(s.deleting && s.delete_dialog.as_ref().unwrap().busy);
        let fx = go(&mut s, Event::Api(ApiEvent::Delete(Err(reason))));
        assert!(fx.is_empty());
        let d = s.delete_dialog.clone().unwrap();
        assert_eq!((d.busy, d.error.as_deref()), (false, Some(texto)), "{reason:?}");
        assert!(!s.deleting, "o fechamento 4410 que vier agora seria surpresa");
    }
    assert!(!s.left);
    // Sem a chave de dono, nem chama a API.
    let mut t = RoomState::new(init());
    go(&mut t, welcome_em(pessoa(ME, "Ana", None), vec![], Some(RoomInfo::Fixed { name: "X".into() }), None, vec![]));
    acao(&mut t, RoomAction::OpenDelete);
    assert!(acao(&mut t, RoomAction::ConfirmDelete).is_empty());
}

#[test]
fn tirar_da_sala_sucesso_cada_falha_e_saida_sozinha() {
    let mut s = dono();
    acao(&mut s, RoomAction::AskRemove { peer_id: "bia".into() });
    let d = s.remove_dialog.clone().unwrap();
    assert_eq!((d.peer_id.as_str(), d.name.as_str(), d.busy), ("bia", "Bia", false));
    assert_eq!(view(&s, NOW).remove_dialog.unwrap().name, "Bia");
    assert_eq!(view(&s, NOW).participants[1].removable, Some(true));
    let fx = acao(&mut s, RoomAction::ConfirmRemove);
    assert_eq!(fx, vec![Effect::Api(ApiCall::Remove { code: "k7f-q2x".into(), key: "k".repeat(64), peer_id: "bia".into() })]);
    // Ocupado: um segundo clique não manda outro pedido.
    assert!(acao(&mut s, RoomAction::ConfirmRemove).is_empty());
    for (reason, texto) in [
        (OwnerFailure::Forbidden, "Só quem criou a sala pode fazer isso."),
        (OwnerFailure::Gone, "Essa sala não existe mais."),
        (OwnerFailure::Name, "Esse nome não dá. Use letras, números e espaços."),
        (OwnerFailure::RateLimited, "Muitas mudanças daqui agora. Tenta de novo em um minuto."),
        (OwnerFailure::Network, "Não deu pra falar com o servidor. Confere a internet e tenta de novo."),
    ] {
        go(&mut s, Event::Api(ApiEvent::Remove { peer_id: "bia".into(), result: Err(reason) }));
        let d = s.remove_dialog.clone().unwrap();
        assert_eq!((d.busy, d.error.as_deref()), (false, Some(texto)), "{reason:?}");
        acao(&mut s, RoomAction::ConfirmRemove);
        assert!(s.remove_dialog.as_ref().unwrap().busy);
    }
    // Resposta de outra pessoa (o modal já é de outra): ignorada.
    let antes = s.clone();
    assert!(go(&mut s, Event::Api(ApiEvent::Remove { peer_id: "lu".into(), result: Ok(()) })).is_empty());
    assert_eq!(s, antes);
    // Sucesso: o modal fecha; a pessoa some pelo `peer-left`.
    go(&mut s, Event::Api(ApiEvent::Remove { peer_id: "bia".into(), result: Ok(()) }));
    assert!(s.remove_dialog.is_none());
    assert_eq!(s.peers.len(), 1);
    // Saiu sozinha enquanto o dono confirmava: não tem mais quem tirar.
    acao(&mut s, RoomAction::AskRemove { peer_id: "bia".into() });
    assert!(s.remove_dialog.is_some());
    go(&mut s, Event::Server(ServerMessage::PeerLeft { id: "bia".into() }));
    assert!(s.remove_dialog.is_none());
}

#[test]
fn tirar_da_sala_so_o_dono_de_sala_fixa_com_a_ligacao_de_pe() {
    let mut s = sala(&[("bia", "Bia", None)]);
    acao(&mut s, RoomAction::AskRemove { peer_id: "bia".into() });
    assert!(s.remove_dialog.is_none(), "sala rápida");
    assert_eq!(view(&s, NOW).participants[1].removable, Some(false));
    let mut n = RoomState::new(init());
    go(&mut n, welcome_em(pessoa(ME, "Ana", None), vec![pessoa("bia", "Bia", None)], Some(RoomInfo::Fixed { name: "X".into() }), None, vec![]));
    acao(&mut n, RoomAction::AskRemove { peer_id: "bia".into() });
    assert!(n.remove_dialog.is_none(), "sem a chave de dono");
    let mut d = dono();
    acao(&mut d, RoomAction::AskRemove { peer_id: "ninguem".into() });
    assert!(d.remove_dialog.is_none(), "pessoa que não está na sala");
    assert_eq!(view(&d, NOW).participants[0].removable, None, "eu mesmo nunca");
    acao(&mut d, RoomAction::AskRemove { peer_id: "bia".into() });
    acao(&mut d, RoomAction::CloseDialogs);
    assert!(d.remove_dialog.is_none() && d.name_dialog.is_none() && d.delete_dialog.is_none() && !d.owner_menu);
    // O 4403 que chega a quem foi tirado.
    let fx = go(&mut d, Event::Link(LinkEvent::Ended(EndReason::Removed)));
    assert!(fx.contains(&Effect::Navigate(Destination::Ended(EndReason::Removed))));
}

// ---------------------------------------------------------------------------
// Sinais de negociação, mensagens do servidor, seletor
// ---------------------------------------------------------------------------

#[test]
fn sinais_de_negociacao_so_passam_adiante() {
    let mut s = sala(&[("bia", "Bia", None)]);
    let desc = || crate::protocol::SignalData::Candidate { candidate: None };
    // Chega sinal de quem ainda não tem ligação: nasce uma ligação não iniciadora.
    let fx = go(&mut s, Event::Server(ServerMessage::Signal { from: "lu".into(), data: desc() }));
    assert_eq!(
        fx,
        vec![Effect::OpenPeer { id: "lu".into(), initiator: false, polite: true }, Effect::Signal { from: "lu".into(), data: desc() }]
    );
    assert!(s.links.contains_key("lu"));
    let fx = go(&mut s, Event::Server(ServerMessage::Signal { from: "lu".into(), data: desc() }));
    assert_eq!(fx, vec![Effect::Signal { from: "lu".into(), data: desc() }]);
    // Do motor para o servidor.
    let fx = go(&mut s, Event::Peer(PeerEvent::Signal { to: "bia".into(), data: desc() }));
    assert_eq!(fx, vec![Effect::Send(ClientMessage::Signal { to: "bia".into(), data: desc() })]);
    // Sem ligações abertas (antes do welcome), ignora.
    let mut v = RoomState::new(init());
    assert!(go(&mut v, Event::Server(ServerMessage::Signal { from: "lu".into(), data: desc() })).is_empty());
    assert!(go(&mut v, Event::Peer(PeerEvent::Signal { to: "bia".into(), data: desc() })).is_empty());
}

#[test]
fn servidores_ice_novos_so_valem_com_ligacoes_e_so_stun() {
    let ice = vec![
        IceServer { urls: Urls::Many(vec!["stun:a:1".into(), "turn:b:2".into()]), username: None, credential: None },
        IceServer { urls: Urls::One("turns:c:3".into()), username: Some("u".into()), credential: Some("c".into()) },
    ];
    let msg = || Event::Server(ServerMessage::IceServers { ice_servers: ice.clone(), ice_expires_at: 1 });
    let mut v = RoomState::new(init());
    assert!(go(&mut v, msg()).is_empty());
    let mut s = sala(&[]);
    let fx = go(&mut s, msg());
    assert_eq!(
        fx,
        vec![Effect::SetIceServers(vec![IceServer { urls: Urls::Many(vec!["stun:a:1".into()]), username: None, credential: None }])]
    );
}

#[test]
fn mensagens_que_a_sinalizacao_trata_nao_mudam_a_sala() {
    let mut s = sala(&[("bia", "Bia", None)]);
    let antes = s.clone();
    for m in [
        ServerMessage::Pong,
        ServerMessage::RoomNotFound,
        ServerMessage::RoomFull { capacity: 10 },
        ServerMessage::ResumeFailed,
        ServerMessage::RoomDeleted,
        ServerMessage::Removed,
    ] {
        assert!(go(&mut s, Event::Server(m)).is_empty());
    }
    assert_eq!(s, antes);
}

#[test]
fn seletor_lista_escolhe_e_lembra_a_ultima_fonte() {
    let mut volta = GoLive::default();
    volta.last_source_key = Some("hollow_knight.exe|Hollow Knight".into());
    let mut s = RoomState::new(RoomInit { go_live_prefs: volta, ..init() });
    go(&mut s, welcome(pessoa(ME, "Ana", None), vec![]));
    acao(&mut s, RoomAction::OpenGoLive);
    assert!(s.sources_loading);
    // Primeira lista: volta a última fonte, se ainda existir.
    go(&mut s, Event::Capture(CaptureEvent::Sources(vec![tela_inteira("m:1", "Tela 1"), janela()])));
    assert_eq!(s.selected_id.as_deref(), Some("w:1"));
    assert!(!s.sources_loading);
    // A escolhida fecha: some a marcação.
    go(&mut s, Event::Capture(CaptureEvent::Sources(vec![tela_inteira("m:1", "Tela 1")])));
    assert_eq!(s.selected_id, None);
    // Só se escolhe o que está na lista.
    acao(&mut s, RoomAction::SelectSource { id: "w:99".into() });
    assert_eq!(s.selected_id, None);
    acao(&mut s, RoomAction::SelectSource { id: "m:1".into() });
    assert_eq!(s.selected_id.as_deref(), Some("m:1"));
    let fx = acao(&mut s, RoomAction::CloseGoLive);
    assert!(fx.contains(&Effect::StopListingSources));
    // Lista que chega depois de fechar: ignorada.
    let antes = s.clone();
    go(&mut s, Event::Capture(CaptureEvent::Sources(vec![])));
    assert_eq!(s, antes);
    // Falha ao listar fecha a lista.
    let mut t = sala(&[]);
    acao(&mut t, RoomAction::OpenGoLive);
    let fx = go(&mut t, Event::Capture(CaptureEvent::SourcesFailed));
    assert!(fx.contains(&Effect::StopListingSources));
}

#[test]
fn transmitir_a_tela_inteira_usa_o_nome_do_monitor_so_com_mais_de_um() {
    let mut s = sala(&[]);
    acao(&mut s, RoomAction::OpenGoLive);
    go(&mut s, Event::Capture(CaptureEvent::Sources(vec![tela_inteira("m:1", "Tela 1"), tela_inteira("m:2", "Tela 2")])));
    acao(&mut s, RoomAction::SelectSource { id: "m:2".into() });
    acao(&mut s, RoomAction::StartGoLive { prefs: GoLive::default() });
    assert_eq!(s.pending_start.as_ref().unwrap().label.as_deref(), Some("Tela 2"));
    assert_eq!(s.pending_start.as_ref().unwrap().prefs.surface, Surface::Monitor);
    assert_eq!(s.pending_start.as_ref().unwrap().prefs.last_source_key.as_deref(), Some("m:Tela 2"));
}

// ---------------------------------------------------------------------------
// Eventos fora de hora, pureza e prévia parada
// ---------------------------------------------------------------------------

#[test]
fn evento_fora_de_hora_e_ignorado_sem_quebrar() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    conectar(&mut s, "bia");
    let antes = s.clone();
    let fora: Vec<Event> = vec![
        Event::Server(ServerMessage::PeerLeft { id: "ninguem".into() }),
        Event::Server(ServerMessage::PeerLive { id: "ninguem".into(), live: Some(live_info(1)) }),
        Event::Server(ServerMessage::LiveAck { request_id: "x".into(), broadcast_seq: 0, live: live_info(1) }),
        Event::Peer(PeerEvent::State { id: "ninguem".into(), conn: ConnState::Connected }),
        Event::Peer(PeerEvent::ControlOpen { id: "ninguem".into() }),
        Event::Peer(PeerEvent::ControlClosed { id: "ninguem".into() }),
        Event::Peer(PeerEvent::VideoFlow { id: "ninguem".into(), flowing: true }),
        Event::Peer(PeerEvent::Remote { id: "ninguem".into(), stream: Some(stream("x")), audio: None, flowing: true }),
        Event::Peer(PeerEvent::ViewSync { id: "ninguem".into(), transmission: minha_chave() }),
        Event::Timer(TimerId::FirstFrame("ninguem".into())),
        Event::Timer(TimerId::Toast("t99".into())),
        Event::Timer(TimerId::LinkDown),
        Event::Timer(TimerId::Copied),
        Event::Capture(CaptureEvent::Ended),
        Event::Capture(CaptureEvent::StartFailed(StartFailure::Other)),
        Event::Capture(CaptureEvent::AudioChanged { kind: AudioKind::None, warn: None }),
        Event::Capture(CaptureEvent::CameraPreview { id: "v".into(), stream: None, busy: false }),
        Event::Api(ApiEvent::Rename(Ok(()))),
        Event::Api(ApiEvent::Remove { peer_id: "bia".into(), result: Ok(()) }),
        Event::Link(LinkEvent::Back),
        Event::Window(WindowEvent::FloatingClosed),
        Event::Window(WindowEvent::FloatingOpened("ninguem".into())),
        Event::Action(RoomAction::Watch { id: "ninguem".into(), on: true }),
        Event::Action(RoomAction::Retry { id: "ninguem".into() }),
        Event::Action(RoomAction::ConfirmRemove),
        Event::Action(RoomAction::ConfirmDelete),
        Event::Action(RoomAction::SubmitRename { name: "x".into() }),
        Event::Action(RoomAction::SelectCamera { id: "x".into() }),
        Event::Action(RoomAction::ToastAction { id: "t99".into() }),
    ];
    for e in fora {
        let (depois, fx) = reduce(s.clone(), e.clone(), NOW);
        assert!(fx.is_empty(), "{e:?} gerou {fx:?}");
        assert_eq!(depois, antes, "{e:?} mudou o estado");
    }
}

#[test]
fn pureza_mesma_entrada_mesma_saida() {
    let mut s = sala(&[("bia", "Bia", Some(100)), ("lu", "Lu", None)]);
    conectar(&mut s, "bia");
    transmitir(&mut s, GoLive { fps: 60, ..GoLive::default() });
    let eventos = vec![
        Event::Action(RoomAction::Watch { id: "bia".into(), on: true }),
        Event::Server(ServerMessage::PeerLive { id: "lu".into(), live: Some(live_info(9)) }),
        Event::Peer(PeerEvent::View { id: "bia".into(), message: ViewMessage { transmission: minha_chave(), revision: 1, mode: ViewMode::Full } }),
        Event::Action(RoomAction::CopyInvite),
        Event::Capture(CaptureEvent::RequestId("r".into())),
        Event::Window(WindowEvent::Visible(false)),
    ];
    for e in eventos {
        let a = reduce(s.clone(), e.clone(), 10);
        let b = reduce(s.clone(), e.clone(), 10);
        assert_eq!(a, b, "{e:?}");
        s = a.0;
    }
    // O relógio só entra pelo parâmetro: o horário de início vem do `agora`.
    let (mut x, mut y) = (RoomState::new(init()), RoomState::new(init()));
    go_at(&mut x, welcome(pessoa(ME, "Ana", None), vec![]), 1);
    go_at(&mut y, welcome(pessoa(ME, "Ana", None), vec![]), 999);
    assert_eq!(x, y);
}

#[test]
fn previa_parada_so_transmitindo_e_com_a_janela_a_vista() {
    let mut s = sala(&[]);
    let fx = transmitir(&mut s, GoLive::default());
    assert!(fx.contains(&Effect::Snapshots { on: true }));
    let fx = go(&mut s, Event::Capture(CaptureEvent::Snapshot(crate::view_model::SnapshotId("q1".into()))));
    assert!(fx.is_empty());
    assert_eq!(view(&s, NOW).my_broadcast.unwrap().snapshot, Some(crate::view_model::SnapshotId("q1".into())));
    // Quadro novo solta o anterior.
    let fx = go(&mut s, Event::Capture(CaptureEvent::Snapshot(crate::view_model::SnapshotId("q2".into()))));
    assert_eq!(fx, vec![Effect::ReleaseSnapshot(crate::view_model::SnapshotId("q1".into()))]);
    // Janela escondida: desliga a prévia; quadro que ainda chega é solto na hora.
    let fx = go(&mut s, Event::Window(WindowEvent::Visible(false)));
    assert!(fx.contains(&Effect::Snapshots { on: false }));
    let fx = go(&mut s, Event::Capture(CaptureEvent::Snapshot(crate::view_model::SnapshotId("q3".into()))));
    assert_eq!(fx, vec![Effect::ReleaseSnapshot(crate::view_model::SnapshotId("q3".into()))]);
    let fx = go(&mut s, Event::Window(WindowEvent::Visible(true)));
    assert!(fx.contains(&Effect::Snapshots { on: true }));
    // Parou de transmitir: a prévia some e a sua tela sai do palco.
    acao(&mut s, RoomAction::ToggleSelfStage);
    assert!(s.self_on_stage);
    let fx = acao(&mut s, RoomAction::StopGoLive);
    assert!(fx.contains(&Effect::ReleaseSnapshot(crate::view_model::SnapshotId("q2".into()))));
    assert!(!s.self_on_stage);
    assert!(s.snapshot.is_none());
}

#[test]
fn sua_tela_no_palco_nunca_vai_para_ninguem_e_sai_com_outro_foco() {
    let mut s = sala(&[("bia", "Bia", Some(100))]);
    conectar(&mut s, "bia");
    assistir(&mut s, "bia");
    // Sem transmitir não há o que mostrar.
    acao(&mut s, RoomAction::ToggleSelfStage);
    assert!(!s.self_on_stage);
    transmitir(&mut s, GoLive::default());
    acao(&mut s, RoomAction::ToggleSelfStage);
    let v = view(&s, NOW);
    assert_eq!(v.streams.len(), 2);
    assert_eq!(v.focused_id.as_deref(), Some(ME));
    assert_eq!(v.streams[1].you, Some(true));
    assert_eq!(v.streams[1].name, "Você");
    // A minha tela não entra no mapa de modos; a da Bia fica em miniatura atrás.
    assert!(!s.sent_modes.contains_key(ME));
    assert_eq!(s.sent_modes["bia"], ViewMode::Thumb);
    // Esc: a sua sai do palco e o foco não muda para outro.
    acao(&mut s, RoomAction::Focus { id: None });
    assert!(!s.self_on_stage);
    assert_eq!(s.sent_modes["bia"], ViewMode::Full);
}

#[test]
fn camera_aba_consulta_permissao_so_ao_abrir_e_prévias_somem_ao_sair() {
    let mut s = sala(&[]);
    acao(&mut s, RoomAction::OpenGoLive);
    let fx = acao(&mut s, RoomAction::SetSurface { surface: Surface::Camera });
    assert_eq!(fx, vec![Effect::QueryCameraPermission]);
    assert!(s.cam_active);
    let fx = go(&mut s, Event::Capture(CaptureEvent::CameraPermission(CameraPermission::Granted)));
    assert_eq!(fx, vec![Effect::ListCameras]);
    let cam = |id: &str, label: &str| DeviceInfo { id: id.into(), group_id: String::new(), label: label.into() };
    let fx = go(&mut s, Event::Capture(CaptureEvent::CameraDevices { videos: vec![cam("a", "Cam A"), cam("b", "Cam B")], audios: vec![] }));
    assert_eq!(s.cam_video_id.as_deref(), Some("a"));
    assert_eq!(fx, vec![Effect::OpenCameraPreviews { ids: vec!["a".into(), "b".into()] }]);
    go(&mut s, Event::Capture(CaptureEvent::CameraPreview { id: "a".into(), stream: Some(stream("pa")), busy: false }));
    go(&mut s, Event::Capture(CaptureEvent::CameraPreview { id: "b".into(), stream: None, busy: true }));
    let v = view(&s, NOW).camera;
    assert_eq!(v.devices[0].preview, Some(stream("pa")));
    assert!(v.devices[1].busy);
    let fx = acao(&mut s, RoomAction::SetSurface { surface: Surface::Window });
    assert!(fx.contains(&Effect::LeaveCameraTab));
    assert!(fx.contains(&Effect::CloseStream(stream("pa"))));
    assert!(!s.cam_active);
    // Prévia que chega depois de sair da aba é fechada.
    let fx = go(&mut s, Event::Capture(CaptureEvent::CameraPreview { id: "a".into(), stream: Some(stream("tarde")), busy: false }));
    assert_eq!(fx, vec![Effect::CloseStream(stream("tarde"))]);
}
