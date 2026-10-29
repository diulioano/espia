// O que dá pra transmitir (research R2, data-model.md "CaptureSource").
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSource {
  /// "w:<hwnd>" ou "m:<hmonitor>".
  pub id: String,
  pub kind: &'static str,
  pub title: String,
  pub app: String,
  pub icon: Option<String>,
  pub thumb: Option<String>,
  pub playing: bool,
  pub primary: bool,
}

/// Atributos de uma janela que decidem se ela aparece na grade.
#[derive(Debug, Clone, Default)]
pub struct WinAttrs {
  pub visible: bool,
  pub title: String,
  pub tool_window: bool,
  pub cloaked: bool,
  pub has_owner: bool,
  pub pid: u32,
  /// Tamanho na tela; janelas de 0 px são de fundo.
  pub width: i32,
  pub height: i32,
}

/// Filtro do seletor: visível, com título, sem ser ferramenta/escondida/filha de outra, fora do Espia.
pub fn is_listable(a: &WinAttrs, own_pid: u32) -> bool {
  a.visible
    && !a.title.trim().is_empty()
    && !a.tool_window
    && !a.cloaked
    && !a.has_owner
    && a.pid != own_pid
    && a.width > 1
    && a.height > 1
}

/// O que identifica uma fonte entre aberturas do seletor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceId {
  Window(isize),
  Monitor(isize),
}

pub fn parse_id(id: &str) -> Option<SourceId> {
  let (kind, n) = id.split_once(':')?;
  let n: isize = n.parse().ok()?;
  match kind {
    "w" => Some(SourceId::Window(n)),
    "m" => Some(SourceId::Monitor(n)),
    _ => None,
  }
}

#[cfg(windows)]
pub use imp::{list, monitor_rect, window_pid, window_alive};

#[cfg(windows)]
mod imp {
  use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
  };

  use windows::{
    Win32::{
      Foundation::{CloseHandle, HWND, LPARAM, RECT, TRUE, WPARAM},
      Graphics::{
        Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute},
        Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
      },
      Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW},
      System::Threading::{OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW},
      UI::{
        Shell::ExtractIconExW,
        WindowsAndMessaging::{
          DestroyIcon, EnumChildWindows, EnumWindows, GCLP_HICON, GW_OWNER, GWL_EXSTYLE, GetClassLongPtrW, GetClassNameW,
          GetWindow, GetWindowLongPtrW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, HICON, ICON_BIG, IsWindow,
          IsWindowVisible, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_GETICON, WS_EX_TOOLWINDOW,
        },
      },
    },
    core::{BOOL, HSTRING},
  };

  use super::{CaptureSource, WinAttrs, is_listable};
  use crate::capture::thumbs;

  const MONITORINFOF_PRIMARY: u32 = 1;

  #[derive(Clone)]
  struct AppInfo {
    name: String,
    icon: Option<String>,
  }

  /// Nome e ícone por executável: não mudam enquanto o programa está aberto.
  static APPS: LazyLock<Mutex<HashMap<String, AppInfo>>> = LazyLock::new(Default::default);

  fn text(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let n = unsafe { GetWindowTextW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
  }

  fn class(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
  }

  pub fn window_pid(hwnd: HWND) -> u32 {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    // App da loja: a moldura é do ApplicationFrameHost; o som e o nome são da CoreWindow filha.
    if class(hwnd) == "ApplicationFrameWindow" {
      let mut found = (pid, 0u32);
      unsafe extern "system" fn child(h: HWND, l: LPARAM) -> BOOL {
        let found = unsafe { &mut *(l.0 as *mut (u32, u32)) };
        let mut p = 0;
        unsafe { GetWindowThreadProcessId(h, Some(&mut p)) };
        if p != found.0 {
          found.1 = p;
          return false.into();
        }
        TRUE
      }
      unsafe {
        let _ = EnumChildWindows(Some(hwnd), Some(child), LPARAM(&mut found as *mut _ as isize));
      }
      if found.1 != 0 {
        return found.1;
      }
    }
    pid
  }

  pub fn window_alive(hwnd: HWND) -> bool {
    unsafe { IsWindow(Some(hwnd)).as_bool() }
  }

  fn attrs(hwnd: HWND) -> WinAttrs {
    unsafe {
      let mut cloaked = 0u32;
      let _ = DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as _, 4);
      let mut r = RECT::default();
      let _ = GetWindowRect(hwnd, &mut r);
      let mut pid = 0;
      GetWindowThreadProcessId(hwnd, Some(&mut pid));
      WinAttrs {
        visible: IsWindowVisible(hwnd).as_bool(),
        title: text(hwnd),
        tool_window: GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0 != 0,
        cloaked: cloaked != 0,
        has_owner: GetWindow(hwnd, GW_OWNER).is_ok_and(|o| !o.is_invalid()),
        pid,
        width: r.right - r.left,
        height: r.bottom - r.top,
      }
    }
  }

  fn exe_path(pid: u32) -> Option<String> {
    unsafe {
      let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
      let mut buf = [0u16; 1024];
      let mut len = buf.len() as u32;
      let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, windows::core::PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
      let _ = CloseHandle(h);
      ok.then(|| String::from_utf16_lossy(&buf[..len as usize]))
    }
  }

  /// FileDescription da versão do executável ("Visual Studio Code"), ou o nome do arquivo.
  fn description(path: &str) -> Option<String> {
    unsafe {
      let wide = HSTRING::from(path);
      let size = GetFileVersionInfoSizeW(&wide, None);
      if size == 0 {
        return None;
      }
      let mut data = vec![0u8; size as usize];
      GetFileVersionInfoW(&wide, None, size, data.as_mut_ptr() as _).ok()?;
      let mut ptr = std::ptr::null_mut();
      let mut len = 0u32;
      if !VerQueryValueW(data.as_ptr() as _, &HSTRING::from("\\VarFileInfo\\Translation"), &mut ptr, &mut len).as_bool() || len < 4 {
        return None;
      }
      let lang = *(ptr as *const u16);
      let cp = *(ptr as *const u16).add(1);
      let key = HSTRING::from(format!("\\StringFileInfo\\{lang:04x}{cp:04x}\\FileDescription"));
      if !VerQueryValueW(data.as_ptr() as _, &key, &mut ptr, &mut len).as_bool() || len == 0 {
        return None;
      }
      let s = std::slice::from_raw_parts(ptr as *const u16, len as usize);
      let s = String::from_utf16_lossy(s).trim_end_matches('\0').trim().to_string();
      (!s.is_empty()).then_some(s)
    }
  }

  fn window_icon(hwnd: HWND, exe: Option<&str>) -> Option<String> {
    unsafe {
      let mut res = 0usize;
      let _ = SendMessageTimeoutW(hwnd, WM_GETICON, WPARAM(ICON_BIG as usize), LPARAM(0), SMTO_ABORTIFHUNG, 50, Some(&mut res));
      if res == 0 {
        res = GetClassLongPtrW(hwnd, GCLP_HICON);
      }
      if res != 0 {
        return thumbs::icon_png(HICON(res as _));
      }
      let exe = exe?;
      let mut large = HICON::default();
      if ExtractIconExW(&HSTRING::from(exe), 0, Some(&mut large), None, 1) > 0 && !large.is_invalid() {
        let png = thumbs::icon_png(large);
        let _ = DestroyIcon(large);
        return png;
      }
      None
    }
  }

  fn app_info(hwnd: HWND, pid: u32) -> AppInfo {
    let path = exe_path(pid);
    let key = path.clone().unwrap_or_else(|| format!("pid:{pid}"));
    if let Some(c) = APPS.lock().unwrap().get(&key) {
      return c.clone();
    }
    let file = path.as_deref().and_then(|p| p.rsplit('\\').next()).unwrap_or("").to_string();
    let name = path.as_deref().and_then(description).unwrap_or(file);
    let icon = window_icon(hwnd, path.as_deref());
    // Sem ícone pode ser só a janela ainda carregando: não guarda.
    if icon.is_some() {
      APPS.lock().unwrap().insert(key, AppInfo { name: name.clone(), icon: icon.clone() });
    }
    AppInfo { name, icon }
  }

  pub struct Listed {
    pub source: CaptureSource,
    /// PID de quem toca o som da janela (0 em monitor).
    pub pid: u32,
  }

  pub fn monitor_rect(handle: isize) -> Option<(RECT, bool)> {
    let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    unsafe { GetMonitorInfoW(HMONITOR(handle as _), &mut info) }
      .as_bool()
      .then_some((info.rcMonitor, info.dwFlags & MONITORINFOF_PRIMARY != 0))
  }

  fn monitors() -> Vec<isize> {
    unsafe extern "system" fn each(m: HMONITOR, _: HDC, _: *mut RECT, l: LPARAM) -> BOOL {
      unsafe { (*(l.0 as *mut Vec<isize>)).push(m.0 as isize) };
      TRUE
    }
    let mut out: Vec<isize> = Vec::new();
    unsafe {
      let _ = EnumDisplayMonitors(None, None, Some(each), LPARAM(&mut out as *mut _ as isize));
    }
    out
  }

  /// Janelas (na ordem de cima para baixo da tela) e depois monitores ("Tela 1", "Tela 2"…).
  pub fn list(thumbs_too: bool) -> Vec<Listed> {
    let own = std::process::id();
    let mut hwnds: Vec<HWND> = Vec::new();
    unsafe extern "system" fn each(h: HWND, l: LPARAM) -> BOOL {
      unsafe { (*(l.0 as *mut Vec<HWND>)).push(h) };
      TRUE
    }
    unsafe {
      let _ = EnumWindows(Some(each), LPARAM(&mut hwnds as *mut _ as isize));
    }
    let mut out = Vec::new();
    for hwnd in hwnds {
      let a = attrs(hwnd);
      if !is_listable(&a, own) {
        continue;
      }
      let pid = window_pid(hwnd);
      if pid == own {
        continue;
      }
      let info = app_info(hwnd, pid);
      out.push(Listed {
        pid,
        source: CaptureSource {
          id: format!("w:{}", hwnd.0 as isize),
          kind: "window",
          title: a.title.trim().to_string(),
          app: info.name,
          icon: info.icon,
          thumb: if thumbs_too { thumbs::window_thumb(hwnd) } else { None },
          playing: false,
          primary: false,
        },
      });
    }
    let mut screens: Vec<(isize, RECT, bool)> = monitors().into_iter().filter_map(|m| monitor_rect(m).map(|(r, p)| (m, r, p))).collect();
    // Principal primeiro, depois da esquerda para a direita.
    screens.sort_by_key(|(_, r, p)| (!p, r.left, r.top));
    for (i, (m, r, primary)) in screens.into_iter().enumerate() {
      out.push(Listed {
        pid: 0,
        source: CaptureSource {
          id: format!("m:{m}"),
          kind: "monitor",
          title: format!("Tela {}", i + 1),
          app: if primary { "Principal".into() } else { String::new() },
          icon: None,
          thumb: if thumbs_too { thumbs::monitor_thumb(r) } else { None },
          playing: false,
          primary,
        },
      });
    }
    out
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ok() -> WinAttrs {
    WinAttrs { visible: true, title: "Hollow Knight".into(), pid: 10, width: 1280, height: 720, ..Default::default() }
  }

  #[test]
  fn janela_normal_aparece() {
    assert!(is_listable(&ok(), 1));
  }

  #[test]
  fn filtros() {
    assert!(!is_listable(&WinAttrs { visible: false, ..ok() }, 1));
    assert!(!is_listable(&WinAttrs { title: "  ".into(), ..ok() }, 1));
    assert!(!is_listable(&WinAttrs { tool_window: true, ..ok() }, 1));
    assert!(!is_listable(&WinAttrs { cloaked: true, ..ok() }, 1));
    assert!(!is_listable(&WinAttrs { has_owner: true, ..ok() }, 1));
    assert!(!is_listable(&WinAttrs { width: 0, ..ok() }, 1));
  }

  #[test]
  fn o_proprio_espia_nao_aparece() {
    assert!(!is_listable(&ok(), 10));
  }

  #[test]
  fn ids() {
    assert_eq!(parse_id("w:123"), Some(SourceId::Window(123)));
    assert_eq!(parse_id("m:-5"), Some(SourceId::Monitor(-5)));
    assert_eq!(parse_id("x:1"), None);
    assert_eq!(parse_id("w:abc"), None);
    assert_eq!(parse_id("w"), None);
  }
}
