//! Sonda G1 (019, T009): entra numa sala do site do Espia como se fosse mais um cliente, com a crate
//! `libwebrtc` 0.4.0 da LiveKit. Veja LEIA-ME.md para os comandos dos itens (a) a (j) do T010.
//!
//! Origem das regras (só leitura): src/lib/peer.ts, src/lib/mesh.ts, src/lib/quality.ts,
//! src/lib/signaling.ts, src/shared/protocol.ts, src/shared/media.ts, src/worker/index.ts.

mod args;
mod media;
mod metrics;
mod proto;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use libwebrtc::data_channel::{DataChannel, DataChannelInit, DataChannelState};
use libwebrtc::media_stream_track::MediaStreamTrack;
use libwebrtc::peer_connection::{
    AnswerOptions, IceConnectionState, OfferOptions, PeerConnection, PeerConnectionState, SignalingState, TrackEvent,
};
use libwebrtc::peer_connection_factory::native::PeerConnectionFactoryExt;
use libwebrtc::peer_connection_factory::{IceServer, PeerConnectionFactory, RtcConfiguration};
use libwebrtc::rtp_parameters::{DegradationPreference, RtpEncodingParameters};
use libwebrtc::rtp_sender::VideoEncoderBackend;
use libwebrtc::rtp_transceiver::{RtpTransceiver, RtpTransceiverDirection, RtpTransceiverInit};
use libwebrtc::session_description::{SdpType, SessionDescription};
use libwebrtc::stats::RtcStats;
use libwebrtc::video_source::native::NativeVideoSource;
use libwebrtc::video_source::VideoResolution;
use libwebrtc::video_track::RtcVideoTrack;
use libwebrtc::MediaType;
use serde_json::{json, Value};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;

use crate::args::Args;
use crate::media::RxCounters;
use crate::proto::*;

const PING: &str = r#"{"t":"ping"}"#;
const AUDIO_BITRATE: u64 = 128_000;
const STREAM_ID: &str = "espia-sonda";

/// Eventos que chegam ao laço principal. As chamadas de volta da libwebrtc só enfileiram aqui, nunca
/// fazem trabalho: elas rodam em threads do motor.
enum PcEv {
    Candidate { mid: String, idx: i32, cand: String },
    NegotiationNeeded,
    Track(TrackEvent),
    DataChannel(DataChannel),
    DcState(DataChannelState),
    DcMsg(String),
    Conn(PeerConnectionState),
    Ice(IceConnectionState),
    Signaling(SignalingState),
}

enum Ev {
    Ws(String),
    WsClosed(Option<u16>, String),
    Pc(String, u64, PcEv),
    Line(String),
    StatsTick,
    PingTick,
    CycleTick,
    MinuteTick,
    CtrlC,
}

macro_rules! log {
    ($($a:tt)*) => {{ println!("[{:>8.1}s] {}", crate::elapsed(), format!($($a)*)); }};
}

static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
fn elapsed() -> f64 {
    START.get_or_init(Instant::now).elapsed().as_secs_f64()
}

struct Link {
    gen: u64,
    pc: PeerConnection,
    polite: bool,
    ignore_offer: bool,
    ctl: Option<DataChannel>,
    ctl_open: bool,
    /// Como eu vejo a tela dessa pessoa (mando para ela).
    my_view: Option<ViewMode>,
    /// Como essa pessoa vê a minha tela (ela me manda).
    their_view: ViewMode,
    grant_revision: i64,
    view_revision: u64,
    remote_key: Option<TransmissionKey>,
    video_tx: Option<RtpTransceiver>,
    audio_tx: Option<RtpTransceiver>,
    restarted: bool,
    remote_fp: Option<String>,
}

#[derive(Default, Clone, Copy)]
struct SyncSample {
    a_est: f64,
    v_est: f64,
    a_jb_ms: f64,
    v_jb_ms: f64,
    have_a: bool,
    have_v: bool,
}

struct App {
    args: Args,
    tx: UnboundedSender<Ev>,
    ws_tx: UnboundedSender<Message>,
    factory: PeerConnectionFactory,
    backend: VideoEncoderBackend,
    me: String,
    ice: Vec<IceServer>,
    links: HashMap<String, Link>,
    next_gen: u64,
    remote_keys: HashMap<String, TransmissionKey>,
    local_key: Option<TransmissionKey>,
    pending_live: Option<String>,
    quality: Quality,
    overrides: Overrides,
    view_pref: ViewMode,
    video_track: Option<RtcVideoTrack>,
    audio_track: Option<libwebrtc::audio_track::RtcAudioTrack>,
    rx: HashMap<String, Arc<RxCounters>>,
    proc: metrics::ProcSampler,
    prev_bytes: HashMap<(String, &'static str), (u64, Instant)>,
    prev_jb: HashMap<(String, &'static str), (f64, f64)>,
    prev_rx_frames: HashMap<String, (u64, u64, Instant)>,
    sync: HashMap<String, SyncSample>,
    cycle_i: usize,
    minute: u64,
    max_abs_est_skew: f64,
    max_abs_pulse_skew: f64,
    gen_video_frames: Arc<AtomicU64>,
    gen_audio_samples: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
}

#[tokio::main]
async fn main() {
    let args = match args::parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::exit(2);
        }
    };
    START.get_or_init(Instant::now);
    if let Err(e) = run(args).await {
        eprintln!("erro: {e}");
        std::process::exit(1);
    }
}

fn parse_backend(s: &str) -> VideoEncoderBackend {
    match s {
        "software" => VideoEncoderBackend::Software,
        "hardware" => VideoEncoderBackend::Hardware,
        "nvenc" => VideoEncoderBackend::Nvenc,
        _ => VideoEncoderBackend::Auto,
    }
}

async fn run(args: Args) -> Result<(), String> {
    log!("sonda G1 | libwebrtc 0.4.0 | sala {} | base {} | Origin {}", args.room, args.base, origin_of(&args.base));
    let factory = if args.low_latency { PeerConnectionFactory::with_zero_playout_delay() } else { PeerConnectionFactory::default() };
    let backend = parse_backend(&args.backend);

    // O que o motor diz sobre codecs e codificadores (item f e preferência de codec).
    let avail: Vec<_> = VideoEncoderBackend::list_available().into_iter().collect();
    log!("codificadores disponíveis neste build: {avail:?}");
    for b in [VideoEncoderBackend::Auto, VideoEncoderBackend::Software, VideoEncoderBackend::Hardware] {
        log!("  video_encoder_codecs({b:?}) = {:?}", factory.video_encoder_codecs(b));
    }
    let caps = factory.get_rtp_sender_capabilities(MediaType::Video);
    log!("capacidades de envio de vídeo (ordem do motor):");
    for c in &caps.codecs {
        log!("  {} fmtp={:?}", c.mime_type, c.sdp_fmtp_line);
    }
    let ordered: Vec<String> = order_codecs(caps.codecs.clone()).iter().take(4).map(|c| format!("{} {:?}", c.mime_type, c.sdp_fmtp_line)).collect();
    log!("ordem depois de orderCodecs (primeiros 4): {ordered:?}");

    // WebSocket com Origin igual ao do endereço base.
    let url = ws_url(&args.base, &args.room);
    let mut req = url.as_str().into_client_request().map_err(|e| format!("endereço inválido: {e}"))?;
    req.headers_mut().insert("Origin", origin_of(&args.base).parse().map_err(|_| "Origin inválido".to_string())?);
    let (ws, resp) = tokio_tungstenite::connect_async(req).await.map_err(|e| format!("não conectou em {url}: {e}"))?;
    log!("WebSocket aberto ({})", resp.status());
    let (mut sink, mut stream) = ws.split();

    let (tx, rx) = unbounded_channel::<Ev>();
    let (ws_tx, mut ws_rx) = unbounded_channel::<Message>();
    tokio::spawn(async move {
        while let Some(m) = ws_rx.recv().await {
            if sink.send(m).await.is_err() {
                break;
            }
        }
    });
    {
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Some(m) = stream.next().await {
                match m {
                    Ok(Message::Text(t)) => {
                        let _ = tx.send(Ev::Ws(t));
                    }
                    Ok(Message::Close(f)) => {
                        let (code, why) = f.map(|f| (Some(u16::from(f.code)), f.reason.to_string())).unwrap_or((None, String::new()));
                        let _ = tx.send(Ev::WsClosed(code, why));
                        return;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        let _ = tx.send(Ev::WsClosed(None, e.to_string()));
                        return;
                    }
                }
            }
            let _ = tx.send(Ev::WsClosed(None, "socket fechado".into()));
        });
    }
    let _ = ws_tx.send(Message::Text(json!({ "t": "join", "nickname": args.nick, "mediaProtocol": 2 }).to_string()));

    // Fontes geradas (a menos que --no-live).
    let stop = Arc::new(AtomicBool::new(false));
    let gen_video_frames = Arc::new(AtomicU64::new(0));
    let gen_audio_samples = Arc::new(AtomicU64::new(0));
    let (mut video_track, mut audio_track) = (None, None);
    if !args.no_live {
        let source = NativeVideoSource::new(VideoResolution { width: args.width, height: args.height }, true);
        media::spawn_video_generator(source.clone(), args.width, args.height, args.gen_fps, stop.clone(), gen_video_frames.clone());
        video_track = Some(factory.create_video_track("sonda-video", source));
        let asrc = media::audio_source();
        media::spawn_audio_generator(asrc.clone(), stop.clone(), gen_audio_samples.clone());
        audio_track = Some(factory.create_audio_track("sonda-som", asrc));
        log!("fontes geradas: vídeo {}x{} a {} qps (barras, contador e clarão) e tom de 440 Hz com bip de 1 kHz a cada segundo", args.width, args.height, args.gen_fps);
    }

    let height = args.height;
    let fps = args.gen_fps.clamp(1, 60);
    let quality = Quality { resolution: height, fps, max_bitrate: bitrate_for(height, fps) };
    let overrides = Overrides { bitrate: args.bitrate, fps: args.fps, scale: args.scale, active: args.active };
    log!("qualidade de referência {height}p{fps}, teto {} bps; ajustes de linha de comando: {}", quality.max_bitrate, overrides.describe());

    // Temporizadores e teclado.
    spawn_ticker(tx.clone(), Duration::from_secs(args.stats_secs.max(1)), || Ev::StatsTick);
    spawn_ticker(tx.clone(), Duration::from_secs(20), || Ev::PingTick);
    spawn_ticker(tx.clone(), Duration::from_secs(60), || Ev::MinuteTick);
    if let Some(s) = args.cycle_secs {
        spawn_ticker(tx.clone(), Duration::from_secs(s.max(2)), || Ev::CycleTick);
    }
    {
        let tx = tx.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut lines = BufReader::new(tokio::io::stdin()).lines();
            while let Ok(Some(l)) = lines.next_line().await {
                let _ = tx.send(Ev::Line(l));
            }
        });
    }
    {
        let tx = tx.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = tx.send(Ev::CtrlC);
        });
    }

    let mut app = App {
        view_pref: args.view,
        args,
        tx,
        ws_tx,
        factory,
        backend,
        me: String::new(),
        ice: vec![],
        links: HashMap::new(),
        next_gen: 0,
        remote_keys: HashMap::new(),
        local_key: None,
        pending_live: None,
        quality,
        overrides,
        video_track: video_track.take(),
        audio_track: audio_track.take(),
        rx: HashMap::new(),
        proc: metrics::ProcSampler::new(),
        prev_bytes: HashMap::new(),
        prev_jb: HashMap::new(),
        prev_rx_frames: HashMap::new(),
        sync: HashMap::new(),
        cycle_i: 0,
        minute: 0,
        max_abs_est_skew: 0.0,
        max_abs_pulse_skew: 0.0,
        gen_video_frames,
        gen_audio_samples,
        stop,
    };
    app.event_loop(rx).await;
    Ok(())
}

fn spawn_ticker(tx: UnboundedSender<Ev>, every: Duration, make: fn() -> Ev) {
    tokio::spawn(async move {
        let mut t = tokio::time::interval(every);
        t.tick().await; // o primeiro tique é imediato; pula
        loop {
            t.tick().await;
            if tx.send(make()).is_err() {
                return;
            }
        }
    });
}

impl App {
    async fn event_loop(&mut self, mut rx: UnboundedReceiver<Ev>) {
        while let Some(ev) = rx.recv().await {
            let keep = match ev {
                Ev::Ws(t) => self.on_ws(&t).await,
                Ev::WsClosed(code, why) => {
                    log!("WebSocket fechado: código {code:?} {why}. A sonda não reconecta; saindo.");
                    false
                }
                Ev::Pc(id, gen, e) => {
                    if self.links.get(&id).map(|l| l.gen) == Some(gen) {
                        self.on_pc(&id, e).await;
                    }
                    true
                }
                Ev::Line(l) => self.on_line(l.trim()).await,
                Ev::StatsTick => {
                    self.report_stats(false).await;
                    true
                }
                Ev::PingTick => {
                    let _ = self.ws_tx.send(Message::Text(PING.into()));
                    true
                }
                Ev::CycleTick => {
                    self.on_cycle().await;
                    true
                }
                Ev::MinuteTick => self.on_minute().await,
                Ev::CtrlC => {
                    log!("Ctrl+C: saindo da sala");
                    false
                }
            };
            if !keep {
                break;
            }
        }
        self.shutdown().await;
    }

    async fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.ws_tx.send(Message::Close(Some(CloseFrame { code: CloseCode::Library(4001), reason: "saiu".into() })));
        for l in self.links.values() {
            l.pc.close();
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
        // O desmonte da libwebrtc pode demorar; a sonda não precisa esperar.
        std::process::exit(0);
    }

    // ------------------------------------------------------------ sinalização

    async fn on_ws(&mut self, text: &str) -> bool {
        if text == r#"{"t":"pong"}"# {
            return true;
        }
        let Ok(v) = serde_json::from_str::<Value>(text) else {
            log!("mensagem do servidor ilegível");
            return true;
        };
        match v["t"].as_str().unwrap_or("") {
            "welcome" => {
                if v["mediaProtocol"].as_u64() != Some(2) || v["mediaPolicy"]["kind"].as_str() != Some("direct-only") {
                    log!("welcome fora do contrato (mediaProtocol/mediaPolicy): seria 'Atualize o Espia'");
                    return false;
                }
                self.me = v["you"]["id"].as_str().unwrap_or("").to_string();
                self.ice = direct_ice_servers(&v["iceServers"]);
                log!("welcome: eu sou {} | sala {} | {} pessoa(s) | servidores STUN usados: {}", self.me, v["room"], v["peers"].as_array().map_or(0, |p| p.len()), self.ice.len());
                let peers = v["peers"].as_array().cloned().unwrap_or_default();
                for p in &peers {
                    let id = p["id"].as_str().unwrap_or("").to_string();
                    if let Some(started) = p["live"]["startedAt"].as_u64() {
                        self.remote_keys.insert(id.clone(), TransmissionKey { peer_id: id.clone(), started_at: started });
                        log!("  {} já está transmitindo", short(&id));
                    }
                }
                for p in &peers {
                    let id = p["id"].as_str().unwrap_or("").to_string();
                    self.create_link(&id, true).await;
                }
                if !self.args.no_live {
                    let seq = match v["broadcastOrder"].get("highestBroadcastSeq").and_then(Value::as_u64) {
                        Some(h) => h + 1,
                        None => 0,
                    };
                    let id = uuid::Uuid::new_v4().to_string();
                    self.pending_live = Some(id.clone());
                    let res = if self.args.height >= 1000 { 1080 } else { 720 };
                    let fps = if self.args.gen_fps >= 45 { 60 } else { 30 };
                    let m = json!({"t":"live","requestId":id,"broadcastSeq":seq,"resolution":res,"fps":fps,"audio":"pc","source":"screen","label":"Sonda G1"});
                    let _ = self.ws_tx.send(Message::Text(m.to_string()));
                    log!("→ live (broadcastSeq {seq}, {res}p{fps})");
                }
            }
            "live-ack" => {
                if self.pending_live.as_deref() == v["requestId"].as_str() {
                    self.pending_live = None;
                    if let Some(started) = v["live"]["startedAt"].as_u64() {
                        let key = TransmissionKey { peer_id: self.me.clone(), started_at: started };
                        log!("live-ack: transmissão {} startedAt={}", short(&key.peer_id), started);
                        self.set_local_key(Some(key)).await;
                    }
                }
            }
            "peer-joined" => log!("entrou: {} ({})", short(v["peer"]["id"].as_str().unwrap_or("?")), v["peer"]["nickname"]),
            "peer-left" => {
                let id = v["id"].as_str().unwrap_or("").to_string();
                log!("saiu: {}", short(&id));
                if let Some(l) = self.links.remove(&id) {
                    l.pc.close();
                }
                self.remote_keys.remove(&id);
                self.rx.remove(&id);
            }
            "peer-live" => {
                let id = v["id"].as_str().unwrap_or("").to_string();
                let key = v["live"]["startedAt"].as_u64().map(|s| TransmissionKey { peer_id: id.clone(), started_at: s });
                log!("peer-live: {} {}", short(&id), if key.is_some() { "transmitindo" } else { "parou" });
                match &key {
                    Some(k) => {
                        self.remote_keys.insert(id.clone(), k.clone());
                    }
                    None => {
                        self.remote_keys.remove(&id);
                    }
                }
                self.set_remote_key(&id, key).await;
            }
            "signal" => {
                let from = v["from"].as_str().unwrap_or("").to_string();
                self.on_signal(&from, &v["data"]).await;
            }
            "ice-servers" => {
                self.ice = direct_ice_servers(&v["iceServers"]);
                log!("ice-servers: {} servidor(es) STUN (os demais são ignorados)", self.ice.len());
            }
            "error" => log!("erro do servidor: {} {}", v["code"], v["message"]),
            t @ ("room-not-found" | "room-full" | "resume-failed" | "room-deleted" | "removed") => {
                log!("o servidor mandou {t}: encerrando");
                return false;
            }
            other => log!("mensagem ignorada: {other}"),
        }
        true
    }

    fn send_signal(&self, to: &str, data: Value) {
        let _ = self.ws_tx.send(Message::Text(json!({"t":"signal","to":to,"data":data}).to_string()));
    }

    async fn on_signal(&mut self, from: &str, data: &Value) {
        // Oferta de uma conexão nova da mesma pessoa (ela recarregou a página): descarta a antiga.
        if let Some(desc) = data.get("description") {
            if desc["type"].as_str() == Some("offer") {
                let new_fp = desc["sdp"].as_str().and_then(fingerprint_of);
                let old_fp = self.links.get(from).and_then(|l| l.remote_fp.clone());
                if let (Some(n), Some(o)) = (&new_fp, &old_fp) {
                    if n != o {
                        log!("{}: certificado novo (a pessoa recarregou); recriando a ligação", short(from));
                        if let Some(l) = self.links.remove(from) {
                            l.pc.close();
                        }
                    }
                }
            }
        }
        if !self.links.contains_key(from) {
            self.create_link(from, false).await;
        }
        let Some(pc) = self.links.get(from).map(|l| l.pc.clone()) else { return };

        if let Some(desc) = data.get("description") {
            let ty = desc["type"].as_str().unwrap_or("");
            let Ok(sdp_type) = SdpType::from_str(ty) else { return };
            let sdp = desc["sdp"].as_str().unwrap_or("");
            let collision = sdp_type == SdpType::Offer && pc.signaling_state() != SignalingState::Stable;
            let polite = self.links.get(from).map_or(false, |l| l.polite);
            let ignore = !polite && collision;
            if let Some(l) = self.links.get_mut(from) {
                l.ignore_offer = ignore;
            }
            if ignore {
                log!("{}: oferta cruzada ignorada (eu não sou educado)", short(from));
                return;
            }
            if collision {
                // Educado: desiste da própria oferta (rollback) e aceita a do outro.
                log!("{}: oferta cruzada; sou educado, faço rollback e aceito a oferta dele", short(from));
                match SessionDescription::parse("", SdpType::Rollback) {
                    Ok(rb) => {
                        if let Err(e) = pc.set_local_description(rb).await {
                            log!("  rollback explícito falhou ({e}); tentando só set_remote_description (rollback implícito)");
                        }
                    }
                    Err(e) => log!("  não consegui montar o rollback: {e:?}"),
                }
            }
            let parsed = match SessionDescription::parse(sdp, sdp_type) {
                Ok(d) => d,
                Err(e) => {
                    log!("{}: SDP inválida: {e}", short(from));
                    return;
                }
            };
            if let Err(e) = pc.set_remote_description(parsed).await {
                log!("{}: set_remote_description({ty}) falhou: {e}", short(from));
                return;
            }
            if let Some(l) = self.links.get_mut(from) {
                l.remote_fp = fingerprint_of(sdp).or(l.remote_fp.take());
            }
            if sdp_type == SdpType::Offer {
                match pc.create_answer(AnswerOptions::default()).await {
                    Ok(ans) => {
                        if let Err(e) = pc.set_local_description(ans.clone()).await {
                            log!("{}: set_local_description(answer) falhou: {e}", short(from));
                            return;
                        }
                        self.send_signal(from, json!({"description":{"type":"answer","sdp":ans.to_string()}}));
                    }
                    Err(e) => log!("{}: create_answer falhou: {e}", short(from)),
                }
            }
        } else if let Some(c) = data.get("candidate").filter(|c| !c.is_null()) {
            let cand = c["candidate"].as_str().unwrap_or("");
            if cand.is_empty() {
                return; // fim dos candidatos
            }
            if is_relay_candidate(cand) {
                log!("{}: candidato typ relay descartado (só STUN)", short(from));
                return;
            }
            let mid = c["sdpMid"].as_str().unwrap_or("");
            let idx = c["sdpMLineIndex"].as_i64().unwrap_or(0) as i32;
            match libwebrtc::ice_candidate::IceCandidate::parse(mid, idx, cand) {
                Ok(ic) => {
                    if let Err(e) = pc.add_ice_candidate(ic).await {
                        let ignoring = self.links.get(from).map_or(false, |l| l.ignore_offer);
                        if !ignoring {
                            log!("{}: add_ice_candidate falhou: {e} ({cand})", short(from));
                        }
                    }
                }
                Err(e) => log!("{}: candidato ilegível: {e}", short(from)),
            }
        }
    }

    // ------------------------------------------------------------ ligações

    async fn create_link(&mut self, id: &str, initiator: bool) {
        let mut cfg = RtcConfiguration::default();
        cfg.ice_servers = self.ice.clone();
        let pc = match self.factory.create_peer_connection(cfg) {
            Ok(pc) => pc,
            Err(e) => {
                log!("{}: create_peer_connection falhou: {e}", short(id));
                return;
            }
        };
        self.next_gen += 1;
        let gen = self.next_gen;
        let mk = |tx: &UnboundedSender<Ev>, id: &str| (tx.clone(), id.to_string());

        let (t, i) = mk(&self.tx, id);
        pc.on_ice_candidate(Some(Box::new(move |c| {
            let _ = t.send(Ev::Pc(i.clone(), gen, PcEv::Candidate { mid: c.sdp_mid(), idx: c.sdp_mline_index(), cand: c.candidate() }));
        })));
        let (t, i) = mk(&self.tx, id);
        pc.on_negotiation_needed(Some(Box::new(move |_| {
            let _ = t.send(Ev::Pc(i.clone(), gen, PcEv::NegotiationNeeded));
        })));
        let (t, i) = mk(&self.tx, id);
        pc.on_track(Some(Box::new(move |ev| {
            let _ = t.send(Ev::Pc(i.clone(), gen, PcEv::Track(ev)));
        })));
        let (t, i) = mk(&self.tx, id);
        pc.on_data_channel(Some(Box::new(move |dc| {
            let _ = t.send(Ev::Pc(i.clone(), gen, PcEv::DataChannel(dc)));
        })));
        let (t, i) = mk(&self.tx, id);
        pc.on_connection_state_change(Some(Box::new(move |s| {
            let _ = t.send(Ev::Pc(i.clone(), gen, PcEv::Conn(s)));
        })));
        let (t, i) = mk(&self.tx, id);
        pc.on_ice_connection_state_change(Some(Box::new(move |s| {
            let _ = t.send(Ev::Pc(i.clone(), gen, PcEv::Ice(s)));
        })));
        let (t, i) = mk(&self.tx, id);
        pc.on_signaling_state_change(Some(Box::new(move |s| {
            let _ = t.send(Ev::Pc(i.clone(), gen, PcEv::Signaling(s)));
        })));

        // Quem é educado é quem tem o id menor (mesh.ts: myId < id).
        let polite = self.me.as_str() < id;
        let mut link = Link {
            gen,
            pc: pc.clone(),
            polite,
            ignore_offer: false,
            ctl: None,
            ctl_open: false,
            my_view: None,
            their_view: ViewMode::Off,
            grant_revision: -1,
            view_revision: 0,
            remote_key: self.remote_keys.get(id).cloned(),
            video_tx: None,
            audio_tx: None,
            restarted: false,
            remote_fp: None,
        };
        log!("{}: ligação criada (iniciador={initiator}, educado={polite})", short(id));
        if initiator {
            match pc.create_data_channel("ctl", DataChannelInit::default()) {
                Ok(dc) => {
                    self.wire_ctl(id, gen, &dc);
                    link.ctl = Some(dc);
                }
                Err(e) => log!("{}: create_data_channel(ctl) falhou: {e}", short(id)),
            }
        }
        let has_key = link.remote_key.is_some();
        self.links.insert(id.to_string(), link);
        if let Some(k) = self.local_key.clone() {
            self.set_local_key_on(id, Some(k)).await;
        }
        if has_key {
            self.set_my_view(id, self.view_pref).await;
        }
    }

    fn wire_ctl(&self, id: &str, gen: u64, dc: &DataChannel) {
        let (t, i) = (self.tx.clone(), id.to_string());
        dc.on_state_change(Some(Box::new(move |s| {
            let _ = t.send(Ev::Pc(i.clone(), gen, PcEv::DcState(s)));
        })));
        let (t, i) = (self.tx.clone(), id.to_string());
        dc.on_message(Some(Box::new(move |b| {
            let _ = t.send(Ev::Pc(i.clone(), gen, PcEv::DcMsg(String::from_utf8_lossy(b.data).to_string())));
        })));
    }

    async fn on_pc(&mut self, id: &str, e: PcEv) {
        match e {
            PcEv::Candidate { mid, idx, cand } => {
                if is_relay_candidate(&cand) {
                    return;
                }
                self.send_signal(id, json!({"candidate":{"candidate":cand,"sdpMid":mid,"sdpMLineIndex":idx}}));
            }
            PcEv::NegotiationNeeded => self.negotiate(id).await,
            PcEv::Track(ev) => {
                let c = self.rx.entry(id.to_string()).or_default().clone();
                match ev.track {
                    MediaStreamTrack::Video(t) => {
                        log!("{}: chegou faixa de VÍDEO do outro lado (id {})", short(id), t.id());
                        media::spawn_video_sink(t, c, self.args.detect_pulse);
                    }
                    MediaStreamTrack::Audio(t) => {
                        log!("{}: chegou faixa de SOM do outro lado (id {})", short(id), t.id());
                        media::spawn_audio_sink(t, c, self.args.detect_pulse);
                    }
                }
            }
            PcEv::DataChannel(dc) => {
                if dc.label() == "ctl" {
                    log!("{}: canal ctl recebido", short(id));
                    let gen = self.links.get(id).map_or(0, |l| l.gen);
                    self.wire_ctl(id, gen, &dc);
                    let open = dc.state() == DataChannelState::Open;
                    if let Some(l) = self.links.get_mut(id) {
                        l.ctl = Some(dc);
                    }
                    if open {
                        self.on_ctl_open(id).await;
                    }
                }
            }
            PcEv::DcState(s) => {
                log!("{}: canal ctl {s:?}", short(id));
                match s {
                    DataChannelState::Open => {
                        if let Some(l) = self.links.get_mut(id) {
                            l.ctl_open = true;
                        }
                        self.on_ctl_open(id).await;
                    }
                    DataChannelState::Closed => {
                        if let Some(l) = self.links.get_mut(id) {
                            l.ctl_open = false;
                            l.their_view = ViewMode::Off;
                            l.grant_revision = -1;
                        }
                        self.set_local(id).await;
                    }
                    _ => {}
                }
            }
            PcEv::DcMsg(text) => self.on_ctl_message(id, &text).await,
            PcEv::Conn(s) => {
                log!("{}: conexão {s:?}", short(id));
                if s == PeerConnectionState::Failed {
                    let again = self.links.get_mut(id).map(|l| !std::mem::replace(&mut l.restarted, true)).unwrap_or(false);
                    if again {
                        log!("{}: restart_ice automático", short(id));
                        if let Some(l) = self.links.get(id) {
                            l.pc.restart_ice();
                        }
                    }
                }
            }
            PcEv::Ice(s) => log!("{}: ICE {s:?}", short(id)),
            PcEv::Signaling(s) => {
                if s == SignalingState::Stable {
                    self.apply_encodings(id).await;
                }
            }
        }
    }

    async fn negotiate(&mut self, id: &str) {
        let Some(pc) = self.links.get(id).map(|l| l.pc.clone()) else { return };
        if pc.signaling_state() != SignalingState::Stable {
            return; // quando assentar o motor avisa de novo, se ainda for preciso
        }
        // None nos dois: mantém a direção dos canais que já existem (o padrão Some(false) os desligaria).
        let opts = OfferOptions { ice_restart: false, offer_to_receive_audio: None, offer_to_receive_video: None };
        match pc.create_offer(opts).await {
            Ok(offer) => {
                if pc.signaling_state() != SignalingState::Stable {
                    return;
                }
                if let Err(e) = pc.set_local_description(offer.clone()).await {
                    log!("{}: set_local_description(offer) falhou: {e}", short(id));
                    return;
                }
                self.send_signal(id, json!({"description":{"type":"offer","sdp":offer.to_string()}}));
            }
            Err(e) => log!("{}: create_offer falhou: {e}", short(id)),
        }
    }

    // ------------------------------------------------------------ canal ctl e ViewMessage

    async fn on_ctl_open(&mut self, id: &str) {
        self.send_view(id);
        self.request_view(id);
    }

    fn send_ctl(&self, id: &str, text: &str) {
        if let Some(Some(dc)) = self.links.get(id).map(|l| l.ctl.as_ref()) {
            if let Err(e) = dc.send(text.as_bytes(), false) {
                log!("{}: ctl.send falhou: {e}", short(id));
            }
        }
    }

    fn ctl_ready(&self, id: &str) -> bool {
        self.links.get(id).and_then(|l| l.ctl.as_ref()).map_or(false, |dc| dc.state() == DataChannelState::Open)
    }

    fn send_view(&mut self, id: &str) {
        let ready = self.ctl_ready(id);
        let Some(l) = self.links.get_mut(id) else { return };
        let Some(key) = l.remote_key.clone() else { return };
        if !ready {
            return;
        }
        l.view_revision += 1;
        let mode = l.my_view.unwrap_or(ViewMode::Off);
        let text = encode_view(mode, &key, l.view_revision);
        log!("{}: ctl → view v2 modo={} revisão={}", short(id), mode.as_str(), l.view_revision);
        self.send_ctl(id, &text);
    }

    fn request_view(&self, id: &str) {
        if let Some(k) = &self.local_key {
            if self.ctl_ready(id) {
                log!("{}: ctl → view-sync", short(id));
                self.send_ctl(id, &encode_view_sync(k));
            }
        }
    }

    async fn on_ctl_message(&mut self, id: &str, text: &str) {
        let Ok(raw) = serde_json::from_str::<Value>(text) else { return };
        if raw["t"].as_str() == Some("view-sync") {
            if let Some(req) = decode_view_sync(&raw) {
                let mine = self.links.get(id).and_then(|l| l.remote_key.clone());
                if same_transmission(Some(&req), mine.as_ref()) {
                    log!("{}: ctl ← view-sync (responde com a view atual)", short(id));
                    self.send_view(id);
                }
            }
            return;
        }
        let Some(grant) = decode_view(&raw) else {
            log!("{}: ctl ← mensagem inválida, ignorada (nunca autoriza mídia)", short(id));
            return;
        };
        if !same_transmission(Some(&grant.transmission), self.local_key.as_ref()) {
            log!("{}: ctl ← view de outra transmissão, ignorada", short(id));
            return;
        }
        let Some(l) = self.links.get_mut(id) else { return };
        if (grant.revision as i64) <= l.grant_revision {
            return;
        }
        l.grant_revision = grant.revision as i64;
        l.their_view = grant.mode;
        log!("{}: ctl ← view v2 modo={} revisão={} (decodeView ok)", short(id), grant.mode.as_str(), grant.revision);
        self.set_local(id).await;
    }

    async fn set_local_key(&mut self, key: Option<TransmissionKey>) {
        self.local_key = key.clone();
        let ids: Vec<String> = self.links.keys().cloned().collect();
        for id in ids {
            self.set_local_key_on(&id, key.clone()).await;
        }
    }

    async fn set_local_key_on(&mut self, id: &str, key: Option<TransmissionKey>) {
        if let Some(l) = self.links.get_mut(id) {
            l.grant_revision = -1;
            l.their_view = ViewMode::Off;
        }
        let _ = key;
        self.set_local(id).await;
        self.request_view(id);
    }

    async fn set_remote_key(&mut self, id: &str, key: Option<TransmissionKey>) {
        let Some(l) = self.links.get_mut(id) else { return };
        if key == l.remote_key {
            return;
        }
        l.remote_key = key.clone();
        l.my_view = Some(ViewMode::Off);
        self.send_view(id);
        if key.is_some() {
            let pref = self.view_pref;
            self.set_my_view(id, pref).await;
        }
    }

    async fn set_my_view(&mut self, id: &str, mode: ViewMode) {
        let Some(l) = self.links.get_mut(id) else { return };
        if l.my_view == Some(mode) {
            return;
        }
        l.my_view = Some(mode);
        self.send_view(id);
    }

    // ------------------------------------------------------------ envio de mídia por ligação

    /// Começa ou para de enviar a nossa imagem e o nosso som a essa pessoa (mesma regra do `setLocal` do site).
    async fn set_local(&mut self, id: &str) {
        let (Some(vt), Some(at)) = (self.video_track.clone(), self.audio_track.clone()) else { return };
        let Some(l) = self.links.get(id) else { return };
        let live = self.local_key.is_some();
        let want_video = live && l.their_view.wants_video();
        let want_sound = live && l.their_view.wants_sound();
        let pc = l.pc.clone();
        let their = l.their_view;
        let (vtx, atx) = (l.video_tx.clone(), l.audio_tx.clone());
        let p = self.overrides.apply(send_params(self.quality, their));

        let mut new_v = None;
        let mut new_a = None;
        match vtx {
            Some(tx) => {
                // Não tira a faixa com set_track(None): nessa crate, remover a faixa de um envio ativo
                // derrubava o processo (crash ao a pessoa passar a me ver em 'hidden'/'off'). Parar o
                // envio é feito por active=false em apply_encodings, como o site (sendParams).
                if want_video {
                    if let Err(e) = tx.sender().set_track(Some(MediaStreamTrack::Video(vt))) {
                        log!("{}: sender.set_track(vídeo) falhou: {e}", short(id));
                    }
                }
            }
            None if want_video => {
                // O teto entra junto com o canal (como no site): antes da 1ª negociação set_parameters não vale.
                let enc = RtpEncodingParameters {
                    active: p.active,
                    max_bitrate: Some(p.max_bitrate),
                    max_framerate: Some(p.max_framerate),
                    scale_resolution_down_by: Some(p.scale_resolution_down_by),
                    ..Default::default()
                };
                let init = RtpTransceiverInit { direction: RtpTransceiverDirection::SendOnly, stream_ids: vec![STREAM_ID.into()], send_encodings: vec![enc] };
                match pc.add_transceiver(MediaStreamTrack::Video(vt), init) {
                    Ok(tx) => {
                        if !self.args.no_h264 {
                            let caps = self.factory.get_rtp_sender_capabilities(MediaType::Video).codecs;
                            if let Err(e) = tx.set_codec_preferences(order_codecs(caps)) {
                                log!("{}: set_codec_preferences falhou: {e}", short(id));
                            }
                        }
                        tx.sender().set_video_encoder_backend(self.backend);
                        log!("{}: canal de vídeo criado (H.264 primeiro={}, backend={:?})", short(id), !self.args.no_h264, self.backend);
                        new_v = Some(tx);
                    }
                    Err(e) => log!("{}: add_transceiver(vídeo) falhou: {e}", short(id)),
                }
            }
            None => {}
        }
        match atx {
            Some(tx) => {
                // Mesmo cuidado do vídeo: o som para com active=false (apply_encodings), sem set_track(None).
                if want_sound {
                    if let Err(e) = tx.sender().set_track(Some(MediaStreamTrack::Audio(at))) {
                        log!("{}: sender.set_track(som) falhou: {e}", short(id));
                    }
                }
            }
            None if want_sound => {
                let enc = RtpEncodingParameters { active: true, max_bitrate: Some(AUDIO_BITRATE), ..Default::default() };
                let init = RtpTransceiverInit { direction: RtpTransceiverDirection::SendOnly, stream_ids: vec![STREAM_ID.into()], send_encodings: vec![enc] };
                match pc.add_transceiver(MediaStreamTrack::Audio(at), init) {
                    Ok(tx) => {
                        log!("{}: canal de som criado", short(id));
                        new_a = Some(tx);
                    }
                    Err(e) => log!("{}: add_transceiver(som) falhou: {e}", short(id)),
                }
            }
            None => {}
        }
        if let Some(l) = self.links.get_mut(id) {
            if new_v.is_some() {
                l.video_tx = new_v;
            }
            if new_a.is_some() {
                l.audio_tx = new_a;
            }
        }
        log!("{}: set_local → vídeo={want_video} som={want_sound} (a pessoa me vê em '{}')", short(id), their.as_str());
        self.apply_encodings(id).await;
    }

    /// Aplica bitrate, fps, scaleResolutionDownBy e active no canal de vídeo (e active/bitrate no som).
    async fn apply_encodings(&mut self, id: &str) {
        let Some(l) = self.links.get(id) else { return };
        if l.pc.signaling_state() != SignalingState::Stable {
            return;
        }
        let p = self.overrides.apply(send_params(self.quality, l.their_view));
        let their = l.their_view;
        if let Some(tx) = &l.video_tx {
            let s = tx.sender();
            if s.track().is_some() {
                let mut params = s.parameters();
                match params.encodings.get_mut(0) {
                    Some(e) => {
                        e.active = p.active;
                        e.max_bitrate = Some(p.max_bitrate);
                        e.max_framerate = Some(p.max_framerate);
                        e.scale_resolution_down_by = Some(p.scale_resolution_down_by);
                        // Tela: mantém resolução (contentPreference do site para 'screen').
                        params.set_degradation_preference(DegradationPreference::MaintainResolution);
                        match s.set_parameters(params) {
                            Ok(()) => {
                                let back = s.parameters();
                                let e0 = back.encodings.first();
                                log!(
                                    "{}: set_parameters(vídeo) ok → pedi active={} bitrate={} fps={} scale={:.2} | o motor guardou {:?}",
                                    short(id), p.active, p.max_bitrate, p.max_framerate, p.scale_resolution_down_by,
                                    e0.map(|e| (e.active, e.max_bitrate, e.max_framerate, e.scale_resolution_down_by))
                                );
                            }
                            Err(e) => log!("{}: set_parameters(vídeo) falhou: {e}", short(id)),
                        }
                    }
                    None => log!("{}: o canal de vídeo ainda não tem encodings; tenta de novo quando assentar", short(id)),
                }
            }
        }
        if let Some(tx) = &l.audio_tx {
            let s = tx.sender();
            if s.track().is_some() {
                let mut params = s.parameters();
                if let Some(e) = params.encodings.get_mut(0) {
                    e.active = their.wants_sound();
                    e.max_bitrate = Some(AUDIO_BITRATE);
                    if let Err(err) = s.set_parameters(params) {
                        log!("{}: set_parameters(som) falhou: {err}", short(id));
                    }
                }
            }
        }
    }

    async fn apply_all_encodings(&mut self) {
        let ids: Vec<String> = self.links.keys().cloned().collect();
        for id in ids {
            self.apply_encodings(&id).await;
        }
    }

    async fn on_cycle(&mut self) {
        let steps = cycle_steps();
        let (label, ov) = steps[self.cycle_i % steps.len()];
        self.cycle_i += 1;
        self.overrides = ov;
        log!("=== CICLO: {label} ===");
        self.apply_all_encodings().await;
    }

    async fn on_line(&mut self, l: &str) -> bool {
        let mut it = l.split_whitespace();
        let cmd = it.next().unwrap_or("");
        let arg = it.next();
        match (cmd, arg) {
            ("", _) => {}
            ("quit" | "q" | "sair", _) => return false,
            ("stats", _) => self.report_stats(true).await,
            ("reset", _) => {
                self.overrides = Overrides::default();
                log!("ajustes limpos");
                self.apply_all_encodings().await;
            }
            ("bitrate", Some(v)) => {
                self.overrides.bitrate = v.parse().ok();
                self.apply_all_encodings().await;
            }
            ("fps", Some(v)) => {
                self.overrides.fps = v.parse().ok();
                self.apply_all_encodings().await;
            }
            ("scale", Some(v)) => {
                self.overrides.scale = v.parse().ok();
                self.apply_all_encodings().await;
            }
            ("active", Some(v)) => {
                self.overrides.active = Some(matches!(v, "on" | "true" | "1" | "sim"));
                self.apply_all_encodings().await;
            }
            ("view", Some(v)) => match ViewMode::parse(v) {
                Some(m) => {
                    self.view_pref = m;
                    let ids: Vec<String> = self.links.iter().filter(|(_, l)| l.remote_key.is_some()).map(|(i, _)| i.clone()).collect();
                    for id in ids {
                        self.set_my_view(&id, m).await;
                    }
                }
                None => log!("view: use full | thumb | hidden | off"),
            },
            _ => log!("comando desconhecido: '{l}' (bitrate N | fps N | scale X | active on|off | reset | view M | stats | quit)"),
        }
        log!("ajustes atuais: {}", self.overrides.describe());
        true
    }

    // ------------------------------------------------------------ estatísticas

    async fn report_stats(&mut self, force_all: bool) {
        let p = self.proc.sample();
        log!(
            "[proc] cpu {:.1}% de um núcleo ({:.1}% da máquina, {} núcleos lógicos) | memória privada {:.0} MB | working set {:.0} MB | gerados: {} quadros, {} amostras",
            p.cpu_one_core_pct, p.cpu_machine_pct, self.proc.cores(), p.private_mb, p.working_set_mb,
            self.gen_video_frames.load(Ordering::Relaxed), self.gen_audio_samples.load(Ordering::Relaxed)
        );
        let pcs: Vec<(String, PeerConnection, ViewMode)> = self.links.iter().map(|(i, l)| (i.clone(), l.pc.clone(), l.their_view)).collect();
        for (id, pc, their) in pcs {
            let stats = match pc.get_stats().await {
                Ok(s) => s,
                Err(e) => {
                    log!("{}: get_stats falhou: {e}", short(&id));
                    continue;
                }
            };
            let mut pairs = HashMap::new();
            let mut locals = HashMap::new();
            let mut remotes = HashMap::new();
            let mut selected = String::new();
            let mut sample = self.sync.get(&id).copied().unwrap_or_default();
            let now = Instant::now();
            for s in &stats {
                match s {
                    RtcStats::Transport(t) => {
                        if !t.transport.selected_candidate_pair_id.is_empty() {
                            selected = t.transport.selected_candidate_pair_id.clone();
                        }
                    }
                    RtcStats::CandidatePair(c) => {
                        pairs.insert(c.rtc.id.clone(), c.candidate_pair.clone());
                    }
                    RtcStats::LocalCandidate(c) => {
                        locals.insert(c.rtc.id.clone(), c.local_candidate.clone());
                    }
                    RtcStats::RemoteCandidate(c) => {
                        remotes.insert(c.rtc.id.clone(), c.remote_candidate.clone());
                    }
                    RtcStats::OutboundRtp(o) => {
                        let kind: &'static str = if o.stream.kind == "video" { "video" } else { "audio" };
                        let key = (id.clone(), kind);
                        let kbps = match self.prev_bytes.insert(key, (o.sent.bytes_sent, now)) {
                            Some((b0, t0)) if o.sent.bytes_sent >= b0 => (o.sent.bytes_sent - b0) as f64 * 8.0 / now.duration_since(t0).as_secs_f64().max(0.001) / 1000.0,
                            _ => 0.0,
                        };
                        if kind == "video" {
                            let o = &o.outbound;
                            log!(
                                "{}: [envio vídeo] vê-me em '{}' | {}x{} a {:.0} qps | enviando {:.0} kbps (alvo {:.0}) | qualityLimitationReason={:?} | encoderImplementation='{}' powerEfficientEncoder={} | quadros {} (chaves {}) pli={} nack={} | active={} | tempo de codificação {:.1} ms/quadro",
                                short(&id), their.as_str(), o.frame_width, o.frame_height, o.frames_per_second, kbps, o.target_bitrate / 1000.0,
                                o.quality_limitation_reason, o.encoder_implementation, o.power_efficient_encoder,
                                o.frames_encoded, o.key_frames_encoded, o.pli_count, o.nack_count, o.active,
                                if o.frames_encoded > 0 { o.total_encode_time * 1000.0 / o.frames_encoded as f64 } else { 0.0 }
                            );
                            if !o.quality_limitation_durations.is_empty() {
                                log!("{}:   tempo por motivo de limitação (s): {:?}", short(&id), o.quality_limitation_durations);
                            }
                        } else {
                            log!("{}: [envio som] {:.0} kbps | active={}", short(&id), kbps, o.outbound.active);
                        }
                    }
                    RtcStats::InboundRtp(i) => {
                        let kind: &'static str = if i.stream.kind == "video" { "video" } else { "audio" };
                        let x = &i.inbound;
                        // atraso médio do buffer de jitter no intervalo (s por unidade emitida)
                        let jb = self.prev_jb.insert((id.clone(), kind), (x.jitter_buffer_delay, x.jitter_buffer_emitted_count as f64));
                        let jb_ms = match jb {
                            Some((d0, c0)) if x.jitter_buffer_emitted_count as f64 > c0 => (x.jitter_buffer_delay - d0) / (x.jitter_buffer_emitted_count as f64 - c0) * 1000.0,
                            _ if x.jitter_buffer_emitted_count > 0 => x.jitter_buffer_delay / x.jitter_buffer_emitted_count as f64 * 1000.0,
                            _ => 0.0,
                        };
                        if kind == "video" {
                            sample.v_est = x.estimated_playout_timestamp;
                            sample.have_v = x.estimated_playout_timestamp > 0.0;
                            sample.v_jb_ms = jb_ms;
                            log!(
                                "{}: [recebe vídeo] {}x{} a {:.0} qps | decodificados {} perdidos(pacotes) {} | decoderImplementation='{}' | atraso do buffer {:.0} ms | congelamentos {} | estimatedPlayoutTimestamp={:.0}",
                                short(&id), x.frame_width, x.frame_height, x.frames_per_second, x.frames_decoded, i.received.packets_lost,
                                x.decoder_implementation, jb_ms, x.freeze_count, x.estimated_playout_timestamp
                            );
                        } else {
                            sample.a_est = x.estimated_playout_timestamp;
                            sample.have_a = x.estimated_playout_timestamp > 0.0;
                            sample.a_jb_ms = jb_ms;
                            log!(
                                "{}: [recebe som] amostras {} (ocultadas {}) | atraso do buffer {:.0} ms | nível {:.3} | estimatedPlayoutTimestamp={:.0}",
                                short(&id), x.total_samples_received, x.concealed_samples, jb_ms, x.audio_level, x.estimated_playout_timestamp
                            );
                        }
                    }
                    _ => {}
                }
            }
            self.sync.insert(id.clone(), sample);
            if let Some(p) = pairs.get(&selected).or_else(|| pairs.values().find(|p| p.nominated)) {
                let l = locals.get(&p.local_candidate_id);
                let r = remotes.get(&p.remote_candidate_id);
                let lt = l.and_then(|c| c.candidate_type);
                let rt = r.and_then(|c| c.candidate_type);
                let relay = lt == Some(libwebrtc::stats::IceCandidateType::Relay) || rt == Some(libwebrtc::stats::IceCandidateType::Relay);
                log!(
                    "{}: [rota] local={:?} remoto={:?} {} | rtt {:.0} ms | banda estimada de saída {:.0} kbps{}",
                    short(&id), lt, rt, l.map_or(String::new(), |c| c.protocol.clone()), p.current_round_trip_time * 1000.0,
                    p.available_outgoing_bitrate / 1000.0, if relay { " | ATENÇÃO: rota por retransmissão (relay)!" } else { "" }
                );
            }
            if let Some(c) = self.rx.get(&id) {
                let (f, a) = (c.video_frames.load(Ordering::Relaxed), c.audio_samples.load(Ordering::Relaxed));
                let prev = self.prev_rx_frames.insert(id.clone(), (f, a, now));
                if let Some((f0, a0, t0)) = prev {
                    let dt = now.duration_since(t0).as_secs_f64().max(0.001);
                    log!(
                        "{}: [recepção na sonda] vídeo {:.1} qps entregues ({} quadros no total, último {}x{}, fora de I420: {}) | som {:.0} amostras/s ({} no total)",
                        short(&id), (f - f0) as f64 / dt, f, c.video_w.load(Ordering::Relaxed), c.video_h.load(Ordering::Relaxed),
                        c.video_non_i420.load(Ordering::Relaxed), (a - a0) as f64 / dt, a
                    );
                }
            }
            if force_all {
                log!("{}: (stats pedidas) {} itens de estatística", short(&id), stats.len());
            }
        }
    }

    async fn on_minute(&mut self) -> bool {
        self.minute += 1;
        let n = self.minute;
        log!("[sync] ===== minuto {n}/{} =====", self.args.minutes);
        for (id, s) in self.sync.clone() {
            if s.have_a && s.have_v {
                let skew = s.a_est - s.v_est;
                self.max_abs_est_skew = self.max_abs_est_skew.max(skew.abs());
                log!(
                    "[sync] {}: estimatedPlayoutTimestamp som - imagem = {:+.1} ms (som {:.0}, imagem {:.0}) | atraso do buffer: som {:.0} ms, imagem {:.0} ms",
                    short(&id), skew, s.a_est, s.v_est, s.a_jb_ms, s.v_jb_ms
                );
            } else {
                log!("[sync] {}: sem estimatedPlayoutTimestamp dos dois lados (som={}, imagem={}); não deu para medir por estatística neste minuto", short(&id), s.have_a, s.have_v);
            }
        }
        if self.args.detect_pulse {
            for (id, c) in &self.rx {
                let mut p = c.pulse.lock().unwrap();
                let v: Vec<f64> = std::mem::take(&mut p.skews_ms);
                if v.is_empty() {
                    log!("[sync] {}: nenhum par clarão+bip detectado neste minuto (a outra ponta é uma sonda transmitindo? quadros em I420?)", short(id));
                } else {
                    let mean = v.iter().sum::<f64>() / v.len() as f64;
                    let (mn, mx) = v.iter().fold((f64::MAX, f64::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
                    self.max_abs_pulse_skew = self.max_abs_pulse_skew.max(mn.abs()).max(mx.abs());
                    log!("[sync] {}: pulso (som - imagem na entrega): {} pares, média {mean:+.1} ms, mín {mn:+.1}, máx {mx:+.1}", short(id), v.len());
                }
            }
        }
        if n >= self.args.minutes {
            log!(
                "[sync] RESULTADO após {n} min: maior |som - imagem| por estatística = {:.1} ms; por pulso = {:.1} ms (critério G1 (g): até 80 ms)",
                self.max_abs_est_skew, self.max_abs_pulse_skew
            );
            if self.args.exit_after {
                return false;
            }
        }
        true
    }
}

fn short(id: &str) -> &str {
    &id[..id.len().min(6)]
}
