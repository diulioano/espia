//! View-models: o estado pronto que cada tela do app recebe, e as ações que ela devolve.
//! Porte de `src/lib/types.ts` (constituição IV: a tela não fala com rede, captura nem mídia).
//!
//! Regras do porte:
//! - Cada função de `RoomActions`, `HomeActions`, `JoinActions` e `NotFoundActions` vira um caso
//!   de um enum de ação (`RoomAction`, `HomeAction`, `JoinAction`, `NotFoundAction`): no app a tela
//!   devolve a ação em vez de chamar uma função.
//! - Tipos que já existem em `protocol` (`Tone`, `AudioKind`, `LiveSource`) e em `device` (`GoLive`
//!   no lugar de `GoLivePrefs`, `Surface` no lugar de `ShareSurface`, `AudioRole`) são reaproveitados.
//! - Ficam de fora (só celular, R17, ou só mock do site): `mockViewer`, `fullscreenId` e a ação
//!   `fullscreen` (visualizador em tela cheia do celular, specs/018), `CameraView.mobile`,
//!   `CameraView.facing` e a ação `setFacing`/`SetFacing` (frontal/traseira), `MobileRoomPage`.
//! - `CameraActions` não existe aparte: as mesmas ações já estão em `RoomAction`.
//! - Sem equivalente claro: `MediaStream` vira `StreamId` e `ImageBitmap` vira `SnapshotId`,
//!   identificadores opacos; quem tem o objeto de verdade (o shell) resolve o identificador.
//! - Sem serde: estes tipos não vão para o disco nem para a rede.

use crate::device::{AudioRole, GoLive, Surface};
use crate::protocol::{AudioKind, LiveSource, Tone};

// ---------------------------------------------------------------------------
// Identificadores opacos
// ---------------------------------------------------------------------------

/// Identificador opaco de um fluxo de mídia (no TS, `MediaStream`). A tela só o repassa ao shell.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamId(pub String);

/// Identificador opaco de um quadro parado da prévia (no TS, `ImageBitmap`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SnapshotId(pub String);

// ---------------------------------------------------------------------------
// Tipos comuns
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileStatus {
    Ok,
    Loading,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileSize {
    Stage,
    Grid,
    Thumb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Join,
    Leave,
    Live,
    Copied,
    Error,
}

/// Aba de janela/tela: o que uma `CaptureSource` do app pode ser (`ScreenSurface`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenSurface {
    Window,
    Monitor,
}

/// Estado de assistir de uma transmissão (`StreamView.watchState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamWatchState {
    Available,
    Opening,
    Playing,
    Stopped,
    Unavailable,
}

// ---------------------------------------------------------------------------
// Sala
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantView {
    pub id: String,
    pub name: String,
    pub tone: Tone,
    pub you: bool,
    pub live: bool,
    /// O que essa pessoa está transmitindo (specs/012).
    pub live_label: Option<String>,
    /// O dono de uma sala fixa pode tirar essa pessoa da sala (specs/014 US4). Nunca em `you`.
    pub removable: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamView {
    pub id: String,
    pub name: String,
    pub tone: Tone,
    /// Ex.: "1080p · 60 fps".
    pub quality: Option<String>,
    /// Texto do placeholder quando não há vídeo (mock).
    pub caption: Option<String>,
    pub status: TileStatus,
    pub muted: bool,
    /// 0..1
    pub volume: f64,
    /// `stream` do TS (`MediaStream`): identificador opaco.
    pub stream: Option<StreamId>,
    /// `audioStream` do TS (`MediaStream`): só o som da pessoa (specs/014), identificador opaco.
    pub audio_stream: Option<StreamId>,
    /// O que está sendo transmitido (specs/012): "Hollow Knight", "Aba do navegador"…
    pub label: Option<String>,
    /// Câmera/placa (specs/006): o tile diz "câmera de …".
    pub source: Option<LiveSource>,
    /// `true` somente após escolha explícita; ausente/`false` nunca autoriza receber mídia.
    pub watching: Option<bool>,
    pub watch_state: Option<StreamWatchState>,
    /// A sua própria transmissão no palco (specs/014 US3): sempre muda, sem controles de som.
    pub you: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToastView {
    pub id: String,
    pub kind: ToastKind,
    pub text: String,
    pub action: Option<String>,
}

/// Permissão de câmera (`CameraView.permission`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPermission {
    Unknown,
    Prompt,
    Granted,
    Denied,
}

/// Tipo do dispositivo de vídeo (`CameraView.devices[].kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraDeviceKind {
    Camera,
    Capture,
}

/// A câmera não abriu: `Busy` = em uso por outro programa; `NotFound` = o Windows não achou.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraError {
    Busy,
    NotFound,
    Other,
}

/// Uma câmera ou placa de captura (`CameraView.devices[]`).
#[derive(Debug, Clone, PartialEq)]
pub struct CameraDevice {
    pub id: String,
    pub label: String,
    pub kind: CameraDeviceKind,
    /// `preview` do TS (`MediaStream | null`): identificador opaco.
    pub preview: Option<StreamId>,
    pub busy: bool,
}

/// Uma entrada de som (`CameraView.audios[]`).
#[derive(Debug, Clone, PartialEq)]
pub struct CameraAudio {
    pub id: String,
    pub label: String,
}

/// Aba Câmeras do seletor (specs/006). Sem `mobile` e `facing` (só celular).
#[derive(Debug, Clone, PartialEq)]
pub struct CameraView {
    pub permission: CameraPermission,
    pub devices: Vec<CameraDevice>,
    pub audios: Vec<CameraAudio>,
    pub video_id: Option<String>,
    /// `None` = sem som.
    pub audio_id: Option<String>,
    pub audio_role: AudioRole,
    pub error: Option<CameraError>,
}

/// Janela ou monitor que o app pode transmitir (specs/005 data-model.md).
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureSource {
    /// "w:<hwnd>" ou "m:<hmonitor>".
    pub id: String,
    pub kind: ScreenSurface,
    pub title: String,
    pub app: String,
    /// `data:image/png` 32×32 no TS; aqui é o mesmo texto, `None` = sem ícone.
    pub icon: Option<String>,
    /// `data:image/jpeg` ≤ 320×180; `None` = minimizada ou sem imagem (mostra o ícone).
    pub thumb: Option<String>,
    /// Selo "tocando som". Sempre `false` em monitor.
    pub playing: bool,
    pub primary: bool,
}

/// Modo do seletor: `Native` = grade do app; `Browser` = seletor do navegador (specs/004).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerMode {
    Native,
    Browser,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickerView {
    pub mode: PickerMode,
    /// Todas as fontes (as duas abas); a grade filtra pela aba.
    pub sources: Vec<CaptureSource>,
    /// Ainda buscando a primeira lista.
    pub loading: bool,
    pub selected_id: Option<String>,
}

/// Sua transmissão (`MyBroadcastView extends GoLivePrefs`): as preferências vão em `prefs`.
#[derive(Debug, Clone, PartialEq)]
pub struct MyBroadcastView {
    /// Os campos de `GoLivePrefs` que o TS herda.
    pub prefs: GoLive,
    pub started_at: u64,
    /// Qualidade baixada sozinha porque tem muita gente assistindo.
    pub adjusted: Option<bool>,
    pub viewers: u32,
    /// `stream` do TS (`MediaStream`): identificador opaco.
    pub stream: Option<StreamId>,
    /// O som que está saindo de verdade (specs/002).
    pub audio_kind: AudioKind,
    /// Sem som porque é tela inteira (`audioReason: 'monitor'`).
    pub audio_reason_monitor: bool,
    /// Título da janela/tela escolhida, ou o nome da câmera. Só pra você; não vai pra sala.
    pub source_title: Option<String>,
    /// O que a sala vê como descrição da sua transmissão (specs/012).
    pub label: Option<String>,
    pub source: Option<LiveSource>,
    /// Prévia parada (specs/014 US3). `snapshot` do TS (`ImageBitmap | null`): identificador opaco.
    pub snapshot: Option<SnapshotId>,
    /// A sua tela está grande no palco, ao vivo (clicou na prévia).
    pub on_stage: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomStatus {
    Connecting,
    Ready,
}

/// Ligação com o servidor (specs/011 US1). Queda não é sair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    Ok,
    Reconnecting,
    Failed,
}

/// Sala rápida, ou fixa com nome. `own` = este aparelho criou (tem a chave do dono).
#[derive(Debug, Clone, PartialEq)]
pub enum RoomKind {
    Quick,
    Fixed { name: String, own: bool },
}

/// Modal de nome da sala fixa: criar (F7) ou renomear (S3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameDialogMode {
    Create,
    Rename,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDialogView {
    pub mode: NameDialogMode,
    pub value: String,
    pub busy: bool,
    pub error: Option<String>,
}

/// Confirmar "Apagar sala" (specs/011 S4).
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteDialogView {
    pub name: String,
    /// Pessoas na sala agora (fora você): o aviso "Elas saem da sala".
    pub people: u32,
    pub busy: bool,
    pub error: Option<String>,
}

/// Confirmar "Tirar da sala" (specs/014 US4).
#[derive(Debug, Clone, PartialEq)]
pub struct RemoveDialogView {
    pub name: String,
    pub busy: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomView {
    pub requires_update: Option<bool>,
    pub room_id: String,
    pub invite_url: String,
    pub status: RoomStatus,
    /// Você primeiro.
    pub participants: Vec<ParticipantView>,
    pub capacity: u32,
    /// Transmissões dos outros (a sua fica só na sidebar).
    pub streams: Vec<StreamView>,
    pub focused_id: Option<String>,
    pub my_broadcast: Option<MyBroadcastView>,
    pub copied: bool,
    pub go_live_open: bool,
    /// `goLivePrefs` do TS (`GoLivePrefs`).
    pub go_live_prefs: GoLive,
    pub picker: PickerView,
    pub camera: CameraView,
    pub permission_denied: bool,
    pub toasts: Vec<ToastView>,
    /// Pode transmitir algo (tela ou câmera).
    pub can_share: bool,
    /// Pode transmitir tela. Sem isso, o seletor mostra só câmeras.
    pub can_share_screen: bool,
    pub link: LinkState,
    pub room: RoomKind,
    /// Dono (specs/011 S2–S4): menu "⋯" ao lado do nome, renomear e apagar.
    pub owner_menu: Option<bool>,
    pub name_dialog: Option<NameDialogView>,
    pub delete_dialog: Option<DeleteDialogView>,
    /// Confirmar "Tirar da sala" (specs/014 US4).
    pub remove_dialog: Option<RemoveDialogView>,
}

/// Ações que a sala devolve (`RoomActions`); quem fornece o `RoomView` decide o que fazer.
/// Sem `Fullscreen` e `SetFacing` (só celular).
#[derive(Debug, Clone, PartialEq)]
pub enum RoomAction {
    CopyInvite,
    OpenGoLive,
    CloseGoLive,
    StartGoLive { prefs: GoLive },
    /// Escolheu um cartão na grade do app (specs/005).
    SelectSource { id: String },
    /// Aba Câmeras (specs/006).
    AllowCamera,
    SelectCamera { id: String },
    SelectAudio { id: Option<String> },
    SetAudioRole { role: AudioRole },
    /// Trocou de aba no seletor (liga/desliga as prévias de câmera).
    SetSurface { surface: Surface },
    StopGoLive,
    DismissPermission,
    Focus { id: Option<String> },
    /// Parar de assistir (`on = false`) ou voltar a assistir (`on = true`) (specs/014 US1).
    Watch { id: String, on: bool },
    /// Uma tela entrou (`Some`) ou saiu (`None`) do picture-in-picture (specs/014 US2).
    Pip { id: Option<String> },
    /// Clicou na prévia "Você está ao vivo" (specs/014 US3).
    ToggleSelfStage,
    ToggleMute { id: String },
    SetVolume { id: String, volume: f64 },
    Retry { id: String },
    ToastAction { id: String },
    DismissToast { id: String },
    /// "Tentar de novo" na tela "Não deu pra voltar pra sala" (specs/011).
    RetryConnection,
    /// Logo (specs/013): sai da sala e vai pro início; transmitindo, pergunta antes.
    GoHome,
    /// Dono da sala fixa (specs/011 US4).
    ToggleOwnerMenu { open: bool },
    OpenRename,
    SubmitRename { name: String },
    OpenDelete,
    ConfirmDelete,
    CloseDialogs,
    /// Dono (specs/014 US4): "Remover da sala" na lista e a confirmação.
    AskRemove { peer_id: String },
    ConfirmRemove,
    Leave,
}

// ---------------------------------------------------------------------------
// Tela inicial
// ---------------------------------------------------------------------------

/// Convite do app (specs/008): oferta, ou a dica depois de baixar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppInvite {
    Offer,
    Hint,
}

/// `SavedRoomView.state`: `Loading` = números ainda não chegaram; `Gone` = apagada, expirada ou
/// não existe mais.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavedRoomState {
    Loading,
    Ok,
    Gone,
}

/// Uma linha de Suas salas (specs/011). Só números, nunca apelidos.
#[derive(Debug, Clone, PartialEq)]
pub struct SavedRoomView {
    pub code: String,
    pub name: String,
    /// Este aparelho criou a sala (tem a chave do dono).
    pub own: bool,
    pub tone: Tone,
    pub state: SavedRoomState,
    pub people: u32,
    pub streaming: u32,
    /// "usada ontem", "há 3 dias"…
    pub recent: String,
    pub menu_open: Option<bool>,
}

/// Um grupo de Suas salas (`SavedRoomsView.groups[]`).
#[derive(Debug, Clone, PartialEq)]
pub struct SavedRoomGroup {
    /// "Com gente agora" / "Usadas recentemente" com 4+ salas; `None` = sem título.
    pub label: Option<String>,
    pub rooms: Vec<SavedRoomView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SavedRoomsView {
    pub groups: Vec<SavedRoomGroup>,
    /// "neste aparelho" ou "5 salas neste aparelho".
    pub note: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HomeView {
    pub nickname: String,
    pub link: String,
    /// Ex.: "Esse link não parece de uma sala."
    pub link_error: Option<String>,
    /// Falha ao criar a sala (rede, limite).
    pub create_error: Option<String>,
    /// Esperando o servidor criar a sala.
    pub creating: Option<bool>,
    /// Ausente = não mostra.
    pub app_invite: Option<AppInvite>,
    /// Aviso ao chegar no início (specs/014 US4): "O dono te tirou da sala".
    pub notice: Option<String>,
    /// Botão "Baixar o app" no topo (specs/013).
    pub download: Option<bool>,
    /// Suas salas. `None` = nenhuma neste aparelho → a ilustração de sempre.
    pub rooms: Option<SavedRoomsView>,
    /// Modal "Criar sala fixa" aberto (F7) ou "Renomear sala" de uma linha de Suas salas.
    pub fixed_dialog: Option<NameDialogView>,
    /// "Apagar sala" pelo "⋯" de Suas salas (dono).
    pub delete_dialog: Option<DeleteDialogView>,
}

/// Ações da tela inicial (`HomeActions`).
#[derive(Debug, Clone, PartialEq)]
pub enum HomeAction {
    CreateRoom { nickname: String },
    OpenLink { text: String },
    ClearLinkError,
    /// Clicou em "Baixar o app".
    DownloadApp,
    DismissAppInvite,
    /// Fechou o aviso do início (specs/014).
    DismissNotice,
    /// Sala fixa (specs/011).
    OpenFixedDialog,
    CloseFixedDialog,
    CreateFixedRoom { nickname: String, name: String },
    /// Suas salas (specs/011 US3).
    EnterSaved { code: String, nickname: String },
    ForgetSaved { code: String },
    /// Abre o "⋯" de uma linha (`None` fecha).
    ToggleSavedMenu { code: Option<String> },
    /// Dono (US4): renomear/apagar pelo "⋯" de Suas salas.
    RenameSaved { code: String },
    SubmitRename { name: String },
    DeleteSaved { code: String },
    ConfirmDelete,
    CloseDeleteDialog,
}

// ---------------------------------------------------------------------------
// Entrada
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct JoinPreviewPerson {
    pub name: String,
    pub tone: Tone,
    pub live: bool,
}

/// "Não existe" virou a tela de sala não encontrada (specs/011).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinStatus {
    Ready,
    Full,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinView {
    pub room_id: String,
    pub status: JoinStatus,
    /// `None` = não deu pra buscar a sala antes de entrar (esconde a pilha de avatares).
    pub people: Option<Vec<JoinPreviewPerson>>,
    pub capacity: u32,
    pub nickname: String,
    pub can_share: bool,
    /// Sala fixa (specs/011 F8): selo "Sala fixa" e o nome no título.
    pub fixed_name: Option<String>,
    /// Você abriu esta sala em outra aba: esta cedeu o lugar.
    pub replaced: Option<bool>,
}

/// Ações da tela de entrada (`JoinActions`).
#[derive(Debug, Clone, PartialEq)]
pub enum JoinAction {
    /// Logo (specs/013).
    GoHome,
    Join { nickname: String },
    CreateRoom,
    PasteOtherLink,
}

// ---------------------------------------------------------------------------
// Sala não encontrada
// ---------------------------------------------------------------------------

/// `Quick` = sala rápida que acabou ou link errado; `Kicked` = apagada com você dentro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotFoundReason {
    Quick,
    Deleted,
    Kicked,
    Expired,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotFoundView {
    pub reason: NotFoundReason,
    /// Nome conhecido neste aparelho (Suas salas).
    pub name: Option<String>,
    /// Está em Suas salas: a ação secundária vira "Tirar de Suas salas".
    pub saved: bool,
}

/// Ações da tela de sala não encontrada (`NotFoundActions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotFoundAction {
    /// Logo (specs/013).
    GoHome,
    CreateRoom,
    PasteOtherLink,
    Forget,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pessoa(id: &str, nome: &str, tone: Tone, you: bool, live: bool) -> ParticipantView {
        ParticipantView {
            id: id.into(),
            name: nome.into(),
            tone,
            you,
            live,
            live_label: if live { Some("Hollow Knight".into()) } else { None },
            removable: if you { None } else { Some(true) },
        }
    }

    fn sala_completa() -> RoomView {
        RoomView {
            requires_update: Some(false),
            room_id: "k7f-q2x".into(),
            invite_url: "https://espia.diuliano.com/k7f-q2x".into(),
            status: RoomStatus::Ready,
            participants: vec![
                pessoa("a1", "Ana", Tone::Deep, true, true),
                pessoa("b2", "Beto", Tone::Sage, false, true),
                pessoa("c3", "Cris", Tone::Sand, false, false),
            ],
            capacity: 10,
            streams: vec![StreamView {
                id: "b2".into(),
                name: "Beto".into(),
                tone: Tone::Sage,
                quality: Some("1080p · 60 fps".into()),
                caption: None,
                status: TileStatus::Ok,
                muted: false,
                volume: 0.8,
                stream: Some(StreamId("fluxo-video-b2".into())),
                audio_stream: Some(StreamId("fluxo-som-b2".into())),
                label: Some("Hollow Knight".into()),
                source: Some(LiveSource::Screen),
                watching: Some(true),
                watch_state: Some(StreamWatchState::Playing),
                you: None,
            }],
            focused_id: Some("b2".into()),
            my_broadcast: Some(MyBroadcastView {
                prefs: GoLive::default(),
                started_at: 1_762_860_000_000,
                adjusted: Some(false),
                viewers: 2,
                stream: Some(StreamId("fluxo-meu".into())),
                audio_kind: AudioKind::Pc,
                audio_reason_monitor: false,
                source_title: Some("Janela do jogo".into()),
                label: Some("Hollow Knight".into()),
                source: Some(LiveSource::Screen),
                snapshot: Some(SnapshotId("quadro-1".into())),
                on_stage: Some(false),
            }),
            copied: false,
            go_live_open: true,
            go_live_prefs: GoLive::default(),
            picker: PickerView {
                mode: PickerMode::Native,
                sources: vec![
                    CaptureSource {
                        id: "w:100".into(),
                        kind: ScreenSurface::Window,
                        title: "Hollow Knight".into(),
                        app: "hollow.exe".into(),
                        icon: Some("data:image/png;base64,AA==".into()),
                        thumb: None,
                        playing: true,
                        primary: false,
                    },
                    CaptureSource {
                        id: "m:1".into(),
                        kind: ScreenSurface::Monitor,
                        title: "Tela 1".into(),
                        app: String::new(),
                        icon: None,
                        thumb: Some("data:image/jpeg;base64,AA==".into()),
                        playing: false,
                        primary: true,
                    },
                ],
                loading: false,
                selected_id: Some("w:100".into()),
            },
            camera: CameraView {
                permission: CameraPermission::Granted,
                devices: vec![CameraDevice {
                    id: "cam1".into(),
                    label: "Webcam".into(),
                    kind: CameraDeviceKind::Camera,
                    preview: None,
                    busy: false,
                }],
                audios: vec![CameraAudio { id: "mic1".into(), label: "Microfone".into() }],
                video_id: Some("cam1".into()),
                audio_id: None,
                audio_role: AudioRole::Voice,
                error: Some(CameraError::Busy),
            },
            permission_denied: false,
            toasts: vec![ToastView {
                id: "t1".into(),
                kind: ToastKind::Live,
                text: "Beto começou a transmitir".into(),
                action: Some("Assistir".into()),
            }],
            can_share: true,
            can_share_screen: true,
            link: LinkState::Ok,
            room: RoomKind::Fixed { name: "Noite de jogo".into(), own: true },
            owner_menu: Some(true),
            name_dialog: Some(NameDialogView {
                mode: NameDialogMode::Rename,
                value: "Noite de jogo".into(),
                busy: false,
                error: None,
            }),
            delete_dialog: Some(DeleteDialogView { name: "Noite de jogo".into(), people: 2, busy: false, error: None }),
            remove_dialog: Some(RemoveDialogView { name: "Cris".into(), busy: false, error: None }),
        }
    }

    #[test]
    fn room_view_de_exemplo_completo() {
        let v = sala_completa();
        assert_eq!(v.clone(), v);
        assert_eq!(v.participants.len(), 3);
        assert!(v.participants[0].you && v.participants[0].removable.is_none());
        assert_eq!(v.streams[0].stream, Some(StreamId("fluxo-video-b2".into())));
        assert_eq!(v.room, RoomKind::Fixed { name: "Noite de jogo".into(), own: true });
        assert_eq!(v.my_broadcast.as_ref().unwrap().prefs.resolution, 1080);
        assert_eq!(v.picker.sources[1].kind, ScreenSurface::Monitor);
        let outra = RoomView { link: LinkState::Reconnecting, ..v.clone() };
        assert_ne!(outra, v);
    }

    #[test]
    fn room_view_minimo_da_sala_rapida() {
        let mut v = sala_completa();
        v.room = RoomKind::Quick;
        v.my_broadcast = None;
        v.name_dialog = None;
        v.delete_dialog = None;
        v.remove_dialog = None;
        v.owner_menu = None;
        v.status = RoomStatus::Connecting;
        assert_eq!(v.room, RoomKind::Quick);
        assert!(v.my_broadcast.is_none());
    }

    #[test]
    fn acoes_da_sala_carregam_os_argumentos() {
        let a = RoomAction::StartGoLive { prefs: GoLive { fps: 60, ..GoLive::default() } };
        assert_eq!(a.clone(), a);
        assert_ne!(RoomAction::Watch { id: "b2".into(), on: true }, RoomAction::Watch { id: "b2".into(), on: false });
        assert_ne!(RoomAction::Pip { id: None }, RoomAction::Pip { id: Some("b2".into()) });
        assert_eq!(RoomAction::SetSurface { surface: Surface::Camera }, RoomAction::SetSurface { surface: Surface::Camera });
        assert_ne!(RoomAction::SelectAudio { id: None }, RoomAction::SelectAudio { id: Some("mic1".into()) });
        let _ = RoomAction::SetVolume { id: "b2".into(), volume: 0.5 };
    }

    fn linha(code: &str, own: bool, state: SavedRoomState) -> SavedRoomView {
        SavedRoomView {
            code: code.into(),
            name: format!("Sala {code}"),
            own,
            tone: Tone::Accent,
            state,
            people: 3,
            streaming: 1,
            recent: "usada ontem".into(),
            menu_open: Some(false),
        }
    }

    #[test]
    fn home_view_com_suas_salas_e_acoes() {
        let v = HomeView {
            nickname: "Ana".into(),
            link: String::new(),
            link_error: Some("Esse link não parece de uma sala.".into()),
            create_error: None,
            creating: Some(false),
            app_invite: Some(AppInvite::Offer),
            notice: Some("O dono te tirou da sala".into()),
            download: Some(true),
            rooms: Some(SavedRoomsView {
                groups: vec![
                    SavedRoomGroup {
                        label: Some("Com gente agora".into()),
                        rooms: vec![linha("aaa-aaa", true, SavedRoomState::Ok)],
                    },
                    SavedRoomGroup { label: None, rooms: vec![linha("bbb-bbb", false, SavedRoomState::Gone)] },
                ],
                note: "2 salas neste aparelho".into(),
            }),
            fixed_dialog: Some(NameDialogView { mode: NameDialogMode::Create, value: String::new(), busy: false, error: None }),
            delete_dialog: None,
        };
        assert_eq!(v.clone(), v);
        assert_eq!(v.rooms.as_ref().unwrap().groups[1].rooms[0].state, SavedRoomState::Gone);
        assert_ne!(HomeAction::ToggleSavedMenu { code: None }, HomeAction::ToggleSavedMenu { code: Some("aaa-aaa".into()) });
        assert_eq!(
            HomeAction::EnterSaved { code: "aaa-aaa".into(), nickname: "Ana".into() },
            HomeAction::EnterSaved { code: "aaa-aaa".into(), nickname: "Ana".into() }
        );
        assert_ne!(HomeAction::DismissNotice, HomeAction::ClearLinkError);
        let sem_salas = HomeView { rooms: None, ..v.clone() };
        assert!(sem_salas.rooms.is_none());
    }

    #[test]
    fn join_view_e_acoes() {
        let v = JoinView {
            room_id: "k7f-q2x".into(),
            status: JoinStatus::Ready,
            people: Some(vec![JoinPreviewPerson { name: "Beto".into(), tone: Tone::Sage, live: true }]),
            capacity: 10,
            nickname: "Ana".into(),
            can_share: true,
            fixed_name: Some("Noite de jogo".into()),
            replaced: Some(false),
        };
        assert_eq!(v.clone(), v);
        let sem_previa = JoinView { people: None, status: JoinStatus::Full, ..v.clone() };
        assert!(sem_previa.people.is_none());
        assert_ne!(sem_previa, v);
        assert_eq!(JoinAction::Join { nickname: "Ana".into() }, JoinAction::Join { nickname: "Ana".into() });
        assert_ne!(JoinAction::GoHome, JoinAction::CreateRoom);
        let _ = JoinAction::PasteOtherLink;
    }

    #[test]
    fn not_found_view_e_acoes() {
        let v = NotFoundView { reason: NotFoundReason::Kicked, name: Some("Noite de jogo".into()), saved: true };
        assert_eq!(v.clone(), v);
        let rapida = NotFoundView { reason: NotFoundReason::Quick, name: None, saved: false };
        assert_ne!(rapida, v);
        for r in [NotFoundReason::Quick, NotFoundReason::Deleted, NotFoundReason::Kicked, NotFoundReason::Expired] {
            assert_eq!(r, r);
        }
        assert_ne!(NotFoundAction::Forget, NotFoundAction::GoHome);
        let _ = (NotFoundAction::CreateRoom, NotFoundAction::PasteOtherLink);
    }
}
