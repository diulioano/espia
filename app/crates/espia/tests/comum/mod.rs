//! Apoio dos testes do `App`: servidor falso (HTTP e WebSocket na mesma porta, como o site), pasta temporária e
//! uma "janela" que só junta o que o `App` entrega.
#![allow(dead_code)] // cada arquivo de teste usa uma parte

use std::sync::{Arc, Mutex};
use std::time::Duration;

use espia::app::{self, App, Config, Entrada, Saida};
use espia_core::device::DeviceData;
use espia_core::protocol::ServerMessage;
use espia_net::signaling::SignalingConfig;
use espia_ui::{Screen, UiAction};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::{WebSocketStream, accept_hdr_async};

pub const LIMITE: Duration = Duration::from_secs(5);
pub const CHAVE: &str = "ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12";

// ---------------------------------------------------------------- servidor falso

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
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).expect("corpo JSON")
    }
}

/// Uma conexão WebSocket aceita pelo servidor falso.
pub struct Conn {
    pub origin: Option<String>,
    pub caminho: String,
    pub ws: WebSocketStream<TcpStream>,
}

impl Conn {
    /// Próxima mensagem de texto do cliente, como JSON.
    pub async fn json(&mut self) -> Value {
        loop {
            match tokio::time::timeout(LIMITE, self.ws.next()).await.expect("o app não mandou nada") {
                Some(Ok(Message::Text(t))) => return serde_json::from_str(t.as_str()).unwrap(),
                Some(Ok(_)) => {}
                outro => panic!("o socket acabou: {outro:?}"),
            }
        }
    }

    pub async fn mandar(&mut self, texto: impl Into<String>) {
        self.ws.send(Message::text(texto.into())).await.unwrap();
    }

    /// Fecha com um código, como o Worker faz (4409 sala cheia, 4000 trocado de lugar…).
    pub async fn fechar_com(&mut self, codigo: u16) {
        let quadro = CloseFrame { code: CloseCode::from(codigo), reason: "".into() };
        let _ = self.ws.close(Some(quadro)).await;
    }

    /// Espera o cliente fechar e devolve o código (4001 = "sair da sala").
    pub async fn codigo_de_fechamento(&mut self) -> Option<u16> {
        loop {
            match tokio::time::timeout(LIMITE, self.ws.next()).await.expect("o app não fechou") {
                Some(Ok(Message::Close(f))) => return f.map(|f| u16::from(f.code)),
                Some(Ok(_)) => {}
                _ => return None,
            }
        }
    }
}

pub struct Falso {
    pub base: String,
    pub reqs: Arc<Mutex<Vec<Req>>>,
    pub conns: UnboundedReceiver<Conn>,
}

impl Falso {
    pub fn pedidos(&self) -> Vec<Req> {
        self.reqs.lock().unwrap().clone()
    }
    pub fn pedidos_de(&self, metodo: &str, caminho: &str) -> Vec<Req> {
        self.pedidos().into_iter().filter(|r| r.method == metodo && r.path == caminho).collect()
    }
    pub async fn conexao(&mut self) -> Conn {
        tokio::time::timeout(LIMITE, self.conns.recv()).await.expect("o app não conectou").expect("servidor fechou")
    }
}

pub fn resp(status: u16, corpo: Value) -> (u16, String) {
    (status, corpo.to_string())
}

/// Sobe o servidor falso. `tratar` responde as chamadas HTTP; WebSocket vai para `Falso::conexao`.
pub async fn servidor(tratar: impl Fn(&Req) -> (u16, String) + Send + Sync + 'static) -> Falso {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let reqs = Arc::new(Mutex::new(Vec::new()));
    let tratar = Arc::new(tratar);
    let (tx, conns) = mpsc::unbounded_channel();
    let guardados = reqs.clone();
    tokio::spawn(async move {
        loop {
            let Ok((s, _)) = listener.accept().await else { return };
            let (tx, tratar, guardados) = (tx.clone(), tratar.clone(), guardados.clone());
            tokio::spawn(async move { atender(s, tx, tratar, guardados).await });
        }
    });
    Falso { base, reqs, conns }
}

async fn atender(
    mut s: TcpStream,
    tx: UnboundedSender<Conn>,
    tratar: Arc<impl Fn(&Req) -> (u16, String) + Send + Sync + 'static>,
    guardados: Arc<Mutex<Vec<Req>>>,
) {
    // Espia o começo do pedido, sem consumir, para saber se é WebSocket.
    let mut visto = vec![0u8; 8192];
    let cabecalho = loop {
        let Ok(n) = s.peek(&mut visto).await else { return };
        if n == 0 {
            return;
        }
        let texto = String::from_utf8_lossy(&visto[..n]).to_string();
        if texto.contains("\r\n\r\n") {
            break texto;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    };
    if cabecalho.to_ascii_lowercase().contains("upgrade: websocket") {
        let (mut origin, mut caminho) = (None, String::new());
        let cb = |req: &Request, res: Response| {
            origin = req.headers().get("origin").and_then(|v| v.to_str().ok()).map(String::from);
            caminho = req.uri().path().to_string();
            Ok(res)
        };
        if let Ok(ws) = accept_hdr_async(s, cb).await {
            let _ = tx.send(Conn { origin, caminho, ws });
        }
        return;
    }

    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    let fim = loop {
        if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break p + 4;
        }
        match s.read(&mut tmp).await {
            Ok(0) | Err(_) => return,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
        }
    };
    let cab = String::from_utf8_lossy(&buf[..fim]).to_string();
    let mut linhas = cab.split("\r\n");
    let mut primeira = linhas.next().unwrap_or("").split(' ');
    let method = primeira.next().unwrap_or("").to_string();
    let path = primeira.next().unwrap_or("").to_string();
    let headers: Vec<(String, String)> =
        linhas.filter_map(|l| l.split_once(':').map(|(n, v)| (n.trim().to_string(), v.trim().to_string()))).collect();
    let tamanho = headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);
    while buf.len() < fim + tamanho {
        match s.read(&mut tmp).await {
            Ok(0) | Err(_) => break,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
        }
    }
    let req = Req { method, path, headers, body: String::from_utf8_lossy(&buf[fim..]).to_string() };
    let (status, corpo) = tratar(&req);
    guardados.lock().unwrap().push(req);
    let texto = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{corpo}",
        corpo.len()
    );
    let _ = s.write_all(texto.as_bytes()).await;
    let _ = s.shutdown().await;
}

// ---------------------------------------------------------------- mensagens do servidor

pub fn pessoa(id: &str, apelido: &str, tone: &str) -> Value {
    json!({ "id": id, "nickname": apelido, "tone": tone, "live": null })
}

pub const EU: &str = "0123456789abcdef";
pub const BETO: &str = "fedcba9876543210";

/// O `welcome` do servidor: `eu` entra com `outros` já na sala; `sala` é `{"kind":"fixed","name":..}` ou `None`.
pub fn welcome(apelido: &str, outros: Vec<Value>, sala: Option<Value>) -> String {
    let mut m = json!({
        "t": "welcome", "mediaProtocol": 2,
        "mediaPolicy": { "kind": "direct-only", "reason": "unverified-budget" },
        "broadcastOrder": null, "you": pessoa(EU, apelido, "deep"), "peers": outros,
        "iceServers": [{ "urls": "stun:stun.exemplo.test:3478" }], "iceExpiresAt": 99,
        "resumeToken": "tok-1", "capacity": 10,
    });
    if let Some(s) = sala {
        m["room"] = s;
    }
    m.to_string()
}

/// Confere que o texto é mesmo uma mensagem que o app entende (o teste não inventa protocolo).
pub fn confere_protocolo(texto: &str) {
    serde_json::from_str::<ServerMessage>(texto).expect("mensagem do servidor inválida");
}

// ---------------------------------------------------------------- a "janela"

pub fn ajuste_rapido(c: &mut SignalingConfig) {
    c.ping_every = Duration::from_millis(60);
    c.pong_timeout = Duration::from_millis(150);
    c.backoff = vec![Duration::from_millis(30)];
}

pub struct Janela {
    pub entradas: UnboundedSender<Entrada>,
    saidas: UnboundedReceiver<Saida>,
    pub pasta: tempfile::TempDir,
    pub copias: Vec<String>,
    pub ultima: Option<Screen>,
}

/// Cria o `App` com pasta temporária (`device_json` já gravado nela, se houver) e começa no `link`.
pub fn abrir(base: &str, device_json: Option<&str>, link: Option<&str>) -> Janela {
    let pasta = tempfile::tempdir().unwrap();
    if let Some(texto) = device_json {
        std::fs::write(pasta.path().join("device.json"), texto).unwrap();
    }
    abrir_em(base, pasta, link)
}

pub fn abrir_em(base: &str, pasta: tempfile::TempDir, link: Option<&str>) -> Janela {
    let mut cfg = Config::nova(base, Some(pasta.path().to_path_buf()));
    cfg.ajuste_sinalizacao = ajuste_rapido;
    let (entradas, recebe) = app::canal();
    let (tx_saida, saidas) = mpsc::unbounded_channel();
    let app = App::novo(cfg, entradas.clone(), move |s| {
        let _ = tx_saida.send(s);
    });
    tokio::spawn(app::laco(app, recebe, link.map(String::from)));
    Janela { entradas, saidas, pasta, copias: Vec::new(), ultima: None }
}

impl Janela {
    pub fn acao(&self, a: UiAction) {
        self.entradas.send(Entrada::Ui(a)).unwrap();
    }

    /// Espera a primeira tela que satisfaz `serve` (as outras são descartadas).
    pub async fn tela(&mut self, serve: impl Fn(&Screen) -> bool) -> Screen {
        let espera = async {
            loop {
                match self.saidas.recv().await.expect("o App parou") {
                    Saida::Copiar(t) => self.copias.push(t),
                    Saida::Tela(t) => {
                        self.ultima = Some(t.clone());
                        if serve(&t) {
                            return t;
                        }
                    }
                }
            }
        };
        match tokio::time::timeout(LIMITE, espera).await {
            Ok(t) => t,
            Err(_) => panic!("a tela esperada não veio; a última foi {:#?}", self.ultima),
        }
    }

    /// Deixa o tempo passar, anotando as telas que chegarem (para provar que nada mudou).
    pub async fn esperar(&mut self, ms: u64) {
        let fim = tokio::time::Instant::now() + Duration::from_millis(ms);
        while let Ok(Some(s)) = tokio::time::timeout_at(fim, self.saidas.recv()).await {
            match s {
                Saida::Copiar(t) => self.copias.push(t),
                Saida::Tela(t) => self.ultima = Some(t),
            }
        }
    }

    /// Espera o `App` pedir uma cópia para a área de transferência.
    pub async fn copia(&mut self) -> String {
        if let Some(c) = self.copias.first() {
            return c.clone();
        }
        let espera = async {
            loop {
                match self.saidas.recv().await.expect("o App parou") {
                    Saida::Copiar(t) => return t,
                    Saida::Tela(t) => self.ultima = Some(t),
                }
            }
        };
        tokio::time::timeout(LIMITE, espera).await.expect("nenhuma cópia veio")
    }

    /// O `device.json` como está no disco agora.
    pub fn device(&self) -> DeviceData {
        let texto = std::fs::read_to_string(self.pasta.path().join("device.json")).unwrap_or_default();
        DeviceData::from_json(&texto, agora_ms())
    }
}

pub fn agora_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
}
