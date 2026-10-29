// Ponte de captura nativa (specs/005, contracts/native-capture.md). Única API nativa que o app expõe à
// página (constituição 1.2.0): 5 comandos, só para a origem do site, só na janela principal.
pub mod audio;
pub mod discord;
pub mod procs;
pub mod sources;
pub mod thumbs;
pub mod video;

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
      atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
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
  pub struct Capture(Mutex<Option<Session>>);

  struct Session {
    video: Video,
    sound: Arc<Mutex<Option<Loopback>>>,
    /// Vigia do Discord (tela inteira com som).
    watching: Arc<AtomicBool>,
    generation: u32,
  }

  impl Session {
    fn stop<R: Runtime>(self, webview: &Webview<R>, reason: &str) {
      self.watching.store(false, Ordering::Relaxed);
      self.sound.lock().unwrap().take();
      self.video.stop(reason);
      SlotSet::release(webview, self.generation);
    }
  }

  /// Para a transmissão nativa (recarregou/navegou a página, fechou a janela).
  pub fn stop_all<R: Runtime>(webview: &Webview<R>, capture: &Capture) {
    if let Some(s) = capture.0.lock().unwrap().take() {
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

  /// Liga o som do plano; se o Windows não deixar, fica sem som e avisa (nunca o PC inteiro).
  fn start_sound(plan: Plan, sound: &Channel<InvokeResponseBody>) -> (Option<Loopback>, Plan) {
    if matches!(plan, Plan::None(_)) {
      return (None, plan);
    }
    let ch = sound.clone();
    match Loopback::start(plan, move |bytes| {
      let _ = ch.send(InvokeResponseBody::Raw(bytes));
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
    let resolution = if resolution >= 1080 { 1080 } else { 720 };
    let fps = if fps >= 60 { 60 } else { 30 };
    let source = parse_id(&id).ok_or("source_gone")?;
    // Uma sessão por vez: começar outra encerra a anterior.
    stop_all(&webview, &state);

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
    let frames2 = frames.clone();
    let sound2 = sound.clone();
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
      Ok::<_, String>((Session { video, sound: sound_slot, watching, generation: 1 }, out))
    })
    .await
    .map_err(|_| "capture_failed".to_string())??;

    let (session, out) = session;
    *state.0.lock().unwrap() = Some(session);
    Ok(out)
  }

  fn watch_discord(
    watching: Arc<AtomicBool>,
    slot: Arc<Mutex<Option<Loopback>>>,
    mut current: Plan,
    sound: Channel<InvokeResponseBody>,
    frames: Channel<Value>,
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
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("not_live")?;
    let next = session.generation + 1;
    let before = session.video.size();
    let (w, h) = session.video.set_quality(resolution, fps, |w, h| SlotSet::post(&webview, next, w, h))?;
    if (w, h) != before {
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
