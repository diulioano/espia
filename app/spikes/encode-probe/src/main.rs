//! Sonda 020 (T009): a sonda do G1 (019, T009) copiada e estendida com o codificador de H.264 da placa. Entra numa sala do site do Espia como se fosse mais um cliente, com a crate
//! `libwebrtc` 0.4.0 da LiveKit. Veja LEIA-ME.md para os comandos dos itens (a) a (j) do T010.
//!
//! Origem das regras (só leitura): src/lib/peer.ts, src/lib/mesh.ts, src/lib/quality.ts,
//! src/lib/signaling.ts, src/shared/protocol.ts, src/shared/media.ts, src/worker/index.ts.

mod args;
mod media;
mod metrics;
mod proto;
mod hw_encoder;
mod texture;

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
use libwebrtc::video_codec::VideoCodec;
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
use crate::media::{HwStream, RxCounters, TierCfg};
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

/// De onde vem o vídeo de uma faixa: a cheia ou a reduzida (comprimidas pela placa) ou a fonte crua (a biblioteca comprime).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Full,
    Reduced,
    Software,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Full => "cheia",
            Kind::Reduced => "reduzida",
            Kind::Software => "processador",
        }
    }
    fn from_placa(self) -> bool {
        self != Kind::Software
    }
}

/// A faixa que está no canal de vídeo de uma ligação (a reduzida pode ser recriada: `gen` muda).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct SrcId {
    kind: Kind,
    gen: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PathKind {
    Placa,
    Cpu,
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
    /// Faixa que está no canal de vídeo agora.
    src: Option<SrcId>,
    /// Forçado pelo comando `tier` (None = a regra do ViewMessage).
    tier_force: Option<Kind>,
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
    placa_mode: bool,
    path: PathKind,
    hw_full: Option<HwStream>,
    hw_reduced: Option<HwStream>,
    /// Relógio comum do vídeo e do som (começo da geração).
    epoch: Instant,
    next_src: u64,
    live_seq: u64,
    /// requestId do `live` em vigor (o `unlive` que o encerra repete o requestId e o número de ordem).
    live_req: String,
    reopen_t0: Option<Instant>,
    nicks: HashMap<String, String>,
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
    if let Some(secs) = args.encode_only {
        // Sem sala e sem rede: só gera, comprime pela placa e imprime o resumo (T010).
        let code = tokio::task::spawn_blocking(move || encode_only(&args, secs)).await.unwrap_or(1);
        std::process::exit(code);
    }
    if let Err(e) = run(args).await {
        eprintln!("erro: {e}");
        std::process::exit(1);
    }
}

/// `--encode-only <segundos>` (T010): gera NV12 no ritmo real, comprime pela placa e imprime o resumo. Não entra em
/// sala. Pede um quadro inteiro a cada 2 s e muda o alvo de taxa uma vez, no meio. Devolve o código de saída:
/// 0 ok, 3 sem codificador por hardware, 4 o codificador achado não iniciou.
fn encode_only(args: &Args, secs: u64) -> i32 {
    use crate::hw_encoder::{Config, HwEncoder, StartError};
    struct Rec {
        t: f64,
        size: usize,
        key: bool,
    }
    let (w, h, fps) = (args.width, args.height, args.gen_fps.clamp(1, 120));
    println!(
        "--encode-only {secs} s: {w}x{h} a {fps} qps, taxa {} bps (muda para {} bps no meio), quadro inteiro pedido a cada 2 s, vídeo gerado (barras, contador e ruído)",
        args.encode_bitrate, args.encode_bitrate_after
    );
    let cfg = Config { width: w, height: h, fps, bitrate: args.encode_bitrate };
    let (mut enc, rx) = match HwEncoder::start(cfg) {
        Ok(x) => x,
        Err(StartError::NotFound) => {
            println!("RESULTADO: não achou codificador de H.264 por hardware neste computador. Nada a comprimir; saindo.");
            return 3;
        }
        Err(e) => {
            println!("RESULTADO: {e}. Nada a comprimir; saindo.");
            return 4;
        }
    };
    let t0 = Instant::now();
    let collector = std::thread::spawn(move || {
        let mut v = Vec::new();
        let mut delays = Vec::new();
        while let Ok(f) = rx.recv() {
            v.push(Rec { t: t0.elapsed().as_secs_f64(), size: f.data.len(), key: f.key });
            delays.push(f.delay_us);
        }
        (v, delays)
    });

    let mut gen = texture::Nv12Generator::new(w, h, true);
    let period = Duration::from_secs_f64(1.0 / fps as f64);
    let total = Duration::from_secs(secs);
    let mut next = t0;
    let mut n: u64 = 0;
    let mut next_key = 2.0f64;
    let mut key_reqs: Vec<f64> = Vec::new();
    let mut rate_change: Option<(f64, Result<(), String>)> = None;
    while t0.elapsed() < total {
        let elapsed = t0.elapsed();
        let frame = gen.frame(n, elapsed);
        if let Err(e) = enc.submit(frame, elapsed.as_micros() as i64) {
            println!("erro ao entregar: {e}");
            break;
        }
        n += 1;
        let now_s = t0.elapsed().as_secs_f64();
        if now_s >= next_key && now_s < total.as_secs_f64() - 0.5 {
            key_reqs.push(now_s);
            let _ = enc.request_keyframe();
            next_key += 2.0;
        }
        if rate_change.is_none() && now_s >= total.as_secs_f64() / 2.0 {
            let r = enc.set_bitrate(args.encode_bitrate_after);
            rate_change = Some((t0.elapsed().as_secs_f64(), r));
        }
        next += period;
        // Dorme até perto e espera o resto girando, porque o sono do Windows tem grão de ~1 ms.
        loop {
            let now = Instant::now();
            if now >= next {
                if now - next > period * 5 {
                    next = now;
                }
                break;
            }
            let left = next - now;
            if left > Duration::from_millis(2) {
                std::thread::sleep(left - Duration::from_millis(1));
            } else {
                std::hint::spin_loop();
            }
        }
    }
    std::thread::sleep(Duration::from_millis(500)); // deixa sair o que o codificador ainda segura
    let stats = enc.stats().clone();
    let name = enc.name().to_string();
    enc.stop();
    let (recs, delays) = collector.join().unwrap_or_default();

    let ld = |a: &std::sync::atomic::AtomicU64| a.load(Ordering::Relaxed);
    let (delivered, accepted, received) = (ld(&stats.delivered), ld(&stats.accepted), ld(&stats.received));
    println!("\n===== resumo do --encode-only =====");
    println!("codificador: {name}");
    println!("quadros entregues: {delivered} | aceitos pelo codificador: {accepted} | recebidos comprimidos: {received} | descartados na fila: {} | erros: {}", ld(&stats.dropped), ld(&stats.errors));
    if !delays.is_empty() {
        let avg = delays.iter().sum::<u64>() as f64 / delays.len() as f64 / 1000.0;
        let max = *delays.iter().max().unwrap_or(&0) as f64 / 1000.0;
        println!("atraso entre entregar e receber: médio {avg:.1} ms, maior {max:.1} ms");
    }
    let keys: Vec<&Rec> = recs.iter().filter(|r| r.key).collect();
    if keys.is_empty() {
        println!("quadros inteiros: 0");
    } else {
        let kavg = keys.iter().map(|r| r.size).sum::<usize>() as f64 / keys.len() as f64;
        println!("quadros inteiros: {} | tamanho médio do quadro inteiro: {:.0} bytes ({:.1} KB)", keys.len(), kavg, kavg / 1024.0);
        let at: Vec<String> = keys.iter().map(|r| format!("{:.2}", r.t)).collect();
        println!("  instantes dos quadros inteiros (s): {}", at.join(" "));
    }
    // Quadro inteiro a pedido: primeiro inteiro que chegou depois de cada pedido.
    let mut served = 0;
    for (i, tr) in key_reqs.iter().enumerate() {
        match recs.iter().position(|r| r.key && r.t > *tr) {
            Some(k) => {
                let between = recs.iter().filter(|r| r.t > *tr && r.t <= recs[k].t).count();
                served += 1;
                println!("  pedido {} em {:.2} s: quadro inteiro chegou {:.0} ms depois (o {}º quadro recebido depois do pedido)", i + 1, tr, (recs[k].t - tr) * 1000.0, between);
            }
            None => println!("  pedido {} em {:.2} s: nenhum quadro inteiro chegou depois", i + 1, tr),
        }
    }
    println!(
        "quadro inteiro a pedido: {served} de {} pedidos atendidos | ForceKeyFrame aceito {} vez(es), recusado {}",
        key_reqs.len(),
        ld(&stats.force_key_ok),
        ld(&stats.force_key_refused)
    );
    // Taxa ao vivo: tamanho médio antes e depois, sem o primeiro segundo de cada janela (o codificador se ajusta).
    match rate_change {
        None => println!("mudança de taxa: não aconteceu (a execução foi curta demais)"),
        Some((tc, r)) => {
            println!("mudança de taxa em {tc:.2} s: {} -> {} bps: {}", args.encode_bitrate, args.encode_bitrate_after, match &r { Ok(()) => "ACEITA ao vivo".to_string(), Err(e) => format!("NÃO ACEITA ({e})") });
            let end = secs as f64;
            let win = |a: f64, b: f64| -> Option<(usize, f64, f64, f64)> {
                let sel: Vec<&Rec> = recs.iter().filter(|r| r.t >= a && r.t < b).collect();
                if sel.is_empty() || b <= a {
                    return None;
                }
                let tot: usize = sel.iter().map(|r| r.size).sum();
                let non: Vec<&&Rec> = sel.iter().filter(|r| !r.key).collect();
                let non_avg = if non.is_empty() { 0.0 } else { non.iter().map(|r| r.size).sum::<usize>() as f64 / non.len() as f64 };
                Some((sel.len(), tot as f64 / sel.len() as f64, non_avg, tot as f64 * 8.0 / (b - a) / 1e6))
            };
            for (label, a, b) in [("antes ", 1.0, tc), ("depois", tc + 1.0, end)] {
                match win(a, b) {
                    Some((cnt, avg, non, mbps)) => println!("  {label} ({a:.1} s a {b:.1} s): {cnt} quadros | tamanho médio {avg:.0} bytes | só parciais {non:.0} bytes | taxa medida {mbps:.2} Mbps"),
                    None => println!("  {label}: sem quadros na janela"),
                }
            }
        }
    }
    println!("propriedades recusadas ao vivo: taxa {} vez(es)", ld(&stats.rate_refused));
    0
}

/// A cheia: o tamanho, os quadros por segundo e a taxa da qualidade de referência.
fn full_cfg(args: &Args, q: Quality) -> TierCfg {
    let bitrate = args.bitrate.unwrap_or(q.max_bitrate).min(u32::MAX as u64) as u32;
    TierCfg { name: "cheia", w: args.width, h: args.height, fps: q.fps, bitrate, noise: args.ruido }
}

/// A reduzida (R6, R18): miniatura 360p15 a 500 kbps; com --reduzida-fraca, 720p30 a 1,5 Mbps (ou, se a cheia já é
/// 720p30 ou menor, o mesmo tamanho da cheia com 1,5 Mbps ou metade da taxa dela, o que for menor).
fn reduced_cfg(args: &Args, full: TierCfg) -> TierCfg {
    if !args.reduzida_fraca {
        return TierCfg { name: "reduzida", w: 640, h: 360, fps: 15, bitrate: 500_000, noise: args.ruido };
    }
    if full.h <= 720 && full.fps <= 30 {
        return TierCfg { name: "reduzida", w: full.w, h: full.h, fps: full.fps, bitrate: 1_500_000.min(full.bitrate / 2), noise: args.ruido };
    }
    TierCfg { name: "reduzida", w: 1280, h: 720, fps: 30, bitrate: 1_500_000, noise: args.ruido }
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
    log!("sonda 020 | libwebrtc 0.4.0 | sala {} | base {} | Origin {}", args.room, args.base, origin_of(&args.base));
    let factory = if args.low_latency { PeerConnectionFactory::with_zero_playout_delay() } else { PeerConnectionFactory::default() };
    // `placa`: o vídeo vem já comprimido pela placa. O caminho pelo processador (e os valores antigos de
    // --backend, que são o caminho da sonda do G1) usa a fonte crua e o codificador da biblioteca.
    let placa_mode = args.backend == "placa";
    let backend = if placa_mode { VideoEncoderBackend::Software } else { parse_backend(&args.backend) };

    // O que o motor diz sobre codecs e codificadores (item f e preferência de codec).
    let avail: Vec<_> = VideoEncoderBackend::list_available().into_iter().collect();
    log!("codificadores disponíveis neste build: {avail:?}");
    for b in [VideoEncoderBackend::Auto, VideoEncoderBackend::Software, VideoEncoderBackend::Hardware, VideoEncoderBackend::PreEncoded] {
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
    let height = args.height;
    let fps = args.gen_fps.clamp(1, 60);
    let quality = Quality { resolution: height, fps, max_bitrate: bitrate_for(height, fps) };
    let epoch = Instant::now();
    let mut hw_full: Option<HwStream> = None;
    let mut path = PathKind::Cpu;
    let mut next_src = 0u64;
    if !args.no_live {
        let asrc = media::audio_source();
        media::spawn_audio_generator(asrc.clone(), stop.clone(), gen_audio_samples.clone());
        audio_track = Some(factory.create_audio_track("sonda-som", asrc));
        let mut software = true;
        if placa_mode {
            if args.sem_placa {
                log!("--sem-placa: caminho pelo processador desde o começo (a biblioteca comprime)");
            } else {
                next_src += 1;
                let cfg = full_cfg(&args, quality);
                let started = tokio::task::block_in_place(|| HwStream::start(&factory, cfg, next_src, epoch, args.sem_aud, gen_video_frames.clone()));
                match started {
                    Ok(s) => {
                        log!("caminho pela placa: cheia {}x{} a {} qps, {} bps, codificador '{}'", cfg.w, cfg.h, cfg.fps, cfg.bitrate, s.encoder_name);
                        hw_full = Some(s);
                        path = PathKind::Placa;
                        software = false;
                    }
                    Err(e) => log!("{e}: caminho pelo processador desde o começo (a biblioteca comprime)"),
                }
            }
        }
        if software {
            let source = NativeVideoSource::new(VideoResolution { width: args.width, height: args.height }, true);
            media::spawn_video_generator(source.clone(), args.width, args.height, args.gen_fps, stop.clone(), gen_video_frames.clone());
            video_track = Some(factory.create_video_track("sonda-video", source));
        }
        log!("fontes geradas: vídeo {}x{} a {} qps (barras, contador e clarão; {}) e tom de 440 Hz com bip de 1 kHz a cada segundo", args.width, args.height, args.gen_fps, if hw_full.is_some() { "NV12 comprimido pela placa" } else { "I420 cru, a biblioteca comprime" });
    }

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
        placa_mode,
        path,
        hw_full,
        hw_reduced: None,
        epoch,
        next_src,
        live_seq: 0,
        live_req: String::new(),
        reopen_t0: None,
        nicks: HashMap::new(),
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
                    self.nicks.insert(id.clone(), p["nickname"].as_str().unwrap_or("").to_string());
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
                    self.send_live(seq);
                }
            }
            "live-ack" => {
                if self.pending_live.as_deref() == v["requestId"].as_str() {
                    self.pending_live = None;
                    if let Some(started) = v["live"]["startedAt"].as_u64() {
                        let key = TransmissionKey { peer_id: self.me.clone(), started_at: started };
                        log!("live-ack: transmissão {} startedAt={}", short(&key.peer_id), started);
                        if let Some(t0) = self.reopen_t0.take() {
                            log!("[troca de caminho] reabertura: live-ack {} ms depois do comando (a imagem volta quando quem assiste mandar a view)", t0.elapsed().as_millis());
                        }
                        self.set_local_key(Some(key)).await;
                    }
                }
            }
            "peer-joined" => {
                if let Some(id) = v["peer"]["id"].as_str() {
                    self.nicks.insert(id.to_string(), v["peer"]["nickname"].as_str().unwrap_or("").to_string());
                }
                log!("entrou: {} ({})", short(v["peer"]["id"].as_str().unwrap_or("?")), v["peer"]["nickname"])
            }
            "peer-left" => {
                let id = v["id"].as_str().unwrap_or("").to_string();
                log!("saiu: {}", short(&id));
                if let Some(l) = self.links.remove(&id) {
                    l.pc.close();
                }
                self.remote_keys.remove(&id);
                self.rx.remove(&id);
                self.nicks.remove(&id);
                self.reconcile_reduced();
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

    /// Anuncia a transmissão (`live`) com esse número de ordem.
    fn send_live(&mut self, seq: u64) {
        let id = uuid::Uuid::new_v4().to_string();
        self.pending_live = Some(id.clone());
        self.live_seq = seq;
        self.live_req = id.clone();
        let res = if self.args.height >= 1000 { 1080 } else { 720 };
        let fps = if self.args.gen_fps >= 45 { 60 } else { 30 };
        let m = json!({"t":"live","requestId":id,"broadcastSeq":seq,"resolution":res,"fps":fps,"audio":"pc","source":"screen","label":"Sonda 020"});
        let _ = self.ws_tx.send(Message::Text(m.to_string()));
        log!("→ live (broadcastSeq {seq}, {res}p{fps})");
    }

    /// Encerra a transmissão (`unlive`): o servidor só aceita repetindo o requestId e o número de ordem do `live` em
    /// vigor (shared/media.ts, nextBroadcastOrder); o `live` seguinte usa o número de ordem + 1. Não há resposta do
    /// servidor: quem assiste recebe `peer-live` sem transmissão.
    fn send_unlive(&mut self) -> u64 {
        let seq = self.live_seq;
        let m = json!({"t":"unlive","requestId":self.live_req,"broadcastSeq":seq});
        let _ = self.ws_tx.send(Message::Text(m.to_string()));
        log!("→ unlive (broadcastSeq {seq})");
        seq
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
            if sdp_type == SdpType::Answer {
                log_video_negotiation(from, "resposta de quem assiste", sdp);
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
                        log_video_negotiation(from, "minha resposta", &ans.to_string());
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
            src: None,
            tier_force: None,
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

    /// O que a ligação recebe agora, pela regra do ViewMessage (R7): `full` a cheia, `thumb` a reduzida; `hidden`, `off`
    /// e sem ViewMessage, nada (quem chama só pergunta se `wants_video`). `tier_force` (comando `tier`) troca uma pela
    /// outra sem esperar regra de banda. No caminho pelo processador só existe a fonte crua.
    fn kind_for(&self, their: ViewMode, force: Option<Kind>) -> Kind {
        if self.path == PathKind::Cpu {
            return Kind::Software;
        }
        force.filter(|k| *k != Kind::Software).unwrap_or(match their {
            ViewMode::Full => Kind::Full,
            _ => Kind::Reduced,
        })
    }

    /// A fonte crua (a biblioteca comprime). No `--backend placa` ela só nasce quando o caminho pelo processador é
    /// preciso (`--sem-placa`, sem codificador, ou `fail`).
    fn ensure_software(&mut self) -> Option<RtcVideoTrack> {
        if self.video_track.is_none() && !self.args.no_live {
            let a = &self.args;
            let source = NativeVideoSource::new(VideoResolution { width: a.width, height: a.height }, true);
            media::spawn_video_generator(source.clone(), a.width, a.height, a.gen_fps, self.stop.clone(), self.gen_video_frames.clone());
            self.video_track = Some(self.factory.create_video_track("sonda-video", source));
            log!("fonte crua criada: vídeo I420 {}x{} a {} qps (a biblioteca comprime, uma vez por ligação)", a.width, a.height, a.gen_fps);
        }
        self.video_track.clone()
    }

    /// Faixa, identificação e backend da fonte `kind`. A reduzida é criada aqui, só quando alguém precisa dela.
    fn src_for(&mut self, kind: Kind) -> Option<(RtcVideoTrack, SrcId, VideoEncoderBackend)> {
        match kind {
            Kind::Software => {
                let t = self.ensure_software()?;
                Some((t, SrcId { kind, gen: 0 }, self.backend))
            }
            Kind::Full => match &self.hw_full {
                Some(s) => Some((s.track.clone(), SrcId { kind, gen: s.gen }, VideoEncoderBackend::PreEncoded)),
                None => self.src_for(Kind::Software),
            },
            Kind::Reduced => {
                if self.hw_reduced.is_none() {
                    let Some(full) = self.hw_full.as_ref().map(|s| s.cfg) else { return self.src_for(Kind::Software) };
                    let cfg = reduced_cfg(&self.args, full);
                    self.next_src += 1;
                    let gen = self.next_src;
                    let t0 = Instant::now();
                    let r = tokio::task::block_in_place(|| HwStream::start(&self.factory, cfg, gen, self.epoch, self.args.sem_aud, Arc::new(AtomicU64::new(0))));
                    match r {
                        Ok(s) => {
                            log!("reduzida criada em {} ms: {}x{} a {} qps, {} bps, codificador '{}'", t0.elapsed().as_millis(), cfg.w, cfg.h, cfg.fps, cfg.bitrate, s.encoder_name);
                            self.hw_reduced = Some(s);
                        }
                        Err(e) => {
                            log!("a reduzida não iniciou ({e}); quem a pediria recebe a cheia");
                            return self.src_for(Kind::Full);
                        }
                    }
                }
                let s = self.hw_reduced.as_ref()?;
                Some((s.track.clone(), SrcId { kind, gen: s.gen }, VideoEncoderBackend::PreEncoded))
            }
        }
    }

    /// Para a reduzida quando nenhuma ligação com vídeo ligado a recebe mais.
    fn reconcile_reduced(&mut self) {
        let Some(r) = self.hw_reduced.as_ref() else { return };
        let gen = r.gen;
        let live = self.local_key.is_some();
        let needed = self.links.values().any(|l| live && l.their_view.wants_video() && l.src.map_or(false, |s| s.kind == Kind::Reduced && s.gen == gen));
        if !needed {
            if let Some(s) = self.hw_reduced.take() {
                log!("reduzida parada: ninguém a recebe");
                tokio::task::block_in_place(|| s.stop());
            }
        }
    }

    /// Taxa nominal da qualidade `kind` (o teto de `max_bitrate` do canal quando o vídeo vem da placa).
    fn nominal_bitrate(&self, kind: Kind) -> u64 {
        match kind {
            Kind::Reduced => self.hw_reduced.as_ref().map(|s| s.cfg.bitrate as u64),
            _ => self.hw_full.as_ref().map(|s| s.cfg.bitrate as u64),
        }
        .unwrap_or(self.quality.max_bitrate)
    }

    /// Parâmetros do canal de vídeo. Vindo da placa, só `active` e `max_bitrate` valem: tamanho e quadros por segundo
    /// já saem do codificador, e `scaleResolutionDownBy` ou `maxFramerate` só fariam a biblioteca descartar quadros
    /// antes do repassador (e quebrar a cadeia de quadros). Vindo da fonte crua, a regra do G1 (`send_params`).
    fn video_params(&self, their: ViewMode, force: Option<Kind>, kind: Option<Kind>) -> SendParams {
        let p = match kind {
            Some(k) if k.from_placa() => SendParams { active: their.wants_video(), scale_resolution_down_by: 1.0, max_framerate: self.quality.fps as f64, max_bitrate: self.nominal_bitrate(k) },
            _ => {
                let mode = match (force, their) {
                    (Some(Kind::Reduced), m) if m.wants_video() => ViewMode::Thumb,
                    (Some(Kind::Full), m) if m.wants_video() => ViewMode::Full,
                    (_, m) => m,
                };
                send_params(self.quality, mode)
            }
        };
        self.overrides.apply(p)
    }

    /// Começa ou para de enviar a nossa imagem e o nosso som a essa pessoa (mesma regra do `setLocal` do site).
    async fn set_local(&mut self, id: &str) {
        let Some(at) = self.audio_track.clone() else { return };
        let Some(l) = self.links.get(id) else { return };
        let live = self.local_key.is_some();
        let want_video = live && l.their_view.wants_video();
        let want_sound = live && l.their_view.wants_sound();
        let pc = l.pc.clone();
        let their = l.their_view;
        let (force, cur) = (l.tier_force, l.src);
        let (vtx, atx) = (l.video_tx.clone(), l.audio_tx.clone());

        let mut chosen = None;
        if want_video {
            let k = self.kind_for(their, force);
            chosen = self.src_for(k);
        }
        let p = self.video_params(their, force, chosen.as_ref().map(|c| c.1.kind).or(cur.map(|c| c.kind)));

        let mut new_v = None;
        let mut new_a = None;
        let mut new_src = None;
        match (vtx, chosen) {
            (Some(tx), Some((vt, sid, backend))) => {
                // Não tira a faixa com set_track(None): nessa crate, remover a faixa de um envio ativo
                // derrubava o processo (crash ao a pessoa passar a me ver em 'hidden'/'off'). Parar o
                // envio é feito por active=false em apply_encodings, como o site (sendParams). Trocar a faixa de
                // uma fonte pela de outra (cheia/reduzida, ou placa/processador) é set_track(Some(...)), sem renegociar.
                if cur != Some(sid) {
                    let was_hw = cur.map_or(false, |c| c.kind.from_placa());
                    if cur.is_none() || was_hw != sid.kind.from_placa() {
                        tx.sender().set_video_encoder_backend(backend);
                    }
                    let t0 = Instant::now();
                    match tx.sender().set_track(Some(MediaStreamTrack::Video(vt))) {
                        Ok(()) => {
                            log!("{}: faixa trocada {} -> {} em {} ms (sem renegociar), backend {backend:?}", short(id), cur.map_or("(nenhuma)", |c| c.kind.name()), sid.kind.name(), t0.elapsed().as_millis());
                            new_src = Some(sid);
                        }
                        Err(e) => log!("{}: sender.set_track(vídeo) falhou: {e}", short(id)),
                    }
                }
            }
            (None, Some((vt, sid, backend))) => {
                // O teto entra junto com o canal (como no site): antes da 1ª negociação set_parameters não vale.
                let hw = sid.kind.from_placa();
                let enc = if hw {
                    RtpEncodingParameters { active: p.active, max_bitrate: Some(p.max_bitrate), ..Default::default() }
                } else {
                    RtpEncodingParameters {
                        active: p.active,
                        max_bitrate: Some(p.max_bitrate),
                        max_framerate: Some(p.max_framerate),
                        scale_resolution_down_by: Some(p.scale_resolution_down_by),
                        ..Default::default()
                    }
                };
                let init = RtpTransceiverInit { direction: RtpTransceiverDirection::SendOnly, stream_ids: vec![STREAM_ID.into()], send_encodings: vec![enc] };
                match pc.add_transceiver(MediaStreamTrack::Video(vt), init) {
                    Ok(tx) => {
                        if !self.args.no_h264 {
                            let mut caps = self.factory.get_rtp_sender_capabilities(MediaType::Video).codecs;
                            if hw {
                                // A ordem vem de video_encoder_codecs(PreEncoded): só entra o codec que o repassador sabe
                                // repassar; H.264 primeiro.
                                let ok = self.factory.video_encoder_codecs(VideoEncoderBackend::PreEncoded);
                                caps.retain(|c| c.mime_type.rsplit('/').next().and_then(|n| n.parse::<VideoCodec>().ok()).map_or(false, |vc| ok.contains(&vc)));
                            }
                            let ordered = order_codecs(caps);
                            if hw {
                                let first: Vec<String> = ordered.iter().take(3).map(|c| format!("{} {:?}", c.mime_type, c.sdp_fmtp_line)).collect();
                                log!("{}: ordem de codecs (PreEncoded, H.264 primeiro): {first:?}", short(id));
                            }
                            if let Err(e) = tx.set_codec_preferences(ordered) {
                                log!("{}: set_codec_preferences falhou: {e}", short(id));
                            }
                        }
                        tx.sender().set_video_encoder_backend(backend);
                        log!("{}: canal de vídeo criado (H.264 primeiro={}, backend={:?}, fonte={})", short(id), !self.args.no_h264, backend, sid.kind.name());
                        new_v = Some(tx);
                        new_src = Some(sid);
                    }
                    Err(e) => log!("{}: add_transceiver(vídeo) falhou: {e}", short(id)),
                }
            }
            _ => {}
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
            if new_src.is_some() {
                l.src = new_src;
            }
            if new_a.is_some() {
                l.audio_tx = new_a;
            }
        }
        log!("{}: set_local → vídeo={want_video} som={want_sound} (a pessoa me vê em '{}')", short(id), their.as_str());
        self.apply_encodings(id).await;
        self.reconcile_reduced();
    }

    /// Aplica bitrate, fps, scaleResolutionDownBy e active no canal de vídeo (e active/bitrate no som).
    async fn apply_encodings(&mut self, id: &str) {
        let Some(l) = self.links.get(id) else { return };
        if l.pc.signaling_state() != SignalingState::Stable {
            return;
        }
        let kind = l.src.map(|s| s.kind);
        let hw = kind.map_or(false, |k| k.from_placa());
        let p = self.video_params(l.their_view, l.tier_force, kind);
        let their = l.their_view;
        if let Some(tx) = &l.video_tx {
            let s = tx.sender();
            if s.track().is_some() {
                let mut params = s.parameters();
                match params.encodings.get_mut(0) {
                    Some(e) => {
                        e.active = p.active;
                        e.max_bitrate = Some(p.max_bitrate);
                        if !hw {
                            e.max_framerate = Some(p.max_framerate);
                            e.scale_resolution_down_by = Some(p.scale_resolution_down_by);
                            // Tela: mantém resolução (contentPreference do site para 'screen').
                            params.set_degradation_preference(DegradationPreference::MaintainResolution);
                        }
                        match s.set_parameters(params) {
                            Ok(()) => {
                                let back = s.parameters();
                                let e0 = back.encodings.first();
                                log!(
                                    "{}: set_parameters(vídeo) ok → pedi active={} bitrate={} fps={} scale={:.2}{} | o motor guardou {:?}",
                                    short(id), p.active, p.max_bitrate, p.max_framerate, p.scale_resolution_down_by,
                                    if hw { " (da placa: só active e bitrate valem)" } else { "" },
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
        let arg2 = it.next();
        match (cmd, arg) {
            ("", _) => {}
            ("quit" | "q" | "sair", _) => return false,
            ("stats", _) => self.report_stats(true).await,
            ("peers", _) => self.cmd_peers(),
            ("tier", Some(p)) => self.cmd_tier(p, arg2).await,
            ("tier", None) => log!("tier: use  tier <início do id> full|reduced|auto"),
            ("fail", a) => self.do_fail(a == Some("--reabrir")).await,
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
            _ => log!("comando desconhecido: '{l}' (bitrate N | fps N | scale X | active on|off | reset | view M | stats | peers | tier ID full|reduced|auto | fail [--reabrir] | quit)"),
        }
        log!("ajustes atuais: {}", self.overrides.describe());
        true
    }

    // ------------------------------------------------------------ comandos da prova (T012)

    /// `peers`: quem recebe o quê.
    fn cmd_peers(&self) {
        let live = self.local_key.is_some();
        log!(
            "[peers] caminho: {} | transmitindo: {} | {} ligação(ões) | cheia: {} | reduzida: {}",
            if self.path == PathKind::Placa { "placa" } else { "processador" },
            live,
            self.links.len(),
            if self.hw_full.is_some() { "no ar" } else { "parada" },
            if self.hw_reduced.is_some() { "no ar" } else { "parada" }
        );
        let mut ids: Vec<&String> = self.links.keys().collect();
        ids.sort();
        for id in ids {
            let l = &self.links[id];
            let recebe = if live && l.their_view.wants_video() {
                l.src.map_or("vídeo ligado, faixa ainda não posta".to_string(), |s| format!("{}{}", s.kind.name(), if s.kind.from_placa() { " (placa)" } else { " (biblioteca)" }))
            } else {
                "nada (vídeo desligado: active=false)".to_string()
            };
            log!(
                "[peers] {} '{}' | vê-me em '{}' | vídeo que recebe: {} | forçado: {} | som: {} | canal ctl: {}",
                short(id),
                self.nicks.get(id).map(String::as_str).unwrap_or("?"),
                l.their_view.as_str(),
                recebe,
                l.tier_force.map_or("não (regra do ViewMessage)", |k| k.name()),
                if live && l.their_view.wants_sound() { "sim" } else { "não" },
                if l.ctl_open || self.ctl_ready(id) { "aberto" } else { "fechado" }
            );
        }
    }

    /// `tier <início do id> full|reduced|auto`: força um espectador para a cheia ou a reduzida, sem esperar regra de banda.
    async fn cmd_tier(&mut self, prefix: &str, which: Option<&str>) {
        let kind = match which {
            Some("full" | "cheia") => Some(Kind::Full),
            Some("reduced" | "reduzida") => Some(Kind::Reduced),
            Some("auto") => None,
            _ => {
                log!("tier: use  tier <início do id> full|reduced|auto");
                return;
            }
        };
        let found: Vec<String> = self.links.keys().filter(|i| i.starts_with(prefix)).cloned().collect();
        let id = match found.as_slice() {
            [] => {
                log!("tier: nenhuma ligação começa com '{prefix}' (veja `peers`)");
                return;
            }
            [one] => one.clone(),
            _ => {
                log!("tier: '{prefix}' serve para {} ligações; digite mais do id", found.len());
                return;
            }
        };
        if self.path == PathKind::Cpu {
            log!("tier: no caminho pelo processador a cheia e a reduzida são os parâmetros da biblioteca nessa ligação (G1)");
        }
        if let Some(l) = self.links.get_mut(&id) {
            l.tier_force = kind;
        }
        let t0 = Instant::now();
        self.set_local(&id).await;
        log!("{}: tier → {} aplicado em {} ms; as outras ligações não foram tocadas", short(&id), kind.map_or("auto", |k| k.name()), t0.elapsed().as_millis());
    }

    /// `fail`: simula a falha do codificador da placa e troca todas as ligações para o caminho pelo processador, sem
    /// renegociar (troca da faixa de cada canal e do backend do envio). `fail --reabrir`: plano B (R11), para e reabre a
    /// transmissão sozinha (`unlive` e `live`). Em qualquer um, imprime a pausa em milissegundos.
    async fn do_fail(&mut self, reopen: bool) {
        if !self.placa_mode {
            log!("fail: só vale com --backend placa");
            return;
        }
        if self.path == PathKind::Cpu {
            log!("fail: já está no caminho pelo processador");
            return;
        }
        let t0 = Instant::now();
        log!("[troca de caminho] placa → processador {} (simulando a falha do codificador da placa)", if reopen { "com o plano B: parar e reabrir a transmissão" } else { "sem renegociar" });
        let live = self.local_key.is_some();
        let targets: Vec<(String, PeerConnection)> =
            self.links.iter().filter(|(_, l)| live && l.their_view.wants_video() && l.video_tx.is_some()).map(|(i, l)| (i.clone(), l.pc.clone())).collect();
        self.path = PathKind::Cpu;
        // Nenhum quadro novo sai da placa a partir daqui.
        if let Some(s) = self.hw_reduced.take() {
            tokio::task::block_in_place(|| s.stop());
        }
        if let Some(s) = self.hw_full.take() {
            tokio::task::block_in_place(|| s.stop());
        }
        log!("[troca de caminho] codificadores da placa parados em {} ms", t0.elapsed().as_millis());
        // Marca de referência: quadros que cada ligação já tinha codificado.
        for (id, pc) in targets {
            let base = video_frames_encoded(&pc).await.unwrap_or(0);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    if t0.elapsed() > Duration::from_secs(20) {
                        log!("[troca de caminho] {}: o vídeo não voltou em 20 s", short(&id));
                        return;
                    }
                    if video_frames_encoded(&pc).await.map_or(false, |c| c > base) {
                        log!("[troca de caminho] {}: o vídeo voltou {} ms depois do comando (pausa, resolução de cerca de 50 ms; veja também 'maior intervalo entre quadros' de quem assiste)", short(&id), t0.elapsed().as_millis());
                        return;
                    }
                }
            });
        }
        self.ensure_software();
        if reopen {
            self.reopen_t0 = Some(t0);
            self.send_unlive();
            self.set_local_key(None).await;
            let seq = self.live_seq + 1;
            self.send_live(seq);
        } else {
            let ids: Vec<String> = self.links.keys().cloned().collect();
            for id in ids {
                self.set_local(&id).await;
            }
        }
        log!("[troca de caminho] comandos aplicados em {} ms", t0.elapsed().as_millis());
    }

    /// Linha `[codificador]` de cada qualidade da placa no ar.
    fn report_encoders(&self) {
        for s in [self.hw_full.as_ref(), self.hw_reduced.as_ref()].into_iter().flatten() {
            let st = &s.stats;
            let ld = |a: &AtomicU64| a.load(Ordering::Relaxed);
            let (delivered, received) = (ld(&st.delivered), ld(&st.received));
            let avg = if received > 0 { ld(&st.delay_sum_us) as f64 / received as f64 / 1000.0 } else { 0.0 };
            let keys = ld(&st.keyframes);
            let key_avg = if keys > 0 { ld(&st.key_bytes) as f64 / keys as f64 / 1024.0 } else { 0.0 };
            log!(
                "[codificador] {} '{}' {}x{} a {} qps | entregues {} recebidos {} (descartados na fila {}) | atraso entregar→receber médio {:.1} ms, maior {:.1} ms | quadros inteiros {} (tamanho médio {:.0} KB; pedidos da biblioteca {}, emitidos {}) | erros {} (recusados pela fonte {}) | NAL 9 tirados {} | taxa pedida pela biblioteca {:?}",
                s.cfg.name, s.encoder_name, s.cfg.w, s.cfg.h, s.cfg.fps, delivered, received, ld(&st.dropped), avg, ld(&st.delay_max_us) as f64 / 1000.0, keys, key_avg,
                ld(&s.fwd.key_requests), ld(&s.fwd.key_issued), ld(&st.errors), ld(&s.fwd.refused), ld(&s.fwd.aud_stripped), s.fwd.last_rate.lock().unwrap().map(|(b, f)| format!("{b} bps a {f:.0} qps"))
            );
        }
    }

    // ------------------------------------------------------------ estatísticas

    async fn report_stats(&mut self, force_all: bool) {
        let p = self.proc.sample();
        log!(
            "[proc] cpu {:.1}% de um núcleo ({:.1}% da máquina, {} núcleos lógicos) | memória privada {:.0} MB | working set {:.0} MB | gerados: {} quadros, {} amostras",
            p.cpu_one_core_pct, p.cpu_machine_pct, self.proc.cores(), p.private_mb, p.working_set_mb,
            self.gen_video_frames.load(Ordering::Relaxed), self.gen_audio_samples.load(Ordering::Relaxed)
        );
        self.report_encoders();
        let hw_names = (self.hw_full.as_ref().map(|s| s.encoder_name.clone()), self.hw_reduced.as_ref().map(|s| s.encoder_name.clone()));
        let pcs: Vec<(String, PeerConnection, ViewMode, Option<SrcId>)> = self.links.iter().map(|(i, l)| (i.clone(), l.pc.clone(), l.their_view, l.src)).collect();
        for (id, pc, their, src) in pcs {
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
                            let (origem, codificador) = match src.map(|s| s.kind) {
                                Some(Kind::Full) => ("placa", hw_names.0.clone().unwrap_or_default()),
                                Some(Kind::Reduced) => ("placa", hw_names.1.clone().unwrap_or_default()),
                                Some(Kind::Software) => ("biblioteca", "biblioteca (veja encoderImplementation)".to_string()),
                                None => ("nenhuma", "-".to_string()),
                            };
                            log!(
                                "{}: [envio vídeo] origem={} codificador='{}' | vê-me em '{}' | {}x{} a {:.0} qps | enviando {:.0} kbps (alvo {:.0}) | qualityLimitationReason={:?} | encoderImplementation='{}' powerEfficientEncoder={} | quadros {} (chaves {}) pli={} nack={} | active={} | tempo de codificação {:.1} ms/quadro",
                                short(&id), origem, codificador, their.as_str(), o.frame_width, o.frame_height, o.frames_per_second, kbps, o.target_bitrate / 1000.0,
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
                        "{}: [recepção na sonda] vídeo {:.1} qps entregues ({} quadros no total, último {}x{}, fora de I420: {}, maior intervalo entre quadros {} ms) | som {:.0} amostras/s ({} no total)",
                        short(&id), (f - f0) as f64 / dt, f, c.video_w.load(Ordering::Relaxed), c.video_h.load(Ordering::Relaxed),
                        c.video_non_i420.load(Ordering::Relaxed), c.max_gap_ms.swap(0, Ordering::Relaxed), (a - a0) as f64 / dt, a
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

/// Quadros de vídeo que o motor já codificou nesta ligação (para medir a pausa de uma troca de caminho).
async fn video_frames_encoded(pc: &PeerConnection) -> Option<u64> {
    let stats = pc.get_stats().await.ok()?;
    for s in stats {
        if let RtcStats::OutboundRtp(o) = s {
            if o.stream.kind == "video" {
                return Some(o.outbound.frames_encoded as u64);
            }
        }
    }
    None
}

/// Linhas de uma SDP que dizem que codec cada seção de vídeo escolheu (o primeiro da lista `m=video`), com o `fmtp`
/// (onde está o `profile-level-id`). Imprime o que a resposta aceitou.
fn log_video_negotiation(peer: &str, who: &str, sdp: &str) {
    let mut in_video = false;
    let mut pt = String::new();
    let mut dir = "sendrecv";
    let mut rtpmap = String::new();
    let mut fmtp = String::new();
    let flush = |dir: &str, pt: &str, rtpmap: &str, fmtp: &str| {
        if !pt.is_empty() {
            log!("{}: [codec negociado] {who}: seção de vídeo ({dir}) primeiro codec pt={pt} {rtpmap} fmtp='{fmtp}'", short(peer));
        }
    };
    for line in sdp.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("m=") {
            if in_video {
                flush(dir, &pt, &rtpmap, &fmtp);
            }
            in_video = rest.starts_with("video");
            pt = if in_video { rest.split_whitespace().nth(3).unwrap_or("").to_string() } else { String::new() };
            dir = "sendrecv";
            rtpmap.clear();
            fmtp.clear();
        } else if in_video {
            for d in ["sendonly", "recvonly", "sendrecv", "inactive"] {
                if line == format!("a={d}") {
                    dir = d;
                }
            }
            if let Some(r) = line.strip_prefix(&format!("a=rtpmap:{pt} ")) {
                rtpmap = r.to_string();
            }
            if let Some(r) = line.strip_prefix(&format!("a=fmtp:{pt} ")) {
                fmtp = r.to_string();
            }
        }
    }
    if in_video {
        flush(dir, &pt, &rtpmap, &fmtp);
    }
}
