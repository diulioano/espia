//! Codificador de H.264 da placa de vídeo pela Media Foundation (T010; contracts/codificador.md, R2 a R4).
//!
//! Como ele é usado (T011 chama de dentro do laço de envio do tokio; T012 cria um SEGUNDO codificador, a reduzida):
//! - Nada de estado global. Cada `HwEncoder::start` abre o seu próprio dispositivo Direct3D 11, o seu gerenciador
//!   de dispositivo, o seu anel de texturas e o seu codificador. Dois codificadores ao mesmo tempo não se enxergam.
//! - Todos os objetos COM vivem numa thread própria ("codificador-h264"). Quem chama só fala por canais:
//!   `submit` (entregar), `set_bitrate` (mudar alvo), `request_keyframe` (pedir quadro inteiro) e o `Receiver`
//!   devolvido por `start` (receber). Nenhuma delas bloqueia o laço do tokio por mais que um instante:
//!   `submit` e `request_keyframe` só enfileiram; `set_bitrate` espera a resposta do codificador (milissegundos).
//! - O codificador por hardware é assíncrono. Uma segunda thread só fica em `GetEvent` e repassa
//!   `METransformNeedInput` e `METransformHaveOutput` para a mesma fila de comandos da thread do codificador.
//!   Assim a thread do codificador bloqueia em UM lugar (o canal) e acorda na hora, sem sono de 15 ms do Windows.
//! - Só a falha de início é fatal (`StartError`). Erro de quadro, propriedade recusada e evento estranho viram
//!   linha de log e contador em `Stats`; a sonda segue.

use std::collections::{HashMap, VecDeque};
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, RecvTimeoutError, Sender, SyncSender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use windows::core::{Interface, GUID};
use windows::Win32::Foundation::LUID;
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, CoUninitialize, COINIT_MULTITHREADED};
use windows::Win32::System::Variant::VARIANT;

use crate::texture::{self, TextureRing};

macro_rules! hwlog {
    ($($a:tt)*) => {{ println!("[{:>8.1}s] [codificador] {}", crate::elapsed(), format!($($a)*)); }};
}

/// Quadro cru que a sonda entrega: NV12 compacto (Y w×h, UV intercalado w×h/2), gerado no processador.
/// No app a entrada será a textura da captura; a sonda sobe esta memória para uma textura NV12 na placa
/// (`texture::TextureRing`) e entrega a textura ao codificador.
struct RawFrame {
    nv12: Vec<u8>,
    capture_us: i64,
    delivered_at: Instant,
}

enum Msg {
    Frame(RawFrame),
    SetBitrate(u32, SyncSender<Result<(), String>>),
    KeyFrame,
    Stop,
    /// Evento da MFT (tipo, status) vindo da thread de eventos.
    Event(i32, i32),
    /// A thread de eventos acabou (a MFT foi desligada ou deu erro).
    PumpEnded(i32),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
}

/// Um quadro comprimido devolvido pelo codificador.
#[derive(Debug)]
pub struct EncodedFrame {
    /// H.264 com códigos de início; os quadros inteiros trazem SPS e PPS na frente.
    pub data: Vec<u8>,
    /// Quadro inteiro (IDR) ou parcial.
    pub key: bool,
    /// Instante de captura que quem chamou deu ao entregar, em microssegundos.
    #[allow(dead_code)] // lido pelo encaminhador do T011; o --encode-only não o usa
    pub capture_us: i64,
    /// Atraso entre entregar o quadro e recebê-lo comprimido, em microssegundos.
    pub delay_us: u64,
}

#[derive(Debug)]
pub enum StartError {
    /// `MFTEnumEx` não achou codificador por hardware de H.264 com entrada NV12.
    NotFound,
    /// Achou, mas nenhum iniciou (a mensagem diz por quê, um por candidato).
    Failed(String),
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartError::NotFound => write!(f, "não achou codificador de H.264 por hardware"),
            StartError::Failed(m) => write!(f, "o codificador por hardware não iniciou: {m}"),
        }
    }
}

/// Contadores (somente leitura para quem chama).
#[derive(Default)]
pub struct Stats {
    /// Quadros entregues por `submit`.
    pub delivered: AtomicU64,
    /// Quadros que o codificador aceitou (`ProcessInput` ok).
    pub accepted: AtomicU64,
    /// Quadros comprimidos recebidos do codificador.
    pub received: AtomicU64,
    pub keyframes: AtomicU64,
    pub key_bytes: AtomicU64,
    pub bytes_out: AtomicU64,
    /// Quadros jogados fora na fila porque o codificador não deu conta a tempo (sinal de "limitado").
    pub dropped: AtomicU64,
    pub errors: AtomicU64,
    pub consecutive_errors: AtomicU64,
    pub delay_sum_us: AtomicU64,
    pub delay_max_us: AtomicU64,
    /// Pedidos de quadro inteiro aplicados com `ForceKeyFrame` aceito e recusado.
    pub force_key_ok: AtomicU64,
    pub force_key_refused: AtomicU64,
    /// Mudanças de taxa aceitas e recusadas.
    pub rate_ok: AtomicU64,
    pub rate_refused: AtomicU64,
}

pub struct HwEncoder {
    name: String,
    tx: Sender<Msg>,
    stats: Arc<Stats>,
    join: Option<JoinHandle<()>>,
}

impl HwEncoder {
    /// Acha o codificador por hardware, abre o dispositivo e o configura. Devolve o codificador e o canal por onde
    /// saem os quadros comprimidos (`receber`). Bloqueia até o início terminar (ou falhar).
    pub fn start(cfg: Config) -> Result<(HwEncoder, Receiver<EncodedFrame>), StartError> {
        let (tx, rx) = channel::<Msg>();
        let (out_tx, out_rx) = channel::<EncodedFrame>();
        let (ready_tx, ready_rx) = sync_channel::<Result<String, StartError>>(1);
        let stats = Arc::new(Stats::default());
        let join = {
            let (stats, tx) = (stats.clone(), tx.clone());
            std::thread::Builder::new()
                .name("codificador-h264".into())
                .spawn(move || worker(cfg, stats, rx, tx, out_tx, ready_tx))
                .map_err(|e| StartError::Failed(format!("thread: {e}")))?
        };
        match ready_rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(name)) => Ok((HwEncoder { name, tx, stats, join: Some(join) }, out_rx)),
            Ok(Err(e)) => {
                let _ = join.join();
                Err(e)
            }
            Err(_) => Err(StartError::Failed("o início não terminou em 30 s".into())),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn stats(&self) -> &Arc<Stats> {
        &self.stats
    }

    /// Entregar: enfileira um quadro NV12 compacto. Não espera a compressão.
    pub fn submit(&self, nv12: Vec<u8>, capture_us: i64) -> Result<(), String> {
        self.stats.delivered.fetch_add(1, Ordering::Relaxed);
        self.tx
            .send(Msg::Frame(RawFrame { nv12, capture_us, delivered_at: Instant::now() }))
            .map_err(|_| "o codificador já parou".to_string())
    }

    /// Mudar alvo: troca a taxa média ao vivo. `Err` = o codificador não aceitou (quem chama decide reabrir).
    pub fn set_bitrate(&self, bps: u32) -> Result<(), String> {
        let (rtx, rrx) = sync_channel(1);
        self.tx.send(Msg::SetBitrate(bps, rtx)).map_err(|_| "o codificador já parou".to_string())?;
        rrx.recv_timeout(Duration::from_secs(5)).map_err(|_| "o codificador não respondeu em 5 s".to_string())?
    }

    /// Pedir quadro inteiro: o próximo quadro entregue ao codificador sai inteiro. O resultado do pedido
    /// (aceito ou recusado pelo `ForceKeyFrame`) entra em `Stats::force_key_ok` e `force_key_refused`.
    pub fn request_keyframe(&self) -> Result<(), String> {
        self.tx.send(Msg::KeyFrame).map_err(|_| "o codificador já parou".to_string())
    }

    /// Parar: desliga o codificador e espera a thread acabar. Pode ser chamada mais de uma vez.
    pub fn stop(&mut self) {
        let _ = self.tx.send(Msg::Stop);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for HwEncoder {
    fn drop(&mut self) {
        self.stop();
    }
}

// ---------------------------------------------------------------- utilidades

fn hr(e: &windows::core::Error) -> String {
    format!("0x{:08X} ({})", e.code().0 as u32, e.message().trim())
}

/// Largura e altura (ou numerador e denominador) no formato de 64 bits da Media Foundation. A função
/// `MFSetAttributeSize` do cabeçalho C é inline e não existe na crate `windows`.
fn pack(hi: u32, lo: u32) -> u64 {
    ((hi as u64) << 32) | lo as u64
}

/// Tipos de NAL de um fluxo com códigos de início (00 00 01 ou 00 00 00 01) e se começa com código de início.
pub fn nal_types(data: &[u8]) -> (bool, Vec<u8>) {
    let starts_with_code = data.starts_with(&[0, 0, 0, 1]) || data.starts_with(&[0, 0, 1]);
    let mut types = Vec::new();
    let mut i = 0;
    while i + 3 < data.len() {
        if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
            types.push(data[i + 3] & 0x1F);
            i += 3;
        } else {
            i += 1;
        }
    }
    (starts_with_code, types)
}

/// Tira o delimitador de acesso (NAL 9) da frente do quadro, se houver (`--sem-aud`). Devolve o quadro sem ele.
pub fn strip_aud(data: &[u8]) -> &[u8] {
    let (n, off) = if data.starts_with(&[0, 0, 0, 1]) { (4, 4) } else if data.starts_with(&[0, 0, 1]) { (3, 3) } else { return data };
    let _ = n;
    if data.len() > off + 1 && data[off] & 0x1F == 9 {
        // Procura o próximo código de início depois do NAL 9.
        let mut i = off + 1;
        while i + 2 < data.len() {
            if data[i] == 0 && data[i + 1] == 0 && (data[i + 2] == 1 || (data[i + 2] == 0 && i + 3 < data.len() && data[i + 3] == 1)) {
                return &data[i..];
            }
            i += 1;
        }
    }
    data
}

/// (profile_idc, byte das constraint_set flags, level_idc) do primeiro SPS.
fn sps_info(data: &[u8]) -> Option<(u8, u8, u8)> {
    let mut i = 0;
    while i + 6 < data.len() {
        if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 && data[i + 3] & 0x1F == 7 {
            return Some((data[i + 4], data[i + 5], data[i + 6]));
        }
        i += 1;
    }
    None
}

fn friendly_name(a: &IMFActivate) -> String {
    unsafe {
        let Ok(len) = a.GetStringLength(&MFT_FRIENDLY_NAME_Attribute) else { return "(sem nome)".into() };
        let mut buf = vec![0u16; len as usize + 1];
        match a.GetString(&MFT_FRIENDLY_NAME_Attribute, &mut buf, None) {
            Ok(()) => String::from_utf16_lossy(&buf[..len as usize]),
            Err(_) => "(sem nome)".into(),
        }
    }
}

fn set_prop(codec: &ICodecAPI, name: &str, guid: &GUID, v: VARIANT) -> bool {
    match unsafe { codec.SetValue(guid, &v) } {
        Ok(()) => {
            hwlog!("propriedade {name}: aceita");
            true
        }
        Err(e) => {
            hwlog!("propriedade {name}: RECUSADA {}", hr(&e));
            false
        }
    }
}

/// Objeto COM que atravessa threads por decisão consciente (a fila de eventos da MFT é livre de thread).
struct SendCom<T>(T);
unsafe impl<T> Send for SendCom<T> {}
impl<T> SendCom<T> {
    fn get(&self) -> &T {
        &self.0
    }
}

// ---------------------------------------------------------------- sessão

fn get_u32(codec: &ICodecAPI, guid: &GUID) -> Option<u32> {
    unsafe {
        let v = codec.GetValue(guid).ok()?;
        let inner = &*v.Anonymous.Anonymous;
        Some(inner.Anonymous.ulVal)
    }
}

/// Propriedades do contrato (R4). Cada recusa vira uma linha; nada derruba a sonda.
fn apply_props(codec: &Option<ICodecAPI>, cfg: &Config) {
    let Some(c) = codec else {
        hwlog!("a MFT não oferece ICodecAPI: nenhuma propriedade pôde ser ajustada");
        return;
    };
    set_prop(c, "CODECAPI_AVLowLatencyMode (baixa latência)", &CODECAPI_AVLowLatencyMode, VARIANT::from(true));
    set_prop(c, "CODECAPI_AVEncCommonRateControlMode (CBR)", &CODECAPI_AVEncCommonRateControlMode, VARIANT::from(eAVEncCommonRateControlMode_CBR.0 as u32));
    set_prop(c, "CODECAPI_AVEncCommonMeanBitRate (alvo inicial)", &CODECAPI_AVEncCommonMeanBitRate, VARIANT::from(cfg.bitrate));
    hwlog!("  lido de volta: modo de taxa {:?} (0 = CBR), alvo {:?} bps", get_u32(c, &CODECAPI_AVEncCommonRateControlMode), get_u32(c, &CODECAPI_AVEncCommonMeanBitRate));
    set_prop(c, "CODECAPI_AVEncMPVDefaultBPictureCount (0 = sem quadros B)", &CODECAPI_AVEncMPVDefaultBPictureCount, VARIANT::from(0u32));
    // Quadro inteiro só a pedido: GOP muito longo. Tenta do maior para o menor (ENCODE_PROBE_GOP força um valor).
    let forced: Option<u32> = std::env::var("ENCODE_PROBE_GOP").ok().and_then(|v| v.parse().ok());
    let list: Vec<u32> = match forced {
        Some(g) => vec![g],
        None => vec![u32::MAX, i32::MAX as u32, 1_000_000, 100_000],
    };
    let mut gop_ok = false;
    for gop in list {
        if set_prop(c, &format!("CODECAPI_AVEncMPVGOPSize ({gop})"), &CODECAPI_AVEncMPVGOPSize, VARIANT::from(gop)) {
            gop_ok = true;
            hwlog!("  GOPSize lido de volta: {:?}", get_u32(c, &CODECAPI_AVEncMPVGOPSize));
            break;
        }
    }
    if !gop_ok {
        hwlog!("nenhum tamanho de GOP longo foi aceito: o codificador pode mandar quadros inteiros sozinho");
    }
}

struct Session {
    name: String,
    mft: IMFTransform,
    activate: IMFActivate,
    codec: Option<ICodecAPI>,
    /// O gerenciador precisa viver enquanto a MFT vive.
    _mgr: IMFDXGIDeviceManager,
    _device: ID3D11Device,
    ring: TextureRing,
    provides_samples: bool,
    out_size: u32,
    fps: u32,
    w: u32,
    h: u32,
}

fn list_candidates() -> Result<Vec<IMFActivate>, String> {
    if std::env::var_os("ENCODE_PROBE_SEM_PLACA").is_some() {
        hwlog!("ENCODE_PROBE_SEM_PLACA definida: fingindo que MFTEnumEx não achou nada");
        return Ok(Vec::new());
    }
    let in_t = MFT_REGISTER_TYPE_INFO { guidMajorType: MFMediaType_Video, guidSubtype: MFVideoFormat_NV12 };
    let out_t = MFT_REGISTER_TYPE_INFO { guidMajorType: MFMediaType_Video, guidSubtype: MFVideoFormat_H264 };
    let mut pp: *mut Option<IMFActivate> = std::ptr::null_mut();
    let mut n = 0u32;
    unsafe {
        MFTEnumEx(MFT_CATEGORY_VIDEO_ENCODER, MFT_ENUM_FLAG_HARDWARE | MFT_ENUM_FLAG_SORTANDFILTER, Some(&in_t), Some(&out_t), &mut pp, &mut n)
            .map_err(|e| format!("MFTEnumEx: {}", hr(&e)))?;
        let mut v = Vec::new();
        if !pp.is_null() {
            for i in 0..n as usize {
                // `take` entrega a posse (o Release acontece quando o IMFActivate sai de escopo).
                if let Some(a) = (*pp.add(i)).take() {
                    v.push(a);
                }
            }
            CoTaskMemFree(Some(pp as *const _));
        }
        Ok(v)
    }
}

fn open_candidate(activate: &IMFActivate, cfg: &Config) -> Result<Session, String> {
    let name = friendly_name(activate);
    unsafe {
        // Dispositivo na mesma placa que o codificador (computador com mais de uma placa).
        let luid = activate.GetUINT64(&MFT_ENUM_ADAPTER_LUID).ok().map(|v| LUID { LowPart: v as u32, HighPart: (v >> 32) as i32 });
        match &luid {
            Some(l) => hwlog!("MFT_ENUM_ADAPTER_LUID do codificador: {:08X}:{:08X}", l.HighPart as u32, l.LowPart),
            None => hwlog!("MFT_ENUM_ADAPTER_LUID: o codificador não informa (usando a placa padrão)"),
        }
        let device = match texture::create_device(luid) {
            Ok(d) => d,
            Err(e) if luid.is_some() => {
                hwlog!("dispositivo na placa do codificador falhou ({e}); tentando a placa padrão");
                texture::create_device(None)?
            }
            Err(e) => return Err(e),
        };
        hwlog!("dispositivo Direct3D 11 na placa: {}", texture::adapter_name(&device));
        let mut token = 0u32;
        let mut mgr: Option<IMFDXGIDeviceManager> = None;
        MFCreateDXGIDeviceManager(&mut token, &mut mgr).map_err(|e| format!("MFCreateDXGIDeviceManager: {}", hr(&e)))?;
        let mgr = mgr.ok_or("MFCreateDXGIDeviceManager não devolveu gerenciador")?;
        mgr.ResetDevice(&device, token).map_err(|e| format!("IMFDXGIDeviceManager::ResetDevice: {}", hr(&e)))?;

        let mft: IMFTransform = activate.ActivateObject().map_err(|e| format!("ActivateObject: {}", hr(&e)))?;
        let fail = |msg: String| -> String {
            let _ = activate.ShutdownObject();
            msg
        };
        let attrs = mft.GetAttributes().map_err(|e| fail(format!("GetAttributes: {}", hr(&e))))?;
        let is_async = attrs.GetUINT32(&MF_TRANSFORM_ASYNC).unwrap_or(0) == 1;
        let d3d_aware = attrs.GetUINT32(&MF_SA_D3D11_AWARE).unwrap_or(0) == 1;
        hwlog!("MFT assíncrona: {} | MF_SA_D3D11_AWARE: {}", if is_async { "sim" } else { "não" }, if d3d_aware { "sim" } else { "não" });
        if !is_async {
            return Err(fail("a MFT não é assíncrona (a sonda só segue o modelo de eventos)".into()));
        }
        attrs.SetUINT32(&MF_TRANSFORM_ASYNC_UNLOCK, 1).map_err(|e| fail(format!("MF_TRANSFORM_ASYNC_UNLOCK: {}", hr(&e))))?;
        mft.ProcessMessage(MFT_MESSAGE_SET_D3D_MANAGER, mgr.as_raw() as usize)
            .map_err(|e| fail(format!("MFT_MESSAGE_SET_D3D_MANAGER: {}", hr(&e))))?;

        let codec: Option<ICodecAPI> = mft.cast().ok();
        // ACHADO: com as propriedades ajustadas DEPOIS dos tipos de mídia, o codificador da NVIDIA aceita o GOPSize
        // mas o troca por 60 (1 s), e manda quadro inteiro sozinho a cada segundo. ANTES, o valor vale.
        // ENCODE_PROBE_PROPS_DEPOIS volta à ordem antiga, só para reproduzir o achado.
        let props_first = std::env::var_os("ENCODE_PROBE_PROPS_DEPOIS").is_none();
        if props_first {
            hwlog!("propriedades ajustadas ANTES dos tipos de mídia");
            apply_props(&codec, cfg);
        }
        // Tipo de saída primeiro (os codificadores exigem), depois o de entrada.
        let make_out = |profile: i32| -> windows::core::Result<IMFMediaType> {
            let t = MFCreateMediaType()?;
            t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
            t.SetUINT32(&MF_MT_AVG_BITRATE, cfg.bitrate)?;
            t.SetUINT64(&MF_MT_FRAME_SIZE, pack(cfg.width, cfg.height))?;
            t.SetUINT64(&MF_MT_FRAME_RATE, pack(cfg.fps, 1))?;
            t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pack(1, 1))?;
            t.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
            t.SetUINT32(&MF_MT_MPEG2_PROFILE, profile as u32)?;
            Ok(t)
        };
        let ot = make_out(eAVEncH264VProfile_ConstrainedBase.0).map_err(|e| fail(format!("tipo de saída: {}", hr(&e))))?;
        match mft.SetOutputType(0, &ot, 0) {
            Ok(()) => hwlog!("tipo de saída: H.264 {}x{} {} qps, perfil Constrained Baseline (eAVEncH264VProfile_ConstrainedBase) aceito", cfg.width, cfg.height, cfg.fps),
            Err(e) => {
                hwlog!("SetOutputType com ConstrainedBase RECUSADO {}; tentando Baseline (66)", hr(&e));
                let ot2 = make_out(eAVEncH264VProfile_Base.0).map_err(|e| fail(format!("tipo de saída: {}", hr(&e))))?;
                mft.SetOutputType(0, &ot2, 0).map_err(|e| fail(format!("SetOutputType: {}", hr(&e))))?;
                hwlog!("tipo de saída: H.264 com perfil Baseline (66) aceito (ConstrainedBase não)");
            }
        }
        let it = MFCreateMediaType().map_err(|e| fail(format!("tipo de entrada: {}", hr(&e))))?;
        let set_in = || -> windows::core::Result<()> {
            it.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            it.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12)?;
            it.SetUINT64(&MF_MT_FRAME_SIZE, pack(cfg.width, cfg.height))?;
            it.SetUINT64(&MF_MT_FRAME_RATE, pack(cfg.fps, 1))?;
            it.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pack(1, 1))?;
            it.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
            Ok(())
        };
        set_in().map_err(|e| fail(format!("tipo de entrada: {}", hr(&e))))?;
        mft.SetInputType(0, &it, 0).map_err(|e| fail(format!("SetInputType NV12: {}", hr(&e))))?;

        // Propriedades do codificador (os itens de configuração do contrato). Recusa = linha de log, sem derrubar.
        if !props_first {
            apply_props(&codec, cfg);
        }

        let info = mft.GetOutputStreamInfo(0).map_err(|e| fail(format!("GetOutputStreamInfo: {}", hr(&e))))?;
        let provides_samples = info.dwFlags & MFT_OUTPUT_STREAM_PROVIDES_SAMPLES.0 as u32 != 0;
        hwlog!("o codificador fornece as amostras de saída: {} (cbSize={})", if provides_samples { "sim" } else { "não" }, info.cbSize);

        let ring = TextureRing::new(&device, cfg.width, cfg.height, 8).map_err(|e| fail(e))?;
        mft.ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0).map_err(|e| fail(format!("BEGIN_STREAMING: {}", hr(&e))))?;
        mft.ProcessMessage(MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0).map_err(|e| fail(format!("START_OF_STREAM: {}", hr(&e))))?;

        Ok(Session {
            name,
            mft,
            activate: activate.clone(),
            codec,
            _mgr: mgr,
            _device: device,
            ring,
            provides_samples,
            out_size: info.cbSize.max(1 << 20),
            fps: cfg.fps,
            w: cfg.width,
            h: cfg.height,
        })
    }
}

fn open(cfg: &Config) -> Result<Session, StartError> {
    let candidates = list_candidates().map_err(StartError::Failed)?;
    if candidates.is_empty() {
        return Err(StartError::NotFound);
    }
    hwlog!("codificadores de H.264 por hardware achados (MFTEnumEx, NV12 -> H.264): {}", candidates.len());
    for (i, a) in candidates.iter().enumerate() {
        hwlog!("  achado {}: {}", i + 1, friendly_name(a));
    }
    let mut errors = Vec::new();
    for a in &candidates {
        let name = friendly_name(a);
        match open_candidate(a, cfg) {
            Ok(s) => {
                hwlog!("INICIOU: {name}");
                return Ok(s);
            }
            Err(e) => {
                hwlog!("não iniciou: {name}: {e}");
                errors.push(format!("{name}: {e}"));
            }
        }
    }
    Err(StartError::Failed(errors.join(" | ")))
}

// ---------------------------------------------------------------- thread do codificador

/// Acompanha o que a sonda precisa saber do formato dos quadros inteiros (R4).
#[derive(Default)]
struct KeyFormat {
    checked: bool,
    /// SPS e PPS para acrescentar antes de cada quadro inteiro que vier sem eles (plano B da R4).
    prefix: Option<Vec<u8>>,
}

fn worker(cfg: Config, stats: Arc<Stats>, rx: Receiver<Msg>, tx: Sender<Msg>, out: Sender<EncodedFrame>, ready: SyncSender<Result<String, StartError>>) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        if let Err(e) = MFStartup(MF_VERSION, MFSTARTUP_FULL) {
            let _ = ready.send(Err(StartError::Failed(format!("MFStartup: {}", hr(&e)))));
            CoUninitialize();
            return;
        }
    }
    let session = match open(&cfg) {
        Ok(s) => s,
        Err(e) => {
            let _ = ready.send(Err(e));
            unsafe {
                let _ = MFShutdown();
                CoUninitialize();
            }
            return;
        }
    };
    let _ = ready.send(Ok(session.name.clone()));
    run(session, &cfg, &stats, &rx, &tx, &out);
    unsafe {
        let _ = MFShutdown();
        CoUninitialize();
    }
}

fn run(mut s: Session, cfg: &Config, stats: &Arc<Stats>, rx: &Receiver<Msg>, tx: &Sender<Msg>, out: &Sender<EncodedFrame>) {
    // Thread de eventos: só repassa.
    if let Ok(gen) = s.mft.cast::<IMFMediaEventGenerator>() {
        let gen = SendCom(gen);
        let tx = tx.clone();
        let _ = std::thread::Builder::new().name("codificador-h264-eventos".into()).spawn(move || {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }
            loop {
                let ev = unsafe { gen.get().GetEvent(MEDIA_EVENT_GENERATOR_GET_EVENT_FLAGS(0)) };
                match ev {
                    Ok(ev) => {
                        let t = unsafe { ev.GetType() }.unwrap_or(0) as i32;
                        let st = unsafe { ev.GetStatus() }.map(|h| h.0).unwrap_or(0);
                        if tx.send(Msg::Event(t, st)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Msg::PumpEnded(e.code().0));
                        break;
                    }
                }
            }
            unsafe { CoUninitialize() };
        });
    } else {
        hwlog!("a MFT não expõe IMFMediaEventGenerator: não há como seguir os eventos");
    }

    let mut credits = 0u32; // METransformNeedInput ainda não usados
    let mut queue: VecDeque<RawFrame> = VecDeque::new();
    let mut want_key = false;
    let mut sent_at: HashMap<i64, Instant> = HashMap::new();
    let mut key_fmt = KeyFormat::default();
    let mut err_logged = 0u32;
    let mut bump_error = |stats: &Stats, what: &str, e: String| {
        stats.errors.fetch_add(1, Ordering::Relaxed);
        stats.consecutive_errors.fetch_add(1, Ordering::Relaxed);
        if err_logged < 10 {
            err_logged += 1;
            hwlog!("erro ({what}): {e}");
        }
    };

    'outer: loop {
        let first = match rx.recv() {
            Ok(m) => m,
            Err(_) => break,
        };
        let mut batch = vec![first];
        while let Ok(m) = rx.try_recv() {
            batch.push(m);
        }
        for m in batch {
            match m {
                Msg::Stop => {
                    break 'outer;
                }
                Msg::Frame(f) => {
                    queue.push_back(f);
                    // O codificador não dá conta: o quadro mais velho vai embora (sinal de "limitado").
                    while queue.len() > 3 {
                        queue.pop_front();
                        let n = stats.dropped.fetch_add(1, Ordering::Relaxed) + 1;
                        if n <= 3 || n % 100 == 0 {
                            hwlog!("fila cheia: quadro descartado (total {n})");
                        }
                    }
                }
                Msg::KeyFrame => want_key = true,
                Msg::SetBitrate(bps, reply) => {
                    let r = match &s.codec {
                        None => Err("sem ICodecAPI".to_string()),
                        Some(c) => unsafe { c.SetValue(&CODECAPI_AVEncCommonMeanBitRate, &VARIANT::from(bps)) }.map_err(|e| hr(&e)),
                    };
                    match &r {
                        Ok(()) => {
                            stats.rate_ok.fetch_add(1, Ordering::Relaxed);
                            hwlog!("alvo de taxa mudado ao vivo para {bps} bps: aceito");
                        }
                        Err(e) => {
                            stats.rate_refused.fetch_add(1, Ordering::Relaxed);
                            hwlog!("alvo de taxa {bps} bps: RECUSADO ao vivo ({e})");
                        }
                    }
                    let _ = reply.send(r);
                }
                Msg::Event(t, st) => {
                    if t == METransformNeedInput.0 {
                        credits += 1;
                    } else if t == METransformHaveOutput.0 {
                        match take_output(&s, &stats, &mut sent_at, &mut key_fmt, out) {
                            Ok(()) => {}
                            Err(e) => bump_error(stats, "ProcessOutput", e),
                        }
                    } else if t == MEError.0 {
                        bump_error(stats, "evento MEError", format!("status 0x{:08X}", st as u32));
                    } else if t == METransformDrainComplete.0 || t == METransformMarker.0 {
                        // Nada a fazer: a sonda não usa dreno nem marcador.
                    } else {
                        hwlog!("evento inesperado da MFT: tipo {t} status 0x{:08X}", st as u32);
                    }
                }
                Msg::PumpEnded(code) => {
                        hwlog!("a fila de eventos da MFT acabou sozinha (0x{:08X}); o codificador não entrega mais nada", code as u32);
                }
            }
        }
        // Alimenta o codificador com o que há de crédito.
        while credits > 0 {
            let Some(f) = queue.pop_front() else { break };
            credits -= 1;
            sent_at.insert(f.capture_us.saturating_mul(10), f.delivered_at);
            if sent_at.len() > 512 {
                sent_at.clear();
            }
            match feed(&mut s, &f, want_key, stats) {
                Ok(()) => {
                    want_key = false;
                    stats.accepted.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => bump_error(stats, "ProcessInput", e),
            }
        }
    }

    // Desligar.
    unsafe {
        let _ = s.mft.ProcessMessage(MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0);
        let _ = s.mft.ProcessMessage(MFT_MESSAGE_COMMAND_FLUSH, 0);
        let _ = s.mft.ProcessMessage(MFT_MESSAGE_NOTIFY_END_STREAMING, 0);
        let _ = s.activate.ShutdownObject();
    }
    // Espera a thread de eventos acabar (GetEvent devolve erro depois do ShutdownObject).
    let t0 = Instant::now();
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Msg::PumpEnded(_)) => break,
            Ok(_) => {}
            Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => {
                if t0.elapsed() > Duration::from_secs(1) {
                    break;
                }
            }
        }
    }
    let _ = (cfg, s.w, s.h);
}

/// Entrega um quadro: sobe para a textura, vira amostra e vai para a MFT.
fn feed(s: &mut Session, f: &RawFrame, want_key: bool, stats: &Stats) -> Result<(), String> {
    let tex = s.ring.upload(&f.nv12)?;
    unsafe {
        let buffer = MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, &tex, 0, false).map_err(|e| format!("MFCreateDXGISurfaceBuffer: {}", hr(&e)))?;
        // Alguns codificadores só aceitam a amostra com o tamanho do quadro declarado.
        let _ = buffer.SetCurrentLength(s.w * s.h * 3 / 2);
        let sample = MFCreateSample().map_err(|e| format!("MFCreateSample: {}", hr(&e)))?;
        sample.AddBuffer(&buffer).map_err(|e| format!("AddBuffer: {}", hr(&e)))?;
        sample.SetSampleTime(f.capture_us.saturating_mul(10)).map_err(|e| format!("SetSampleTime: {}", hr(&e)))?;
        let _ = sample.SetSampleDuration(10_000_000 / s.fps.max(1) as i64);
        if want_key {
            match &s.codec {
                Some(c) => match c.SetValue(&CODECAPI_AVEncVideoForceKeyFrame, &VARIANT::from(1u32)) {
                    Ok(()) => {
                        stats.force_key_ok.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        let n = stats.force_key_refused.fetch_add(1, Ordering::Relaxed) + 1;
                        if n <= 3 {
                            hwlog!("propriedade CODECAPI_AVEncVideoForceKeyFrame: RECUSADA {}", hr(&e));
                        }
                    }
                },
                None => {
                    stats.force_key_refused.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        s.mft.ProcessInput(0, &sample, 0).map_err(|e| format!("ProcessInput: {}", hr(&e)))
    }
}

/// Um `METransformHaveOutput`: tira um quadro comprimido da MFT e manda para `out`.
fn take_output(s: &Session, stats: &Stats, sent_at: &mut HashMap<i64, Instant>, key_fmt: &mut KeyFormat, out: &Sender<EncodedFrame>) -> Result<(), String> {
    unsafe {
        // Se o codificador não fornece a amostra, quem chama precisa dar uma.
        let given = if s.provides_samples {
            None
        } else {
            let sample = MFCreateSample().map_err(|e| format!("MFCreateSample: {}", hr(&e)))?;
            let buf = MFCreateMemoryBuffer(s.out_size).map_err(|e| format!("MFCreateMemoryBuffer: {}", hr(&e)))?;
            sample.AddBuffer(&buf).map_err(|e| format!("AddBuffer: {}", hr(&e)))?;
            Some(sample)
        };
        let mut bufs = [MFT_OUTPUT_DATA_BUFFER { dwStreamID: 0, pSample: ManuallyDrop::new(given), dwStatus: 0, pEvents: ManuallyDrop::new(None) }];
        let mut status = 0u32;
        let r = s.mft.ProcessOutput(0, &mut bufs, &mut status);
        let sample = ManuallyDrop::take(&mut bufs[0].pSample);
        let _events = ManuallyDrop::take(&mut bufs[0].pEvents);
        match r {
            Ok(()) => {}
            Err(e) if e.code() == MF_E_TRANSFORM_NEED_MORE_INPUT => return Ok(()),
            Err(e) if e.code() == MF_E_TRANSFORM_STREAM_CHANGE => return Err("MF_E_TRANSFORM_STREAM_CHANGE (a sonda não renegocia o tipo de saída)".into()),
            Err(e) => return Err(hr(&e)),
        }
        let Some(sample) = sample else { return Err("ProcessOutput ok, mas sem amostra".into()) };
        let time = sample.GetSampleTime().unwrap_or(0);
        let clean = sample.GetUINT32(&MFSampleExtension_CleanPoint).unwrap_or(0) == 1;
        let buffer = sample.ConvertToContiguousBuffer().map_err(|e| format!("ConvertToContiguousBuffer: {}", hr(&e)))?;
        let mut p: *mut u8 = std::ptr::null_mut();
        let mut len = 0u32;
        buffer.Lock(&mut p, None, Some(&mut len)).map_err(|e| format!("Lock: {}", hr(&e)))?;
        let mut data = std::slice::from_raw_parts(p, len as usize).to_vec();
        let _ = buffer.Unlock();

        let (annexb, types) = nal_types(&data);
        let key = types.contains(&5) || clean;
        if key {
            let has_sps = types.contains(&7);
            let has_pps = types.contains(&8);
            if !key_fmt.checked {
                key_fmt.checked = true;
                let prof = sps_info(&data).map(|(p, c, l)| format!("profile_idc={p} constraint_flags=0x{c:02X} level_idc={l}")).unwrap_or_else(|| "sem SPS para ler".into());
                hwlog!(
                    "primeiro quadro inteiro: {} bytes | códigos de início (Annex B): {} | SPS junto: {} | PPS junto: {} | NALs: {:?} | CleanPoint: {} | {}",
                    data.len(),
                    if annexb { "sim" } else { "NÃO" },
                    if has_sps { "sim" } else { "NÃO" },
                    if has_pps { "sim" } else { "NÃO" },
                    types,
                    if clean { "sim" } else { "não" },
                    prof
                );
                if !has_sps || !has_pps {
                    // Plano B da R4: pegar o SPS/PPS do cabeçalho de sequência do tipo de saída.
                    key_fmt.prefix = s.mft.GetOutputCurrentType(0).ok().and_then(|t| {
                        let n = t.GetBlobSize(&MF_MT_MPEG_SEQUENCE_HEADER).ok()? as usize;
                        let mut b = vec![0u8; n];
                        t.GetBlob(&MF_MT_MPEG_SEQUENCE_HEADER, &mut b, None).ok()?;
                        Some(b)
                    });
                    match &key_fmt.prefix {
                        Some(b) => {
                            let (a, ty) = nal_types(b);
                            hwlog!("o quadro inteiro veio SEM SPS/PPS: acrescentando MF_MT_MPEG_SEQUENCE_HEADER ({} bytes, Annex B: {}, NALs {:?}) antes de cada quadro inteiro", b.len(), if a { "sim" } else { "não" }, ty);
                        }
                        None => hwlog!("o quadro inteiro veio SEM SPS/PPS e o tipo de saída não tem MF_MT_MPEG_SEQUENCE_HEADER: não há de onde tirar"),
                    }
                } else {
                    hwlog!("o quadro inteiro já traz SPS e PPS: nada a acrescentar");
                }
            }
            if !(has_sps && has_pps) {
                if let Some(prefix) = &key_fmt.prefix {
                    let mut v = Vec::with_capacity(prefix.len() + data.len());
                    v.extend_from_slice(prefix);
                    v.extend_from_slice(&data);
                    data = v;
                }
            }
        }
        let delay_us = sent_at.remove(&time).map(|t| t.elapsed().as_micros() as u64).unwrap_or(0);
        stats.received.fetch_add(1, Ordering::Relaxed);
        stats.consecutive_errors.store(0, Ordering::Relaxed);
        stats.bytes_out.fetch_add(data.len() as u64, Ordering::Relaxed);
        stats.delay_sum_us.fetch_add(delay_us, Ordering::Relaxed);
        stats.delay_max_us.fetch_max(delay_us, Ordering::Relaxed);
        if key {
            stats.keyframes.fetch_add(1, Ordering::Relaxed);
            stats.key_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
        }
        let _ = out.send(EncodedFrame { data, key, capture_us: time / 10, delay_us });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nals_com_codigo_de_inicio() {
        let d = [0, 0, 0, 1, 0x67, 66, 0xE0, 31, 0, 0, 0, 1, 0x68, 1, 2, 0, 0, 1, 0x65, 9, 9];
        let (a, t) = nal_types(&d);
        assert!(a);
        assert_eq!(t, vec![7, 8, 5]);
        assert_eq!(sps_info(&d), Some((66, 0xE0, 31)));
    }

    #[test]
    fn tira_o_delimitador_de_acesso() {
        let d = [0, 0, 0, 1, 0x09, 0xF0, 0, 0, 0, 1, 0x67, 66, 0xE0, 31, 0, 0, 0, 1, 0x65, 9];
        assert_eq!(strip_aud(&d), &d[6..]);
        let sem = [0, 0, 0, 1, 0x67, 1, 2];
        assert_eq!(strip_aud(&sem), &sem[..]);
    }

    #[test]
    fn sem_codigo_de_inicio() {
        let (a, t) = nal_types(&[0, 0, 0, 5, 0x65, 1, 2, 3, 4]);
        assert!(!a);
        assert!(t.is_empty());
    }
}
