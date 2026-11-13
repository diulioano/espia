//! Liga `RoomView` à tela `ui/pages/room.slint` (estado em `ui/components/room-state.slint`). Os textos derivados
//! (subtítulo, "você está assistindo", estatísticas da sua transmissão, dicas do seletor) e as regras de layout do
//! palco (`stage_layout`, `grid_size`) vêm de `espia-core`; aqui só se montam as frases que são da tela.
//!
//! Ponto de entrada do vídeo: `set_frame` guarda o último quadro RGBA de cada transmissão (por id) e o põe no tile.
//! Os quadros ficam numa tabela da thread da interface, para um `apply` novo não os perder.

use crate::{
    AppWindow, AudioOption, CamError, CamPermission, Emit, GoLiveData, LinkMode, PersonData, RoomState, SourceData,
    SourceKind, StageMode, SurfaceTab, TileData, TileStatus, ToastData, ToastKind, UiAction,
    bind::{home, initial, tone},
};
use espia_core::audio_source::{CaptureSurface, NoSoundReason, audio_label_text, native_audio_text};
use espia_core::device::{AudioRole, GoLive, ROOM_NAME_MAX, Surface};
use espia_core::protocol::{LiveSource, Tone};
use espia_core::room::{StageLayout, focused_stream, format_elapsed, grid_size, header, stage_layout, watching_id};
use espia_core::room_name::normalize_room_name;
use espia_core::view_model as vm;
use espia_core::view_model::{ParticipantView, RoomAction, RoomView, StreamView};
use slint::{ComponentHandle, Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

thread_local! {
    /// Último quadro de cada transmissão (id da transmissão; para a prévia da barra, o id da própria pessoa).
    static QUADROS: RefCell<HashMap<String, Image>> = RefCell::new(HashMap::new());
    /// Preferências do seletor da última vista: `StartGoLive` as devolve com a aba, a qualidade e o som escolhidos.
    static PREFS: RefCell<GoLive> = RefCell::new(GoLive::default());
    /// Relógio do "Você está ao vivo há 12:04".
    static RELOGIO: slint::Timer = slint::Timer::default();
}

fn modelo<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(v))
}

// ---------------------------------------------------------------------------
// Textos
// ---------------------------------------------------------------------------

/// O convite sem "https://" (`inviteLabel`).
pub(crate) fn invite_label(url: &str) -> String {
    url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or(url).to_string()
}

/// Linha de baixo de "Na sala" (`sub` de `Participant.svelte`).
pub(crate) fn participant_sub(p: &ParticipantView, watching: bool) -> String {
    if watching {
        "você está assistindo".into()
    } else if p.live {
        p.live_label
            .clone()
            .unwrap_or_else(|| if p.you { "transmitindo pra sala".into() } else { "transmitindo".into() })
    } else {
        String::new()
    }
}

/// Estatísticas do cartão "Você está ao vivo".
pub(crate) fn broadcast_stats(b: &vm::MyBroadcastView, down: bool) -> String {
    if down {
        return "Sua transmissão continua · volta sozinha".into();
    }
    let label = b.label.as_deref().filter(|l| !l.is_empty());
    let mut s = String::new();
    if let Some(l) = label {
        s.push_str(l);
        s.push_str(" · ");
    }
    s.push_str(&format!("{} assistindo · {}p · {} fps", b.viewers, b.prefs.resolution, b.prefs.fps));
    if b.adjusted == Some(true) {
        s.push_str(" · ajustado");
    }
    let motivo = b.audio_reason_monitor.then_some(NoSoundReason::Monitor);
    s.push_str(" · ");
    s.push_str(audio_label_text(b.audio_kind, motivo));
    if label.is_none() && b.source == Some(LiveSource::Camera) {
        if let Some(t) = b.source_title.as_deref().filter(|t| !t.is_empty()) {
            s.push_str(&format!(" · câmera {t}"));
        }
    }
    s
}

/// Dica de baixo do seletor (`footHint`).
pub(crate) fn foot_hint(people: i32, resolution: i32, fps: i32) -> String {
    if people > 5 {
        format!("Com {people} pessoas na sala, a qualidade baixa sozinha se muita gente estiver assistindo.")
    } else if resolution == 1080 && fps == 60 {
        "1080p a 60 fps usa mais internet. Se travar pra galera, tente 720p.".into()
    } else {
        format!("{resolution}p · {fps} fps — leve pra todo mundo.")
    }
}

/// Texto sob "Compartilhar áudio" (`audioHint`, sem as ramificações do celular).
pub(crate) fn audio_hint(
    surface: Surface,
    native: bool,
    audio: bool,
    selected_title: &str,
    cam_granted: bool,
    cam_audio_label: &str,
) -> String {
    let screen = surface == Surface::Monitor;
    if surface == Surface::Camera {
        return if !audio {
            "Só a imagem vai pra sala.".into()
        } else if !cam_granted {
            "A entrada de som você escolhe depois de permitir as câmeras.".into()
        } else if !cam_audio_label.is_empty() {
            format!("Vai o som de {cam_audio_label} junto com a câmera.")
        } else {
            "Escolha abaixo a entrada de som que vai junto.".into()
        };
    }
    if native {
        let s = if screen { CaptureSurface::Monitor } else { CaptureSurface::Window };
        return native_audio_text(s, audio, Some(selected_title));
    }
    if screen {
        "Tela inteira vai sem som, pra não vazar o Discord. Pra levar o som, escolha uma janela.".into()
    } else if audio {
        "Só o som da janela que você escolher — Discord e notificações ficam de fora.".into()
    } else {
        "Só a imagem vai pra sala.".into()
    }
}

/// Título do diálogo de tirar da sala.
pub(crate) fn remove_title(name: &str) -> String {
    format!("Tirar {name} da sala?")
}

/// Nome do tile: "Você", "câmera de Lu" (sem descrição) ou o nome.
fn tile_label(s: &StreamView, you: bool) -> String {
    if you {
        "Você".into()
    } else if s.source == Some(LiveSource::Camera) && s.label.is_none() {
        format!("câmera de {}", s.name)
    } else {
        s.name.clone()
    }
}

/// Legenda do placeholder (`view.caption ?? "tela de Lu"`).
fn tile_caption(s: &StreamView) -> String {
    s.caption.clone().unwrap_or_else(|| {
        format!("{} de {}", if s.source == Some(LiveSource::Camera) { "câmera" } else { "tela" }, s.name)
    })
}

// ---------------------------------------------------------------------------
// Conversões
// ---------------------------------------------------------------------------

fn quadro(id: &str) -> Image {
    QUADROS.with(|q| q.borrow().get(id).cloned()).unwrap_or_default()
}

fn status(s: vm::TileStatus) -> TileStatus {
    match s {
        vm::TileStatus::Ok => TileStatus::Ok,
        vm::TileStatus::Loading => TileStatus::Loading,
        vm::TileStatus::Failed => TileStatus::Failed,
    }
}

fn tile(s: &StreamView) -> TileData {
    let you = s.you == Some(true);
    TileData {
        id: s.id.as_str().into(),
        name: s.name.as_str().into(),
        label: tile_label(s, you).into(),
        what: if you { String::new() } else { s.label.clone().unwrap_or_default() }.into(),
        tone: tone(s.tone),
        caption: tile_caption(s).into(),
        status: status(s.status),
        muted: s.muted,
        volume: s.volume as f32,
        you,
        watching: s.watching == Some(true),
        stopped: s.watch_state == Some(vm::StreamWatchState::Stopped),
        has_video: s.stream.is_some(),
        image: quadro(&s.id),
    }
}

/// A prévia "Você está ao vivo": a sua própria tela, parada, sem controles.
fn preview(view: &RoomView) -> TileData {
    let me = view.participants.iter().find(|p| p.you);
    let b = view.my_broadcast.as_ref();
    let camera = b.and_then(|b| b.source) == Some(LiveSource::Camera);
    let id = me.map(|p| p.id.clone()).unwrap_or_default();
    let image = quadro(&id);
    let sem_quadro_e_com_snapshot = image.size().width == 0 && b.is_some_and(|b| b.snapshot.is_some());
    let base = if camera { "sua câmera" } else { "sua tela" };
    TileData {
        id: id.as_str().into(),
        name: me.map(|p| p.name.clone()).unwrap_or_default().into(),
        label: "Você".into(),
        what: SharedString::new(),
        tone: me.map(|p| tone(p.tone)).unwrap_or(crate::Tone::Deep),
        // O mock do site desenha "sua tela · quadro parado" quando há snapshot: aqui o mesmo, sem o quadro de verdade.
        caption: if sem_quadro_e_com_snapshot { format!("{base} · quadro parado") } else { base.to_string() }.into(),
        status: TileStatus::Ok,
        muted: true,
        volume: 0.0,
        you: true,
        watching: true,
        stopped: false,
        has_video: false,
        image,
    }
}

/// Altura da lista de "Na sala": 54px por linha (59px com a linha de baixo) e 4px entre elas.
fn altura_da_lista(gente: &[PersonData]) -> f32 {
    let linhas: f32 = gente.iter().map(|p| if p.sub.is_empty() { 54.0 } else { 59.0 }).sum();
    linhas + 4.0 * gente.len().saturating_sub(1) as f32
}

fn pessoa(p: &ParticipantView, watching: bool) -> PersonData {
    PersonData {
        id: p.id.as_str().into(),
        name: p.name.as_str().into(),
        letter: initial(&p.name).into(),
        tone: tone(p.tone),
        you: p.you,
        live: p.live,
        sub: participant_sub(p, watching).into(),
        watching,
        removable: p.removable == Some(true) && !p.you,
    }
}

fn aba(s: Surface) -> SurfaceTab {
    match s {
        Surface::Window => SurfaceTab::Window,
        Surface::Monitor => SurfaceTab::Monitor,
        Surface::Camera => SurfaceTab::Camera,
    }
}

fn aba_para_core(t: SurfaceTab) -> Surface {
    match t {
        SurfaceTab::Window => Surface::Window,
        SurfaceTab::Monitor => Surface::Monitor,
        SurfaceTab::Camera => Surface::Camera,
    }
}

fn tom(nome: &str) -> Tone {
    match nome {
        "sage" => Tone::Sage,
        "sand" => Tone::Sand,
        "deep" => Tone::Deep,
        _ => Tone::Accent,
    }
}

/// Miniatura de mentira do mock: `mock:<tom>:<rótulo>` (as listras do `fakeThumb` com o rótulo "janela"/"tela").
/// Miniatura de verdade (`data:image/...`): ainda sem decodificador aqui, sai sem miniatura (mostra o ícone).
pub(crate) fn thumb_falsa(thumb: Option<&str>) -> Option<(Tone, String)> {
    let resto = thumb?.strip_prefix("mock:")?;
    let (t, rotulo) = resto.split_once(':')?;
    Some((tom(t), rotulo.to_string()))
}

/// Ícone de mentira do mock: `mock:<letra>:<tom>` (a pastilha do `fakeIcon`).
pub(crate) fn icone_falso(icon: Option<&str>) -> Option<(String, Tone)> {
    let resto = icon?.strip_prefix("mock:")?;
    let (letra, t) = resto.split_once(':')?;
    Some((letra.to_string(), tom(t)))
}

fn primeira_letra(texto: &str) -> String {
    let l = initial(texto);
    if l.is_empty() { "?".into() } else { l }
}

fn fonte(c: &vm::CaptureSource, selecionada: bool) -> SourceData {
    let monitor = c.kind == vm::ScreenSurface::Monitor;
    let icone = icone_falso(c.icon.as_deref());
    let miniatura = thumb_falsa(c.thumb.as_deref());
    // Sem ícone: a letra do programa (ou o número da tela).
    let fallback = {
        let base = if c.app.is_empty() { &c.title } else { &c.app };
        let letra = primeira_letra(base);
        if monitor {
            let digitos: String = c.title.chars().filter(|ch| ch.is_ascii_digit()).collect();
            if digitos.is_empty() { letra } else { digitos }
        } else {
            letra
        }
    };
    SourceData {
        id: c.id.as_str().into(),
        kind: if monitor { SourceKind::Monitor } else { SourceKind::Window },
        title: c.title.as_str().into(),
        app: c.app.as_str().into(),
        letter: icone.as_ref().map(|(l, _)| l.clone()).unwrap_or(fallback).into(),
        has_icon: icone.is_some(),
        icon_tone: icone.map(|(_, t)| tone(t)).unwrap_or(crate::Tone::Accent),
        thumb_label: miniatura.as_ref().map(|(_, r)| r.clone()).unwrap_or_default().into(),
        thumb_tone: miniatura.map(|(t, _)| tone(t)).unwrap_or(crate::Tone::Sand),
        thumb: Image::default(),
        playing: c.playing,
        busy: false,
        selected: selecionada,
    }
}

fn camera(d: &vm::CameraDevice, indice: usize, selecionada: bool) -> SourceData {
    let capture = d.kind == vm::CameraDeviceKind::Capture;
    let app = if capture { "Placa de captura" } else { "Câmera" };
    // Prévia de mentira do mock: fundo verde, quente ou cinza em rodízio, com o rótulo "câmera"/"console".
    let tons = [Tone::Sage, Tone::Accent, Tone::Sand];
    SourceData {
        id: d.id.as_str().into(),
        kind: SourceKind::Camera,
        title: d.label.as_str().into(),
        app: app.into(),
        letter: primeira_letra(app).into(),
        has_icon: false,
        icon_tone: crate::Tone::Accent,
        thumb_label: if d.preview.is_some() { if capture { "console" } else { "câmera" } } else { "" }.into(),
        thumb_tone: tone(tons[indice % tons.len()]),
        thumb: Image::default(),
        playing: false,
        busy: d.busy,
        selected: selecionada,
    }
}

fn go_live(view: &RoomView) -> GoLiveData {
    let p = &view.picker;
    let cam = &view.camera;
    let sel = p.selected_id.as_deref();
    let da_aba = |k: vm::ScreenSurface| p.sources.iter().filter(move |s| s.kind == k);
    let escolhida = |k: vm::ScreenSurface| da_aba(k).find(|s| Some(s.id.as_str()) == sel);
    let janela = escolhida(vm::ScreenSurface::Window);
    let tela = escolhida(vm::ScreenSurface::Monitor);
    let audio_label = cam
        .audio_id
        .as_deref()
        .and_then(|id| cam.audios.iter().find(|a| a.id == id))
        .map(|a| a.label.clone())
        .unwrap_or_default();
    GoLiveData {
        surface: aba(view.go_live_prefs.surface),
        resolution: view.go_live_prefs.resolution as i32,
        fps: view.go_live_prefs.fps as i32,
        audio: view.go_live_prefs.audio,
        cameras_only: !view.can_share_screen,
        native: p.mode == vm::PickerMode::Native,
        loading: p.loading,
        windows: modelo(
            da_aba(vm::ScreenSurface::Window).map(|s| fonte(s, Some(s.id.as_str()) == sel)).collect(),
        ),
        monitors: modelo(
            da_aba(vm::ScreenSurface::Monitor).map(|s| fonte(s, Some(s.id.as_str()) == sel)).collect(),
        ),
        selected_window: janela.map(|s| s.title.clone()).unwrap_or_default().into(),
        selected_monitor: tela.map(|s| s.title.clone()).unwrap_or_default().into(),
        has_selected_window: janela.is_some(),
        has_selected_monitor: tela.is_some(),
        cam_permission: match cam.permission {
            vm::CameraPermission::Unknown => CamPermission::Unknown,
            vm::CameraPermission::Prompt => CamPermission::Prompt,
            vm::CameraPermission::Granted => CamPermission::Granted,
            vm::CameraPermission::Denied => CamPermission::Denied,
        },
        cams: modelo(
            cam.devices
                .iter()
                .enumerate()
                .map(|(i, d)| camera(d, i, Some(d.id.as_str()) == cam.video_id.as_deref()))
                .collect(),
        ),
        cam_audios: modelo(
            cam.audios.iter().map(|a| AudioOption { id: a.id.as_str().into(), label: a.label.as_str().into() }).collect(),
        ),
        has_cam_video: cam.video_id.is_some(),
        cam_audio_id: cam.audio_id.clone().unwrap_or_default().into(),
        cam_audio_label: audio_label.into(),
        cam_role_game: cam.audio_role == AudioRole::Game,
        cam_error: match cam.error {
            None => CamError::None,
            Some(vm::CameraError::Busy) => CamError::Busy,
            Some(vm::CameraError::NotFound) => CamError::NotFound,
            Some(vm::CameraError::Other) => CamError::Other,
        },
        people: view.participants.len() as i32,
    }
}

fn agora_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// Põe o "12:04" na tela e o mantém andando (uma vez por segundo) enquanto você transmite.
fn relogio(app: &AppWindow, iniciou: Option<u64>) {
    let s = app.global::<RoomState>();
    match iniciou {
        None => RELOGIO.with(|t| t.stop()),
        Some(inicio) => {
            s.set_elapsed(format_elapsed(agora_ms().saturating_sub(inicio)).into());
            let fraca = app.as_weak();
            RELOGIO.with(|t| {
                t.start(slint::TimerMode::Repeated, Duration::from_secs(1), move || {
                    if let Some(app) = fraca.upgrade() {
                        app.global::<RoomState>()
                            .set_elapsed(format_elapsed(agora_ms().saturating_sub(inicio)).into());
                    }
                });
            });
        }
    }
}

// ---------------------------------------------------------------------------
// apply / connect
// ---------------------------------------------------------------------------

/// Copia o view-model para as propriedades de `RoomState`. Com `manter_texto`, o nome que a pessoa está digitando no
/// diálogo de renomear não é sobrescrito enquanto o diálogo continua aberto.
pub(crate) fn apply(app: &AppWindow, view: &RoomView, manter_texto: bool) {
    let s = app.global::<RoomState>();
    let layout = stage_layout(view);
    let head = header(view);
    let caiu = view.link != vm::LinkState::Ok;
    let vendo = watching_id(view);

    s.set_requires_update(view.requires_update.unwrap_or(false));
    s.set_connecting(layout == StageLayout::Connecting);
    s.set_link(match view.link {
        vm::LinkState::Ok => LinkMode::Ok,
        vm::LinkState::Reconnecting => LinkMode::Reconnecting,
        vm::LinkState::Failed => LinkMode::Failed,
    });
    s.set_title(head.title.into());
    s.set_subtitle(head.subtitle.into());
    s.set_invite_label(invite_label(&view.invite_url).into());
    s.set_copied(view.copied);

    // Barra lateral.
    let gente: Vec<PersonData> =
        view.participants.iter().map(|p| pessoa(p, vendo.as_deref() == Some(p.id.as_str()))).collect();
    s.set_people_height(altura_da_lista(&gente));
    s.set_people(modelo(gente));
    s.set_people_count(format!("{} de {}", view.participants.len(), view.capacity).into());
    match &view.my_broadcast {
        Some(b) => {
            s.set_broadcasting(true);
            s.set_preview(preview(view));
            s.set_on_stage(b.on_stage == Some(true));
            s.set_stats(broadcast_stats(b, caiu).into());
            relogio(app, Some(b.started_at));
        }
        None => {
            s.set_broadcasting(false);
            s.set_on_stage(false);
            s.set_stats(SharedString::new());
            relogio(app, None);
        }
    }

    // Cabeçalho do dono.
    s.set_owner_button(matches!(view.room, vm::RoomKind::Fixed { own: true, .. }));
    s.set_owner_menu(view.owner_menu == Some(true));

    // Palco.
    s.set_stage(match layout {
        StageLayout::Connecting => StageMode::Connecting,
        StageLayout::Empty => StageMode::Empty,
        StageLayout::Single => StageMode::Single,
        StageLayout::Grid => StageMode::Grid,
        StageLayout::Focus => StageMode::Focus,
    });
    let n = view.streams.len();
    let (colunas, linhas) = grid_size(n);
    s.set_cols(colunas as i32);
    s.set_rows(linhas as i32);
    s.set_grid_hint(n < (colunas * linhas) as usize);
    let (em_foco, faixa): (Option<&StreamView>, Vec<&StreamView>) = match (layout, focused_stream(view)) {
        (StageLayout::Focus, Some(f)) => (Some(f), view.streams.iter().filter(|x| x.id != f.id).collect()),
        _ => (None, Vec::new()),
    };
    s.set_tiles(modelo(match layout {
        StageLayout::Single | StageLayout::Grid => view.streams.iter().map(tile).collect(),
        _ => Vec::new(),
    }));
    s.set_focused_tile(em_foco.map(tile).unwrap_or_default());
    s.set_strip(modelo(faixa.into_iter().map(tile).collect()));
    s.set_can_share(view.can_share);
    s.set_permission_denied(view.permission_denied);
    let avisos: Vec<ToastData> = view
        .toasts
        .iter()
        .map(|t| ToastData {
            id: t.id.as_str().into(),
            kind: match t.kind {
                vm::ToastKind::Join => ToastKind::Join,
                vm::ToastKind::Leave => ToastKind::Leave,
                vm::ToastKind::Live => ToastKind::Live,
                vm::ToastKind::Copied => ToastKind::Copied,
                vm::ToastKind::Error => ToastKind::Error,
            },
            text: t.text.as_str().into(),
            action: t.action.clone().unwrap_or_default().into(),
        })
        .collect();
    s.set_toasts(modelo(avisos));

    // Diálogos.
    match &view.name_dialog {
        Some(d) => {
            if !(manter_texto && s.get_name_open()) {
                s.set_name_value(d.value.as_str().into());
            }
            s.set_name_open(true);
            s.set_name_busy(d.busy);
            s.set_name_error(d.error.clone().unwrap_or_default().into());
        }
        None => s.set_name_open(false),
    }
    match &view.delete_dialog {
        Some(d) => {
            s.set_delete_open(true);
            s.set_delete_title(home::delete_title(&d.name).into());
            s.set_delete_warn(home::delete_warn(d.people).into());
            s.set_delete_busy(d.busy);
            s.set_delete_error(d.error.clone().unwrap_or_default().into());
        }
        None => s.set_delete_open(false),
    }
    match &view.remove_dialog {
        Some(d) => {
            s.set_remove_open(true);
            s.set_remove_title(remove_title(&d.name).into());
            s.set_remove_busy(d.busy);
            s.set_remove_error(d.error.clone().unwrap_or_default().into());
        }
        None => s.set_remove_open(false),
    }

    // Seletor.
    PREFS.with(|p| *p.borrow_mut() = view.go_live_prefs.clone());
    s.set_go_live_open(view.go_live_open);
    if view.go_live_open {
        s.set_go_live(go_live(view));
    }
}

/// Liga os callbacks de `RoomState` às ações devolvidas por `emit`.
pub(crate) fn connect(app: &AppWindow, emit: &Emit) {
    let s = app.global::<RoomState>();
    s.on_limit_room_name(|t| t.chars().take(ROOM_NAME_MAX).collect::<String>().into());
    s.on_normalize_room_name(|t| normalize_room_name(&t).unwrap_or_default().into());
    s.on_audio_hint(|aba, nativo, som, titulo, liberadas, entrada| {
        audio_hint(aba_para_core(aba), nativo, som, &titulo, liberadas, &entrada).into()
    });
    s.on_foot_hint(|pessoas, res, fps| foot_hint(pessoas, res, fps).into());

    macro_rules! acao {
        ($on:ident, || $a:expr) => {{
            let e = emit.clone();
            s.$on(move || e(UiAction::Room($a)));
        }};
        ($on:ident, |$($p:ident),+| $a:expr) => {{
            let e = emit.clone();
            s.$on(move |$($p),*| e(UiAction::Room($a)));
        }};
    }
    // Texto vazio vale como "nenhum" (tirar o foco, sem som).
    fn opcional(t: &SharedString) -> Option<String> {
        (!t.is_empty()).then(|| t.to_string())
    }
    acao!(on_go_home, || RoomAction::GoHome);
    acao!(on_copy_invite, || RoomAction::CopyInvite);
    acao!(on_toggle_owner_menu, |aberto| RoomAction::ToggleOwnerMenu { open: aberto });
    acao!(on_open_rename, || RoomAction::OpenRename);
    acao!(on_submit_rename, |nome| RoomAction::SubmitRename { name: nome.to_string() });
    acao!(on_open_delete, || RoomAction::OpenDelete);
    acao!(on_confirm_delete, || RoomAction::ConfirmDelete);
    acao!(on_close_dialogs, || RoomAction::CloseDialogs);
    acao!(on_ask_remove, |id| RoomAction::AskRemove { peer_id: id.to_string() });
    acao!(on_confirm_remove, || RoomAction::ConfirmRemove);
    acao!(on_toggle_self_stage, || RoomAction::ToggleSelfStage);
    acao!(on_focus, |id| RoomAction::Focus { id: opcional(&id) });
    acao!(on_watch, |id, ligar| RoomAction::Watch { id: id.to_string(), on: ligar });
    acao!(on_toggle_mute, |id| RoomAction::ToggleMute { id: id.to_string() });
    acao!(on_set_volume, |id, v| RoomAction::SetVolume { id: id.to_string(), volume: f64::from(v).clamp(0.0, 1.0) });
    acao!(on_retry, |id| RoomAction::Retry { id: id.to_string() });
    acao!(on_pip, |id, ligar| RoomAction::Pip { id: ligar.then(|| id.to_string()) });
    acao!(on_toast_action, |id| RoomAction::ToastAction { id: id.to_string() });
    acao!(on_dismiss_toast, |id| RoomAction::DismissToast { id: id.to_string() });
    acao!(on_retry_connection, || RoomAction::RetryConnection);
    acao!(on_open_go_live, || RoomAction::OpenGoLive);
    acao!(on_close_go_live, || RoomAction::CloseGoLive);
    acao!(on_start_go_live, |aba, res, fps, som| {
        let base = PREFS.with(|p| p.borrow().clone());
        RoomAction::StartGoLive {
            prefs: GoLive { resolution: res as u16, fps: fps as u16, audio: som, surface: aba_para_core(aba), ..base },
        }
    });
    acao!(on_select_source, |id| RoomAction::SelectSource { id: id.to_string() });
    acao!(on_allow_camera, || RoomAction::AllowCamera);
    acao!(on_select_camera, |id| RoomAction::SelectCamera { id: id.to_string() });
    acao!(on_select_audio, |id| RoomAction::SelectAudio { id: opcional(&id) });
    acao!(on_set_audio_role, |jogo| RoomAction::SetAudioRole {
        role: if jogo { AudioRole::Game } else { AudioRole::Voice }
    });
    acao!(on_set_surface, |aba| RoomAction::SetSurface { surface: aba_para_core(aba) });
    acao!(on_dismiss_permission, || RoomAction::DismissPermission);
    acao!(on_stop_go_live, || RoomAction::StopGoLive);
    acao!(on_leave, || RoomAction::Leave);
    let e = emit.clone();
    s.on_reload(move || e(UiAction::Reload));
}

// ---------------------------------------------------------------------------
// Quadros de vídeo
// ---------------------------------------------------------------------------

/// Guarda o quadro RGBA (`largura` x `altura`, 4 bytes por pixel) da transmissão `id` e o põe no tile que a mostra
/// (palco, grade, faixa ou, com o id da própria pessoa, a prévia da barra lateral). Devolve `false` e ignora o quadro se
/// o tamanho do vetor não bate com as dimensões.
pub(crate) fn set_frame(app: &AppWindow, id: &str, largura: u32, altura: u32, rgba: &[u8]) -> bool {
    if largura == 0 || altura == 0 || rgba.len() != largura as usize * altura as usize * 4 {
        return false;
    }
    let buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(rgba, largura, altura);
    let img = Image::from_rgba8(buf);
    QUADROS.with(|q| q.borrow_mut().insert(id.to_string(), img.clone()));

    let s = app.global::<RoomState>();
    let colocar = |m: ModelRc<TileData>| {
        for i in 0..m.row_count() {
            if let Some(mut t) = m.row_data(i) {
                if t.id == id {
                    t.image = img.clone();
                    m.set_row_data(i, t);
                }
            }
        }
    };
    colocar(s.get_tiles());
    colocar(s.get_strip());
    let mut foco = s.get_focused_tile();
    if foco.id == id {
        foco.image = img.clone();
        s.set_focused_tile(foco);
    }
    let mut previa = s.get_preview();
    if previa.id == id {
        previa.image = img;
        s.set_preview(previa);
    }
    true
}

/// A transmissão `id` acabou: esquece o último quadro.
pub(crate) fn clear_frame(app: &AppWindow, id: &str) {
    QUADROS.with(|q| q.borrow_mut().remove(id));
    let s = app.global::<RoomState>();
    let limpar = |m: ModelRc<TileData>| {
        for i in 0..m.row_count() {
            if let Some(mut t) = m.row_data(i) {
                if t.id == id {
                    t.image = Image::default();
                    m.set_row_data(i, t);
                }
            }
        }
    };
    limpar(s.get_tiles());
    limpar(s.get_strip());
    let mut foco = s.get_focused_tile();
    if foco.id == id {
        foco.image = Image::default();
        s.set_focused_tile(foco);
    }
    let mut previa = s.get_preview();
    if previa.id == id {
        previa.image = Image::default();
        s.set_preview(previa);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use espia_core::protocol::AudioKind;
    use espia_core::view_model::{MyBroadcastView, ScreenSurface};

    fn pessoa_de(live: bool, you: bool, rotulo: Option<&str>) -> ParticipantView {
        ParticipantView {
            id: "x".into(),
            name: "Lu".into(),
            tone: Tone::Accent,
            you,
            live,
            live_label: rotulo.map(String::from),
            removable: None,
        }
    }

    fn transmissao() -> MyBroadcastView {
        MyBroadcastView {
            prefs: GoLive { resolution: 1080, fps: 60, ..GoLive::default() },
            started_at: 0,
            adjusted: None,
            viewers: 3,
            stream: None,
            audio_kind: AudioKind::Window,
            audio_reason_monitor: false,
            source_title: None,
            label: Some("Celeste".into()),
            source: None,
            snapshot: None,
            on_stage: None,
        }
    }

    #[test]
    fn convite_sem_https() {
        assert_eq!(invite_label("https://espia.diuliano.com/s/k7f-q2x"), "espia.diuliano.com/s/k7f-q2x");
        assert_eq!(invite_label("http://x/y"), "x/y");
        assert_eq!(invite_label("x/y"), "x/y");
    }

    #[test]
    fn linha_de_baixo_da_pessoa() {
        assert_eq!(participant_sub(&pessoa_de(true, false, Some("Hollow Knight")), true), "você está assistindo");
        assert_eq!(participant_sub(&pessoa_de(true, false, Some("Hollow Knight")), false), "Hollow Knight");
        assert_eq!(participant_sub(&pessoa_de(true, false, None), false), "transmitindo");
        assert_eq!(participant_sub(&pessoa_de(true, true, None), false), "transmitindo pra sala");
        assert_eq!(participant_sub(&pessoa_de(false, false, Some("x")), false), "");
    }

    #[test]
    fn estatisticas_da_sua_transmissao() {
        let mut b = transmissao();
        assert_eq!(broadcast_stats(&b, false), "Celeste · 3 assistindo · 1080p · 60 fps · com som da janela");
        assert_eq!(broadcast_stats(&b, true), "Sua transmissão continua · volta sozinha");
        b.adjusted = Some(true);
        b.label = None;
        b.source = Some(LiveSource::Camera);
        b.source_title = Some("HD Pro Webcam C920".into());
        b.audio_kind = AudioKind::None;
        b.audio_reason_monitor = true;
        assert_eq!(
            broadcast_stats(&b, false),
            "3 assistindo · 1080p · 60 fps · ajustado · sem som (tela inteira) · câmera HD Pro Webcam C920"
        );
    }

    #[test]
    fn dica_do_rodape() {
        assert_eq!(foot_hint(2, 1080, 30), "1080p · 30 fps — leve pra todo mundo.");
        assert_eq!(foot_hint(2, 1080, 60), "1080p a 60 fps usa mais internet. Se travar pra galera, tente 720p.");
        assert!(foot_hint(6, 720, 30).starts_with("Com 6 pessoas na sala"));
        assert!(foot_hint(5, 720, 30).starts_with("720p"));
    }

    #[test]
    fn dica_do_som() {
        assert_eq!(audio_hint(Surface::Window, true, false, "", false, ""), "Só a imagem vai pra sala.");
        assert_eq!(
            audio_hint(Surface::Monitor, true, true, "", false, ""),
            "Todo o som do computador vai junto — menos o Discord."
        );
        assert_eq!(
            audio_hint(Surface::Window, true, true, "Hollow Knight", false, ""),
            "Só o som de Hollow Knight — nada de notificação vazando."
        );
        assert!(audio_hint(Surface::Monitor, false, true, "", false, "").starts_with("Tela inteira vai sem som"));
        assert!(audio_hint(Surface::Camera, true, true, "", false, "").starts_with("A entrada de som você escolhe"));
        assert_eq!(
            audio_hint(Surface::Camera, true, true, "", true, "Microfone"),
            "Vai o som de Microfone junto com a câmera."
        );
        assert!(audio_hint(Surface::Camera, true, true, "", true, "").starts_with("Escolha abaixo"));
    }

    #[test]
    fn titulo_de_tirar_da_sala() {
        assert_eq!(remove_title("Tomás"), "Tirar Tomás da sala?");
    }

    #[test]
    fn miniatura_e_icone_falsos_do_mock() {
        assert_eq!(thumb_falsa(Some("mock:sage:janela")), Some((Tone::Sage, "janela".into())));
        assert_eq!(thumb_falsa(Some("data:image/png;base64,AA==")), None);
        assert_eq!(thumb_falsa(None), None);
        assert_eq!(icone_falso(Some("mock:H:deep")), Some(("H".into(), Tone::Deep)));
        assert_eq!(icone_falso(Some("data:image/png;base64,AA==")), None);
    }

    #[test]
    fn letra_da_pastilha_do_cartao() {
        let janela = vm::CaptureSource {
            id: "w:1".into(),
            kind: ScreenSurface::Window,
            title: "Downloads".into(),
            app: "Explorador de arquivos".into(),
            icon: None,
            thumb: None,
            playing: false,
            primary: false,
        };
        assert_eq!(fonte(&janela, false).letter.as_str(), "E");
        let tela = vm::CaptureSource { kind: ScreenSurface::Monitor, title: "Tela 2".into(), app: String::new(), ..janela.clone() };
        assert_eq!(fonte(&tela, false).letter.as_str(), "2");
        let com_icone = vm::CaptureSource { icon: Some("mock:H:accent".into()), ..janela };
        let f = fonte(&com_icone, true);
        assert!(f.has_icon && f.selected);
        assert_eq!(f.letter.as_str(), "H");
    }
}
