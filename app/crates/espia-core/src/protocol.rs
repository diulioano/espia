//! Protocolo de sinalização (espelha `src/shared/protocol.ts`). O app nativo é só mais um cliente do
//! mesmo servidor: nenhuma mensagem nova, nenhum campo novo. As fixtures de `app/fixtures/protocol/`
//! são conferidas aqui (`tests/protocol_fixtures.rs`) e no site (`test/shared/protocol-fixtures.test.ts`).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::media_policy::{BroadcastOrder, MediaPolicy};

/// Pessoas por sala.
pub const CAPACITY: u32 = 10;
/// Apelido: até 24 caracteres (depois da limpeza do servidor).
pub const NICKNAME_MAX: usize = 24;
/// Mensagem do WebSocket: até 64 KiB.
pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;
/// Texto do que está sendo transmitido: até 60 caracteres.
pub const LIVE_LABEL_MAX: usize = 60;
/// Nome de sala fixa: até 40 caracteres.
pub const ROOM_NAME_MAX: usize = 40;
/// Salas fixas que cada aparelho lembra.
pub const SAVED_ROOMS_MAX: usize = 5;

/// Keep-alive: tem de ser exatamente esta string (o objeto Durable responde sem acordar).
pub const PING: &str = r#"{"t":"ping"}"#;
pub const PONG: &str = r#"{"t":"pong"}"#;

/// Códigos de fechamento do WebSocket (`CLOSE`).
pub mod close {
    pub const UPDATE: u16 = 4426;
    pub const POLICY: u16 = 1008;
    pub const NOT_FOUND: u16 = 4404;
    pub const FULL: u16 = 4409;
    /// Outra conexão retomou a mesma identidade.
    pub const REPLACED: u16 = 4000;
    /// Saída voluntária: não espera a tolerância de 60 s.
    pub const LEAVE: u16 = 4001;
    /// O dono apagou a sala fixa: não reconecta.
    pub const DELETED: u16 = 4410;
    /// O dono tirou você da sala: não reconecta.
    pub const REMOVED: u16 = 4403;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Accent,
    Sage,
    Sand,
    Deep,
}

/// Som que sai na transmissão (`AudioKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioKind {
    Tab,
    Window,
    Pc,
    Device,
    None,
}

/// O que está sendo transmitido (`LiveSource`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LiveSource {
    Screen,
    Camera,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveInfo {
    /// 720 ou 1080.
    pub resolution: u16,
    /// 30 ou 60.
    pub fps: u8,
    pub audio: AudioKind,
    pub source: LiveSource,
    pub started_at: u64,
    /// Ausente = cliente antigo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub nickname: String,
    pub tone: Tone,
    /// Sempre presente no JSON; `null` quando a pessoa não transmite.
    pub live: Option<LiveInfo>,
}

/// `urls` do RTCIceServer: uma string ou uma lista.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Urls {
    One(String),
    Many(Vec<String>),
}

impl Urls {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Urls::One(u) => vec![u.clone()],
            Urls::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Urls,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

/// Tipo da sala: rápida (some vazia) ou fixa (com nome).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RoomInfo {
    Quick,
    Fixed { name: String },
}

/// `RTCSessionDescriptionInit`: o servidor só repassa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdpDescription {
    /// `offer`, `answer`, `pranswer` ou `rollback`.
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdp: Option<String>,
}

/// SDP ou candidato ICE; `candidate: null` marca o fim dos candidatos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalData {
    Description { description: SdpDescription },
    Candidate { candidate: Option<Map<String, Value>> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum ClientMessage {
    /// `media_protocol` é sempre 2 (`MEDIA_PROTOCOL`).
    Join { nickname: String, media_protocol: u32 },
    Resume { peer_id: String, resume_token: String, media_protocol: u32 },
    Signal { to: String, data: SignalData },
    Live {
        request_id: String,
        broadcast_seq: u64,
        resolution: u16,
        fps: u8,
        audio: AudioKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<LiveSource>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    Unlive { request_id: String, broadcast_seq: u64 },
    TurnRefresh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    BadMessage,
    NotJoined,
    UnknownPeer,
    RateLimited,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum ServerMessage {
    Welcome {
        media_protocol: u32,
        media_policy: MediaPolicy,
        /// Sempre presente no JSON; `null` quando a sala ainda não teve transmissão.
        broadcast_order: Option<BroadcastOrder>,
        you: Peer,
        peers: Vec<Peer>,
        ice_servers: Vec<IceServer>,
        ice_expires_at: u64,
        resume_token: String,
        capacity: u32,
        /// Servidor antigo não manda: trate como sala rápida.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        room: Option<RoomInfo>,
    },
    LiveAck { request_id: String, broadcast_seq: u64, live: LiveInfo },
    RoomNotFound,
    RoomFull { capacity: u32 },
    ResumeFailed,
    PeerJoined { peer: Peer },
    PeerLeft { id: String },
    PeerLive { id: String, live: Option<LiveInfo> },
    Signal { from: String, data: SignalData },
    IceServers { ice_servers: Vec<IceServer>, ice_expires_at: u64 },
    Error { code: ErrorCode, message: String },
    RoomRenamed { name: String },
    RoomDeleted,
    Removed,
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media_policy::{BroadcastStatus, DIRECT_ONLY, MEDIA_PROTOCOL};
    use serde_json::json;

    fn ida_e_volta<T: Serialize + for<'de> Deserialize<'de>>(texto: &str) -> Value {
        let msg: T = serde_json::from_str(texto).expect("decodifica");
        serde_json::to_value(&msg).unwrap()
    }

    #[test]
    fn constantes_do_site() {
        assert_eq!(PING, r#"{"t":"ping"}"#);
        assert_eq!(PONG, r#"{"t":"pong"}"#);
        assert_eq!(close::UPDATE, 4426);
        assert_eq!(close::POLICY, 1008);
        assert_eq!(close::NOT_FOUND, 4404);
        assert_eq!(close::FULL, 4409);
        assert_eq!(close::REPLACED, 4000);
        assert_eq!(close::LEAVE, 4001);
        assert_eq!(close::DELETED, 4410);
        assert_eq!(close::REMOVED, 4403);
        assert_eq!(NICKNAME_MAX, 24);
        assert_eq!(LIVE_LABEL_MAX, 60);
        assert_eq!(SAVED_ROOMS_MAX, 5);
        assert_eq!(ROOM_NAME_MAX, 40);
        assert_eq!(CAPACITY, 10);
        assert_eq!(MAX_MESSAGE_BYTES, 65536);
    }

    #[test]
    fn join_usa_t_e_camel_case() {
        let m = ClientMessage::Join { nickname: "Ana".into(), media_protocol: MEDIA_PROTOCOL };
        assert_eq!(serde_json::to_string(&m).unwrap(), r#"{"t":"join","nickname":"Ana","mediaProtocol":2}"#);
    }

    #[test]
    fn turn_refresh_e_so_o_tipo() {
        assert_eq!(serde_json::to_string(&ClientMessage::TurnRefresh).unwrap(), r#"{"t":"turn-refresh"}"#);
        assert_eq!(serde_json::to_string(&ServerMessage::Pong).unwrap(), PONG);
    }

    #[test]
    fn opcionais_ausentes_ficam_de_fora() {
        let live = ClientMessage::Live {
            request_id: "00000000-0000-4000-8000-000000000001".into(),
            broadcast_seq: 0,
            resolution: 720,
            fps: 30,
            audio: AudioKind::None,
            source: None,
            label: None,
        };
        let v = serde_json::to_value(&live).unwrap();
        assert!(v.get("source").is_none() && v.get("label").is_none());
        // sem source nem label também decodifica (cliente antigo)
        let de: ClientMessage = serde_json::from_value(v).unwrap();
        assert_eq!(de, live);

        let info = LiveInfo { resolution: 1080, fps: 60, audio: AudioKind::Pc, source: LiveSource::Screen, started_at: 5, label: None };
        assert!(serde_json::to_value(&info).unwrap().get("label").is_none());
        let ice = IceServer { urls: Urls::One("stun:a".into()), username: None, credential: None };
        assert_eq!(serde_json::to_string(&ice).unwrap(), r#"{"urls":"stun:a"}"#);
    }

    #[test]
    fn welcome_de_servidor_antigo_sem_room_e_sem_ordem() {
        let texto = r#"{"t":"welcome","mediaProtocol":2,"mediaPolicy":{"kind":"direct-only","reason":"exhausted"},
            "broadcastOrder":null,"you":{"id":"a1b2c3d4e5f60718","nickname":"Ana","tone":"deep","live":null},
            "peers":[],"iceServers":[],"iceExpiresAt":1,"resumeToken":"x","capacity":10}"#;
        let m: ServerMessage = serde_json::from_str(texto).unwrap();
        match &m {
            ServerMessage::Welcome { room, broadcast_order, media_policy, .. } => {
                assert!(room.is_none());
                assert!(broadcast_order.is_none());
                assert_ne!(*media_policy, DIRECT_ONLY);
            }
            _ => panic!("não é welcome"),
        }
        let v = serde_json::to_value(&m).unwrap();
        assert!(v.get("room").is_none());
        assert_eq!(v["broadcastOrder"], Value::Null);
    }

    #[test]
    fn welcome_com_ordem_e_sala_fixa() {
        let v = json!({"t":"welcome","mediaProtocol":2,"mediaPolicy":{"kind":"direct-only","reason":"unverified-budget"},
            "broadcastOrder":{"highestBroadcastSeq":3,"requestId":"00000000-0000-4000-8000-000000000001","lastStartedAt":9,"status":"ended"},
            "you":{"id":"a","nickname":"Ana","tone":"sand","live":null},"peers":[],"iceServers":[],"iceExpiresAt":1,
            "resumeToken":"x","capacity":10,"room":{"kind":"fixed","name":"Turma"}});
        let m: ServerMessage = serde_json::from_value(v.clone()).unwrap();
        if let ServerMessage::Welcome { broadcast_order: Some(o), room: Some(r), .. } = &m {
            assert_eq!(o.status, BroadcastStatus::Ended);
            assert_eq!(*r, RoomInfo::Fixed { name: "Turma".into() });
        } else {
            panic!("welcome sem ordem ou sala");
        }
        assert_eq!(serde_json::to_value(&m).unwrap(), v);
        let q: RoomInfo = serde_json::from_str(r#"{"kind":"quick"}"#).unwrap();
        assert_eq!(q, RoomInfo::Quick);
    }

    #[test]
    fn peer_live_nulo_continua_nulo() {
        let v = ida_e_volta::<ServerMessage>(r#"{"t":"peer-live","id":"a","live":null}"#);
        assert_eq!(v, json!({"t":"peer-live","id":"a","live":null}));
    }

    #[test]
    fn sinal_descricao_candidato_e_fim_de_candidatos() {
        let desc = r#"{"t":"signal","to":"a","data":{"description":{"type":"answer"}}}"#;
        assert_eq!(ida_e_volta::<ClientMessage>(desc), serde_json::from_str::<Value>(desc).unwrap());
        let fim = r#"{"t":"signal","from":"a","data":{"candidate":null}}"#;
        assert_eq!(ida_e_volta::<ServerMessage>(fim), serde_json::from_str::<Value>(fim).unwrap());
        let m: ServerMessage = serde_json::from_str(fim).unwrap();
        assert!(matches!(m, ServerMessage::Signal { data: SignalData::Candidate { candidate: None }, .. }));
    }

    #[test]
    fn urls_como_texto_ou_lista() {
        let a: IceServer = serde_json::from_str(r#"{"urls":"stun:a"}"#).unwrap();
        let b: IceServer = serde_json::from_str(r#"{"urls":["stun:a","stun:b"],"username":"u","credential":"c"}"#).unwrap();
        assert_eq!(a.urls.to_vec(), vec!["stun:a"]);
        assert_eq!(b.urls.to_vec(), vec!["stun:a", "stun:b"]);
        assert_eq!(b.username.as_deref(), Some("u"));
    }

    #[test]
    fn rejeita_tipo_desconhecido_e_valores_fora_do_conjunto() {
        assert!(serde_json::from_str::<ClientMessage>(r#"{"t":"ping"}"#).is_err());
        assert!(serde_json::from_str::<ServerMessage>(r#"{"t":"nada"}"#).is_err());
        assert!(serde_json::from_str::<ServerMessage>(r#"{"t":"error","code":"outro","message":"x"}"#).is_err());
        assert!(serde_json::from_str::<Peer>(r#"{"id":"a","nickname":"A","tone":"azul","live":null}"#).is_err());
    }
}
