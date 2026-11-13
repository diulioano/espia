//! Eventos da sala: tudo o que chega ao redutor. O laço do binário os produz (a rede, a mídia, a
//! captura, o aparelho, a tela e o relógio) e os entrega a `reduce`, um de cada vez.

use serde_json::{json, Value};

use crate::devices::DeviceInfo;
use crate::media_policy::{is_safe_sequence, TransmissionKey};
use crate::protocol::{AudioKind, ServerMessage, SignalData};
use crate::quality::ViewMode;
use crate::view_model::{CameraError, CameraPermission, CaptureSource, RoomAction, SnapshotId, StreamId};

use super::state::{AudioWarn, ConnState, EndReason, OwnerFailure, RoomSession};

/// Um temporizador nomeado. Agendar um que já existe o substitui.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimerId {
    /// Some o aviso com esse id (`TOAST_MS`).
    Toast(String),
    /// O "Link copiado" volta ao normal.
    Copied,
    /// Sem voltar em `RECONNECT_FAIL_MS`: a ligação vira "falhou".
    LinkDown,
    /// O vídeo dessa pessoa não chegou em `FIRST_FRAME_TIMEOUT_MS`.
    FirstFrame(String),
}

/// A mensagem `view` (versão 2) do canal `ctl`: como alguém vê a tela de outra pessoa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewMessage {
    pub transmission: TransmissionKey,
    pub revision: u64,
    pub mode: ViewMode,
}

/// Nome do modo na mensagem do canal `ctl` (`ViewMode` do site).
fn mode_name(mode: ViewMode) -> &'static str {
    match mode {
        ViewMode::Full => "full",
        ViewMode::Thumb => "thumb",
        ViewMode::Hidden => "hidden",
        ViewMode::Off => "off",
    }
}

fn parse_mode(name: &str) -> Option<ViewMode> {
    match name {
        "full" => Some(ViewMode::Full),
        "thumb" => Some(ViewMode::Thumb),
        "hidden" => Some(ViewMode::Hidden),
        "off" => Some(ViewMode::Off),
        _ => None,
    }
}

/// Inteiro seguro e não negativo, como o `isSafeSequence` do site (aceita `100.0`, como o JavaScript).
fn safe_number(v: &Value) -> Option<u64> {
    let n = v.as_u64().or_else(|| v.as_f64().filter(|f| f.fract() == 0.0 && *f >= 0.0 && *f <= 9.007_199_254_740_991e15).map(|f| f as u64))?;
    is_safe_sequence(n).then_some(n)
}

/// A chave da transmissão no JSON do `ctl`: `peerId` com 16 hexadecimais minúsculos.
fn decode_key(v: &Value) -> Option<TransmissionKey> {
    let peer_id = v.get("peerId")?.as_str()?;
    if peer_id.len() != 16 || !peer_id.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return None;
    }
    Some(TransmissionKey { peer_id: peer_id.to_string(), started_at: safe_number(v.get("startedAt")?)? })
}

fn key_json(key: &TransmissionKey) -> Value {
    json!({ "peerId": key.peer_id, "startedAt": key.started_at })
}

impl ViewMessage {
    /// O JSON que vai pelo canal `ctl` (`encodeView`): versão 2, nunca a 1.
    pub fn to_json(&self) -> Value {
        json!({ "t": "view", "version": 2, "transmission": key_json(&self.transmission), "revision": self.revision, "mode": mode_name(self.mode) })
    }

    /// Lê o que chegou pelo `ctl` (`decodeView`). Mensagem estranha, sem versão 2, com chave, revisão ou
    /// modo inválidos: `None`. Entrada externa inválida nunca autoriza mídia.
    pub fn decode(raw: &Value) -> Option<ViewMessage> {
        if raw.get("t")?.as_str()? != "view" || raw.get("version")?.as_u64()? != 2 {
            return None;
        }
        Some(ViewMessage {
            transmission: decode_key(raw.get("transmission")?)?,
            revision: safe_number(raw.get("revision")?)?,
            mode: parse_mode(raw.get("mode")?.as_str()?)?,
        })
    }
}

/// O JSON do pedido `view-sync` pelo canal `ctl`.
pub fn view_sync_json(transmission: &TransmissionKey) -> Value {
    json!({ "t": "view-sync", "version": 2, "transmission": key_json(transmission) })
}

/// Lê um `view-sync` (a chave da transmissão que a outra pessoa quer rever), ou `None`.
pub fn decode_view_sync(raw: &Value) -> Option<TransmissionKey> {
    if raw.get("t")?.as_str()? != "view-sync" || raw.get("version")?.as_u64()? != 2 {
        return None;
    }
    decode_key(raw.get("transmission")?)
}

/// A sinalização com o servidor (o `Signaling` do site).
#[derive(Debug, Clone, PartialEq)]
pub enum LinkEvent {
    Connecting,
    Open,
    /// Caiu: a sala continua na tela, com o aviso no palco.
    Reconnecting,
    /// Voltou depois de uma queda (o welcome seguinte traz o estado).
    Back,
    /// Identidade nova (ou `None` = esquecida) para retomar o lugar.
    Session(Option<RoomSession>),
    /// A sala acabou de vez, sem reconectar.
    Ended(EndReason),
}

/// O que o motor de mídia mediu num envio (uma pessoa com vídeo saindo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendReading {
    pub id: String,
    /// O motor está limitando o envio pelo processador (`qualityLimitationReason == 'cpu'`).
    pub cpu_limited: bool,
}

/// Da ligação com cada pessoa.
#[derive(Debug, Clone, PartialEq)]
pub enum PeerEvent {
    /// O estado da ligação mudou.
    State { id: String, conn: ConnState },
    /// O canal `ctl` abriu.
    ControlOpen { id: String },
    /// O canal `ctl` fechou.
    ControlClosed { id: String },
    /// Chegou uma `view` dessa pessoa (como ela vê a minha tela), já decodificada.
    View { id: String, message: ViewMessage },
    /// Chegou um `view-sync`: ela pede que eu diga de novo como vejo a tela dela.
    ViewSync { id: String, transmission: TransmissionKey },
    /// Chegou (ou mudou) o vídeo e o som dessa pessoa. `flowing`: quadros de vídeo chegando agora.
    Remote { id: String, stream: Option<StreamId>, audio: Option<StreamId>, flowing: bool },
    /// Os quadros de vídeo começaram ou pararam de chegar.
    VideoFlow { id: String, flowing: bool },
    /// Um sinal de negociação para mandar a essa pessoa.
    Signal { to: String, data: SignalData },
    /// Uma leitura de envio (uma por segundo enquanto há ligações).
    SendStats(Vec<SendReading>),
}

/// Por que a captura não começou.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartFailure {
    /// A janela escolhida fechou antes de começar.
    SourceGone,
    /// A câmera está sendo usada por outro programa.
    Busy,
    PermissionDenied,
    Other,
}

/// Da captura de tela, janela, câmera e som.
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureEvent {
    /// Lista de janelas e telas atualizada.
    Sources(Vec<CaptureSource>),
    /// A lista não deu para buscar.
    SourcesFailed,
    /// A captura pedida começou. `request_id` é um UUID v4 novo, para o pedido de transmissão.
    Started { request_id: String, stream: StreamId, audio_kind: AudioKind, audio_warn: Option<AudioWarn> },
    StartFailed(StartFailure),
    /// A captura acabou sozinha (janela fechada, câmera desplugada, botão do sistema).
    Ended,
    /// O som mudou no meio da transmissão (vigia do Discord).
    AudioChanged { kind: AudioKind, warn: Option<AudioWarn> },
    /// A tela está saindo preta.
    BlackScreen,
    /// Quadro parado novo da sua transmissão.
    Snapshot(SnapshotId),
    /// Identificador de pedido novo (resposta a `Effect::NewRequestId`).
    RequestId(String),
    /// Resposta à consulta da permissão da câmera.
    CameraPermission(CameraPermission),
    /// Resposta ao pedido de permissão da câmera.
    CameraAllowed { permission: CameraPermission, error: Option<CameraError> },
    /// Câmeras e entradas de som listadas.
    CameraDevices { videos: Vec<DeviceInfo>, audios: Vec<DeviceInfo> },
    /// Uma prévia de câmera abriu (`stream`) ou não (`None`); `busy`: em uso por outro programa.
    CameraPreview { id: String, stream: Option<StreamId>, busy: bool },
}

/// Respostas da API HTTP do dono.
#[derive(Debug, Clone, PartialEq)]
pub enum ApiEvent {
    Rename(Result<(), OwnerFailure>),
    Delete(Result<(), OwnerFailure>),
    Remove { peer_id: String, result: Result<(), OwnerFailure> },
}

/// Da janela do app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowEvent {
    /// Janela à vista (`true`) ou escondida: minimizada ou na bandeja (`false`). Nunca pelo foco.
    Visible(bool),
    /// A janela flutuante dessa tela abriu.
    FloatingOpened(String),
    FloatingClosed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Server(ServerMessage),
    Link(LinkEvent),
    Peer(PeerEvent),
    Capture(CaptureEvent),
    Api(ApiEvent),
    /// O que a pessoa fez na tela.
    Action(RoomAction),
    Window(WindowEvent),
    /// Um temporizador venceu.
    Timer(TimerId),
}
