// Vídeo da fonte escolhida (research R4/R5): Windows.Graphics.Capture → reduz e converte para NV12
// na GPU (ID3D11VideoProcessor) → copia para um dos 3 buffers compartilhados com a página.

/// Tamanho de saída: altura `res` (720/1080) mantendo a proporção, sem ampliar, par nos dois lados.
pub fn target_size(src_w: u32, src_h: u32, res: u32) -> (u32, u32) {
  let even = |v: u32| (v.max(2)) & !1;
  if src_w == 0 || src_h == 0 {
    return (even(res * 16 / 9), even(res));
  }
  let max_w = res * 16 / 9;
  let scale = (res as f64 / src_h as f64).min(max_w as f64 / src_w as f64).min(1.0);
  (even((src_w as f64 * scale).round() as u32), even((src_h as f64 * scale).round() as u32))
}

/// Retângulo onde o conteúdo (cw×ch) cabe centralizado em (w×h), mantendo a proporção.
pub fn letterbox(cw: u32, ch: u32, w: u32, h: u32) -> (i32, i32, i32, i32) {
  if cw == 0 || ch == 0 {
    return (0, 0, w as i32, h as i32);
  }
  let scale = (w as f64 / cw as f64).min(h as f64 / ch as f64);
  let (dw, dh) = ((cw as f64 * scale).round() as i32, (ch as f64 * scale).round() as i32);
  let (x, y) = ((w as i32 - dw) / 2, (h as i32 - dh) / 2);
  (x, y, x + dw, y + dh)
}

/// Bytes de um quadro NV12 compacto (Y w×h + UV w×h/2).
pub fn nv12_size(w: u32, h: u32) -> usize {
  (w * h * 3 / 2) as usize
}

#[cfg(windows)]
pub use imp::{Item, SlotSet, Video};

#[cfg(windows)]
mod imp {
  use std::{
    mem::ManuallyDrop,
    sync::{
      Arc, Mutex,
      atomic::{AtomicBool, AtomicU64, Ordering},
      mpsc,
    },
    time::{Duration, Instant},
  };

  use serde_json::json;
  use super::super::FrameSink;
  use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_SHARED_BUFFER_ACCESS_READ_ONLY, ICoreWebView2_17, ICoreWebView2Environment12, ICoreWebView2SharedBuffer,
  };
  use windows::{
    Graphics::{
      Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession},
      DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
    },
    Win32::{
      Foundation::{HMODULE, HWND, RECT},
      Graphics::{
        Direct3D::D3D_DRIVER_TYPE_HARDWARE,
        Direct3D11::{
          D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
          D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_SDK_VERSION, D3D11_TEX2D_VPIV,
          D3D11_TEX2D_VPOV, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING, D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
          D3D11_VIDEO_PROCESSOR_COLOR_SPACE, D3D11_VIDEO_PROCESSOR_CONTENT_DESC, D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC,
          D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0, D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC, D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0,
          D3D11_VIDEO_PROCESSOR_STREAM, D3D11_VIDEO_USAGE_OPTIMAL_SPEED, D3D11_VPIV_DIMENSION_TEXTURE2D, D3D11_VPOV_DIMENSION_TEXTURE2D,
          D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Multithread, ID3D11Texture2D, ID3D11VideoContext, ID3D11VideoDevice,
          ID3D11VideoProcessor, ID3D11VideoProcessorEnumerator, ID3D11VideoProcessorInputView, ID3D11VideoProcessorOutputView,
        },
        Dxgi::{
          Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_NV12, DXGI_RATIONAL, DXGI_SAMPLE_DESC},
          IDXGIDevice,
        },
        Gdi::HMONITOR,
      },
      System::WinRT::{
        Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess},
        Graphics::Capture::IGraphicsCaptureItemInterop,
      },
    },
    core::{HSTRING, Interface},
  };

  use super::{letterbox, nv12_size, target_size};

  // ---------- Buffers compartilhados com a página ----------

  struct SendPtr(*mut u8);
  unsafe impl Send for SendPtr {}
  unsafe impl Sync for SendPtr {}

  thread_local! {
    /// Os ICoreWebView2SharedBuffer vivem na thread da UI (onde foram criados), por geração.
    static BUFFERS: std::cell::RefCell<Vec<(u32, ICoreWebView2SharedBuffer)>> = const { std::cell::RefCell::new(Vec::new()) };
  }

  /// Os 3 slots de uma geração (contracts/native-capture.md "Buffers compartilhados").
  pub struct SlotSet {
    pub generation: u32,
    pub width: u32,
    pub height: u32,
    ptrs: [SendPtr; 3],
  }

  impl SlotSet {
    /// Cria e posta os 3 buffers para a página (precisa rodar a criação na thread da UI).
    pub fn post<R: tauri::Runtime>(webview: &tauri::Webview<R>, generation: u32, width: u32, height: u32) -> Result<Self, String> {
      let size = nv12_size(width, height);
      let (tx, rx) = mpsc::channel::<Result<[usize; 3], String>>();
      webview
        .with_webview(move |wv| {
          let res = (|| unsafe {
            let env: ICoreWebView2Environment12 = wv.environment().cast().map_err(|e| e.to_string())?;
            let core: ICoreWebView2_17 = wv.controller().CoreWebView2().and_then(|c| c.cast()).map_err(|e| e.to_string())?;
            let mut ptrs = [0usize; 3];
            for (slot, out) in ptrs.iter_mut().enumerate() {
              let buffer = env.CreateSharedBuffer(size as u64).map_err(|e| e.to_string())?;
              let mut p = std::ptr::null_mut();
              buffer.Buffer(&mut p).map_err(|e| e.to_string())?;
              *out = p as usize;
              let meta = json!({ "espia": "frame-slot", "gen": generation, "slot": slot, "width": width, "height": height, "format": "NV12" });
              core
                .PostSharedBufferToScript(&buffer, COREWEBVIEW2_SHARED_BUFFER_ACCESS_READ_ONLY, &HSTRING::from(meta.to_string()))
                .map_err(|e| e.to_string())?;
              BUFFERS.with(|b| b.borrow_mut().push((generation, buffer)));
            }
            Ok(ptrs)
          })();
          let _ = tx.send(res);
        })
        .map_err(|e| e.to_string())?;
      let ptrs = rx.recv_timeout(Duration::from_secs(5)).map_err(|_| "shared_buffer_timeout".to_string())??;
      Ok(Self { generation, width, height, ptrs: ptrs.map(|p| SendPtr(p as *mut u8)) })
    }

    /// Libera (na thread da UI) as gerações até `upto`, inclusive.
    pub fn release<R: tauri::Runtime>(webview: &tauri::Webview<R>, upto: u32) {
      let _ = webview.with_webview(move |_| {
        BUFFERS.with(|b| {
          b.borrow_mut().retain(|(g, buf)| {
            if *g <= upto {
              unsafe {
                let _ = buf.Close();
              }
              false
            } else {
              true
            }
          })
        });
      });
    }
  }

  // ---------- GPU: BGRA (tamanho da fonte) → NV12 (tamanho alvo) → staging ----------

  pub struct Pipeline {
    device: ID3D11Device,
    ctx: ID3D11DeviceContext,
    _vdev: ID3D11VideoDevice,
    vctx: ID3D11VideoContext,
    /// Tamanho da textura de entrada (o do frame pool).
    in_size: (u32, u32),
    out_size: (u32, u32),
    input: ID3D11Texture2D,
    in_view: ID3D11VideoProcessorInputView,
    /// Mantido vivo junto com o processador e as views criadas a partir dele.
    _enumerator: ID3D11VideoProcessorEnumerator,
    processor: ID3D11VideoProcessor,
    output: ID3D11Texture2D,
    out_view: ID3D11VideoProcessorOutputView,
    staging: ID3D11Texture2D,
  }

  unsafe impl Send for Pipeline {}

  fn texture(device: &ID3D11Device, w: u32, h: u32, format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT, staging: bool, bind: u32) -> windows::core::Result<ID3D11Texture2D> {
    let desc = D3D11_TEXTURE2D_DESC {
      Width: w,
      Height: h,
      MipLevels: 1,
      ArraySize: 1,
      Format: format,
      SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
      Usage: if staging { D3D11_USAGE_STAGING } else { D3D11_USAGE_DEFAULT },
      BindFlags: bind,
      CPUAccessFlags: if staging { D3D11_CPU_ACCESS_READ.0 as u32 } else { 0 },
      MiscFlags: 0,
    };
    let mut tex = None;
    unsafe { device.CreateTexture2D(&desc, None, Some(&mut tex))? };
    tex.ok_or_else(windows::core::Error::empty)
  }

  impl Pipeline {
    fn new(device: ID3D11Device, in_size: (u32, u32), out_size: (u32, u32)) -> windows::core::Result<Self> {
      unsafe {
        let ctx = device.GetImmediateContext()?;
        let vdev: ID3D11VideoDevice = device.cast()?;
        let vctx: ID3D11VideoContext = ctx.cast()?;
        let rate = DXGI_RATIONAL { Numerator: 60, Denominator: 1 };
        let content = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
          InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
          InputFrameRate: rate,
          InputWidth: in_size.0,
          InputHeight: in_size.1,
          OutputFrameRate: rate,
          OutputWidth: out_size.0,
          OutputHeight: out_size.1,
          Usage: D3D11_VIDEO_USAGE_OPTIMAL_SPEED,
        };
        let enumerator = vdev.CreateVideoProcessorEnumerator(&content)?;
        let processor = vdev.CreateVideoProcessor(&enumerator, 0)?;
        let input = texture(&device, in_size.0, in_size.1, DXGI_FORMAT_B8G8R8A8_UNORM, false, (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32)?;
        let output = texture(&device, out_size.0, out_size.1, DXGI_FORMAT_NV12, false, D3D11_BIND_RENDER_TARGET.0 as u32)?;
        let staging = texture(&device, out_size.0, out_size.1, DXGI_FORMAT_NV12, true, 0)?;
        let in_desc = D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
          FourCC: 0,
          ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
          Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 { Texture2D: D3D11_TEX2D_VPIV { MipSlice: 0, ArraySlice: 0 } },
        };
        let mut in_view = None;
        vdev.CreateVideoProcessorInputView(&input, &enumerator, &in_desc, Some(&mut in_view))?;
        let out_desc = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
          ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
          Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 { Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 } },
        };
        let mut out_view = None;
        vdev.CreateVideoProcessorOutputView(&output, &enumerator, &out_desc, Some(&mut out_view))?;
        // Entrada RGB 0–255; saída YCbCr BT.709 16–235 (a página cria o VideoFrame com esse colorSpace).
        vctx.VideoProcessorSetStreamColorSpace(&processor, 0, &D3D11_VIDEO_PROCESSOR_COLOR_SPACE { _bitfield: 0 });
        vctx.VideoProcessorSetOutputColorSpace(&processor, &D3D11_VIDEO_PROCESSOR_COLOR_SPACE { _bitfield: 0b01_0100 });
        vctx.VideoProcessorSetStreamFrameFormat(&processor, 0, D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE);
        vctx.VideoProcessorSetStreamAutoProcessingMode(&processor, 0, false);
        Ok(Self {
          device,
          ctx,
          _vdev: vdev,
          vctx,
          in_size,
          out_size,
          input,
          in_view: in_view.ok_or_else(windows::core::Error::empty)?,
          _enumerator: enumerator,
          processor,
          output,
          out_view: out_view.ok_or_else(windows::core::Error::empty)?,
          staging,
        })
      }
    }

    /// Converte o quadro e copia o NV12 compacto para `dst`. Devolve a média aproximada de Y.
    fn convert(&mut self, frame: &ID3D11Texture2D, content: (u32, u32), dst: *mut u8) -> windows::core::Result<u8> {
      unsafe {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        frame.GetDesc(&mut desc);
        if (desc.Width, desc.Height) != self.in_size {
          *self = Pipeline::new(self.device.clone(), (desc.Width, desc.Height), self.out_size)?;
        }
        self.ctx.CopyResource(&self.input, frame);
        let (cw, ch) = (content.0.min(self.in_size.0), content.1.min(self.in_size.1));
        let src = RECT { left: 0, top: 0, right: cw as i32, bottom: ch as i32 };
        let (l, t, r, b) = letterbox(cw, ch, self.out_size.0, self.out_size.1);
        let dest = RECT { left: l, top: t, right: r, bottom: b };
        self.vctx.VideoProcessorSetStreamSourceRect(&self.processor, 0, true, Some(&src));
        self.vctx.VideoProcessorSetStreamDestRect(&self.processor, 0, true, Some(&dest));
        let stream = D3D11_VIDEO_PROCESSOR_STREAM {
          Enable: true.into(),
          pInputSurface: ManuallyDrop::new(Some(self.in_view.clone())),
          ..Default::default()
        };
        let streams = [stream];
        let blt = self.vctx.VideoProcessorBlt(&self.processor, &self.out_view, 0, &streams);
        let [mut stream] = streams;
        ManuallyDrop::drop(&mut stream.pInputSurface);
        blt?;
        self.ctx.CopyResource(&self.staging, &self.output);
        let mut map = D3D11_MAPPED_SUBRESOURCE::default();
        self.ctx.Map(&self.staging, 0, D3D11_MAP_READ, 0, Some(&mut map))?;
        let (w, h) = (self.out_size.0 as usize, self.out_size.1 as usize);
        let pitch = map.RowPitch as usize;
        let src = map.pData as *const u8;
        let mut luma: u64 = 0;
        // Y: h linhas; UV: h/2 linhas logo depois das h linhas (com o mesmo pitch).
        for row in 0..h {
          let line = src.add(row * pitch);
          std::ptr::copy_nonoverlapping(line, dst.add(row * w), w);
          luma += *line.add(w / 2) as u64;
        }
        let uv_src = src.add(pitch * h);
        let uv_dst = dst.add(w * h);
        for row in 0..h / 2 {
          std::ptr::copy_nonoverlapping(uv_src.add(row * pitch), uv_dst.add(row * w), w);
        }
        self.ctx.Unmap(&self.staging, 0);
        Ok((luma / h.max(1) as u64) as u8)
      }
    }
  }

  // ---------- Sessão de captura ----------

  /// HWND/HMONITOR como número: passa entre threads (o handle vale em qualquer uma).
  pub enum Item {
    Window(isize),
    Monitor(isize),
  }

  /// O IDirect3DDevice do WinRT é ágil (pode ser usado de qualquer thread).
  #[derive(Clone)]
  struct AgileDevice(IDirect3DDevice);
  unsafe impl Send for AgileDevice {}
  unsafe impl Sync for AgileDevice {}
  impl AgileDevice {
    // Método (e não `.0`): a closure captura o AgileDevice inteiro, que é Send.
    fn get(&self) -> &IDirect3DDevice {
      &self.0
    }
  }

  struct Shared {
    pipeline: Mutex<Pipeline>,
    slots: Mutex<SlotSet>,
    /// Intervalo mínimo entre quadros enviados (1/fps).
    interval_us: AtomicU64,
    seq: AtomicU64,
    last: Mutex<Option<Instant>>,
    started: Instant,
    dark_frames: AtomicU64,
    warned_black: AtomicBool,
    ended: AtomicBool,
    pool_size: Mutex<(i32, i32)>,
  }

  pub struct Video {
    shared: Arc<Shared>,
    pool: Direct3D11CaptureFramePool,
    session: GraphicsCaptureSession,
    device: AgileDevice,
    frames: FrameSink,
    content: (u32, u32),
  }

  // Os objetos do WGC são ágeis; o resto já é Send.
  unsafe impl Send for Video {}

  /// Quadros com Y médio abaixo disso contam como pretos (preto "studio" é 16).
  const BLACK_Y: u8 = 20;
  const BLACK_FRAMES: u64 = 30;

  fn d3d() -> windows::core::Result<(ID3D11Device, IDirect3DDevice)> {
    unsafe {
      let mut device = None;
      D3D11CreateDevice(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),
        D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
        None,
        D3D11_SDK_VERSION,
        Some(&mut device),
        None,
        None,
      )?;
      let device = device.ok_or_else(windows::core::Error::empty)?;
      // WGC usa o mesmo dispositivo em outras threads.
      if let Ok(mt) = device.cast::<ID3D11Multithread>() {
        let _ = mt.SetMultithreadProtected(true);
      }
      let dxgi: IDXGIDevice = device.cast()?;
      let winrt: IDirect3DDevice = CreateDirect3D11DeviceFromDXGIDevice(&dxgi)?.cast()?;
      Ok((device, winrt))
    }
  }

  pub fn capture_item(item: &Item) -> windows::core::Result<GraphicsCaptureItem> {
    let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    unsafe {
      match item {
        Item::Window(h) => interop.CreateForWindow(HWND(*h as _)),
        Item::Monitor(m) => interop.CreateForMonitor(HMONITOR(*m as _)),
      }
    }
  }

  /// Pede ao Windows no máximo `fps` quadros por segundo. Sem isso, um jogo a 170 fps num monitor de 165 Hz
  /// faz o DWM copiar ~165 quadros/s para o pool, e o descarte por intervalo jogava fora quase 2 de cada 3.
  /// Windows 11 24H2+; antes disso falha e fica só o descarte por intervalo.
  fn limit_rate(session: &GraphicsCaptureSession, fps: u32) {
    // Mesma folga de 10% do descarte por intervalo, para não perder quadro por arredondamento do vsync.
    let hundred_ns = 10_000_000 / fps.max(1) as i64 * 9 / 10;
    let _ = session.SetMinUpdateInterval(windows::Foundation::TimeSpan { Duration: hundred_ns });
  }

  impl Video {
    /// Começa a capturar. `slots` já foi postado para a página com o tamanho alvo.
    pub fn start(item: &Item, resolution: u32, fps: u32, frames: FrameSink, slots_for: impl FnOnce(u32, u32) -> Result<SlotSet, String>) -> Result<Self, String> {
      let e = |e: windows::core::Error| e.to_string();
      let item = capture_item(item).map_err(e)?;
      let size = item.Size().map_err(e)?;
      let (w, h) = target_size(size.Width as u32, size.Height as u32, resolution);
      let slots = slots_for(w, h)?;
      let (device, winrt) = d3d().map_err(e)?;
      let pipeline = Pipeline::new(device, (size.Width.max(1) as u32, size.Height.max(1) as u32), (w, h)).map_err(e)?;
      let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(&winrt, DirectXPixelFormat::B8G8R8A8UIntNormalized, 2, size).map_err(e)?;
      let session = pool.CreateCaptureSession(&item).map_err(e)?;
      let _ = session.SetIsCursorCaptureEnabled(true);
      limit_rate(&session, fps);
      // Windows 11: sem a borda amarela (pede o acesso "sem borda" antes; no 10 a borda fica).
      if windows::Graphics::Capture::GraphicsCaptureAccess::RequestAccessAsync(windows::Graphics::Capture::GraphicsCaptureAccessKind::Borderless)
        .and_then(|op| op.join())
        .is_ok()
      {
        let _ = session.SetIsBorderRequired(false);
      }
      let shared = Arc::new(Shared {
        pipeline: Mutex::new(pipeline),
        slots: Mutex::new(slots),
        interval_us: AtomicU64::new(1_000_000 / fps.max(1) as u64),
        seq: AtomicU64::new(0),
        last: Mutex::new(None),
        started: Instant::now(),
        dark_frames: AtomicU64::new(0),
        warned_black: AtomicBool::new(false),
        ended: AtomicBool::new(false),
        pool_size: Mutex::new((size.Width, size.Height)),
      });

      let on_frame = {
        let shared = shared.clone();
        let frames = frames.clone();
        let winrt = AgileDevice(winrt.clone());
        windows::Foundation::TypedEventHandler::<Direct3D11CaptureFramePool, windows::core::IInspectable>::new(move |pool, _| {
          let Some(pool) = pool.as_ref() else { return Ok(()) };
          if shared.ended.load(Ordering::SeqCst) { return Ok(()); }
          let mut frame = pool.TryGetNextFrame()?;
          // O pool tem dois buffers: após uma pausa, usar o mais recente disponível.
          while let Ok(newer) = pool.TryGetNextFrame() { frame = newer; }
          let content = frame.ContentSize()?;
          // A janela mudou de tamanho: o pool acompanha (o tamanho de saída não muda).
          {
            let mut ps = shared.pool_size.lock().unwrap();
            if (content.Width, content.Height) != *ps && content.Width > 0 && content.Height > 0 {
              *ps = (content.Width, content.Height);
              let _ = pool.Recreate(winrt.get(), DirectXPixelFormat::B8G8R8A8UIntNormalized, 2, content);
            }
          }
          let now = Instant::now();
          {
            let mut last = shared.last.lock().unwrap();
            let interval = Duration::from_micros(shared.interval_us.load(Ordering::Relaxed) * 9 / 10);
            if last.is_some_and(|l| now - l < interval) {
              return Ok(());
            }
            *last = Some(now);
          }
          let access: IDirect3DDxgiInterfaceAccess = frame.Surface()?.cast()?;
          let tex: ID3D11Texture2D = unsafe { access.GetInterface()? };
          // Ordem fixa dos travamentos (pipeline → slots), a mesma do set_quality: o tamanho do NV12
          // e o do buffer de destino nunca se desencontram.
          let mut pipeline = shared.pipeline.lock().unwrap();
          if shared.ended.load(Ordering::SeqCst) { return Ok(()); }
          let seq = shared.seq.fetch_add(1, Ordering::Relaxed);
          let slot = (seq % 3) as usize;
          let (generation, ptr) = {
            let s = shared.slots.lock().unwrap();
            if (s.width, s.height) != pipeline.out_size {
              return Ok(());
            }
            (s.generation, s.ptrs[slot].0)
          };
          let luma = pipeline.convert(&tex, (content.Width.max(0) as u32, content.Height.max(0) as u32), ptr)?;
          drop(pipeline);
          if shared.ended.load(Ordering::SeqCst) { return Ok(()); }
          let ts = (now - shared.started).as_micros() as u64;
          let _ = frames.send(json!({ "t": "frame", "gen": generation, "slot": slot, "seq": seq, "ts": ts }));
          // Jogo em tela cheia exclusiva costuma sair preto (T546).
          if seq < BLACK_FRAMES {
            if luma < BLACK_Y {
              shared.dark_frames.fetch_add(1, Ordering::Relaxed);
            }
            if seq == BLACK_FRAMES - 1
              && shared.dark_frames.load(Ordering::Relaxed) >= BLACK_FRAMES
              && !shared.warned_black.swap(true, Ordering::Relaxed)
            {
              let _ = frames.send(json!({ "t": "warn", "reason": "black" }));
            }
          }
          Ok(())
        })
      };
      pool.FrameArrived(&on_frame).map_err(e)?;

      let on_closed = {
        let shared = shared.clone();
        let frames = frames.clone();
        windows::Foundation::TypedEventHandler::<GraphicsCaptureItem, windows::core::IInspectable>::new(move |_, _| {
          if !shared.ended.swap(true, Ordering::SeqCst) {
            let _ = frames.send(json!({ "t": "ended", "reason": "source_closed" }));
          }
          Ok(())
        })
      };
      item.Closed(&on_closed).map_err(e)?;
      session.StartCapture().map_err(e)?;
      let content = (size.Width.max(0) as u32, size.Height.max(0) as u32);
      Ok(Self { shared, pool, session, device: AgileDevice(winrt), frames, content })
    }

    pub fn size(&self) -> (u32, u32) {
      let s = self.shared.slots.lock().unwrap();
      (s.width, s.height)
    }

    /// Troca resolução/fps no ar: novo tamanho → nova geração de slots.
    pub fn set_quality(&self, resolution: u32, fps: u32, slots_for: impl FnOnce(u32, u32) -> Result<SlotSet, String>) -> Result<(u32, u32), String> {
      self.shared.interval_us.store(1_000_000 / fps.max(1) as u64, Ordering::Relaxed);
      limit_rate(&self.session, fps);
      let (cw, ch) = *self.shared.pool_size.lock().unwrap();
      let src = if cw > 0 && ch > 0 { (cw as u32, ch as u32) } else { self.content };
      let (w, h) = target_size(src.0, src.1, resolution);
      if (w, h) == self.size() {
        return Ok((w, h));
      }
      let slots = slots_for(w, h)?;
      let mut pipeline = self.shared.pipeline.lock().unwrap();
      let in_size = pipeline.in_size;
      *pipeline = Pipeline::new(pipeline.device.clone(), in_size, (w, h)).map_err(|e| e.to_string())?;
      *self.shared.slots.lock().unwrap() = slots;
      drop(pipeline);
      Ok((w, h))
    }

    pub fn stop(&self, reason: &str) {
      let was_ended = self.shared.ended.swap(true, Ordering::SeqCst);
      // Aguarda uma cópia já em andamento antes de liberar os buffers compartilhados.
      drop(self.shared.pipeline.lock().unwrap());
      let _ = self.session.Close();
      let _ = self.pool.Close();
      if !was_ended {
        let _ = self.frames.send(json!({ "t": "ended", "reason": reason }));
      }
      let _ = &self.device;
    }
  }

}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn alvo_mantem_proporcao_e_e_par() {
    assert_eq!(target_size(1920, 1080, 1080), (1920, 1080));
    assert_eq!(target_size(1920, 1080, 720), (1280, 720));
    assert_eq!(target_size(2560, 1440, 1080), (1920, 1080));
    let (w, h) = target_size(3440, 1440, 1080);
    assert_eq!((w % 2, h % 2), (0, 0));
    assert!(w <= 1920 && h <= 1080);
  }

  #[test]
  fn alvo_nao_amplia_janela_pequena() {
    assert_eq!(target_size(801, 601, 1080), (800, 600));
  }

  #[test]
  fn alvo_com_tamanho_zero() {
    assert_eq!(target_size(0, 0, 720), (1280, 720));
  }

  #[test]
  fn letterbox_centraliza() {
    assert_eq!(letterbox(1920, 1080, 1280, 720), (0, 0, 1280, 720));
    assert_eq!(letterbox(1000, 1000, 1280, 720), (280, 0, 1000, 720));
  }

  #[test]
  fn tamanho_nv12() {
    assert_eq!(nv12_size(1920, 1080), 1920 * 1080 * 3 / 2);
  }
}
