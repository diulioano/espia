// Motor de página do app (specs/009-memoria-do-app, contracts/engine.md): escondido na bandeja e fora de
// sala por 30 s, a janela e o motor (WebView2) são destruídos e recriados ao abrir; minimizado ou
// escondido numa sala, o motor só é avisado para gastar menos memória. Ao vivo, nada muda.
use std::time::Duration;

/// Espera antes de soltar o motor: quem esconde e reabre logo não espera a página recarregar.
pub const RELEASE_AFTER: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
  Visible,
  Minimized,
  /// Escondida na bandeja (o X, specs/007).
  Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineAction {
  Normal,
  /// Pede ao motor que solte o que não usa (research R3).
  Low,
  /// Memória baixa e, se nada mudar até lá, solta o motor.
  ReleaseAfter(Duration),
}

/// A página deve se saber à vista? (specs/014 US2, research R2). Minimizada ou na bandeja, não: assim
/// ela esconde as telas dos outros (o som continua). Vale também ao vivo: o envio não depende disso.
pub fn page_visible(window: WindowMode) -> bool {
  window == WindowMode::Visible
}

/// O que fazer com o motor (data-model.md).
pub fn engine_action(window: WindowMode, in_room: bool, live: bool) -> EngineAction {
  match window {
    _ if live => EngineAction::Normal,
    WindowMode::Visible => EngineAction::Normal,
    WindowMode::Minimized => EngineAction::Low,
    WindowMode::Hidden if in_room => EngineAction::Low,
    WindowMode::Hidden => EngineAction::ReleaseAfter(RELEASE_AFTER),
  }
}

#[cfg(windows)]
pub use imp::*;

#[cfg(windows)]
mod imp {
  use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
  };

  use tauri::{AppHandle, Manager, Runtime, Webview, WebviewWindow};
  use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Controller,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL, ICoreWebView2_19,
  };
  use windows::core::Interface;

  use super::*;
  use crate::{capture, tray::TrayState};

  pub struct EngineState {
    window: Mutex<WindowMode>,
    in_room: AtomicBool,
    /// Cada `apply` aumenta; um timer antigo que expira não faz nada.
    timer_gen: AtomicU64,
  }

  impl Default for EngineState {
    fn default() -> Self {
      Self { window: Mutex::new(WindowMode::Visible), in_room: AtomicBool::new(false), timer_gen: AtomicU64::new(0) }
    }
  }

  impl EngineState {
    fn action<R: Runtime>(&self, app: &AppHandle<R>) -> EngineAction {
      let live = app.state::<TrayState>().live();
      engine_action(*self.window.lock().unwrap(), self.in_room.load(Ordering::Relaxed), live)
    }

    pub fn window(&self) -> WindowMode {
      *self.window.lock().unwrap()
    }
  }

  /// A janela mudou (mostrada, minimizada, escondida). Só reage se mudou de fato.
  pub fn set_window<R: Runtime>(app: &AppHandle<R>, mode: WindowMode) {
    let state = app.state::<EngineState>();
    let changed = std::mem::replace(&mut *state.window.lock().unwrap(), mode) != mode;
    if changed {
      apply(app);
    }
  }

  pub fn set_in_room<R: Runtime>(app: &AppHandle<R>, in_room: bool) {
    if app.state::<EngineState>().in_room.swap(in_room, Ordering::Relaxed) != in_room {
      apply(app);
    }
  }

  /// Página → app: entrou ou saiu de uma sala (contracts/engine.md).
  #[tauri::command]
  pub fn app_set_room<R: Runtime>(webview: Webview<R>, in_room: bool) {
    if capture::origin_ok(&webview) {
      set_in_room(webview.app_handle(), in_room);
    }
  }

  /// Aplica a regra ao estado atual.
  pub fn apply<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<EngineState>();
    let generation = state.timer_gen.fetch_add(1, Ordering::Relaxed) + 1;
    let action = state.action(app);
    let Some(window) = app.get_webview_window("main") else { return };
    set_memory_level(&window, action != EngineAction::Normal);
    set_page_visible(&window, page_visible(state.window()));
    if let EngineAction::ReleaseAfter(wait) = action {
      let app = app.clone();
      std::thread::spawn(move || {
        std::thread::sleep(wait);
        let state = app.state::<EngineState>();
        let same = state.timer_gen.load(Ordering::Relaxed) == generation;
        if same && matches!(state.action(&app), EngineAction::ReleaseAfter(_)) {
          release(&app);
        }
      });
    }
  }

  /// Destrói a janela e, com ela, o motor de página. O ícone da bandeja recria (`crate::create_main`).
  pub fn release<R: Runtime>(app: &AppHandle<R>) {
    use tauri_plugin_window_state::{AppHandleExt, StateFlags};
    let Some(window) = app.get_webview_window("main") else { return };
    capture::stop_all(window.as_ref(), &app.state::<capture::Capture>());
    let _ = app.save_window_state(StateFlags::all());
    if let Err(e) = window.destroy() {
      eprintln!("[espia] não deu pra soltar o motor: {e}");
      return;
    }
    eprintln!("[espia] motor solto (escondido fora de sala)");
  }

  /// Sem isso o WebView2 diz "visible" à página mesmo minimizado ou escondido na bandeja (spike T1401
  /// da specs/014). Com `IsVisible = false` a página recebe `visibilitychange` → "hidden".
  fn set_page_visible<R: Runtime>(window: &WebviewWindow<R>, visible: bool) {
    let _ = window.with_webview(move |wv| unsafe {
      let controller: ICoreWebView2Controller = wv.controller();
      if let Err(e) = controller.SetIsVisible(visible) {
        eprintln!("[espia] visibilidade da página: {e}");
      }
    });
  }

  /// Nível de memória pedido ao motor (research R3). Runtime antigo sem a interface: só log.
  fn set_memory_level<R: Runtime>(window: &WebviewWindow<R>, low: bool) {
    let level = if low { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW } else { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL };
    let _ = window.with_webview(move |wv| unsafe {
      let res = wv.controller().CoreWebView2().and_then(|c| c.cast::<ICoreWebView2_19>()).and_then(|c| c.SetMemoryUsageTargetLevel(level));
      if let Err(e) = res {
        eprintln!("[espia] nível de memória do motor: {e}");
      }
    });
  }
}

#[cfg(test)]
mod tests {
  use super::{EngineAction::*, WindowMode::*, *};

  #[test]
  fn regra_da_data_model() {
    assert_eq!(engine_action(Visible, false, false), Normal);
    assert_eq!(engine_action(Visible, true, false), Normal);
    assert_eq!(engine_action(Minimized, false, false), Low);
    assert_eq!(engine_action(Minimized, true, false), Low);
    assert_eq!(engine_action(Hidden, true, false), Low);
    assert_eq!(engine_action(Hidden, false, false), ReleaseAfter(RELEASE_AFTER));
  }

  #[test]
  fn ao_vivo_nada_muda() {
    for w in [Visible, Minimized, Hidden] {
      for in_room in [false, true] {
        assert_eq!(engine_action(w, in_room, true), Normal);
      }
    }
  }

  #[test]
  fn minimizado_nunca_solta() {
    for in_room in [false, true] {
      assert!(!matches!(engine_action(Minimized, in_room, false), ReleaseAfter(_)));
    }
  }

  #[test]
  fn pagina_a_vista_so_com_a_janela_visivel() {
    assert!(page_visible(Visible));
    assert!(!page_visible(Minimized));
    assert!(!page_visible(Hidden));
  }

  #[test]
  fn espera_de_30_s() {
    assert_eq!(RELEASE_AFTER, Duration::from_secs(30));
  }
}
