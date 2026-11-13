//! Cenas da tela `room` do mock. Origem: `src/mocks/room.ts` (47 cenas portadas, contando as seis watch-* geradas por
//! .map). Todas as cenas não marcadas `device: 'mobile'`, com os mesmos `id`, `code` e `label` do site (a paridade é
//! testada na T033), mais uma só do app (`pickerloading`, "carregando" do seletor, que o site não tem).
//!
//! Como no site, o `RoomView` de cada cena é montado por uma função só (`cena`, o `buildRoomScene` do TS), que parte de
//! uma cena-base e muda o que a cena pede. Para acrescentar uma cena: ponha `id`, `code` e `label` em `CENAS` e, se ela
//! precisar de algo novo, um ramo em `bruta`. Os ajudantes (`pessoa`, `transmissao`, `fonte_*`, `camera_*`,
//! `aviso`) montam gente, transmissões, fontes do seletor e avisos.
//!
//! Miniaturas e ícones de janela são de mentira e embutidos no próprio texto (`mock:<tom>:<rótulo>` e
//! `mock:<letra>:<tom>`): a tela os desenha (listras com o rótulo, pastilha com a letra). Nada de captura de verdade.

use super::Scene;
use espia_core::device::{AudioRole, GoLive, Surface};
use espia_core::protocol::{AudioKind, LiveSource, Tone};
use espia_core::view_model::{
    CameraAudio, CameraDevice, CameraDeviceKind, CameraPermission, CameraView, CaptureSource, DeleteDialogView,
    LinkState, MyBroadcastView, NameDialogMode, NameDialogView, ParticipantView, PickerMode, PickerView,
    RemoveDialogView, RoomKind, RoomStatus, RoomView, ScreenSurface, SnapshotId, StreamId, StreamView,
    StreamWatchState, TileStatus, ToastKind, ToastView,
};

/// `MOCK_ROOM_ID` de `src/mocks/room.ts`.
pub const SALA: &str = "k7f-q2x";

/// Os seis estados de `watch-*` (o `.map` do TS).
const ESTADOS_ASSISTIR: [&str; 6] = ["available", "stopped", "opening", "playing", "unavailable", "reconnecting"];

/// `ROOM_SCENES` (sem os `watch-*`, que o `.map` gera): `(id, code, label)`, na ordem do site.
const CENAS: [(&str, &str, &str); 40] = [
    ("connecting", "E1", "Conectando"),
    ("empty", "D3", "Ninguém ao vivo"),
    ("copied", "D3", "Convite copiado"),
    ("modal", "A1", "Seletor · Janelas"),
    ("modalscreen", "A2", "Seletor · Telas inteiras"),
    ("modalmute", "A3", "Seletor · Sem áudio"),
    ("pickerwindows", "A1·app", "App · grade de janelas"),
    ("pickerscreens", "A2·app", "App · telas inteiras com som"),
    ("pickerempty", "A1·app", "App · nenhuma janela"),
    ("pickerfallback", "A1·app", "App · sem ponte (seletor do Windows)"),
    ("pickercamperm", "A5", "Câmeras · pedir permissão"),
    ("pickercams", "A5", "Câmeras · lista com prévias"),
    ("pickercammic", "A5", "Câmeras · placa com som de jogo"),
    ("pickercamnone", "A5", "Câmeras · nenhuma encontrada"),
    ("pickercamdenied", "A5", "Câmeras · bloqueada"),
    ("denied", "E3", "Permissão negada"),
    ("single", "D4", "Uma transmissão"),
    ("grid", "D5", "Várias (grade)"),
    ("focus", "D5b", "Foco + miniaturas"),
    ("streamfail", "E4", "Falha na transmissão"),
    ("live", "D7", "Você ao vivo"),
    ("livescreen", "D7", "Ao vivo · tela inteira sem som"),
    ("livepc", "D7·app", "Ao vivo · tela inteira com som do PC (app)"),
    ("livecam", "D7·cam", "Ao vivo · câmera (Lu também de câmera)"),
    ("crowd", "D5+", "Sala cheia (10, 6 ao vivo)"),
    ("reconnecting", "Q1", "Reconectando… (você ao vivo)"),
    ("back", "Q2", "De volta!"),
    ("reconnectfail", "Q3", "Não deu pra voltar"),
    ("fixed", "S·", "Sala fixa vazia (dono)"),
    ("created", "S1", "Sala fixa recém-criada"),
    ("ownermenu", "S2", "Menu do dono"),
    ("rename", "S3", "Renomear"),
    ("delete", "S4", "Apagar · com gente na sala"),
    ("guest", "S5", "Sala fixa (não é o dono)"),
    ("unwatched", "W1", "Parou de assistir uma tela"),
    ("unwatched-all", "W2", "Parou de assistir todas"),
    ("unwatched-focus", "W3", "Foco com miniatura sem assistir"),
    ("self-stage", "W4", "Sua tela grande no palco"),
    ("owner-remove", "R1", "Dono: remover da sala (passe o mouse na lista)"),
    ("remove-confirm", "R2", "Dono: confirmar \"Tirar da sala\""),
];

/// Cenas do seletor aberto (`PICKER_SCENES`) e as da aba Câmeras (`CAMERA_SCENES`).
const CENAS_DO_SELETOR: [&str; 13] = [
    "modal",
    "modalscreen",
    "modalmute",
    "pickerwindows",
    "pickerscreens",
    "pickerempty",
    "pickerfallback",
    "pickercamperm",
    "pickercams",
    "pickercammic",
    "pickercamnone",
    "pickercamdenied",
    "pickerloading",
];
const CENAS_DE_CAMERA: [&str; 5] = ["pickercamperm", "pickercams", "pickercammic", "pickercamnone", "pickercamdenied"];

// ---------------------------------------------------------------------------
// Ajudantes: gente, transmissões, avisos
// ---------------------------------------------------------------------------

/// Uma pessoa da lista, sem transmitir.
pub fn pessoa(id: &str, nome: &str, tone: Tone, voce: bool) -> ParticipantView {
    ParticipantView { id: id.into(), name: nome.into(), tone, you: voce, live: false, live_label: None, removable: None }
}

/// `PEOPLE` de `room.ts`: você (Rafa) primeiro.
pub fn pessoas() -> Vec<ParticipantView> {
    vec![
        pessoa("rafa", "Rafa", Tone::Deep, true),
        pessoa("lu", "Lu", Tone::Accent, false),
        pessoa("bia", "Bia", Tone::Sage, false),
        pessoa("tomas", "Tomás", Tone::Sand, false),
    ]
}

/// `CROWD` de `room.ts` (cena `crowd`): o resto do grupo, com os tons em rodízio como no servidor.
pub fn multidao() -> Vec<ParticipantView> {
    let tons = [Tone::Deep, Tone::Accent, Tone::Sage, Tone::Sand];
    ["Caio", "Duda", "Gabi", "Nina", "Theo", "Vini"]
        .iter()
        .enumerate()
        .map(|(i, nome)| pessoa(&nome.to_lowercase(), nome, tons[i % 4], false))
        .collect()
}

/// Uma transmissão de outra pessoa, assistindo, com o texto do placeholder (sem vídeo).
pub fn transmissao(id: &str, nome: &str, tone: Tone, qualidade: &str, legenda: &str, rotulo: Option<&str>) -> StreamView {
    StreamView {
        id: id.into(),
        name: nome.into(),
        tone,
        quality: Some(qualidade.into()),
        caption: Some(legenda.into()),
        status: TileStatus::Ok,
        muted: false,
        volume: 0.68,
        stream: None,
        audio_stream: None,
        label: rotulo.map(String::from),
        source: None,
        watching: None,
        watch_state: None,
        you: None,
    }
}

/// `STREAMS` de `room.ts`: Lu, Bia e Tomás (o Tomás ainda conectando).
fn transmissao_fixa(id: &str) -> Option<StreamView> {
    Some(match id {
        "lu" => transmissao("lu", "Lu", Tone::Accent, "1080p · 60 fps · com som", "tela de Lu · Hollow Knight", Some("Hollow Knight")),
        "bia" => StreamView {
            muted: true,
            ..transmissao("bia", "Bia", Tone::Sage, "720p · 30 fps · com som", "aba de Bia · YouTube", Some("Aba do navegador"))
        },
        "tomas" => StreamView {
            muted: true,
            status: TileStatus::Loading,
            ..transmissao("tomas", "Tomás", Tone::Sand, "1080p · 30 fps · sem som", "janela de Tomás · VS Code", Some("Visual Studio Code"))
        },
        _ => return None,
    })
}

/// `crowdStream` de `room.ts`: um texto comprido (cortado) e um de versão antiga (sem texto), specs/012.
fn transmissao_da_multidao(id: &str) -> StreamView {
    let gente = multidao();
    let p = gente.iter().find(|c| c.id == id).expect("pessoa da multidão");
    let rotulo = match id {
        "caio" => Some("Microsoft Flight Simulator 2024 — Edição Premium Deluxe"),
        "duda" => None,
        _ => Some("Tela inteira"),
    };
    StreamView {
        muted: true,
        ..transmissao(id, &p.name, p.tone, "1080p · 30 fps · sem som", &format!("tela de {}", p.name), rotulo)
    }
}

/// Um aviso com o id da cena (`scene-<id>`).
pub fn aviso(cena: &str, kind: ToastKind, texto: &str, acao: Option<&str>) -> ToastView {
    ToastView { id: format!("scene-{cena}"), kind, text: texto.into(), action: acao.map(String::from) }
}

/// `TOASTS` de `room.ts`: o aviso padrão de algumas cenas.
fn aviso_da_cena(id: &str) -> Option<ToastView> {
    Some(match id {
        "single" => aviso(id, ToastKind::Join, "Bia entrou na sala", None),
        "grid" => aviso(id, ToastKind::Live, "Tomás começou a transmitir", Some("Assistir")),
        "focus" => aviso(id, ToastKind::Leave, "Caio saiu da sala", None),
        "copied" => aviso(id, ToastKind::Copied, "Link copiado — é só mandar pra galera", None),
        "live" => aviso(id, ToastKind::Live, "Você está ao vivo pra sala", None),
        "livescreen" => aviso(
            id,
            ToastKind::Live,
            "Tela inteira vai sem som. Pra levar o som, transmita a janela do jogo ou uma aba.",
            None,
        ),
        "crowd" => aviso(id, ToastKind::Join, "Vini entrou na sala", None),
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Ajudantes: seletor
// ---------------------------------------------------------------------------

/// Uma janela do seletor. `icone` é `(letra, tom)` da pastilha; `miniatura` é `(tom, rótulo)` das listras.
pub fn fonte_janela(
    id: &str,
    titulo: &str,
    app: &str,
    icone: (&str, &str),
    miniatura: Option<(&str, &str)>,
    tocando: bool,
) -> CaptureSource {
    CaptureSource {
        id: id.into(),
        kind: ScreenSurface::Window,
        title: titulo.into(),
        app: app.into(),
        icon: Some(format!("mock:{}:{}", icone.0, icone.1)),
        thumb: miniatura.map(|(tom, rotulo)| format!("mock:{tom}:{rotulo}")),
        playing: tocando,
        primary: false,
    }
}

/// Uma tela inteira do seletor (nunca "tocando som", sem ícone).
pub fn fonte_tela(id: &str, titulo: &str, app: &str, tom: &str, principal: bool) -> CaptureSource {
    CaptureSource {
        id: id.into(),
        kind: ScreenSurface::Monitor,
        title: titulo.into(),
        app: app.into(),
        icon: None,
        thumb: Some(format!("mock:{tom}:tela")),
        playing: false,
        primary: principal,
    }
}

/// `MOCK_SOURCES` de `room.ts`: seis janelas (uma sem miniatura) e duas telas.
pub fn fontes() -> Vec<CaptureSource> {
    vec![
        fonte_janela("w:1", "Hollow Knight", "hollow_knight.exe", ("H", "accent"), Some(("accent", "janela")), true),
        fonte_janela("w:2", "YouTube — Firefox", "Firefox", ("Y", "sage"), Some(("sage", "janela")), true),
        fonte_janela("w:3", "espia-app — Visual Studio Code", "Code", ("e", "sand"), Some(("sand", "janela")), false),
        fonte_janela("w:4", "Spotify", "Spotify", ("S", "sage"), Some(("sage", "janela")), true),
        fonte_janela("w:5", "Planilha do campeonato", "LibreOffice Calc", ("P", "deep"), Some(("deep", "janela")), false),
        fonte_janela("w:6", "Downloads", "Explorador de arquivos", ("D", "sand"), None, false),
        fonte_tela("m:1", "Tela 1", "Principal", "sand", true),
        fonte_tela("m:2", "Tela 2", "", "accent", false),
    ]
}

fn seletor_do_app(fontes: Vec<CaptureSource>, escolhida: Option<&str>) -> PickerView {
    PickerView { mode: PickerMode::Native, sources: fontes, loading: false, selected_id: escolhida.map(String::from) }
}

/// `BROWSER_PICKER`: o seletor do navegador, sem lista.
fn seletor_do_navegador() -> PickerView {
    PickerView { mode: PickerMode::Browser, sources: Vec::new(), loading: false, selected_id: None }
}

fn seletor(cena: &str) -> PickerView {
    match cena {
        "pickerwindows" => seletor_do_app(fontes(), Some("w:1")),
        "pickerscreens" => seletor_do_app(fontes(), Some("m:1")),
        "pickerempty" => {
            seletor_do_app(fontes().into_iter().filter(|s| s.kind == ScreenSurface::Monitor).collect(), None)
        }
        // Só do app: ainda procurando a primeira lista.
        "pickerloading" => PickerView { loading: true, ..seletor_do_app(Vec::new(), None) },
        _ => seletor_do_navegador(),
    }
}

/// Uma câmera ou placa de captura com prévia (a prévia é de mentira: a tela desenha o rótulo).
pub fn camera_dispositivo(id: &str, nome: &str, placa: bool) -> CameraDevice {
    CameraDevice {
        id: id.into(),
        label: nome.into(),
        kind: if placa { CameraDeviceKind::Capture } else { CameraDeviceKind::Camera },
        preview: Some(StreamId(format!("mock:{id}"))),
        busy: false,
    }
}

/// `MOCK_CAMERA_DEVICES` de `room.ts`.
pub fn cameras() -> Vec<CameraDevice> {
    vec![
        camera_dispositivo("cam1", "HD Pro Webcam C920", false),
        camera_dispositivo("cam2", "Elgato HD60 X", true),
        camera_dispositivo("cam3", "OBS Virtual Camera", false),
    ]
}

/// `MOCK_AUDIO_INPUTS` de `room.ts`.
pub fn entradas_de_som() -> Vec<CameraAudio> {
    [
        ("mic1", "Microfone (HD Pro Webcam C920)"),
        ("mic2", "Digital Audio Interface (HD60 X)"),
        ("mic3", "Headset Microphone (HyperX Cloud II)"),
    ]
    .iter()
    .map(|(id, label)| CameraAudio { id: (*id).into(), label: (*label).into() })
    .collect()
}

/// `NO_CAMERA`: nenhuma permissão pedida ainda.
fn sem_camera() -> CameraView {
    CameraView {
        permission: CameraPermission::Unknown,
        devices: Vec::new(),
        audios: Vec::new(),
        video_id: None,
        audio_id: None,
        audio_role: AudioRole::Voice,
        error: None,
    }
}

fn camera_da_cena(cena: &str) -> CameraView {
    match cena {
        "pickercamperm" => CameraView { permission: CameraPermission::Prompt, ..sem_camera() },
        "pickercamdenied" => CameraView { permission: CameraPermission::Denied, ..sem_camera() },
        "pickercamnone" => {
            CameraView { permission: CameraPermission::Granted, audios: entradas_de_som(), ..sem_camera() }
        }
        "pickercams" => CameraView {
            permission: CameraPermission::Granted,
            devices: cameras(),
            audios: entradas_de_som(),
            video_id: Some("cam1".into()),
            audio_id: Some("mic1".into()),
            audio_role: AudioRole::Voice,
            error: None,
        },
        "pickercammic" => CameraView {
            permission: CameraPermission::Granted,
            devices: cameras(),
            audios: entradas_de_som(),
            video_id: Some("cam2".into()),
            audio_id: Some("mic2".into()),
            audio_role: AudioRole::Game,
            error: None,
        },
        _ => sem_camera(),
    }
}

// ---------------------------------------------------------------------------
// A cena
// ---------------------------------------------------------------------------

fn agora_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// `buildRoomScene`: a cena pronta; quem não disse se está assistindo, está.
pub fn cena(id: &str) -> RoomView {
    let mut v = bruta(id);
    for s in &mut v.streams {
        s.watching.get_or_insert(true);
    }
    v
}

/// `rawRoomScene`. Cada ramo parte de outra cena (como no TS) e muda só o que a cena pede.
fn bruta(id: &str) -> RoomView {
    if id == "update-required" {
        return RoomView { requires_update: Some(true), ..cena("empty") };
    }
    if let Some(estado) = id.strip_prefix("watch-") {
        let mut v = cena("grid");
        v.streams.truncate(2);
        v.toasts.clear();
        if estado == "reconnecting" {
            v.link = LinkState::Reconnecting;
        }
        for p in &mut v.participants {
            p.live = v.streams.iter().any(|s| s.id == p.id);
        }
        for s in &mut v.streams {
            s.watching = Some(matches!(estado, "opening" | "playing" | "unavailable" | "reconnecting"));
            let estado_de_assistir = match estado {
                "stopped" => StreamWatchState::Stopped,
                "opening" | "reconnecting" => StreamWatchState::Opening,
                "playing" => StreamWatchState::Playing,
                "unavailable" => StreamWatchState::Unavailable,
                _ => StreamWatchState::Available,
            };
            s.watch_state = Some(estado_de_assistir);
            s.status = match estado_de_assistir {
                StreamWatchState::Opening => TileStatus::Loading,
                StreamWatchState::Unavailable => TileStatus::Failed,
                _ => TileStatus::Ok,
            };
        }
        return v;
    }
    // Sala fixa: a mesma sala, com o nome no lugar do título.
    if matches!(id, "fixed" | "created" | "guest" | "ownermenu" | "rename" | "delete") {
        let base = match id {
            "guest" => "single",
            "created" => "copied",
            _ => "empty",
        };
        let mut v = cena(base);
        v.room = RoomKind::Fixed { name: "CS da galera".into(), own: id != "guest" };
        if id == "ownermenu" {
            v.owner_menu = Some(true);
        }
        if id == "rename" {
            v.name_dialog = Some(NameDialogView {
                mode: NameDialogMode::Rename,
                value: "CS da galera 2".into(),
                busy: false,
                error: None,
            });
        }
        if id == "delete" {
            v.delete_dialog =
                Some(DeleteDialogView { name: "CS da galera".into(), people: 3, busy: false, error: None });
        }
        if id == "created" {
            v.toasts = vec![aviso("created", ToastKind::Copied, "Sala criada — fixe o link no grupo", None)];
        }
        return v;
    }
    if matches!(id, "unwatched" | "unwatched-all" | "unwatched-focus") {
        let mut v = cena(if id == "unwatched-focus" { "focus" } else { "grid" });
        v.toasts.clear();
        let alvo = if id == "unwatched" { "bia" } else { "tomas" };
        for s in &mut v.streams {
            if id == "unwatched-all" || s.id == alvo {
                s.watching = Some(false);
            }
        }
        if id == "unwatched-focus" {
            if let Some(s) = v.streams.iter_mut().find(|s| s.id == "tomas") {
                s.status = TileStatus::Ok;
            }
        }
        return v;
    }
    if matches!(id, "owner-remove" | "remove-confirm") {
        let mut v = cena("single");
        v.toasts.clear();
        v.room = RoomKind::Fixed { name: "CS da galera".into(), own: true };
        for p in &mut v.participants {
            p.removable = Some(!p.you);
        }
        if id == "remove-confirm" {
            v.remove_dialog = Some(RemoveDialogView { name: "Tomás".into(), busy: false, error: None });
        }
        return v;
    }
    if id == "self-stage" {
        // Clicou na prévia: a sua tela grande, em foco; a de Lu vai pra faixa.
        let mut v = cena("live");
        v.toasts.clear();
        let rotulo = v.my_broadcast.as_ref().and_then(|b| b.label.clone());
        if let Some(b) = v.my_broadcast.as_mut() {
            b.on_stage = Some(true);
        }
        v.streams.push(StreamView {
            id: "rafa".into(),
            name: "Você".into(),
            tone: Tone::Deep,
            quality: None,
            caption: Some("sua tela · Celeste".into()),
            status: TileStatus::Ok,
            muted: true,
            volume: 0.0,
            stream: None,
            audio_stream: None,
            label: rotulo,
            source: None,
            watching: None,
            watch_state: None,
            you: Some(true),
        });
        v.focused_id = Some("rafa".into());
        return v;
    }
    // Queda (specs/011 Q1–Q3): a mesma sala de "Você ao vivo", com o aviso por cima.
    if matches!(id, "reconnecting" | "back" | "reconnectfail") {
        let mut v = cena("live");
        v.link = match id {
            "back" => LinkState::Ok,
            "reconnecting" => LinkState::Reconnecting,
            _ => LinkState::Failed,
        };
        v.toasts = if id == "back" { vec![aviso("back", ToastKind::Copied, "De volta!", None)] } else { Vec::new() };
        return v;
    }
    base(id)
}

/// O fim de `rawRoomScene`: as cenas que se montam do zero (palco, "você ao vivo", seletor, sala cheia...).
fn base(id: &str) -> RoomView {
    let ids: &[&str] = match id {
        "single" | "live" | "livescreen" | "livepc" | "livecam" => &["lu"],
        "grid" | "focus" => &["lu", "bia", "tomas"],
        "streamfail" => &["lu", "bia"],
        "crowd" => &["lu", "bia", "tomas", "caio", "duda", "gabi"],
        _ => &[],
    };
    let mut streams: Vec<StreamView> =
        ids.iter().map(|i| transmissao_fixa(i).unwrap_or_else(|| transmissao_da_multidao(i))).collect();
    if id == "crowd" {
        for s in &mut streams {
            s.quality = Some("720p · 30 fps · ajustado · sem som".into());
        }
    }
    let mut mexer = |alvo: &str, f: &dyn Fn(&mut StreamView)| {
        if let Some(s) = streams.iter_mut().find(|s| s.id == alvo) {
            f(s);
        }
    };
    // Na grade o Tomás ainda está conectando; no foco ele já carregou.
    if id == "focus" {
        mexer("tomas", &|s| s.status = TileStatus::Ok);
    }
    if id == "streamfail" {
        mexer("bia", &|s| s.status = TileStatus::Failed);
    }

    let eu_ao_vivo = matches!(id, "live" | "livescreen" | "livepc" | "livecam");
    if id == "livecam" {
        for s in &mut streams {
            s.source = Some(LiveSource::Camera);
            s.label = Some("Câmera · Logitech Brio".into());
        }
    }
    let meu_rotulo = match id {
        "live" => Some("Celeste"),
        "livecam" => Some("Câmera · HD Pro Webcam C920"),
        "livepc" | "livescreen" => Some("Tela inteira"),
        _ => None,
    };

    let mut gente = pessoas();
    if id == "crowd" {
        gente.extend(multidao());
    }
    for p in &mut gente {
        p.live = ids.contains(&p.id.as_str()) || (p.you && eu_ao_vivo);
        p.live_label = if p.you {
            if eu_ao_vivo { meu_rotulo.map(String::from) } else { None }
        } else {
            streams.iter().find(|s| s.id == p.id).and_then(|s| s.label.clone())
        };
    }

    let minha = eu_ao_vivo.then(|| {
        let mut prefs = GoLive { resolution: 1080, fps: 60, ..GoLive::default() };
        let (audio_kind, reason_monitor, source, titulo) = match id {
            "live" => (AudioKind::Window, false, None, None),
            "livecam" => {
                prefs.surface = Surface::Camera;
                (AudioKind::Device, false, Some(LiveSource::Camera), Some("HD Pro Webcam C920"))
            }
            "livepc" => {
                prefs.surface = Surface::Monitor;
                (AudioKind::Pc, false, None, Some("Tela 1"))
            }
            _ => {
                prefs.surface = Surface::Monitor;
                (AudioKind::None, true, None, None)
            }
        };
        MyBroadcastView {
            prefs,
            started_at: agora_ms().saturating_sub((12 * 60 + 4) * 1000),
            adjusted: None,
            viewers: 3,
            stream: None,
            audio_kind,
            audio_reason_monitor: reason_monitor,
            source_title: titulo.map(String::from),
            label: meu_rotulo.map(String::from),
            source,
            snapshot: Some(SnapshotId("mock:quadro".into())),
            on_stage: None,
        }
    });

    let superficie = if CENAS_DE_CAMERA.contains(&id) {
        Surface::Camera
    } else if matches!(id, "modalscreen" | "pickerscreens") {
        Surface::Monitor
    } else {
        Surface::Window
    };

    RoomView {
        requires_update: None,
        room_id: SALA.into(),
        invite_url: format!("https://espia.diuliano.com/s/{SALA}"),
        status: if id == "connecting" { RoomStatus::Connecting } else { RoomStatus::Ready },
        participants: gente,
        capacity: 10,
        streams,
        focused_id: matches!(id, "focus" | "streamfail").then(|| "bia".to_string()),
        my_broadcast: minha,
        copied: id == "copied",
        go_live_open: CENAS_DO_SELETOR.contains(&id),
        go_live_prefs: GoLive {
            resolution: 1080,
            fps: if id == "pickerwindows" { 60 } else { 30 },
            audio: id != "modalmute",
            surface: superficie,
            ..GoLive::default()
        },
        picker: seletor(id),
        camera: camera_da_cena(id),
        permission_denied: id == "denied",
        toasts: aviso_da_cena(id).into_iter().collect(),
        can_share: true,
        can_share_screen: true,
        link: LinkState::Ok,
        room: RoomKind::Quick,
        owner_menu: None,
        name_dialog: None,
        delete_dialog: None,
        remove_dialog: None,
    }
}

/// Todas as cenas da sala, na ordem do site (`update-required`, os seis `watch-*` e as demais), mais a do app.
pub fn scenes() -> Vec<Scene<RoomView>> {
    let mut lista = vec![Scene::new("update-required", "017", "Atualização necessária", cena("update-required"))];
    for estado in ESTADOS_ASSISTIR {
        let id = format!("watch-{estado}");
        let view = cena(&id);
        lista.push(Scene::new(id, "017", format!("Assistir: {estado}"), view));
    }
    for (id, code, label) in CENAS {
        lista.push(Scene::new(id, code, label, cena(id)));
    }
    // Só do app (o site não tem estado de carregando no seletor).
    lista.push(Scene::new("pickerloading", "A1·app", "App · procurando janelas", cena("pickerloading")));
    lista
}

#[cfg(test)]
mod tests {
    use super::*;
    use espia_core::room::{StageLayout, stage_layout};

    #[test]
    fn quarenta_e_sete_do_site_mais_uma_do_app() {
        let ids: Vec<_> = scenes().into_iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), 48);
        let mut unicos = ids.clone();
        unicos.sort();
        unicos.dedup();
        assert_eq!(unicos.len(), 48, "ids repetidos");
        assert_eq!(ids[0], "update-required");
        assert!(ids.contains(&"watch-reconnecting".to_string()));
        assert!(ids.contains(&"single".to_string()));
    }

    #[test]
    fn cena_padrao_e_uma_transmissao() {
        let v = cena("single");
        assert_eq!(stage_layout(&v), StageLayout::Single);
        assert_eq!(v.streams[0].id, "lu");
        assert_eq!(v.streams[0].watching, Some(true));
        assert_eq!(v.toasts.len(), 1);
    }

    #[test]
    fn grade_e_foco() {
        assert_eq!(stage_layout(&cena("grid")), StageLayout::Grid);
        let foco = cena("focus");
        assert_eq!(stage_layout(&foco), StageLayout::Focus);
        assert_eq!(foco.focused_id.as_deref(), Some("bia"));
        assert_eq!(foco.streams.iter().find(|s| s.id == "tomas").unwrap().status, TileStatus::Ok);
        assert_eq!(cena("grid").streams.iter().find(|s| s.id == "tomas").unwrap().status, TileStatus::Loading);
    }

    #[test]
    fn estados_de_assistir() {
        let parado = cena("watch-stopped");
        assert_eq!(parado.streams.len(), 2);
        assert!(parado.streams.iter().all(|s| s.watching == Some(false) && s.watch_state == Some(StreamWatchState::Stopped)));
        let abrindo = cena("watch-opening");
        assert!(abrindo.streams.iter().all(|s| s.watching == Some(true) && s.status == TileStatus::Loading));
        assert_eq!(cena("watch-unavailable").streams[0].status, TileStatus::Failed);
        let reconectando = cena("watch-reconnecting");
        assert_eq!(reconectando.link, LinkState::Reconnecting);
        assert_eq!(reconectando.streams[0].watch_state, Some(StreamWatchState::Opening));
        assert!(cena("watch-playing").participants.iter().filter(|p| p.live).count() == 2);
    }

    #[test]
    fn quedas_e_sala_fixa() {
        assert_eq!(cena("reconnecting").link, LinkState::Reconnecting);
        assert_eq!(cena("reconnectfail").link, LinkState::Failed);
        assert_eq!(cena("back").toasts[0].text, "De volta!");
        assert!(cena("reconnecting").my_broadcast.is_some());
        assert_eq!(cena("ownermenu").owner_menu, Some(true));
        assert_eq!(cena("guest").room, RoomKind::Fixed { name: "CS da galera".into(), own: false });
        assert!(cena("rename").name_dialog.is_some());
        assert_eq!(cena("delete").delete_dialog.unwrap().people, 3);
        assert_eq!(cena("remove-confirm").remove_dialog.unwrap().name, "Tomás");
        assert!(cena("owner-remove").participants.iter().filter(|p| p.removable == Some(true)).count() == 3);
        assert_eq!(cena("update-required").requires_update, Some(true));
    }

    #[test]
    fn seletor_abre_nas_cenas_do_seletor() {
        for id in ["modal", "pickerwindows", "pickercams", "pickerloading"] {
            assert!(cena(id).go_live_open, "{id}");
        }
        assert!(!cena("single").go_live_open);
        assert_eq!(cena("pickerscreens").go_live_prefs.surface, Surface::Monitor);
        assert_eq!(cena("pickercams").go_live_prefs.surface, Surface::Camera);
        assert!(!cena("modalmute").go_live_prefs.audio);
        assert_eq!(cena("pickerempty").picker.sources.len(), 2);
        assert!(cena("pickerloading").picker.loading);
        assert_eq!(cena("pickerfallback").picker.mode, PickerMode::Browser);
        assert_eq!(cena("pickercammic").camera.audio_role, AudioRole::Game);
        assert!(cena("denied").permission_denied);
    }

    #[test]
    fn sala_cheia_tem_dez_e_a_sua_tela_vai_pro_palco() {
        let c = cena("crowd");
        assert_eq!(c.participants.len(), 10);
        assert_eq!(c.streams.len(), 6);
        let s = cena("self-stage");
        assert_eq!(s.focused_id.as_deref(), Some("rafa"));
        assert_eq!(s.streams.last().unwrap().you, Some(true));
        assert_eq!(s.my_broadcast.unwrap().on_stage, Some(true));
    }
}
