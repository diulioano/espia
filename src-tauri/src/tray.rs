// Bandeja do app desktop (specs/007, contracts/tray.md): o X esconde a janela, o ícone traz de volta,
// o menu tem Parar transmissão e Sair (que pergunta se você está ao vivo).
use serde_json::{Value, json};

pub const TIP: &str = "Espia";
pub const TIP_LIVE: &str = "Espia · ao vivo";
pub const NOTICE_TITLE: &str = "O Espia continua aberto";
pub const NOTICE_BODY: &str =
  "Ele fica aqui perto do relógio (às vezes na setinha ^). Pra sair de vez, clique com o botão direito no ícone e escolha Sair.";

/// Preferências guardadas em `tray.json` (data-model.md).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrayPrefs {
  /// `false` (padrão): o X esconde. `true`: o X fecha.
  pub close_on_x: bool,
  /// O aviso da primeira vez já apareceu.
  pub tray_notice_shown: bool,
}

/// Arquivo ausente, ilegível ou com campos estranhos → padrão (campos desconhecidos são ignorados).
pub fn parse_prefs(text: &str) -> TrayPrefs {
  let Ok(v) = serde_json::from_str::<Value>(text) else { return TrayPrefs::default() };
  let flag = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(false);
  TrayPrefs { close_on_x: flag("closeOnX"), tray_notice_shown: flag("trayNoticeShown") }
}

pub fn prefs_json(p: &TrayPrefs) -> String {
  json!({ "closeOnX": p.close_on_x, "trayNoticeShown": p.tray_notice_shown }).to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseAction {
  Hide,
  Exit,
  ConfirmExit,
}

/// O que o X faz.
pub fn on_close(prefs: &TrayPrefs, live: bool) -> CloseAction {
  if !prefs.close_on_x {
    CloseAction::Hide
  } else {
    on_quit(live)
  }
}

/// O que "Sair" faz.
pub fn on_quit(live: bool) -> CloseAction {
  if live { CloseAction::ConfirmExit } else { CloseAction::Exit }
}

#[cfg(windows)]
pub use imp::*;

#[cfg(windows)]
mod imp {
  use std::{
    path::PathBuf,
    sync::{
      Mutex,
      atomic::{AtomicBool, Ordering},
    },
  };

  use tauri::{
    AppHandle, Manager, Runtime, State, Webview, Window,
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
  };
  use tauri_plugin_notification::NotificationExt;

  use super::*;

  const ICON: &[u8] = include_bytes!("../icons/tray.png");
  const ICON_LIVE: &[u8] = include_bytes!("../icons/tray-live.png");

  #[derive(Default)]
  pub struct TrayState {
    live: AtomicBool,
    prefs: Mutex<TrayPrefs>,
    path: Option<PathBuf>,
    /// Saindo de verdade: o CloseRequested deixa fechar.
    quitting: AtomicBool,
    /// A confirmação já está na tela.
    asking: AtomicBool,
  }

  impl TrayState {
    pub fn load<R: Runtime>(app: &AppHandle<R>) -> Self {
      let path = app.path().app_config_dir().ok().map(|d| d.join("tray.json"));
      let prefs = path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).map(|t| parse_prefs(&t)).unwrap_or_default();
      Self { prefs: Mutex::new(prefs), path, ..Default::default() }
    }

    fn prefs(&self) -> TrayPrefs {
      *self.prefs.lock().unwrap()
    }

    fn update(&self, f: impl FnOnce(&mut TrayPrefs)) {
      let mut p = self.prefs.lock().unwrap();
      f(&mut p);
      let Some(path) = &self.path else { return };
      let saved = path.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(path, prefs_json(&p)));
      if let Err(e) = saved {
        eprintln!("[espia] não deu pra gravar {}: {e}", path.display());
      }
    }

    pub fn live(&self) -> bool {
      self.live.load(Ordering::Relaxed)
    }
  }

  fn build_menu<R: Runtime>(app: &AppHandle<R>, live: bool, prefs: &TrayPrefs) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(app, "open", "Abrir o Espia", true, None::<&str>)?)?;
    if live {
      menu.append(&MenuItem::with_id(app, "stop", "Parar transmissão", true, None::<&str>)?)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&CheckMenuItem::with_id(app, "close-on-x", "Fechar no X", true, prefs.close_on_x, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?)?;
    Ok(menu)
  }

  pub fn setup<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let state = app.state::<TrayState>();
    TrayIconBuilder::with_id("main")
      .icon(Image::from_bytes(ICON)?)
      .tooltip(TIP)
      .menu(&build_menu(app, false, &state.prefs())?)
      .show_menu_on_left_click(false)
      .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
          show(tray.app_handle());
        }
      })
      .on_menu_event(|app, event| match event.id().as_ref() {
        "open" => show(app),
        "stop" => {
          if let Some(w) = app.get_webview_window("main") {
            let _ = w.eval("window.espiaStopLive?.()");
          }
        }
        "close-on-x" => {
          app.state::<TrayState>().update(|p| p.close_on_x = !p.close_on_x);
          refresh(app);
        }
        "quit" => quit(app),
        _ => {}
      })
      .build(app)?;
    Ok(())
  }

  fn show<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
      crate::focus(&w);
    }
  }

  /// Ícone, dica e menu conforme o estado (FR-706).
  pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let Some(tray) = app.tray_by_id("main") else { return };
    let state = app.state::<TrayState>();
    let live = state.live();
    let _ = tray.set_icon(Image::from_bytes(if live { ICON_LIVE } else { ICON }).ok());
    let _ = tray.set_tooltip(Some(if live { TIP_LIVE } else { TIP }));
    if let Ok(menu) = build_menu(app, live, &state.prefs()) {
      let _ = tray.set_menu(Some(menu));
    }
  }

  pub fn set_live<R: Runtime>(app: &AppHandle<R>, live: bool) {
    if app.state::<TrayState>().live.swap(live, Ordering::Relaxed) != live {
      refresh(app);
    }
  }

  /// Página → app: começou ou parou de transmitir (contracts/tray.md).
  #[tauri::command]
  pub fn app_set_live<R: Runtime>(webview: Webview<R>, live: bool) {
    if crate::capture::origin_ok(&webview) {
      set_live(webview.app_handle(), live);
    }
  }

  /// O X da janela principal (R3).
  pub fn on_close_requested<R: Runtime>(window: &Window<R>, api: &tauri::CloseRequestApi) {
    let app = window.app_handle();
    let state = app.state::<TrayState>();
    if state.quitting.load(Ordering::Relaxed) {
      return;
    }
    api.prevent_close();
    match on_close(&state.prefs(), state.live()) {
      CloseAction::Hide => hide(app, window, &state),
      _ => quit(app),
    }
  }

  fn hide<R: Runtime>(app: &AppHandle<R>, window: &Window<R>, state: &State<TrayState>) {
    let _ = window.hide();
    if !state.prefs().tray_notice_shown {
      let _ = app.notification().builder().title(NOTICE_TITLE).body(NOTICE_BODY).show();
      state.update(|p| p.tray_notice_shown = true);
    }
  }

  /// Sair de verdade; transmitindo, pergunta antes (R5).
  pub fn quit<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<TrayState>();
    match on_quit(state.live()) {
      CloseAction::ConfirmExit => {
        if state.asking.swap(true, Ordering::Relaxed) {
          return;
        }
        let window = app.get_webview_window("main");
        if let Some(w) = &window {
          crate::focus(w);
        }
        let hwnd = window.and_then(|w| w.hwnd().ok()).map_or(0, |h| h.0 as isize);
        let app = app.clone();
        // Fora do laço de eventos, para a bandeja não travar enquanto a caixa está aberta.
        std::thread::spawn(move || {
          let yes = confirm_exit(hwnd);
          app.state::<TrayState>().asking.store(false, Ordering::Relaxed);
          if yes {
            exit(&app);
          }
        });
      }
      _ => exit(&app.clone()),
    }
  }

  fn exit<R: Runtime>(app: &AppHandle<R>) {
    use tauri_plugin_window_state::{AppHandleExt, StateFlags};
    app.state::<TrayState>().quitting.store(true, Ordering::Relaxed);
    let _ = app.save_window_state(StateFlags::all());
    app.exit(0);
  }

  fn confirm_exit(hwnd: isize) -> bool {
    use windows::{
      Win32::{Foundation::HWND, UI::WindowsAndMessaging::*},
      core::w,
    };
    let owner = HWND(hwnd as *mut _);
    let flags = MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2 | MB_SETFOREGROUND;
    unsafe { MessageBoxW(Some(owner), w!("Você está transmitindo. Sair mesmo assim?"), w!("Espia"), flags) == IDYES }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn prefs_ausentes_ou_invalidas_viram_padrao() {
    for text in ["", "{", "null", "[]", "42", r#"{"closeOnX":"sim","trayNoticeShown":1}"#] {
      assert_eq!(parse_prefs(text), TrayPrefs::default(), "{text}");
    }
  }

  #[test]
  fn prefs_lidas_ignorando_campos_extras() {
    let p = parse_prefs(r#"{"closeOnX":true,"trayNoticeShown":true,"outro":5}"#);
    assert_eq!(p, TrayPrefs { close_on_x: true, tray_notice_shown: true });
    assert_eq!(parse_prefs(r#"{"trayNoticeShown":true}"#), TrayPrefs { close_on_x: false, tray_notice_shown: true });
  }

  #[test]
  fn prefs_ida_e_volta() {
    for close_on_x in [false, true] {
      for tray_notice_shown in [false, true] {
        let p = TrayPrefs { close_on_x, tray_notice_shown };
        assert_eq!(parse_prefs(&prefs_json(&p)), p);
      }
    }
  }

  #[test]
  fn x_esconde_por_padrao_mesmo_ao_vivo() {
    let p = TrayPrefs::default();
    assert_eq!(on_close(&p, false), CloseAction::Hide);
    assert_eq!(on_close(&p, true), CloseAction::Hide);
  }

  #[test]
  fn x_fecha_com_a_opcao_e_pergunta_ao_vivo() {
    let p = TrayPrefs { close_on_x: true, ..Default::default() };
    assert_eq!(on_close(&p, false), CloseAction::Exit);
    assert_eq!(on_close(&p, true), CloseAction::ConfirmExit);
  }

  #[test]
  fn sair_pergunta_so_ao_vivo() {
    assert_eq!(on_quit(false), CloseAction::Exit);
    assert_eq!(on_quit(true), CloseAction::ConfirmExit);
  }
}
