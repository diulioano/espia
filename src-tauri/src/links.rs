// Links de sala e quais endereços ficam dentro do app (contracts/deep-link.md).
use url::Url;

/// Onde o Espia está publicado. O app só abre esta origem; o resto vai pro navegador.
pub const ORIGIN: &str = "https://espia.espia.workers.dev";
pub const HOST: &str = "espia.espia.workers.dev";

/// Mesmo alfabeto de src/lib/room-id.ts (sem 0/1/i/l/o).
const ALPHABET: &str = "abcdefghjkmnpqrstuvwxyz23456789";

fn valid_code(code: &str) -> bool {
  let b = code.as_bytes();
  b.len() == 7
    && b[3] == b'-'
    && code.chars().enumerate().all(|(i, c)| i == 3 || ALPHABET.contains(c))
}

/// "/s/k7f-q2x" a partir de `espia://s/k7f-q2x` ou `https://espia.espia.workers.dev/s/k7f-q2x`.
/// Qualquer outra coisa (inclusive código inválido) dá None.
pub fn room_path(link: &str) -> Option<String> {
  let url = Url::parse(link.trim()).ok()?;
  let code = match url.scheme() {
    // espia://s/k7f-q2x → host "s", caminho "/k7f-q2x"
    "espia" if url.host_str() == Some("s") => url.path().trim_matches('/').to_owned(),
    "https" if url.host_str() == Some(HOST) => url.path().strip_prefix("/s/")?.trim_end_matches('/').to_owned(),
    _ => return None,
  };
  let code = code.to_ascii_lowercase();
  valid_code(&code).then(|| format!("/s/{code}"))
}

/// Primeiro argumento de linha de comando que é um link de sala (o Windows passa o link como argumento).
pub fn room_path_from_args<S: AsRef<str>>(args: &[S]) -> Option<String> {
  args.iter().skip(1).find_map(|a| room_path(a.as_ref()))
}

/// Navegações permitidas dentro da janela: o Espia publicado e a tela local de carregamento.
pub fn is_internal(url: &Url) -> bool {
  match url.scheme() {
    "https" => url.host_str() == Some(HOST),
    "http" | "tauri" => matches!(url.host_str(), Some("tauri.localhost") | Some("localhost")),
    "about" | "blob" | "data" => true,
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn espia_scheme() {
    assert_eq!(room_path("espia://s/k7f-q2x").as_deref(), Some("/s/k7f-q2x"));
    assert_eq!(room_path("espia://s/K7F-Q2X/").as_deref(), Some("/s/k7f-q2x"));
  }

  #[test]
  fn https_link() {
    assert_eq!(room_path("https://espia.espia.workers.dev/s/k7f-q2x").as_deref(), Some("/s/k7f-q2x"));
    assert_eq!(room_path("https://espia.espia.workers.dev/s/k7f-q2x?x=1").as_deref(), Some("/s/k7f-q2x"));
  }

  #[test]
  fn rejects_bad_links() {
    for bad in [
      "espia://qualquer-coisa",
      "espia://s/",
      "espia://s/k7f-q2",
      "espia://s/k7f-q2o", // "o" não está no alfabeto
      "espia://x/k7f-q2x",
      "https://evil.example/s/k7f-q2x",
      "http://espia.espia.workers.dev/s/k7f-q2x",
      "https://espia.espia.workers.dev/k7f-q2x",
      "k7f-q2x",
      "",
    ] {
      assert_eq!(room_path(bad), None, "{bad}");
    }
  }

  #[test]
  fn args_skip_exe() {
    let args = ["C:\\espia.exe", "--flag", "espia://s/abc-def"];
    assert_eq!(room_path_from_args(&args).as_deref(), Some("/s/abc-def"));
    assert_eq!(room_path_from_args(&["espia://s/abc-def"]), None);
  }

  #[test]
  fn internal_urls() {
    let ok = |s: &str| is_internal(&Url::parse(s).unwrap());
    assert!(ok("https://espia.espia.workers.dev/s/abc-def"));
    assert!(ok("http://tauri.localhost/index.html"));
    assert!(!ok("https://github.com/DiulianoVargas/espia"));
    assert!(!ok("https://espia.espia.workers.dev.evil.example/"));
    assert!(!ok("file:///C:/Windows"));
  }
}
