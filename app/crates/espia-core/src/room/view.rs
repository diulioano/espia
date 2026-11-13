//! Do estado da sala para o que a tela mostra: `view()` (o `buildView` de `RoomStore`) e as
//! derivações puras de `src/lib/room-view.ts` (`stageLayout`, `header`, `viewModes`...).
//! Fora, por serem só de celular: `keepFullscreen` e o `mobile` de `viewModes`. `viewer_stream` entra com o
//! id da tela cheia por parâmetro (o `RoomView` do app não tem `fullscreenId`).

use std::collections::{BTreeMap, BTreeSet};

use crate::devices::is_capture_card;
use crate::protocol::{AudioKind, LiveInfo};
use crate::quality::{is_adjusted, wants_video, ViewMode};
use crate::view_model::{
    CameraAudio, CameraDevice, CameraDeviceKind, CameraView, LinkState, MyBroadcastView, ParticipantView, PickerMode,
    PickerView, RemoveDialogView, RoomKind, RoomStatus, RoomView, StreamView, StreamWatchState, TileStatus,
};

use super::state::{AudioPref, ConnState, RoomState, DEFAULT_VOLUME};

// ---------------------------------------------------------------------------
// Derivações puras (room-view.ts)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageLayout {
    Connecting,
    Empty,
    Single,
    Grid,
    Focus,
}

pub fn stage_layout(view: &RoomView) -> StageLayout {
    if view.status == RoomStatus::Connecting {
        return StageLayout::Connecting;
    }
    match view.streams.len() {
        0 => StageLayout::Empty,
        1 => StageLayout::Single,
        _ if focused_stream(view).is_some() => StageLayout::Focus,
        _ => StageLayout::Grid,
    }
}

/// Grade para n transmissões (máx. 9 = 10 pessoas menos você): 2×1, 2×2, 3×2, 3×3.
pub fn grid_size(n: usize) -> (u32, u32) {
    match n {
        0..=2 => (2, 1),
        3..=4 => (2, 2),
        5..=6 => (3, 2),
        _ => (3, 3),
    }
}

pub fn focused_stream(view: &RoomView) -> Option<&StreamView> {
    view.streams.iter().find(|s| Some(&s.id) == view.focused_id.as_ref())
}

/// Quem você está vendo em destaque (palco cheio ou foco).
pub fn watching_id(view: &RoomView) -> Option<String> {
    let shown = match stage_layout(view) {
        StageLayout::Single => view.streams.first(),
        StageLayout::Focus => focused_stream(view),
        _ => None,
    };
    // Parou de assistir (specs/014): não está vendo, mesmo em destaque.
    // A sua própria tela no palco (specs/014 US3) também não conta.
    shown.filter(|s| s.watching != Some(false) && s.you != Some(true)).map(|s| s.id.clone())
}

/// A transmissão aberta no visualizador em tela cheia (`viewerStream`, specs/018 R5): só enquanto ela
/// existe e estou assistindo; some com a ligação em `Failed`; `Reconnecting` mantém aberto.
pub fn viewer_stream<'a>(view: &'a RoomView, fullscreen_id: Option<&str>) -> Option<&'a StreamView> {
    let id = fullscreen_id?;
    if view.link == LinkState::Failed {
        return None;
    }
    view.streams.iter().find(|s| s.id == id).filter(|s| s.watching == Some(true))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub title: String,
    pub subtitle: String,
}

fn head(title: impl Into<String>, subtitle: impl Into<String>) -> Header {
    Header { title: title.into(), subtitle: subtitle.into() }
}

pub fn header(view: &RoomView) -> Header {
    let n = view.streams.len();
    let layout = stage_layout(view);
    if layout == StageLayout::Connecting {
        return head(format!("Sala {}", view.room_id), "Conectando…");
    }
    // A sua tela no palco (specs/014 US3).
    let shown = match layout {
        StageLayout::Single => view.streams.first(),
        StageLayout::Focus => focused_stream(view),
        _ => None,
    };
    if shown.is_some_and(|s| s.you == Some(true)) {
        let others = n - 1;
        let sub = if others > 0 {
            format!("{others} {} da galera · Esc volta", if others == 1 { "tela" } else { "telas" })
        } else {
            "do jeito que a sala vê · Esc volta".to_string()
        };
        if let RoomKind::Fixed { name, .. } = &view.room {
            return head(name.clone(), format!("Sua tela, ao vivo · {sub}"));
        }
        return head("Sua tela, ao vivo", capitalize(&sub));
    }
    // Sala fixa (specs/011): o nome é o título; o subtítulo resume o que está acontecendo.
    if let RoomKind::Fixed { name, .. } = &view.room {
        return head(name.clone(), fixed_subtitle(view, layout, n));
    }
    match layout {
        StageLayout::Empty => {
            if view.my_broadcast.is_some() {
                head("Você está ao vivo", "Ninguém mais transmitindo agora")
            } else {
                head("Sala da galera", "Ninguém transmitindo agora")
            }
        }
        StageLayout::Single => {
            let s = &view.streams[0];
            head(
                format!("{} está ao vivo", s.name),
                if view.my_broadcast.is_some() { "Você também está transmitindo".to_string() } else { stream_line(s) },
            )
        }
        StageLayout::Grid => head(format!("{n} telas ao vivo"), "Escolha qual assistir"),
        _ => head(format!("Vendo {}", focused_stream(view).map_or("", |s| s.name.as_str())), format!("{n} telas ao vivo")),
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// "Hollow Knight · 1080p · 60 fps · com som" (o que está sendo transmitido vem primeiro, specs/012).
fn stream_line(s: &StreamView) -> String {
    [s.label.as_deref(), s.quality.as_deref()]
        .into_iter()
        .flatten()
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

fn fixed_subtitle(view: &RoomView, layout: StageLayout, n: usize) -> String {
    let mine = view.my_broadcast.is_some();
    match layout {
        StageLayout::Empty => {
            if mine { "Você está ao vivo · ninguém mais transmitindo" } else { "Ninguém transmitindo agora" }.to_string()
        }
        StageLayout::Single => {
            let s = &view.streams[0];
            format!(
                "{} está ao vivo · {}",
                s.name,
                if mine { "você também está transmitindo".to_string() } else { stream_line(s) }
            )
        }
        StageLayout::Grid => format!("{n} telas ao vivo · escolha qual assistir"),
        _ => format!("Vendo {} · {n} telas ao vivo", focused_stream(view).map_or("", |s| s.name.as_str())),
    }
}

/// "12:04" ou "1:02:03".
pub fn format_elapsed(ms: u64) -> String {
    let total = ms / 1000;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m:02}:{s:02}") }
}

/// Entrada de `view_modes` (`ViewModesInput` sem `mobile`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewModesInput {
    /// Ids das telas na sala, na ordem da tela.
    pub streams: Vec<String>,
    pub layout: StageLayout,
    pub focused_id: Option<String>,
    /// Quem escolhi assistir nesta transmissão (specs/017).
    pub watching: BTreeSet<String>,
    /// A janela está à vista (specs/014 US2): minimizada ou na bandeja = false.
    pub visible: bool,
    /// Tela na janela flutuante: continua chegando com a janela escondida.
    pub floating_id: Option<String>,
    /// A minha própria tela no palco (specs/014 US3): nunca vai para ninguém.
    pub self_id: Option<String>,
}

/// Como estou vendo a tela de cada pessoa (specs/014 research R5). Sem escolha explícita: `Off`.
/// `Hidden` (segundo plano) pausa só o vídeo; `Off` (parou de assistir) pausa vídeo e som.
/// É a regra do Assistir (FR-021): abrir, fechar, aproximar ou destacar uma tela nunca mexe em
/// `watching`; só muda o modo entre `Full` e `Thumb`, e só para quem já está sendo assistido.
pub fn view_modes(i: &ViewModesInput) -> BTreeMap<String, ViewMode> {
    let mut out = BTreeMap::new();
    for id in i.streams.iter().filter(|id| Some(*id) != i.self_id.as_ref()) {
        let mode = if !i.watching.contains(id) {
            // Parou de assistir (ou nunca escolheu): nem vídeo nem som.
            ViewMode::Off
        } else if !i.visible && Some(id) != i.floating_id.as_ref() {
            // Segundo plano: sem vídeo, o som continua.
            ViewMode::Hidden
        } else if i.layout == StageLayout::Focus {
            if Some(id) == i.focused_id.as_ref() { ViewMode::Full } else { ViewMode::Thumb }
        } else {
            ViewMode::Full
        };
        out.insert(id.clone(), mode);
    }
    out
}

// ---------------------------------------------------------------------------
// view()
// ---------------------------------------------------------------------------

/// "1080p · 60 fps · com som".
fn quality_text(live: &LiveInfo) -> String {
    format!(
        "{}p · {} fps · {}",
        live.resolution,
        live.fps,
        if live.audio == AudioKind::None { "sem som" } else { "com som" }
    )
}

/// O `RoomView` que a tela mostra. `_now_ms` fica na assinatura por simetria com `reduce`: nada da
/// tela depende do relógio (o tempo da transmissão a tela conta com `started_at`).
pub fn view(state: &RoomState, _now_ms: u64) -> RoomView {
    let me = state.me.as_ref();
    let fixed = matches!(state.room_info, crate::protocol::RoomInfo::Fixed { .. });
    // Só o dono de uma sala fixa tira alguém (specs/014 US4), e só com a ligação de pé.
    let can_remove = fixed && state.owner_key.is_some() && state.link == LinkState::Ok;
    let quality = state.quality();
    let their_views = state.their_views();

    let mut participants: Vec<ParticipantView> = Vec::with_capacity(state.peers.len() + 1);
    if let Some(me) = me {
        participants.push(ParticipantView {
            id: me.id.clone(),
            name: me.nickname.clone(),
            tone: me.tone,
            you: true,
            live: state.my_broadcast.is_some(),
            live_label: state.my_broadcast.as_ref().and_then(|b| b.label.clone()),
            removable: None,
        });
    }
    for p in &state.peers {
        participants.push(ParticipantView {
            id: p.id.clone(),
            name: p.nickname.clone(),
            tone: p.tone,
            you: false,
            live: p.live.is_some(),
            live_label: p.live.as_ref().and_then(|l| l.label.clone()),
            removable: Some(can_remove),
        });
    }

    let mut streams: Vec<StreamView> = Vec::new();
    for p in &state.peers {
        let Some(live) = &p.live else { continue };
        let pref = state.audio.get(&p.id).copied().unwrap_or_else(|| AudioPref {
            muted: true,
            volume: state.saved_volume(&p.nickname).unwrap_or(DEFAULT_VOLUME),
        });
        let media = state.media.get(&p.id);
        let flowing = media.is_some_and(|m| m.video && m.stream.is_some());
        let failed = media.is_some_and(|m| m.conn == ConnState::Failed) || state.timed_out.contains(&p.id);
        let choice = state.watching.get(&p.id);
        let watch_state = if choice.is_some() {
            if failed {
                StreamWatchState::Unavailable
            } else if flowing {
                StreamWatchState::Playing
            } else {
                StreamWatchState::Opening
            }
        } else if state.stopped.get(&p.id) == Some(&live.started_at) {
            StreamWatchState::Stopped
        } else {
            StreamWatchState::Available
        };
        streams.push(StreamView {
            id: p.id.clone(),
            name: p.nickname.clone(),
            tone: p.tone,
            quality: Some(quality_text(live)),
            caption: None,
            status: if flowing {
                TileStatus::Ok
            } else if failed {
                TileStatus::Failed
            } else {
                TileStatus::Loading
            },
            muted: pref.muted,
            volume: pref.volume,
            stream: if flowing { media.and_then(|m| m.stream.clone()) } else { None },
            audio_stream: media.and_then(|m| m.audio.clone()),
            label: live.label.clone(),
            source: Some(live.source),
            watching: Some(choice.is_some_and(|c| c.transmission.started_at == live.started_at)),
            watch_state: Some(watch_state),
            you: None,
        });
    }

    // A sua tela no palco (specs/014 US3): uma tela a mais, sempre muda, fora do que vai para a sala.
    let on_stage = match (&state.my_broadcast, me) {
        (Some(b), Some(me)) if state.self_on_stage && b.stream.is_some() => Some((b, me)),
        _ => None,
    };
    if let Some((b, me)) = on_stage {
        streams.push(StreamView {
            id: me.id.clone(),
            name: "Você".to_string(),
            tone: me.tone,
            quality: None,
            caption: None,
            status: TileStatus::Ok,
            muted: true,
            volume: 0.0,
            stream: b.stream.clone(),
            audio_stream: None,
            label: b.label.clone(),
            source: b.source,
            watching: Some(true),
            watch_state: Some(StreamWatchState::Playing),
            you: Some(true),
        });
    }
    let focused_id = match on_stage {
        Some((_, me)) => Some(me.id.clone()),
        None => state.focused_id.clone().filter(|f| streams.iter().any(|s| &s.id == f)),
    };

    let my_broadcast = state.my_broadcast.as_ref().map(|b| {
        let mut prefs = b.prefs.clone();
        // Mostra a qualidade efetiva, não só a escolhida.
        if let Some(q) = &quality {
            prefs.resolution = q.resolution;
            prefs.fps = q.fps;
        }
        MyBroadcastView {
            adjusted: Some(quality.as_ref().is_some_and(|q| is_adjusted(&b.prefs, q))),
            prefs,
            started_at: b.started_at,
            // Quem está recebendo o vídeo de verdade (specs/014): parar de assistir ou esconder a janela não conta.
            viewers: their_views.values().filter(|m| wants_video(Some(**m))).count() as u32,
            stream: b.stream.clone(),
            audio_kind: b.audio_kind,
            audio_reason_monitor: b.audio_reason_monitor,
            source_title: b.source_title.clone(),
            label: b.label.clone(),
            source: b.source,
            snapshot: state.snapshot.clone(),
            on_stage: Some(state.self_on_stage),
        }
    });

    let room = match &state.room_info {
        crate::protocol::RoomInfo::Fixed { name } => RoomKind::Fixed { name: name.clone(), own: state.owner_key.is_some() },
        crate::protocol::RoomInfo::Quick => RoomKind::Quick,
    };

    RoomView {
        requires_update: None,
        room_id: state.code.clone(),
        invite_url: state.invite_url(),
        status: if state.connected { RoomStatus::Ready } else { RoomStatus::Connecting },
        participants,
        capacity: state.capacity,
        streams,
        focused_id,
        my_broadcast,
        copied: state.copied,
        go_live_open: state.go_live_open,
        go_live_prefs: state.go_live_prefs.clone(),
        picker: PickerView {
            mode: PickerMode::Native,
            sources: state.sources.clone(),
            loading: state.sources_loading,
            selected_id: state.selected_id.clone(),
        },
        camera: CameraView {
            permission: state.cam_permission,
            devices: state
                .cam_videos
                .iter()
                .map(|v| CameraDevice {
                    id: v.id.clone(),
                    label: v.label.clone(),
                    kind: if is_capture_card(&v.label) { CameraDeviceKind::Capture } else { CameraDeviceKind::Camera },
                    preview: state.cam_previews.get(&v.id).cloned(),
                    busy: state.cam_busy.contains(&v.id),
                })
                .collect(),
            audios: state.cam_audios.iter().map(|a| CameraAudio { id: a.id.clone(), label: a.label.clone() }).collect(),
            video_id: state.cam_video_id.clone(),
            audio_id: state.cam_audio_id.clone(),
            audio_role: state.cam_audio_role,
            error: state.cam_error,
        },
        permission_denied: state.permission_denied,
        toasts: state.toasts.clone(),
        // No app, transmitir tela e câmera sempre existem.
        can_share: true,
        can_share_screen: true,
        link: state.link,
        room,
        owner_menu: Some(state.owner_menu),
        name_dialog: state.name_dialog.clone(),
        delete_dialog: state.delete_dialog.clone(),
        remove_dialog: state
            .remove_dialog
            .as_ref()
            .map(|d| RemoveDialogView { name: d.name.clone(), busy: d.busy, error: d.error.clone() }),
    }
}
