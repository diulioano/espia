// Espia Desktop: uma janela leve (WebView2) que abre o Espia publicado.
// Ver specs/003-espia-desktop/ (spec, plan, contracts/deep-link.md).
mod links;
#[cfg(windows)]
mod share_bar;

use tauri::{Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder, webview::NewWindowResponse};
use tauri_plugin_opener::OpenerExt;

/// Leva a janela já aberta para a sala. A página decide (pergunta se você está transmitindo);
/// se a página ainda não tem esse gancho, só navega.
fn open_room<R: tauri::Runtime>(window: &WebviewWindow<R>, path: &str) {
  // `path` já foi validado por links::room_path ("/s/xxx-xxx"), então é seguro no JS.
  let js = format!(
    "(window.espiaOpen || ((p) => location.assign({origin:?} + p)))({path:?})",
    origin = links::ORIGIN
  );
  let _ = window.eval(js);
}

fn focus<R: tauri::Runtime>(window: &WebviewWindow<R>) {
  let _ = window.unminimize();
  let _ = window.show();
  let _ = window.set_focus();
}

/// Primeira tela: a página local de carregamento (src-tauri/www), que confere a conexão e
/// segue para o site. Em builds de debug, ESPIA_START troca o endereço inicial (testes).
fn start_url(room: Option<&str>) -> WebviewUrl {
  #[cfg(debug_assertions)]
  if let Some(url) = std::env::var("ESPIA_START").ok().and_then(|u| u.parse().ok()) {
    return WebviewUrl::External(url);
  }
  match room {
    Some(path) => WebviewUrl::App(format!("index.html?to={path}").into()),
    None => WebviewUrl::App("index.html".into()),
  }
}

fn allowed(url: &Url) -> bool {
  links::is_internal(url) || (cfg!(debug_assertions) && url.host_str() == Some("127.0.0.1"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    // Precisa ser o primeiro plugin: a segunda abertura (atalho ou link espia://) cai aqui.
    .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
      if let Some(window) = app.get_webview_window("main") {
        focus(&window);
        if let Some(path) = links::room_path_from_args(&args) {
          open_room(&window, &path);
        }
      }
    }))
    .plugin(tauri_plugin_deep_link::init())
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      // O instalador registra espia://; rodando sem instalar (dev), registramos aqui.
      #[cfg(debug_assertions)]
      {
        use tauri_plugin_deep_link::DeepLinkExt;
        let _ = app.deep_link().register_all();
      }

      let args: Vec<String> = std::env::args().collect();
      let room = links::room_path_from_args(&args);

      let opener = app.handle().clone();
      let popups = app.handle().clone();
      #[allow(unused_mut)]
      let mut builder = WebviewWindowBuilder::new(app, "main", start_url(room.as_deref()))
        .title("Espia")
        .theme(Some(tauri::Theme::Dark))
        .inner_size(1280.0, 800.0)
        .min_inner_size(720.0, 480.0)
        .center()
        // Fora do Espia, abre no navegador padrão (FR-007).
        .on_navigation(move |url| {
          if allowed(url) {
            return true;
          }
          if matches!(url.scheme(), "https" | "http") {
            let _ = opener.opener().open_url(url.as_str(), None::<&str>);
          }
          false
        })
        .on_new_window(move |url, _features| {
          if matches!(url.scheme(), "https" | "http") {
            let _ = popups.opener().open_url(url.as_str(), None::<&str>);
          }
          NewWindowResponse::Deny
        });

      #[cfg(debug_assertions)]
      if let Ok(extra) = std::env::var("ESPIA_BROWSER_ARGS") {
        // Mantém os padrões do wry e acrescenta (ex.: --auto-select-desktop-capture-source=…).
        builder = builder.additional_browser_args(&format!(
          "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection {extra}"
        ));
      }

      #[cfg(debug_assertions)]
      if let Ok(title) = std::env::var("ESPIA_TITLE") {
        builder = builder.title(title);
      }

      let window = builder.build()?;
      #[cfg(windows)]
      if !(cfg!(debug_assertions) && std::env::var("ESPIA_KEEP_BAR").is_ok()) {
        share_bar::install(&window);
      }
      #[cfg(debug_assertions)]
      if std::env::var("ESPIA_START").is_ok() {
        focus(&window);
      }
      let _ = window;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("erro ao iniciar o Espia");
}
