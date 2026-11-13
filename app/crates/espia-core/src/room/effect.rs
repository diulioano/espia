//! Efeitos da sala: tudo o que o redutor pede ao laço do binário, na ordem em que vieram. O redutor
//! não faz nada disso: só descreve.

use crate::device::{AudioRole, GoLive};
use crate::media_policy::TransmissionKey;
use crate::protocol::{ClientMessage, IceServer, SignalData};
use crate::quality::SendParams;
use crate::view_model::{CaptureSource, SnapshotId, StreamId};

use super::event::{TimerId, ViewMessage};
use super::state::{EndReason, RoomSession};

/// Pedido à API HTTP do dono (renomear, apagar, tirar da sala).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiCall {
    Rename { code: String, key: String, name: String },
    Delete { code: String, key: String },
    Remove { code: String, key: String, peer_id: String },
}

/// O que capturar.
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureTarget {
    /// Janela ou tela escolhida na grade.
    Source(CaptureSource),
    Camera { video_id: String, audio_id: Option<String>, audio_role: AudioRole },
}

/// O que gravar no aparelho (`device.json`).
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceChange {
    /// Preferências do seletor (`saveGoLivePrefs`).
    GoLivePrefs(GoLive),
    /// Volume daquele apelido (`saveVolume`).
    Volume { nickname: String, volume: f64 },
    /// Identidade para retomar o lugar; `None` esquece.
    Session { code: String, session: Option<RoomSession> },
    /// Entrou numa sala fixa: fica em Suas salas (`rememberRoom`). `at_ms`: quando.
    RememberRoom { code: String, name: String, at_ms: u64 },
    /// O dono renomeou a sala (`renameSaved`).
    RenameRoom { code: String, name: String },
    /// Tirar de Suas salas (`forgetRoom`).
    ForgetRoom { code: String },
    /// Esquecer a chave de dono (`forgetOwnerKey`).
    ForgetOwnerKey { code: String },
}

/// Para onde ir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    /// A tela inicial.
    Home,
    /// A sala acabou (apagada, removido, cheia, não achada, trocada de lugar, atualizar): a rota
    /// mostra a tela do motivo.
    Ended(EndReason),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    // ---- Servidor ----
    Send(ClientMessage),
    /// "Sair da sala": os outros recebem a saída na hora.
    Leave,
    /// "Tentar de novo": tenta já, sem esperar o intervalo da reconexão.
    RetryNow,
    Api(ApiCall),

    // ---- Ligações ----
    /// Abre a ligação com essa pessoa. `polite`: o meu id é menor que o dele.
    OpenPeer { id: String, initiator: bool, polite: bool },
    ClosePeer { id: String },
    /// Recomeça a ligação com essa pessoa ("Tentar de novo" do tile).
    RetryPeer { id: String },
    /// Entrega um sinal de negociação à ligação com `from`.
    Signal { from: String, data: SignalData },
    /// Servidores STUN para as ligações (só STUN).
    SetIceServers(Vec<IceServer>),
    /// Manda `view` pelo canal `ctl` dessa pessoa.
    SendView { to: String, message: ViewMessage },
    /// Manda `view-sync` pelo canal `ctl`: pede que ela diga de novo como vê a minha tela.
    RequestView { to: String, transmission: TransmissionKey },
    /// Parâmetros de envio de vídeo para essa pessoa. `sound`: o som também vai.
    SetSendParams { to: String, params: SendParams, sound: bool },
    /// Reduz na captura (alivia o processador de todos os envios de uma vez).
    SetCaptureQuality { resolution: u16, fps: u16 },

    // ---- Captura ----
    StartListingSources,
    StopListingSources,
    StartCapture { prefs: GoLive, target: CaptureTarget },
    /// Para a captura em andamento, ou cancela a que ainda está abrindo.
    StopCapture,
    /// Liga ou desliga os quadros parados da prévia (transmitindo e janela à vista).
    Snapshots { on: bool },
    ReleaseSnapshot(SnapshotId),
    /// Pede um identificador de pedido de transmissão novo (UUID v4).
    NewRequestId,
    QueryCameraPermission,
    RequestCameraPermission,
    ListCameras,
    /// Prévias de câmera que devem estar ligadas, a escolhida primeiro.
    OpenCameraPreviews { ids: Vec<String> },
    LeaveCameraTab,
    CloseStream(StreamId),

    // ---- Aparelho ----
    /// Copia o convite (sempre com o endereço oficial).
    CopyInvite(String),
    Save(DeviceChange),

    // ---- Tempo ----
    Schedule { timer: TimerId, after_ms: u64 },
    Cancel { timer: TimerId },

    // ---- Rota e tela ----
    Navigate(Destination),
    /// Pergunta "Você está transmitindo. Parar e sair da sala?" (`CONFIRM_LEAVE`). Se a pessoa
    /// confirmar, o laço devolve `RoomAction::Leave`.
    ConfirmLeave,
}
