//! Base do servidor e `Origin`, comuns à API HTTP e à sinalização.
//!
//! A base é sempre um valor passado às funções. `server_base()` devolve a base do processo: a oficial,
//! ou (só em build de depuração) a de `ESPIA_SERVER`.

use std::sync::Once;

/// Endereço oficial do servidor. Em release é o único.
pub const OFFICIAL_BASE: &str = "https://espia.diuliano.com";

/// Base em uso: em depuração, `ESPIA_SERVER` (se for um endereço http/https válido); em release,
/// sempre a oficial.
pub fn server_base() -> String {
    resolve_base(cfg!(debug_assertions), std::env::var("ESPIA_SERVER").ok().as_deref())
}

/// Parte pura de `server_base`: `depuracao` e o valor da variável entram como parâmetro.
pub fn resolve_base(depuracao: bool, variavel: Option<&str>) -> String {
    if depuracao
        && let Some(v) = variavel
    {
        let v = v.trim().trim_end_matches('/');
        if let Ok(url) = reqwest::Url::parse(v)
            && matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some()
        {
            return v.to_string();
        }
    }
    OFFICIAL_BASE.to_string()
}

/// Origem (`esquema://host[:porta]`) da base: é o que vai no cabeçalho `Origin`.
pub fn origin_of(base: &str) -> String {
    match reqwest::Url::parse(base) {
        Ok(url) => url.origin().ascii_serialization(),
        Err(_) => base.trim_end_matches('/').to_string(),
    }
}

/// `GET /ws/:code` na base: o esquema vira `ws://` ou `wss://` conforme a base.
pub fn ws_url(base: &str, code: &str) -> String {
    let origin = origin_of(base);
    let (esquema, resto) = match origin.strip_prefix("https://") {
        Some(r) => ("wss", r),
        None => ("ws", origin.strip_prefix("http://").unwrap_or(&origin)),
    };
    format!("{esquema}://{resto}/ws/{code}")
}

/// Instala o provedor criptográfico (ring) do rustls, uma vez por processo.
pub(crate) fn install_crypto() {
    static UMA_VEZ: Once = Once::new();
    UMA_VEZ.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_ignora_a_variavel() {
        assert_eq!(resolve_base(false, Some("http://127.0.0.1:8787")), OFFICIAL_BASE);
        assert_eq!(resolve_base(false, None), OFFICIAL_BASE);
    }

    #[test]
    fn depuracao_usa_a_variavel_valida() {
        assert_eq!(resolve_base(true, Some("http://127.0.0.1:8787/")), "http://127.0.0.1:8787");
        assert_eq!(resolve_base(true, Some("  https://exemplo.test ")), "https://exemplo.test");
        assert_eq!(resolve_base(true, None), OFFICIAL_BASE);
        assert_eq!(resolve_base(true, Some("")), OFFICIAL_BASE);
        assert_eq!(resolve_base(true, Some("lixo")), OFFICIAL_BASE);
        assert_eq!(resolve_base(true, Some("ftp://x.test")), OFFICIAL_BASE);
    }

    #[test]
    fn origem_e_esquema_do_websocket() {
        assert_eq!(origin_of("https://espia.diuliano.com"), "https://espia.diuliano.com");
        assert_eq!(origin_of("https://espia.diuliano.com/"), "https://espia.diuliano.com");
        assert_eq!(origin_of("http://127.0.0.1:8787"), "http://127.0.0.1:8787");
        assert_eq!(ws_url("https://espia.diuliano.com", "abc-def"), "wss://espia.diuliano.com/ws/abc-def");
        assert_eq!(ws_url("http://127.0.0.1:8787", "abc-def"), "ws://127.0.0.1:8787/ws/abc-def");
    }
}

/// Servidor HTTP falso (127.0.0.1, porta livre) para os testes da API.
#[cfg(test)]
pub(crate) mod testutil {
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[derive(Debug, Clone)]
    pub struct Req {
        pub method: String,
        pub path: String,
        pub headers: Vec<(String, String)>,
        pub body: String,
    }

    impl Req {
        pub fn header(&self, nome: &str) -> Option<&str> {
            self.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(nome)).map(|(_, v)| v.as_str())
        }
    }

    pub struct Resp {
        pub status: u16,
        pub body: String,
    }

    pub fn resp(status: u16, body: &str) -> Resp {
        Resp { status, body: body.to_string() }
    }

    pub struct Fake {
        pub base: String,
        pub reqs: Arc<Mutex<Vec<Req>>>,
    }

    impl Fake {
        pub fn ultimo(&self) -> Req {
            self.reqs.lock().unwrap().last().cloned().expect("nenhum pedido chegou")
        }
    }

    pub async fn serve(handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> Fake {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let reqs = Arc::new(Mutex::new(Vec::new()));
        let handler = Arc::new(handler);
        let guardados = reqs.clone();
        tokio::spawn(async move {
            loop {
                let Ok((mut s, _)) = listener.accept().await else { return };
                let handler = handler.clone();
                let guardados = guardados.clone();
                tokio::spawn(async move {
                    let mut buf = Vec::new();
                    let mut tmp = [0u8; 4096];
                    let fim_cabecalho = loop {
                        if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                            break p + 4;
                        }
                        match s.read(&mut tmp).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        }
                    };
                    let cabecalho = String::from_utf8_lossy(&buf[..fim_cabecalho]).to_string();
                    let mut linhas = cabecalho.split("\r\n");
                    let mut primeira = linhas.next().unwrap_or("").split(' ');
                    let method = primeira.next().unwrap_or("").to_string();
                    let path = primeira.next().unwrap_or("").to_string();
                    let headers: Vec<(String, String)> = linhas
                        .filter_map(|l| l.split_once(':').map(|(n, v)| (n.trim().to_string(), v.trim().to_string())))
                        .collect();
                    let tamanho = headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case("content-length"))
                        .and_then(|(_, v)| v.parse::<usize>().ok())
                        .unwrap_or(0);
                    while buf.len() < fim_cabecalho + tamanho {
                        match s.read(&mut tmp).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        }
                    }
                    let body = String::from_utf8_lossy(&buf[fim_cabecalho..]).to_string();
                    let req = Req { method, path, headers, body };
                    let r = handler(&req);
                    guardados.lock().unwrap().push(req);
                    let texto = format!(
                        "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        r.status,
                        r.body.len(),
                        r.body
                    );
                    let _ = s.write_all(texto.as_bytes()).await;
                    let _ = s.shutdown().await;
                });
            }
        });
        Fake { base, reqs }
    }
}
