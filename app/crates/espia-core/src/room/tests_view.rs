//! Testes das derivações puras da tela da sala e do `view()`.
//!
//! Portados de `test/lib/room-view.test.ts` (13 casos no TS, 10 portados: os 3 de `keepFullscreen`
//! são só de celular e ficam de fora) e de `test/lib/view-modes.test.ts` (13 casos no TS, 10 portados:
//! os 3 que usam `mobile` ficam de fora). Total: 26 casos no TS, 20 portados.

use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::device::GoLive;
use crate::media_policy::TransmissionKey;
use crate::protocol::{AudioKind, LiveInfo, LiveSource, Peer, RoomInfo, ServerMessage, Tone};
use crate::quality::{send_params, wants_sound, wants_video, Quality, ViewMode};
use crate::view_model::{
    LinkState, MyBroadcastView, RoomKind, RoomStatus, RoomView, StreamView, StreamWatchState, TileStatus,
};

// ---------------------------------------------------------------------------
// Montagem
// ---------------------------------------------------------------------------

fn lu() -> StreamView {
    StreamView {
        id: "lu".into(),
        name: "Lu".into(),
        tone: Tone::Accent,
        quality: Some("1080p · 60 fps · com som".into()),
        caption: None,
        status: TileStatus::Ok,
        muted: true,
        volume: 1.0,
        stream: None,
        audio_stream: None,
        label: None,
        source: None,
        watching: None,
        watch_state: None,
        you: None,
    }
}

fn bia() -> StreamView {
    StreamView { id: "bia".into(), name: "Bia".into(), tone: Tone::Sage, ..lu() }
}

/// `view({...})` do teste do site: sala rápida pronta, sem transmissões.
fn base() -> RoomView {
    let state = RoomState::new(RoomInit {
        code: "k7f-q2x".into(),
        go_live_prefs: GoLive::default(),
        owner_key: None,
        volumes: BTreeMap::new(),
        just_created: false,
    });
    RoomView { status: RoomStatus::Ready, ..view(&state, 0) }
}

fn com(streams: Vec<StreamView>) -> RoomView {
    RoomView { streams, ..base() }
}

fn fixa() -> RoomKind {
    RoomKind::Fixed { name: "CS da galera".into(), own: false }
}

fn transmitindo() -> Option<MyBroadcastView> {
    Some(MyBroadcastView {
        prefs: GoLive::default(),
        started_at: 0,
        adjusted: None,
        viewers: 0,
        stream: None,
        audio_kind: AudioKind::None,
        audio_reason_monitor: false,
        source_title: None,
        label: None,
        source: None,
        snapshot: None,
        on_stage: None,
    })
}

fn h(view: &RoomView) -> (String, String) {
    let h = header(view);
    (h.title, h.subtitle)
}

fn par(a: &str, b: &str) -> (String, String) {
    (a.to_string(), b.to_string())
}

// ---------------------------------------------------------------------------
// header (room-view.test.ts)
// ---------------------------------------------------------------------------

#[test]
fn header_sala_rapida_igual_a_antes() {
    assert_eq!(h(&base()), par("Sala da galera", "Ninguém transmitindo agora"));
}

#[test]
fn header_sala_fixa_o_nome_e_o_titulo() {
    let fixa_com = |streams, focused: Option<&str>| RoomView {
        room: fixa(),
        streams,
        focused_id: focused.map(String::from),
        ..base()
    };
    assert_eq!(h(&fixa_com(vec![], None)), par("CS da galera", "Ninguém transmitindo agora"));
    assert_eq!(h(&fixa_com(vec![lu()], None)).1, "Lu está ao vivo · 1080p · 60 fps · com som");
    assert_eq!(h(&fixa_com(vec![lu(), bia()], None)).1, "2 telas ao vivo · escolha qual assistir");
    assert_eq!(h(&fixa_com(vec![lu(), bia()], Some("bia"))).1, "Vendo Bia · 2 telas ao vivo");
}

#[test]
fn header_uma_transmissao_o_que_transmite_vem_antes_da_qualidade() {
    let hollow = StreamView { label: Some("Hollow Knight".into()), ..lu() };
    assert_eq!(h(&com(vec![hollow.clone()])).1, "Hollow Knight · 1080p · 60 fps · com som");
    assert_eq!(
        h(&RoomView { room: fixa(), ..com(vec![hollow]) }).1,
        "Lu está ao vivo · Hollow Knight · 1080p · 60 fps · com som"
    );
}

#[test]
fn header_sala_fixa_conectando_ainda_nao_sabe_o_nome() {
    let v = RoomView { room: fixa(), status: RoomStatus::Connecting, ..base() };
    assert_eq!(h(&v).1, "Conectando…");
    assert_eq!(h(&RoomView { status: RoomStatus::Connecting, ..base() }), par("Sala k7f-q2x", "Conectando…"));
}

#[test]
fn header_sua_tela_no_palco_so_a_sua() {
    let eu = StreamView { id: "eu".into(), name: "Você".into(), you: Some(true), ..lu() };
    assert_eq!(h(&com(vec![eu])), par("Sua tela, ao vivo", "Do jeito que a sala vê · Esc volta"));
}

#[test]
fn header_sua_tela_no_palco_com_outras_em_foco() {
    let eu = StreamView { id: "eu".into(), name: "Você".into(), you: Some(true), ..lu() };
    let v = RoomView { focused_id: Some("eu".into()), ..com(vec![lu(), eu]) };
    assert_eq!(h(&v), par("Sua tela, ao vivo", "1 tela da galera · Esc volta"));
    let eu = StreamView { id: "eu".into(), name: "Você".into(), you: Some(true), ..lu() };
    let v = RoomView { focused_id: Some("eu".into()), ..com(vec![lu(), bia(), eu]) };
    assert_eq!(h(&v).1, "2 telas da galera · Esc volta");
}

#[test]
fn header_sua_tela_no_palco_sala_fixa_o_nome_continua_no_titulo() {
    let eu = StreamView { id: "eu".into(), name: "Você".into(), you: Some(true), ..lu() };
    let v = RoomView { room: fixa(), ..com(vec![eu]) };
    assert_eq!(h(&v).0, "CS da galera");
    assert_eq!(h(&v).1, "Sua tela, ao vivo · do jeito que a sala vê · Esc volta");
}

#[test]
fn header_demais_casos_do_site() {
    // Vazia, transmitindo eu.
    let v = RoomView { my_broadcast: transmitindo(), ..base() };
    assert_eq!(h(&v), par("Você está ao vivo", "Ninguém mais transmitindo agora"));
    let v = RoomView { room: fixa(), my_broadcast: transmitindo(), ..base() };
    assert_eq!(h(&v).1, "Você está ao vivo · ninguém mais transmitindo");
    // Uma tela, eu também transmitindo.
    let v = RoomView { my_broadcast: transmitindo(), ..com(vec![lu()]) };
    assert_eq!(h(&v), par("Lu está ao vivo", "Você também está transmitindo"));
    let v = RoomView { room: fixa(), my_broadcast: transmitindo(), ..com(vec![lu()]) };
    assert_eq!(h(&v).1, "Lu está ao vivo · você também está transmitindo");
    // Grade e foco na sala rápida.
    assert_eq!(h(&com(vec![lu(), bia()])), par("2 telas ao vivo", "Escolha qual assistir"));
    let v = RoomView { focused_id: Some("bia".into()), ..com(vec![lu(), bia()]) };
    assert_eq!(h(&v), par("Vendo Bia", "2 telas ao vivo"));
}

// ---------------------------------------------------------------------------
// viewerStream (room-view.test.ts)
// ---------------------------------------------------------------------------

fn visualizador(link: LinkState) -> RoomView {
    let assistida = StreamView { watching: Some(true), ..lu() };
    let parada = StreamView { watching: Some(false), ..bia() };
    RoomView { link, ..com(vec![assistida, parada]) }
}

#[test]
fn viewer_stream_devolve_a_transmissao_aberta_que_estou_assistindo() {
    let v = visualizador(LinkState::Ok);
    assert_eq!(viewer_stream(&v, Some("lu")).map(|s| s.id.as_str()), Some("lu"));
}

#[test]
fn viewer_stream_fechado_id_que_nao_existe_ou_nao_assistida_nada() {
    let v = visualizador(LinkState::Ok);
    assert!(viewer_stream(&v, None).is_none());
    assert!(viewer_stream(&v, Some("tomas")).is_none());
    assert!(viewer_stream(&v, Some("bia")).is_none());
}

#[test]
fn viewer_stream_ligacao_em_failed_esconde_reconnecting_mantem() {
    assert!(viewer_stream(&visualizador(LinkState::Failed), Some("lu")).is_none());
    let v = visualizador(LinkState::Reconnecting);
    assert_eq!(viewer_stream(&v, Some("lu")).map(|s| s.id.as_str()), Some("lu"));
}

// ---------------------------------------------------------------------------
// viewModes (view-modes.test.ts)
// ---------------------------------------------------------------------------

fn entrada() -> ViewModesInput {
    ViewModesInput {
        streams: vec!["lu".into(), "bia".into()],
        layout: StageLayout::Grid,
        focused_id: None,
        watching: ["lu", "bia"].iter().map(|s| s.to_string()).collect(),
        visible: true,
        floating_id: None,
        self_id: None,
    }
}

fn conjunto(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

fn modos(pares: &[(&str, ViewMode)]) -> BTreeMap<String, ViewMode> {
    pares.iter().map(|(id, m)| (id.to_string(), *m)).collect()
}

#[test]
fn view_modes_sem_escolha_nao_recebe_midia_nem_em_destaque() {
    let i = ViewModesInput { watching: conjunto(&[]), focused_id: Some("lu".into()), ..entrada() };
    assert_eq!(view_modes(&i), modos(&[("lu", ViewMode::Off), ("bia", ViewMode::Off)]));
}

#[test]
fn view_modes_grade_somente_escolhas_explicitas() {
    assert_eq!(view_modes(&entrada()), modos(&[("lu", ViewMode::Full), ("bia", ViewMode::Full)]));
}

#[test]
fn view_modes_foco_focada_cheia_outras_miniatura() {
    let i = ViewModesInput { layout: StageLayout::Focus, focused_id: Some("bia".into()), ..entrada() };
    assert_eq!(view_modes(&i), modos(&[("lu", ViewMode::Thumb), ("bia", ViewMode::Full)]));
}

#[test]
fn view_modes_parou_de_assistir_sem_video_e_sem_som_mesmo_focada() {
    let i = ViewModesInput { watching: conjunto(&["bia"]), ..entrada() };
    assert_eq!(view_modes(&i), modos(&[("lu", ViewMode::Off), ("bia", ViewMode::Full)]));
    let i = ViewModesInput {
        layout: StageLayout::Focus,
        focused_id: Some("lu".into()),
        watching: conjunto(&["bia"]),
        ..entrada()
    };
    assert_eq!(view_modes(&i), modos(&[("lu", ViewMode::Off), ("bia", ViewMode::Thumb)]));
}

#[test]
fn view_modes_janela_escondida_sem_video_som_continua_menos_a_flutuante() {
    let i = ViewModesInput { visible: false, ..entrada() };
    assert_eq!(view_modes(&i), modos(&[("lu", ViewMode::Hidden), ("bia", ViewMode::Hidden)]));
    let i = ViewModesInput { visible: false, floating_id: Some("bia".into()), ..entrada() };
    assert_eq!(view_modes(&i), modos(&[("lu", ViewMode::Hidden), ("bia", ViewMode::Full)]));
}

#[test]
fn view_modes_nao_assistida_ganha_da_janela_flutuante() {
    let i = ViewModesInput { visible: false, floating_id: Some("bia".into()), watching: conjunto(&["lu"]), ..entrada() };
    assert_eq!(view_modes(&i), modos(&[("lu", ViewMode::Hidden), ("bia", ViewMode::Off)]));
}

#[test]
fn view_modes_a_minha_propria_tela_no_palco_nunca_entra_no_mapa() {
    let i = ViewModesInput {
        streams: vec!["lu".into(), "eu".into()],
        layout: StageLayout::Focus,
        focused_id: Some("eu".into()),
        self_id: Some("eu".into()),
        ..entrada()
    };
    assert_eq!(view_modes(&i), modos(&[("lu", ViewMode::Thumb)]));
}

// ---------------------------------------------------------------------------
// Mensagem de como estou vendo (view-modes.test.ts, segundo describe)
// ---------------------------------------------------------------------------

#[test]
fn mensagem_v2_identifica_a_transmissao_e_a_revisao_nao_aceita_v1() {
    let chave = TransmissionKey { peer_id: "0123456789abcdef".into(), started_at: 100 };
    for modo in [ViewMode::Full, ViewMode::Thumb, ViewMode::Hidden, ViewMode::Off] {
        let msg = ViewMessage { transmission: chave.clone(), revision: 1, mode: modo };
        let json = msg.to_json();
        assert_eq!(json["version"], 2);
        assert_eq!(json["t"], "view");
        assert_eq!(ViewMessage::decode(&json), Some(msg));
    }
    // Versão 1 (sem chave nem revisão) não vale.
    assert_eq!(ViewMessage::decode(&serde_json::json!({ "t": "view", "mode": "off" })), None);
    // `view-sync` com o mesmo cuidado.
    let sync = view_sync_json(&chave);
    assert_eq!(decode_view_sync(&sync), Some(chave));
    assert_eq!(decode_view_sync(&serde_json::json!({ "t": "view-sync", "version": 1 })), None);
}

#[test]
fn mensagem_estranha_e_ignorada() {
    use serde_json::json;
    assert_eq!(ViewMessage::decode(&json!({ "t": "view", "mode": "off" })), None);
    assert_eq!(ViewMessage::decode(&json!({ "t": "outra", "mode": "full" })), None);
    assert_eq!(ViewMessage::decode(&json!(null)), None);
    let boa = json!({ "t": "view", "version": 2, "transmission": { "peerId": "0123456789abcdef", "startedAt": 1 }, "revision": 1, "mode": "full" });
    assert!(ViewMessage::decode(&boa).is_some());
    // Cada campo inválido derruba a mensagem inteira.
    for (campo, valor) in [
        ("mode", json!("quase")),
        ("revision", json!(-1)),
        ("revision", json!(1.5)),
        ("revision", json!("1")),
        ("revision", json!(9_007_199_254_740_993_u64)),
        ("version", json!(3)),
        ("transmission", json!({ "peerId": "0123456789ABCDEF", "startedAt": 1 })),
        ("transmission", json!({ "peerId": "0123", "startedAt": 1 })),
        ("transmission", json!({ "peerId": "0123456789abcdef", "startedAt": -5 })),
        ("transmission", json!({ "startedAt": 1 })),
    ] {
        let mut m = boa.clone();
        m[campo] = valor.clone();
        assert_eq!(ViewMessage::decode(&m), None, "{campo}={valor}");
    }
}

#[test]
fn segundo_plano_sem_video_com_som_parou_de_assistir_sem_os_dois() {
    assert_eq!([wants_video(Some(ViewMode::Hidden)), wants_sound(Some(ViewMode::Hidden))], [false, true]);
    assert_eq!([wants_video(Some(ViewMode::Off)), wants_sound(Some(ViewMode::Off))], [false, false]);
    assert_eq!([wants_video(Some(ViewMode::Full)), wants_sound(Some(ViewMode::Full))], [true, true]);
    let q = Quality { resolution: 1080, fps: 60, max_bitrate: 6_000_000 };
    assert!(!send_params(&q, ViewMode::Off).active);
}

// ---------------------------------------------------------------------------
// stageLayout, gridSize, watchingId, formatElapsed
// ---------------------------------------------------------------------------

#[test]
fn layout_do_palco() {
    assert_eq!(stage_layout(&RoomView { status: RoomStatus::Connecting, ..base() }), StageLayout::Connecting);
    assert_eq!(stage_layout(&base()), StageLayout::Empty);
    assert_eq!(stage_layout(&com(vec![lu()])), StageLayout::Single);
    assert_eq!(stage_layout(&com(vec![lu(), bia()])), StageLayout::Grid);
    let focada = RoomView { focused_id: Some("bia".into()), ..com(vec![lu(), bia()]) };
    assert_eq!(stage_layout(&focada), StageLayout::Focus);
    // Foco em quem não existe não vale.
    let fantasma = RoomView { focused_id: Some("x".into()), ..com(vec![lu(), bia()]) };
    assert_eq!(stage_layout(&fantasma), StageLayout::Grid);
}

#[test]
fn tamanho_da_grade() {
    assert_eq!([0, 1, 2].map(grid_size), [(2, 1); 3]);
    assert_eq!([3, 4].map(grid_size), [(2, 2); 2]);
    assert_eq!([5, 6].map(grid_size), [(3, 2); 2]);
    assert_eq!([7, 9].map(grid_size), [(3, 3); 2]);
}

#[test]
fn quem_estou_vendo_em_destaque() {
    assert_eq!(watching_id(&com(vec![lu()])), Some("lu".into()));
    let parou = StreamView { watching: Some(false), ..lu() };
    assert_eq!(watching_id(&com(vec![parou])), None);
    let eu = StreamView { id: "eu".into(), you: Some(true), ..lu() };
    assert_eq!(watching_id(&com(vec![eu])), None);
    assert_eq!(watching_id(&com(vec![lu(), bia()])), None, "na grade ninguém está em destaque");
    let focada = RoomView { focused_id: Some("bia".into()), ..com(vec![lu(), bia()]) };
    assert_eq!(watching_id(&focada), Some("bia".into()));
}

#[test]
fn tempo_decorrido() {
    assert_eq!(format_elapsed(0), "00:00");
    assert_eq!(format_elapsed(724_000), "12:04");
    assert_eq!(format_elapsed(3_723_000), "1:02:03");
    assert_eq!(format_elapsed(999), "00:00");
}

// ---------------------------------------------------------------------------
// view()
// ---------------------------------------------------------------------------

fn pessoa(id: &str, nick: &str, started: Option<u64>) -> Peer {
    Peer {
        id: id.into(),
        nickname: nick.into(),
        tone: Tone::Sand,
        live: started.map(|started_at| LiveInfo {
            resolution: 720,
            fps: 30,
            audio: AudioKind::None,
            source: LiveSource::Camera,
            started_at,
            label: None,
        }),
    }
}

fn estado(outros: Vec<Peer>) -> RoomState {
    let mut s = RoomState::new(RoomInit {
        code: "k7f-q2x".into(),
        go_live_prefs: GoLive::default(),
        owner_key: None,
        volumes: BTreeMap::new(),
        just_created: false,
    });
    let (n, _) = reduce(
        s.clone(),
        Event::Server(ServerMessage::Welcome {
            media_protocol: 2,
            media_policy: crate::media_policy::DIRECT_ONLY,
            broadcast_order: None,
            you: pessoa("eu", "Ana", None),
            peers: outros,
            ice_servers: vec![],
            ice_expires_at: 0,
            resume_token: "x".into(),
            capacity: 10,
            room: Some(RoomInfo::Quick),
        }),
        0,
    );
    s = n;
    s
}

#[test]
fn view_lista_eu_primeiro_e_so_quem_transmite_vira_tela() {
    let s = estado(vec![pessoa("bia", "Bia", Some(100)), pessoa("lu", "Lu", None)]);
    let v = view(&s, 0);
    assert_eq!(v.participants.iter().map(|p| (p.name.as_str(), p.you, p.live)).collect::<Vec<_>>(), vec![("Ana", true, false), ("Bia", false, true), ("Lu", false, false)]);
    assert_eq!(v.streams.len(), 1);
    assert_eq!(v.streams[0].quality.as_deref(), Some("720p · 30 fps · sem som"));
    assert_eq!(v.streams[0].source, Some(LiveSource::Camera));
    assert_eq!(v.streams[0].watching, Some(false));
    assert_eq!(v.streams[0].watch_state, Some(StreamWatchState::Available));
    assert_eq!(v.streams[0].status, TileStatus::Loading);
    assert_eq!(v.capacity, 10);
    assert_eq!(v.invite_url, "https://espia.diuliano.com/s/k7f-q2x");
    assert_eq!(v.room, RoomKind::Quick);
    assert!(v.can_share && v.can_share_screen);
}

#[test]
fn view_foco_em_quem_nao_transmite_mais_nao_vale() {
    let mut s = estado(vec![pessoa("bia", "Bia", Some(100)), pessoa("lu", "Lu", Some(200))]);
    s.focused_id = Some("bia".into());
    assert_eq!(view(&s, 0).focused_id.as_deref(), Some("bia"));
    s.focused_id = Some("ninguem".into());
    assert_eq!(view(&s, 0).focused_id, None);
}
