//! Estado da sala: os campos de `RoomStore` (`src/lib/stores/room.svelte.ts`) sem rede, mídia nem
//! tela. Só o `reduce` muda este estado; `view()` o transforma no `RoomView` que a tela mostra.

use std::collections::{BTreeMap, BTreeSet};

use crate::cpu_governor::CpuGovernor;
use crate::device::{AudioRole, GoLive, Surface};
use crate::devices::DeviceInfo;
use crate::media_policy::TransmissionKey;
use crate::protocol::{AudioKind, IceServer, LiveSource, Peer, RoomInfo, CAPACITY};
use crate::quality::{count_full_viewers, effective_quality, Quality, ViewMode};
use crate::room_id::room_path;
use crate::view_model::{
    CameraError, CameraPermission, CaptureSource, DeleteDialogView, LinkState, NameDialogView, SnapshotId, StreamId,
    ToastView,
};
use crate::watch_state::Choices;

// ---------------------------------------------------------------------------
// Números e textos do site (mesmos, caractere por caractere)
// ---------------------------------------------------------------------------

/// Volume de quem ainda não teve volume guardado (`DEFAULT_VOLUME`).
pub const DEFAULT_VOLUME: f64 = 0.68;
/// Quanto um aviso fica na tela (`TOAST_MS`). Aviso de erro não some sozinho.
pub const TOAST_MS: u64 = 4_000;
/// Quanto o "Link copiado" fica marcado (o `setTimeout` de `copyInvite`).
pub const COPIED_MS: u64 = 2_000;
/// Sem quadro de vídeo nesse tempo depois de "começou a transmitir" → tile "falhou".
pub const FIRST_FRAME_TIMEOUT_MS: u64 = 10_000;
/// Sem voltar nesse tempo → "Não deu pra voltar pra sala" (specs/011 FR-004).
pub const RECONNECT_FAIL_MS: u64 = 60_000;
/// Endereço oficial do site: o convite do app é sempre este (`OFFICIAL_ORIGIN` de `site-origin.ts`).
pub const OFFICIAL_ORIGIN: &str = "https://espia.diuliano.com";
/// Aviso da transmissão com a tela preta (`BLACK_WARN`).
pub const BLACK_WARN: &str = "A tela tá saindo preta — tenta deixar o jogo em modo janela ou sem bordas.";
/// Pergunta ao sair da sala transmitindo (o `confirm` de `goHome`).
pub const CONFIRM_LEAVE: &str = "Você está transmitindo. Parar e sair da sala?";

/// Avisos do som no app desktop (`NATIVE_AUDIO_WARN`, specs/005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioWarn {
    Unsupported,
    ManyDiscords,
}

impl AudioWarn {
    pub fn text(self) -> &'static str {
        match self {
            AudioWarn::Unsupported => "Seu Windows não deixa separar o som da janela — foi só a imagem.",
            AudioWarn::ManyDiscords => "Mais de um Discord aberto — tela inteira vai sem som.",
        }
    }
}

/// Por que a sala acabou de vez, sem reconectar (`SignalingEnd`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndReason {
    NotFound,
    Full,
    /// Outra aba ou janela assumiu o lugar.
    Replaced,
    Kicked,
    /// O dono apagou a sala.
    Deleted,
    /// O dono tirou você da sala.
    Removed,
    /// O app precisa ser atualizado.
    Update,
}

/// Por que um pedido do dono (renomear, apagar, tirar) falhou (`OwnerFailure` de `api.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerFailure {
    Forbidden,
    Gone,
    Name,
    RateLimited,
    Network,
}

/// Mensagem do modal de renomear, apagar e tirar da sala (`ownerErrorText`).
pub fn owner_error_text(reason: OwnerFailure) -> &'static str {
    match reason {
        OwnerFailure::Forbidden => "Só quem criou a sala pode fazer isso.",
        OwnerFailure::Gone => "Essa sala não existe mais.",
        OwnerFailure::Name => "Esse nome não dá. Use letras, números e espaços.",
        OwnerFailure::RateLimited => "Muitas mudanças daqui agora. Tenta de novo em um minuto.",
        OwnerFailure::Network => "Não deu pra falar com o servidor. Confere a internet e tenta de novo.",
    }
}

// ---------------------------------------------------------------------------
// Pedaços do estado
// ---------------------------------------------------------------------------

/// Identidade para retomar o lugar (`RoomSession`): quem guarda no disco é o aparelho.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomSession {
    pub peer_id: String,
    pub resume_token: String,
}

/// Estado da ligação com uma pessoa (`RTCPeerConnectionState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
    Closed,
}

/// Vídeo e ligação de cada pessoa (`RemoteMedia`).
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteMedia {
    pub stream: Option<StreamId>,
    /// Só o som da pessoa (specs/014): toca à parte do vídeo.
    pub audio: Option<StreamId>,
    /// Chegando quadros de vídeo agora, de quem escolhi assistir.
    pub video: bool,
    pub conn: ConnState,
}

impl Default for RemoteMedia {
    fn default() -> Self {
        RemoteMedia { stream: None, audio: None, video: false, conn: ConnState::New }
    }
}

/// O lado da sala de cada ligação (o que o `PeerLink` do site guarda e que muda o que sai e entra).
#[derive(Debug, Clone, PartialEq)]
pub struct PeerLinkState {
    /// O canal de controle `ctl` está aberto.
    pub ctl_open: bool,
    /// Transmissão atual dessa pessoa, se ela transmite.
    pub remote_key: Option<TransmissionKey>,
    /// Como eu vejo a tela dela (o que já mandei). `None` = ainda não mandei nada.
    pub my_view: Option<ViewMode>,
    /// Numeração das mensagens `view` que mando a ela.
    pub view_revision: u64,
    /// Como ela vê a minha tela (ela me manda). Sem autorização válida, nada sai: `Off`.
    pub their_view: ViewMode,
    /// Última revisão aceita de quem recebe a minha tela; -1 = nenhuma.
    pub grant_revision: i64,
}

impl PeerLinkState {
    pub fn new(remote_key: Option<TransmissionKey>) -> Self {
        PeerLinkState {
            ctl_open: false,
            remote_key,
            my_view: None,
            view_revision: 0,
            their_view: ViewMode::Off,
            grant_revision: -1,
        }
    }
}

/// Mudo e volume de quem transmite (`audio`). Começa mudo (autoplay).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioPref {
    pub muted: bool,
    pub volume: f64,
}

/// Pedido de transmissão em andamento (`captureRequest`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveRequest {
    pub request_id: String,
    pub broadcast_seq: u64,
}

/// Sua transmissão (`MyBroadcastView` sem os campos derivados).
#[derive(Debug, Clone, PartialEq)]
pub struct MyBroadcast {
    pub prefs: GoLive,
    pub started_at: u64,
    pub stream: Option<StreamId>,
    pub audio_kind: AudioKind,
    pub audio_reason_monitor: bool,
    pub source_title: Option<String>,
    pub label: Option<String>,
    pub source: Option<LiveSource>,
}

/// A captura foi pedida e ainda não respondeu: guarda o que a transmissão vai mostrar.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingStart {
    pub prefs: GoLive,
    pub source: Option<LiveSource>,
    pub source_title: Option<String>,
    pub label: Option<String>,
}

/// "Tirar da sala" (specs/014 US4): quem e o estado do pedido.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoveDialog {
    pub peer_id: String,
    pub name: String,
    pub busy: bool,
    pub error: Option<String>,
}

/// `CpuGovernor` não compara; aqui compara pelo texto de `Debug` (todos os campos entram nele).
#[derive(Debug, Clone, Default)]
pub struct CpuState(pub CpuGovernor);

impl PartialEq for CpuState {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self.0) == format!("{:?}", other.0)
    }
}

// ---------------------------------------------------------------------------
// Estado
// ---------------------------------------------------------------------------

/// O que o aparelho já sabe ao abrir a sala (`loadGoLivePrefs`, `loadOwnerKey`, volumes guardados).
#[derive(Debug, Clone, PartialEq)]
pub struct RoomInit {
    pub code: String,
    pub go_live_prefs: GoLive,
    /// Chave de dono desta sala neste aparelho.
    pub owner_key: Option<String>,
    /// Volumes guardados: apelido em minúsculas → 0..1.
    pub volumes: BTreeMap<String, f64>,
    /// Acabou de criar a sala fixa (specs/011 S1): copia o convite e avisa no primeiro welcome.
    pub just_created: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomState {
    // ---- Ligação com o servidor ----
    pub code: String,
    pub connected: bool,
    pub ended: Option<EndReason>,
    /// Saí da sala (botão, voltar, apaguei): nada mais vale.
    pub left: bool,
    pub link: LinkState,
    pub session: Option<RoomSession>,

    // ---- Sala e pessoas ----
    pub room_info: RoomInfo,
    pub capacity: u32,
    pub me: Option<Peer>,
    pub peers: Vec<Peer>,
    /// Servidores STUN em uso (só STUN: o app nunca usa TURN).
    pub ice_servers: Vec<IceServer>,
    /// Id com que as ligações foram abertas; `None` = nenhuma ligação aberta (`mesh`).
    pub mesh_id: Option<String>,
    pub owner_key: Option<String>,
    pub just_created: bool,

    // ---- O que eu vejo ----
    pub focused_id: Option<String>,
    /// Quem escolhi assistir, por transmissão (`watching`).
    pub watching: Choices,
    /// Quem parei de assistir → início daquela transmissão (`stopped`).
    pub stopped: BTreeMap<String, u64>,
    /// Último modo mandado a cada pessoa (`sentModes`).
    pub sent_modes: BTreeMap<String, ViewMode>,
    pub media: BTreeMap<String, RemoteMedia>,
    pub links: BTreeMap<String, PeerLinkState>,
    /// Começou a transmitir e o vídeo não chegou a tempo (`timedOut`).
    pub timed_out: BTreeSet<String>,
    /// Pessoas com o temporizador do primeiro quadro correndo.
    pub frame_timers: BTreeSet<String>,
    pub audio: BTreeMap<String, AudioPref>,
    /// Volumes guardados no aparelho: apelido em minúsculas → 0..1.
    pub volumes: BTreeMap<String, f64>,
    /// Janela à vista (specs/014 US2): minimizada ou na bandeja = false. Nunca pelo foco.
    pub visible: bool,
    /// Tela na janela flutuante (no site, picture-in-picture): continua chegando com a janela escondida.
    pub floating_id: Option<String>,
    /// A sua tela grande no palco, ao vivo (specs/014 US3).
    pub self_on_stage: bool,
    pub snapshot: Option<SnapshotId>,
    /// Os quadros parados da prévia estão ligados (transmitindo e janela à vista).
    pub snapshots_on: bool,

    // ---- O que eu transmito ----
    pub my_broadcast: Option<MyBroadcast>,
    pub pending_start: Option<PendingStart>,
    pub capture_request: Option<LiveRequest>,
    /// Maior sequência de transmissão vista na sala; -1 = nenhuma (`highestBroadcastSeq`).
    pub highest_broadcast_seq: i64,
    /// Chave da minha transmissão atual (`mesh.setLocalKey`).
    pub local_key: Option<TransmissionKey>,
    /// Última qualidade aplicada na captura e anunciada (`appliedQuality`).
    pub applied_quality: Option<Quality>,
    /// Degraus baixados porque o PC não dá conta de comprimir (specs/010).
    pub cpu_steps: u32,
    pub governor: CpuState,
    /// Identificador de pedido de reserva que o laço entregou; o redutor não sorteia.
    pub spare_request_id: Option<String>,
    /// Já pedi um identificador novo e ele ainda não chegou.
    pub request_id_asked: bool,
    /// Preciso anunciar a transmissão assim que chegar um identificador.
    pub live_pending: bool,
    pub go_live_prefs: GoLive,

    // ---- Seletor de transmissão ----
    pub go_live_open: bool,
    pub tab: Surface,
    pub sources: Vec<CaptureSource>,
    pub sources_loading: bool,
    pub selected_id: Option<String>,
    /// A lista de janelas está sendo atualizada (`stopPoll != null`).
    pub listing: bool,
    /// Falta a primeira lista (volta a última fonte).
    pub first_list: bool,
    pub permission_denied: bool,
    pub cam_permission: CameraPermission,
    pub cam_error: Option<CameraError>,
    pub cam_videos: Vec<DeviceInfo>,
    pub cam_audios: Vec<DeviceInfo>,
    pub cam_previews: BTreeMap<String, StreamId>,
    pub cam_busy: BTreeSet<String>,
    pub cam_video_id: Option<String>,
    pub cam_audio_id: Option<String>,
    pub cam_audio_role: AudioRole,
    /// A pessoa trocou o som ou o perfil nesta abertura: não sugerir por cima.
    pub cam_audio_manual: bool,
    pub cam_role_manual: bool,
    /// A aba Câmeras está aberta (prévias de uma abertura antiga são descartadas).
    pub cam_active: bool,
    /// A aba Câmeras já foi aberta alguma vez (`camGen > 0`).
    pub cam_entered: bool,

    // ---- Avisos e diálogos ----
    pub toasts: Vec<ToastView>,
    pub toast_seq: u64,
    /// Aviso "começou a transmitir" → quem focar no "Assistir".
    pub toast_peer: BTreeMap<String, String>,
    pub copied: bool,
    pub owner_menu: bool,
    pub name_dialog: Option<NameDialogView>,
    pub delete_dialog: Option<DeleteDialogView>,
    pub remove_dialog: Option<RemoveDialog>,
    /// Confirmou "Apagar sala": o fechamento 4410 que vem em seguida não é surpresa.
    pub deleting: bool,
}

impl RoomState {
    pub fn new(init: RoomInit) -> Self {
        RoomState {
            code: init.code,
            connected: false,
            ended: None,
            left: false,
            link: LinkState::Ok,
            session: None,
            room_info: RoomInfo::Quick,
            capacity: CAPACITY,
            me: None,
            peers: Vec::new(),
            ice_servers: Vec::new(),
            mesh_id: None,
            owner_key: init.owner_key,
            just_created: init.just_created,
            focused_id: None,
            watching: Choices::new(),
            stopped: BTreeMap::new(),
            sent_modes: BTreeMap::new(),
            media: BTreeMap::new(),
            links: BTreeMap::new(),
            timed_out: BTreeSet::new(),
            frame_timers: BTreeSet::new(),
            audio: BTreeMap::new(),
            volumes: init.volumes,
            visible: true,
            floating_id: None,
            self_on_stage: false,
            snapshot: None,
            snapshots_on: false,
            my_broadcast: None,
            pending_start: None,
            capture_request: None,
            highest_broadcast_seq: -1,
            local_key: None,
            applied_quality: None,
            cpu_steps: 0,
            governor: CpuState::default(),
            spare_request_id: None,
            request_id_asked: false,
            live_pending: false,
            go_live_prefs: init.go_live_prefs,
            go_live_open: false,
            tab: Surface::Window,
            sources: Vec::new(),
            sources_loading: false,
            selected_id: None,
            listing: false,
            first_list: false,
            permission_denied: false,
            cam_permission: CameraPermission::Unknown,
            cam_error: None,
            cam_videos: Vec::new(),
            cam_audios: Vec::new(),
            cam_previews: BTreeMap::new(),
            cam_busy: BTreeSet::new(),
            cam_video_id: None,
            cam_audio_id: None,
            cam_audio_role: AudioRole::Voice,
            cam_audio_manual: false,
            cam_role_manual: false,
            cam_active: false,
            cam_entered: false,
            toasts: Vec::new(),
            toast_seq: 0,
            toast_peer: BTreeMap::new(),
            copied: false,
            owner_menu: false,
            name_dialog: None,
            delete_dialog: None,
            remove_dialog: None,
            deleting: false,
        }
    }

    /// Convite da sala: sempre o endereço oficial (specs/019 US1).
    pub fn invite_url(&self) -> String {
        format!("{OFFICIAL_ORIGIN}{}", room_path(&self.code))
    }

    pub fn broadcasting(&self) -> bool {
        self.my_broadcast.is_some()
    }

    pub fn peer(&self, id: &str) -> Option<&Peer> {
        self.peers.iter().find(|p| p.id == id)
    }

    /// Como cada pessoa conectada está vendo a minha tela (`mesh.theirViews`): só ligação de pé.
    pub fn their_views(&self) -> BTreeMap<String, ViewMode> {
        self.links
            .iter()
            .filter(|(id, _)| self.media.get(*id).is_some_and(|m| m.conn == ConnState::Connected))
            .map(|(id, l)| (id.clone(), l.their_view))
            .collect()
    }

    /// Qualidade efetiva da minha transmissão: o escolhido, com teto pelo número de espectadores
    /// cheios e pelo processador.
    pub fn quality(&self) -> Option<Quality> {
        self.my_broadcast.as_ref().map(|b| {
            effective_quality(&b.prefs, count_full_viewers(self.their_views().into_values()), self.cpu_steps)
        })
    }

    /// Transmissão atual de uma pessoa, pelo que o servidor disse.
    pub fn remote_key(&self, id: &str) -> Option<TransmissionKey> {
        let p = self.peer(id)?;
        let live = p.live.as_ref()?;
        Some(TransmissionKey { peer_id: p.id.clone(), started_at: live.started_at })
    }

    /// Volume guardado para o apelido (`loadVolume`).
    pub fn saved_volume(&self, nickname: &str) -> Option<f64> {
        self.volumes.get(&nickname.to_lowercase()).copied()
    }
}
