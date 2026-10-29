// Esconde a barra "… está compartilhando …" que o WebView2 põe por cima da tela ao transmitir
// (o mesmo aviso flutuante do Chrome). No app, parar é pelo botão "Parar" do próprio Espia.
//
// O WebView2 não tem opção para desligar a barra. Quando a página pede a captura
// (evento ScreenCaptureStarting), procuramos por até 2 min (o seletor fica aberto nesse meio-tempo)
// uma janela pequena do processo do WebView2 com o endereço do site no título, e a escondemos.
// Se o runtime for antigo ou a barra mudar de cara, ela simplesmente continua aparecendo.
use std::{
  sync::atomic::{AtomicU64, Ordering},
  thread,
  time::{Duration, Instant},
};

use webview2_com::{Microsoft::Web::WebView2::Win32::ICoreWebView2_27, ScreenCaptureStartingEventHandler};
use windows::{
  Win32::{
    Foundation::{HWND, LPARAM, RECT},
    UI::WindowsAndMessaging::{
      EnumWindows, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, SW_HIDE,
      ShowWindowAsync,
    },
  },
  core::{BOOL, Interface},
};

/// A barra tem ~50 px; o seletor de tela é bem maior.
const MAX_BAR_HEIGHT: i32 = 240;
const WATCH_FOR: Duration = Duration::from_secs(120);
const POLL_EVERY: Duration = Duration::from_millis(150);

/// Uma captura nova encerra a vigia da anterior.
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn install<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
  let _ = window.with_webview(|wv| unsafe {
    let Ok(core) = wv.controller().CoreWebView2() else { return };
    let Ok(core27) = core.cast::<ICoreWebView2_27>() else { return };
    let handler = ScreenCaptureStartingEventHandler::create(Box::new(|sender, _args| {
      let mut pid = 0u32;
      if let Some(core) = sender {
        let _ = core.BrowserProcessId(&mut pid);
      }
      if pid != 0 {
        watch(pid);
      }
      Ok(())
    }));
    let mut token = Default::default();
    let _ = core27.add_ScreenCaptureStarting(&handler, &mut token);
  });
}

fn watch(browser_pid: u32) {
  let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
  thread::spawn(move || {
    let end = Instant::now() + WATCH_FOR;
    while Instant::now() < end && GENERATION.load(Ordering::SeqCst) == generation {
      if hide_bar(browser_pid) {
        return;
      }
      thread::sleep(POLL_EVERY);
    }
    // Não achou: em debug, lista as janelas do WebView2 pra ajustar a busca.
    if cfg!(debug_assertions) {
      for (title, height) in browser_windows(browser_pid) {
        eprintln!("[share_bar] não escondida: {title:?} ({height}px)");
      }
    }
  });
}

struct Search {
  pid: u32,
  hidden: bool,
  /// Só para diagnóstico: (título, altura) das janelas visíveis do WebView2.
  seen: Option<Vec<(String, i32)>>,
}

fn hide_bar(browser_pid: u32) -> bool {
  let mut search = Search { pid: browser_pid, hidden: false, seen: None };
  unsafe {
    let _ = EnumWindows(Some(visit), LPARAM(&mut search as *mut Search as isize));
  }
  search.hidden
}

fn browser_windows(browser_pid: u32) -> Vec<(String, i32)> {
  let mut search = Search { pid: browser_pid, hidden: false, seen: Some(Vec::new()) };
  unsafe {
    let _ = EnumWindows(Some(visit), LPARAM(&mut search as *mut Search as isize));
  }
  search.seen.unwrap_or_default()
}

unsafe extern "system" fn visit(hwnd: HWND, lparam: LPARAM) -> BOOL {
  let search = unsafe { &mut *(lparam.0 as *mut Search) };
  let mut owner = 0u32;
  unsafe { GetWindowThreadProcessId(hwnd, Some(&mut owner)) };
  if owner != search.pid || !unsafe { IsWindowVisible(hwnd) }.as_bool() {
    return BOOL::from(true);
  }
  let (title, height) = describe(hwnd);
  if let Some(seen) = &mut search.seen {
    seen.push((title, height));
  } else if height <= MAX_BAR_HEIGHT && title_is_bar(&title) {
    let _ = unsafe { ShowWindowAsync(hwnd, SW_HIDE) };
    search.hidden = true;
  }
  BOOL::from(true)
}

fn describe(hwnd: HWND) -> (String, i32) {
  let mut rect = RECT::default();
  let height = match unsafe { GetWindowRect(hwnd, &mut rect) } {
    Ok(()) => rect.bottom - rect.top,
    Err(_) => i32::MAX,
  };
  let mut buf = [0u16; 512];
  let len = unsafe { GetWindowTextW(hwnd, &mut buf) } as usize;
  (String::from_utf16_lossy(&buf[..len]), height)
}

/// O título da barra cita o site que está capturando ("espia.espia.workers.dev está compartilhando…").
fn title_is_bar(title: &str) -> bool {
  title.contains(crate::links::HOST) || (cfg!(debug_assertions) && title.contains("127.0.0.1"))
}

#[cfg(test)]
mod tests {
  use super::title_is_bar;

  #[test]
  fn matches_only_our_site() {
    assert!(title_is_bar("espia.espia.workers.dev está compartilhando uma janela."));
    assert!(title_is_bar("espia.espia.workers.dev is sharing your screen."));
    assert!(!title_is_bar("Espia"));
    assert!(!title_is_bar("youtube.com está compartilhando sua tela."));
  }
}
