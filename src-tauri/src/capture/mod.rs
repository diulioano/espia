// Ponte de captura nativa (specs/005, contracts/native-capture.md). Única API nativa que o app expõe à
// página (constituição 1.2.0): 5 comandos, só para a origem do site, só na janela principal.
pub mod audio;
pub mod discord;
pub mod procs;
pub mod sources;
pub mod thumbs;
pub mod video;
mod publish;

#[cfg(windows)]
pub use imp::*;

/// Inicializa COM (MTA) na thread atual e desfaz ao sair do escopo.
#[cfg(windows)]
pub struct ComGuard(bool);

#[cfg(windows)]
pub fn com_init() -> ComGuard {
  use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};
  ComGuard(unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok())
}

#[cfg(windows)]
impl Drop for ComGuard {
  fn drop(&mut self) {
    if self.0 {
      unsafe { windows::Win32::System::Com::CoUninitialize() };
    }
  }
}

#[cfg(windows)]
mod imp {
  use std::{
    sync::{
      Arc, Mutex,
      atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
  };

  use serde_json::{Value, json};
  use tauri::{
    Runtime, State, Webview,
    ipc::{Channel, InvokeResponseBody},
  };
  use windows::Win32::Foundation::HWND;

  use super::{
    audio::{self, Loopback, Plan, Target},
    discord::discord_roots,
    procs::{self, tree_root},
    sources::{self, CaptureSource, SourceId, parse_id},
    video::{Item, SlotSet, Video},
  };
  use crate::links;
  use super::publish::{Message, PublishQueue};

  const VERSION: u32 = 1;

  /// A página que chama é a do Espia, na janela principal (R9).
  pub(crate) fn origin_ok<R: Runtime>(webview: &Webview<R>) -> bool {
    if webview.label() != "main" {
      return false;
    }
    let Ok(url) = webview.url() else { return false };
    let site = url.scheme() == "https" && url.host_str() == links::ORIGIN.strip_prefix("https://");
    let dev = cfg!(debug_assertions) && url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1" | "localhost"));
    site || dev
  }

  fn denied() -> String {
    "forbidden".into()
  }

  /// Uma transmissão nativa por vez.
  #[derive(Default)]
  pub struct Capture {
    session: Mutex<Option<Session>>,
    epoch: AtomicU64,
    pending: Mutex<Option<Arc<Mutex<PublishQueue>>>>,
    starting: tauri::async_runtime::Mutex<()>,
  }

  struct Session {
    publication: Arc<Mutex<PublishQueue>>,
    video: Video,
    sound: Arc<Mutex<Option<Loopback>>>,
    /// Vigia do Discord (tela inteira com som).
    watching: Arc<AtomicBool>,
    generation: u32,
  }

  impl Session {
    fn stop<R: Runtime>(self, webview: &Webview<R>, reason: &str) {
      self.publication.lock().unwrap().stop();
      self.watching.store(false, Ordering::Relaxed);
      self.sound.lock().unwrap().take();
      self.video.stop(reason);
      SlotSet::release(webview, self.generation);
    }
  }

  /// Para a transmissão nativa (recarregou/navegou a página, fechou a janela).
  pub fn stop_all<R: Runtime>(webview: &Webview<R>, capture: &Capture) {
    capture.epoch.fetch_add(1, Ordering::SeqCst);
    if let Some(q) = capture.pending.lock().unwrap().take() { q.lock().unwrap().stop(); }
    if let Some(s) = capture.session.lock().unwrap().take() {
      s.stop(webview, "stopped");
    }
  }

  #[tauri::command]
  pub async fn capture_supported<R: Runtime>(webview: Webview<R>) -> Value {
    if !origin_ok(&webview) {
      return json!({ "ok": false, "version": VERSION, "win11": false });
    }
    let ok = windows::Graphics::Capture::GraphicsCaptureSession::IsSupported().unwrap_or(false);
    json!({ "ok": ok, "version": VERSION, "win11": is_win11() })
  }

  fn is_win11() -> bool {
    use windows::Win32::System::SystemInformation::OSVERSIONINFOW;
    // RtlGetVersion não mente sobre a versão (GetVersionEx mente sem manifesto).
    #[link(name = "ntdll")]
    unsafe extern "system" {
      fn RtlGetVersion(info: *mut OSVERSIONINFOW) -> i32;
    }
    let mut info = OSVERSIONINFOW { dwOSVersionInfoSize: size_of::<OSVERSIONINFOW>() as u32, ..Default::default() };
    unsafe { RtlGetVersion(&mut info) == 0 && info.dwBuildNumber >= 22000 }
  }

  #[tauri::command]
  pub async fn capture_list<R: Runtime>(webview: Webview<R>, thumbs: bool) -> Result<Vec<CaptureSource>, String> {
    if !origin_ok(&webview) {
      return Err(denied());
    }
    // Miniaturas só com o Espia em primeiro plano (o seletor aberto na frente): a página não
    // consegue "espiar" as outras janelas em segundo plano.
    let thumbs = thumbs && webview.window().is_focused().unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || {
      let _com = super::com_init();
      let table = procs::snapshot();
      let playing = audio::playing_roots(&table);
      sources::list(thumbs)
        .into_iter()
        .map(|mut l| {
          l.source.playing = l.pid != 0 && playing.contains(&tree_root(l.pid, &table));
          l.source
        })
        .collect()
    })
    .await
    .map_err(|_| "list_failed".to_string())
  }

  /// Plano de som para a fonte e o estado atual do Discord.
  fn plan_for(target: Target, wants: bool) -> Plan {
    let table = procs::snapshot();
    let target = match target {
      Target::Window(pid) => Target::Window(tree_root(pid, &table)),
      t => t,
    };
    audio::audio_plan(target, wants, &discord_roots(&table), std::process::id())
  }

  /// Para onde vão os pedaços de som (10 ms, f32 intercalado).
  pub(crate) type SoundSink = Arc<dyn Fn(&[u8]) + Send + Sync>;

  /// Avisos de quadro e de estado (`{t: 'frame' | 'audio' | 'warn' | 'ended', …}`), pelo mesmo canal de mensagens
  /// do WebView2 que o som, como `{"espia":"f","m":…}` (specs/010): 60 avisos por segundo por `eval` também
  /// faziam a página crescer.
  #[derive(Clone)]
  pub struct FrameSink(Arc<dyn Fn(Value) + Send + Sync>);

  impl FrameSink {
    pub fn send(&self, msg: Value) {
      (self.0)(msg);
    }
  }

  fn schedule_publish<R: Runtime>(webview: &Webview<R>, queue: &Arc<Mutex<PublishQueue>>) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
    use windows::core::HSTRING;
    {
      let mut q = queue.lock().unwrap();
      if q.scheduled || !q.pending() { return; }
      q.scheduled = true;
    }
    let q2 = queue.clone();
    let webview2 = webview.clone();
    let result = webview.with_webview(move |wv| unsafe {
      let messages = q2.lock().unwrap().take(Instant::now());
      let core: Result<ICoreWebView2, _> = wv.controller().CoreWebView2();
      if let Ok(core) = core {
        for message in messages {
          if q2.lock().unwrap().stopped { break; }
          let json = match message {
            Message::Frame(m) => json!({"espia":"f","m":m}).to_string(),
            Message::Pcm(bytes) => sound_json(&bytes),
          };
          let _ = core.PostWebMessageAsJson(&HSTRING::from(json));
        }
      }
      q2.lock().unwrap().scheduled = false;
      schedule_publish(&webview2, &q2);
    });
    if result.is_err() { queue.lock().unwrap().scheduled = false; }
  }

  fn frame_sink<R: Runtime>(webview: &Webview<R>, queue: &Arc<Mutex<PublishQueue>>) -> FrameSink {
    let webview = webview.clone();
    let queue = queue.clone();
    FrameSink(Arc::new(move |m| {
      queue.lock().unwrap().frame(m, Instant::now());
      schedule_publish(&webview, &queue);
    }))
  }

  /// Mensagem de som para a página: `{"espia":"pcm","d":"<base64>"}`.
  pub(crate) fn sound_json(bytes: &[u8]) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    json!({ "espia": "pcm", "d": STANDARD.encode(bytes) }).to_string()
  }

  /// O som vai pelo canal de mensagens do WebView2 (`chrome.webview` "message"), não por um canal do Tauri:
  /// cada mensagem do Tauri vira um script executado na página (`eval`), e 100 por segundo faziam a página do
  /// motor crescer sem parar (specs/010, research R2).
  fn sound_sink<R: Runtime>(webview: &Webview<R>, queue: &Arc<Mutex<PublishQueue>>) -> SoundSink {
    let webview = webview.clone();
    let queue = queue.clone();
    Arc::new(move |bytes| {
      queue.lock().unwrap().pcm(bytes, Instant::now());
      schedule_publish(&webview, &queue);
    })
  }

  /// Liga o som do plano; se o Windows não deixar, fica sem som e avisa (nunca o PC inteiro).
  fn start_sound(plan: Plan, sound: &SoundSink) -> (Option<Loopback>, Plan) {
    if matches!(plan, Plan::None(_)) {
      return (None, plan);
    }
    let ch = sound.clone();
    match Loopback::start(plan, move |bytes| {
      ch(&bytes);
    }) {
      Ok(l) => (Some(l), plan),
      Err(e) => {
        eprintln!("[espia] som por processo indisponível: {e}");
        (None, Plan::None(Some(audio::Warn::Unsupported)))
      }
    }
  }

  fn result(video: &Video, plan: Plan) -> Value {
    let (w, h) = video.size();
    let mut out = json!({ "width": w, "height": h, "audio": plan.kind() });
    if let Some(w) = plan.warn() {
      out["audioWarn"] = json!(w.as_str());
    }
    out
  }

  #[tauri::command]
  #[allow(clippy::too_many_arguments)]
  pub async fn capture_start<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, Capture>,
    id: String,
    resolution: u32,
    fps: u32,
    audio: bool,
    frames: Channel<Value>,
    sound: Channel<InvokeResponseBody>,
  ) -> Result<Value, String> {
    if !origin_ok(&webview) {
      return Err(denied());
    }
    let _starting = state.starting.lock().await;
    let resolution = if resolution >= 1080 { 1080 } else { 720 };
    let fps = if fps >= 60 { 60 } else { 30 };
    let source = parse_id(&id).ok_or("source_gone")?;
    // Uma sessão por vez: começar outra encerra a anterior.
    stop_all(&webview, &state);
    let epoch = state.epoch.load(Ordering::SeqCst);
    let publication = Arc::new(Mutex::new(PublishQueue::default()));
    *state.pending.lock().unwrap() = Some(publication.clone());

    let (item, target) = match source {
      SourceId::Window(h) => {
        let hwnd = HWND(h as _);
        if !sources::window_alive(hwnd) {
          return Err("source_gone".into());
        }
        (Item::Window(h), Target::Window(sources::window_pid(hwnd)))
      }
      SourceId::Monitor(m) => {
        if sources::monitor_rect(m).is_none() {
          return Err("source_gone".into());
        }
        (Item::Monitor(m), Target::Monitor)
      }
    };

    let wv = webview.clone();
    // Os canais `frames`/`sound` do Tauri continuam no contrato (páginas antigas mandam), mas não são mais usados.
    let _ = frames;
    let frames2 = frame_sink(&webview, &publication);
    let _ = sound;
    let sound2 = sound_sink(&webview, &publication);
    let cleanup = publication.clone();
    let session = tauri::async_runtime::spawn_blocking(move || {
      let _com = super::com_init();
      let video = Video::start(&item, resolution, fps, frames2.clone(), |w, h| SlotSet::post(&wv, 1, w, h)).map_err(|e| {
        eprintln!("[espia] captura falhou: {e}");
        "capture_failed".to_string()
      })?;
      let (loopback, plan) = start_sound(plan_for(target, audio), &sound2);
      let sound_slot = Arc::new(Mutex::new(loopback));
      let watching = Arc::new(AtomicBool::new(false));
      // Tela inteira com som: o Discord pode abrir/reiniciar no meio (R7).
      if target == Target::Monitor && audio {
        watching.store(true, Ordering::Relaxed);
        watch_discord(watching.clone(), sound_slot.clone(), plan, sound2.clone(), frames2.clone());
      }
      let out = result(&video, plan);
      Ok::<_, String>((Session { publication, video, sound: sound_slot, watching, generation: 1 }, out))
    })
    .await
    .map_err(|_| "capture_failed".to_string())
    .and_then(|r| r);

    let session = match session {
      Ok(s) => s,
      Err(e) => {
        cleanup.lock().unwrap().stop();
        SlotSet::release(&webview, 1);
        return Err(e);
      }
    };

    let (session, out) = session;
    let mut guard = state.session.lock().unwrap();
    if state.epoch.load(Ordering::SeqCst) != epoch {
      session.stop(&webview, "stopped");
      return Err("capture_failed".into());
    }
    *guard = Some(session);
    Ok(out)
  }

  fn watch_discord(
    watching: Arc<AtomicBool>,
    slot: Arc<Mutex<Option<Loopback>>>,
    mut current: Plan,
    sound: SoundSink,
    frames: FrameSink,
  ) {
    let mut roots = discord_roots(&procs::snapshot());
    thread::spawn(move || {
      let _com = super::com_init();
      while watching.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));
        if !watching.load(Ordering::Relaxed) {
          break;
        }
        let now = discord_roots(&procs::snapshot());
        if now == roots {
          continue;
        }
        roots = now;
        let wanted = audio::audio_plan(Target::Monitor, true, &roots, std::process::id());
        // Para o som antigo antes de abrir o novo: nunca os dois (nem o Discord) juntos.
        slot.lock().unwrap().take();
        let (loopback, plan) = start_sound(wanted, &sound);
        {
          // Confere a parada com o slot travado: Session::stop desliga a vigia antes de esvaziar o
          // slot, então nenhum som novo fica ligado depois de parar.
          let mut guard = slot.lock().unwrap();
          if !watching.load(Ordering::Relaxed) {
            break;
          }
          *guard = loopback;
        }
        if plan.kind() != current.kind() || plan.warn() != current.warn() {
          let mut msg = json!({ "t": "audio", "kind": plan.kind() });
          if let Some(w) = plan.warn() {
            msg["warn"] = json!(w.as_str());
          }
          let _ = frames.send(msg);
        }
        current = plan;
      }
    });
  }

  #[tauri::command]
  pub async fn capture_set_quality<R: Runtime>(webview: Webview<R>, state: State<'_, Capture>, resolution: u32, fps: u32) -> Result<Value, String> {
    if !origin_ok(&webview) {
      return Err(denied());
    }
    let resolution = if resolution >= 1080 { 1080 } else { 720 };
    let fps = if fps >= 60 { 60 } else { 30 };
    let mut guard = state.session.lock().unwrap();
    let session = guard.as_mut().ok_or("not_live")?;
    let next = session.generation + 1;
    let before = session.video.size();
    let (w, h) = session.video.set_quality(resolution, fps, |w, h| SlotSet::post(&webview, next, w, h))?;
    if (w, h) != before {
      session.publication.lock().unwrap().generation(next as u64);
      SlotSet::release(&webview, session.generation);
      session.generation = next;
    }
    Ok(json!({ "width": w, "height": h }))
  }

  #[tauri::command]
  pub async fn capture_stop<R: Runtime>(webview: Webview<R>, state: State<'_, Capture>) -> Result<(), String> {
    if origin_ok(&webview) {
      stop_all(&webview, &state);
    }
    Ok(())
  }
}

#[cfg(all(test, windows))]
mod tests {
  use super::{audio::CHUNK_FRAMES, sound_json};

  #[test]
  fn mensagem_de_som_volta_igual() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let bytes: Vec<u8> = (0..CHUNK_FRAMES * 2 * 4).map(|i| i as u8).collect();
    let v: serde_json::Value = serde_json::from_str(&sound_json(&bytes)).unwrap();
    assert_eq!(v["espia"], "pcm");
    assert_eq!(STANDARD.decode(v["d"].as_str().unwrap()).unwrap(), bytes);
  }
}
