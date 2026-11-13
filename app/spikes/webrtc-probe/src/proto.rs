//! Espelho em Rust das regras do site que a sonda precisa repetir:
//! `ViewMessage` versão 2 (`src/lib/peer.ts`: encodeView/decodeView), `sendParams` e `effectiveQuality`
//! (`src/lib/quality.ts`), `directIceServers` (`src/shared/media.ts`), `orderCodecs` (`peer.ts`).

use libwebrtc::peer_connection_factory::IceServer;
use libwebrtc::rtp_parameters::RtpCodecCapability;
use serde_json::{json, Value};

pub const MAX_SAFE: u64 = 9_007_199_254_740_991; // Number.MAX_SAFE_INTEGER

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewMode {
    Full,
    Thumb,
    Hidden,
    Off,
}

impl ViewMode {
    pub fn parse(s: &str) -> Option<ViewMode> {
        match s {
            "full" => Some(ViewMode::Full),
            "thumb" => Some(ViewMode::Thumb),
            "hidden" => Some(ViewMode::Hidden),
            "off" => Some(ViewMode::Off),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            ViewMode::Full => "full",
            ViewMode::Thumb => "thumb",
            ViewMode::Hidden => "hidden",
            ViewMode::Off => "off",
        }
    }
    /// `wantsVideo` de quality.ts.
    pub fn wants_video(self) -> bool {
        matches!(self, ViewMode::Full | ViewMode::Thumb)
    }
    /// `wantsSound` de quality.ts: só `off` corta o som.
    pub fn wants_sound(self) -> bool {
        self != ViewMode::Off
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransmissionKey {
    pub peer_id: String,
    pub started_at: u64,
}

impl TransmissionKey {
    pub fn to_json(&self) -> Value {
        json!({ "peerId": self.peer_id, "startedAt": self.started_at })
    }
}

/// `sameTransmission` de media.ts.
pub fn same_transmission(a: Option<&TransmissionKey>, b: Option<&TransmissionKey>) -> bool {
    matches!((a, b), (Some(a), Some(b)) if a == b)
}

#[derive(Clone, Debug)]
pub struct ViewMessage {
    pub transmission: TransmissionKey,
    pub revision: u64,
    pub mode: ViewMode,
}

/// `encodeView`: `{t:'view', version:2, transmission, revision, mode}`.
pub fn encode_view(mode: ViewMode, key: &TransmissionKey, revision: u64) -> String {
    json!({
        "t": "view",
        "version": 2,
        "transmission": key.to_json(),
        "revision": revision,
        "mode": mode.as_str(),
    })
    .to_string()
}

/// `{t:'view-sync', version:2, transmission}`: quem transmite pede a `view` atual de quem assiste.
pub fn encode_view_sync(key: &TransmissionKey) -> String {
    json!({ "t": "view-sync", "version": 2, "transmission": key.to_json() }).to_string()
}

fn safe_seq(v: &Value) -> Option<u64> {
    let n = v.as_u64()?;
    (n <= MAX_SAFE).then_some(n)
}

fn is_peer_id(s: &str) -> bool {
    s.len() == 16 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn decode_key(v: &Value) -> Option<TransmissionKey> {
    let o = v.as_object()?;
    let peer_id = o.get("peerId")?.as_str()?;
    if !is_peer_id(peer_id) {
        return None;
    }
    Some(TransmissionKey { peer_id: peer_id.to_string(), started_at: safe_seq(o.get("startedAt")?)? })
}

/// `decodeView`: mesma validação do site; qualquer desvio devolve `None` (e nunca autoriza mídia).
pub fn decode_view(v: &Value) -> Option<ViewMessage> {
    let o = v.as_object()?;
    if o.get("t")?.as_str()? != "view" || o.get("version")?.as_u64()? != 2 {
        return None;
    }
    let transmission = decode_key(o.get("transmission")?)?;
    let revision = safe_seq(o.get("revision")?)?;
    let mode = ViewMode::parse(o.get("mode")?.as_str()?)?;
    Some(ViewMessage { transmission, revision, mode })
}

/// Pedido `view-sync`: o site o valida com `decodeView({...raw, t:'view', revision:0, mode:'off'})`.
pub fn decode_view_sync(v: &Value) -> Option<TransmissionKey> {
    let o = v.as_object()?;
    if o.get("t")?.as_str()? != "view-sync" || o.get("version")?.as_u64()? != 2 {
        return None;
    }
    decode_key(o.get("transmission")?)
}

// ---- qualidade (quality.ts) ----

#[derive(Clone, Copy, Debug)]
pub struct Quality {
    pub resolution: u32,
    pub fps: u32,
    pub max_bitrate: u64,
}

/// `BITRATE` de quality.ts.
pub fn bitrate_for(resolution: u32, fps: u32) -> u64 {
    match (resolution, fps) {
        (720, 30) => 2_500_000,
        (720, 60) => 4_000_000,
        (1080, 30) => 4_500_000,
        _ => 6_000_000,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SendParams {
    pub active: bool,
    pub scale_resolution_down_by: f64,
    pub max_framerate: f64,
    pub max_bitrate: u64,
}

/// `sendParams` de quality.ts.
pub fn send_params(q: Quality, mode: ViewMode) -> SendParams {
    match mode {
        ViewMode::Hidden | ViewMode::Off => SendParams {
            active: false,
            scale_resolution_down_by: 1.0,
            max_framerate: q.fps as f64,
            max_bitrate: q.max_bitrate,
        },
        ViewMode::Thumb => SendParams {
            active: true,
            scale_resolution_down_by: q.resolution as f64 / 360.0,
            max_framerate: 15.0,
            max_bitrate: 500_000,
        },
        ViewMode::Full => SendParams {
            active: true,
            scale_resolution_down_by: 1.0,
            max_framerate: q.fps as f64,
            max_bitrate: q.max_bitrate,
        },
    }
}

/// Mudanças pedidas pela linha de comando, pelo teclado ou pelo ciclo, por cima de `send_params`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Overrides {
    pub bitrate: Option<u64>,
    pub fps: Option<f64>,
    pub scale: Option<f64>,
    pub active: Option<bool>,
}

impl Overrides {
    pub fn apply(&self, mut p: SendParams) -> SendParams {
        if let Some(v) = self.bitrate {
            p.max_bitrate = v;
        }
        if let Some(v) = self.fps {
            p.max_framerate = v;
        }
        if let Some(v) = self.scale {
            p.scale_resolution_down_by = v;
        }
        if let Some(v) = self.active {
            p.active = v;
        }
        p
    }
    pub fn describe(&self) -> String {
        let mut v = vec![];
        if let Some(x) = self.bitrate {
            v.push(format!("bitrate={x}"));
        }
        if let Some(x) = self.fps {
            v.push(format!("fps={x}"));
        }
        if let Some(x) = self.scale {
            v.push(format!("scale={x}"));
        }
        if let Some(x) = self.active {
            v.push(format!("active={x}"));
        }
        if v.is_empty() {
            "(nenhuma, só a regra do modo)".into()
        } else {
            v.join(" ")
        }
    }
}

/// Passos do `--cycle`: (rótulo, o que muda).
pub fn cycle_steps() -> Vec<(&'static str, Overrides)> {
    let none = Overrides::default();
    vec![
        ("cheio: 1080p60, 6 Mbps (regra do modo full)", none),
        ("miniatura: scale=3 (360p), fps=15, 0,5 Mbps", Overrides { bitrate: Some(500_000), fps: Some(15.0), scale: Some(3.0), active: None }),
        ("fps=30", Overrides { fps: Some(30.0), ..none }),
        ("bitrate=1 Mbps", Overrides { bitrate: Some(1_000_000), ..none }),
        ("scale=2 (540p)", Overrides { scale: Some(2.0), ..none }),
        ("active=false", Overrides { active: Some(false), ..none }),
    ]
}

// ---- ICE (media.ts) ----

/// `directIceServers`: só `stun:` e `stuns:`; servidor sem URL sobrando é descartado.
pub fn direct_ice_servers(servers: &Value) -> Vec<IceServer> {
    let Some(list) = servers.as_array() else { return vec![] };
    list.iter()
        .filter_map(|s| {
            let urls: Vec<String> = match s.get("urls")? {
                Value::String(u) => vec![u.clone()],
                Value::Array(a) => a.iter().filter_map(|u| u.as_str().map(str::to_string)).collect(),
                _ => return None,
            };
            let urls: Vec<String> = urls
                .into_iter()
                .filter(|u| {
                    let l = u.to_ascii_lowercase();
                    l.starts_with("stun:") || l.starts_with("stuns:")
                })
                .collect();
            (!urls.is_empty()).then(|| IceServer { urls, username: String::new(), password: String::new() })
        })
        .collect()
}

/// Candidato `typ relay` (`/\btyp relay\b/i`).
pub fn is_relay_candidate(c: &str) -> bool {
    let l = c.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = l[from..].find("typ relay") {
        let start = from + i;
        let end = start + "typ relay".len();
        let before_ok = start == 0 || !l.as_bytes()[start - 1].is_ascii_alphanumeric() && l.as_bytes()[start - 1] != b'_';
        let after_ok = end == l.len() || !l.as_bytes()[end].is_ascii_alphanumeric() && l.as_bytes()[end] != b'_';
        if before_ok && after_ok {
            return true;
        }
        from = end;
    }
    false
}

/// `orderCodecs`: H.264 com packetization-mode=1, depois os outros H.264, depois o resto (ordem estável).
pub fn order_codecs(codecs: Vec<RtpCodecCapability>) -> Vec<RtpCodecCapability> {
    let rank = |c: &RtpCodecCapability| -> u8 {
        if !c.mime_type.eq_ignore_ascii_case("video/h264") {
            2
        } else if c.sdp_fmtp_line.as_deref().unwrap_or("").contains("packetization-mode=1") {
            0
        } else {
            1
        }
    };
    let mut v: Vec<(usize, RtpCodecCapability)> = codecs.into_iter().enumerate().collect();
    v.sort_by_key(|(i, c)| (rank(c), *i));
    v.into_iter().map(|(_, c)| c).collect()
}

/// "a=fingerprint:sha-256 AB:CD:…" de uma SDP.
pub fn fingerprint_of(sdp: &str) -> Option<String> {
    sdp.lines().find_map(|l| l.trim().strip_prefix("a=fingerprint:").map(|s| s.trim().to_string()))
}

/// Origem (`esquema://host[:porta]`) de um endereço base.
pub fn origin_of(base: &str) -> String {
    let b = base.trim_end_matches('/');
    match b.find("://") {
        Some(i) => match b[i + 3..].find('/') {
            Some(j) => b[..i + 3 + j].to_string(),
            None => b.to_string(),
        },
        None => b.to_string(),
    }
}

/// Endereço do WebSocket da sala a partir do endereço base.
pub fn ws_url(base: &str, room: &str) -> String {
    let origin = origin_of(base);
    let ws = if let Some(rest) = origin.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = origin.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        origin
    };
    format!("{ws}/ws/{room}")
}
