//! Sinalização por WebSocket (espelha `src/lib/signaling.ts`, `contracts/sinalizacao-nativa.md`):
//! entra, mantém viva com o PING exato, reconecta e retoma a mesma identidade com `resume`.
//!
//! Não sabe nada de tela. Quem usa fala por canais do tokio: recebe `SignalingEvent` e manda
//! `SignalingCommand`. Só STUN: `turn-refresh` nunca sai e `ice-servers` chega filtrado.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::{Instant, MissedTickBehavior, interval_at, sleep, sleep_until, timeout};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use espia_core::media_policy::{MEDIA_PROTOCOL, MediaPolicy, direct_ice_servers};
use espia_core::protocol::{ClientMessage, PING, ServerMessage, close};

use crate::server::{install_crypto, origin_of, ws_url};

/// Intervalo do PING (`PING_EVERY_MS`).
pub const PING_EVERY: Duration = Duration::from_secs(20);
/// Sem `pong` neste prazo o socket é abandonado e a conexão refeita (`PONG_TIMEOUT_MS`).
pub const PONG_TIMEOUT: Duration = Duration::from_secs(8);
/// Espera crescente entre tentativas: 1, 2, 4, 8 s e depois 10 s (`BACKOFF_MS`).
pub const BACKOFF: [Duration; 5] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
    Duration::from_secs(8),
    Duration::from_secs(10),
];
/// Prazo para abrir o socket (o site deixa ao navegador; aqui evita ficar pendurado).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalingStatus {
    Connecting,
    Open,
    Reconnecting,
}

/// Motivos que encerram a sala de vez (sem reconectar). Mesmos do site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalingEnd {
    NotFound,
    Full,
    Replaced,
    Kicked,
    Deleted,
    Removed,
    Update,
}

/// Identidade guardada para retomar a sala.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomSession {
    pub peer_id: String,
    pub resume_token: String,
}

/// O que chega a quem usa, na ordem em que acontece.
#[derive(Debug, Clone, PartialEq)]
pub enum SignalingEvent {
    /// Mensagem do servidor (menos `pong`; `welcome` e `ice-servers` já só com STUN).
    Message(ServerMessage),
    Status(SignalingStatus),
    /// Sessão para guardar (`Some`) ou apagar (`None`).
    Session(Option<RoomSession>),
    /// Voltou depois de uma queda (`welcome` depois de `Reconnecting`).
    Back,
    /// A sala acabou; nenhum evento depois deste.
    End(SignalingEnd),
}

/// O que quem usa pode pedir.
#[derive(Debug, Clone, PartialEq)]
pub enum SignalingCommand {
    /// Envia se estiver conectado; mensagens perdidas na queda não fazem falta (o estado volta no
    /// `welcome`). `join`, `resume` e `turn-refresh` são ignorados: o primeiro envio é nosso e só há STUN.
    Send(ClientMessage),
    /// "Sair da sala": fecha com 4001 (os outros recebem `peer-left` na hora) e apaga a sessão.
    Leave,
    /// "Tentar de novo" / internet voltou: reconecta já, sem esperar a espera crescente.
    RetryNow,
}

#[derive(Debug, Clone)]
pub struct SignalingConfig {
    /// Base do servidor (`server::server_base()`; o servidor local nos testes).
    pub base: String,
    pub code: String,
    pub nickname: String,
    pub session: Option<RoomSession>,
    pub ping_every: Duration,
    pub pong_timeout: Duration,
    pub backoff: Vec<Duration>,
}

impl SignalingConfig {
    pub fn new(base: &str, code: &str, nickname: &str, session: Option<RoomSession>) -> Self {
        Self {
            base: base.to_string(),
            code: code.to_string(),
            nickname: nickname.to_string(),
            session,
            ping_every: PING_EVERY,
            pong_timeout: PONG_TIMEOUT,
            backoff: BACKOFF.to_vec(),
        }
    }
}

/// Ponta de quem usa: manda comandos e lê eventos.
pub struct Signaling {
    pub commands: UnboundedSender<SignalingCommand>,
    pub events: UnboundedReceiver<SignalingEvent>,
}

impl Signaling {
    /// Começa a conectar já. Precisa de um runtime do tokio.
    pub fn start(config: SignalingConfig) -> Self {
        install_crypto();
        let (commands, cmd_rx) = mpsc::unbounded_channel();
        let (ev_tx, events) = mpsc::unbounded_channel();
        tokio::spawn(Tarefa { cfg: config.clone(), cmds: cmd_rx, ev: ev_tx, session: config.session, attempt: 0, was_down: false }.run());
        Self { commands, events }
    }

    pub fn send(&self, msg: ClientMessage) {
        let _ = self.commands.send(SignalingCommand::Send(msg));
    }

    pub fn leave(&self) {
        let _ = self.commands.send(SignalingCommand::Leave);
    }

    pub fn retry_now(&self) {
        let _ = self.commands.send(SignalingCommand::RetryNow);
    }
}

/// Fechamento sem volta, como `Signaling.dropped` do site.
fn end_for_close(code: u16) -> Option<SignalingEnd> {
    Some(match code {
        close::REPLACED => SignalingEnd::Replaced,
        close::UPDATE => SignalingEnd::Update,
        close::POLICY => SignalingEnd::Kicked,
        close::NOT_FOUND => SignalingEnd::NotFound,
        close::FULL => SignalingEnd::Full,
        close::DELETED => SignalingEnd::Deleted,
        close::REMOVED => SignalingEnd::Removed,
        _ => return None,
    })
}

type Socket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

enum Saida {
    /// A conexão caiu (código de fechamento; 1006 para queda sem código).
    Caiu(u16),
    /// A sala acabou por mensagem do servidor.
    Fim(SignalingEnd),
    /// Quem usa pediu para sair (ou largou o canal).
    Saiu,
}

enum Conexao {
    Pronta(Box<Socket>),
    Falhou,
    Saiu,
}

enum Espera {
    Passou,
    Agora,
    Saiu,
}

enum Tratada {
    Segue,
    Pong,
    Entrar,
    Fim(SignalingEnd),
}

struct Tarefa {
    cfg: SignalingConfig,
    cmds: UnboundedReceiver<SignalingCommand>,
    ev: UnboundedSender<SignalingEvent>,
    session: Option<RoomSession>,
    attempt: usize,
    /// Caiu desde o último welcome: o próximo avisa `Back`.
    was_down: bool,
}

impl Tarefa {
    fn emit(&self, e: SignalingEvent) {
        let _ = self.ev.send(e);
    }

    async fn run(mut self) {
        loop {
            if self.ev.is_closed() {
                return;
            }
            let estado = if self.attempt == 0 && self.session.is_none() { SignalingStatus::Connecting } else { SignalingStatus::Reconnecting };
            self.emit(SignalingEvent::Status(estado));
            let saida = match self.conectar().await {
                Conexao::Pronta(ws) => self.conexao(*ws).await,
                Conexao::Falhou => Saida::Caiu(1006),
                Conexao::Saiu => Saida::Saiu,
            };
            match saida {
                Saida::Saiu => {
                    self.session = None;
                    self.emit(SignalingEvent::Session(None));
                    return;
                }
                Saida::Fim(motivo) => return self.terminar(motivo),
                Saida::Caiu(codigo) => {
                    if let Some(motivo) = end_for_close(codigo) {
                        return self.terminar(motivo);
                    }
                    self.was_down = true;
                    self.emit(SignalingEvent::Status(SignalingStatus::Reconnecting));
                    let i = self.attempt.min(self.cfg.backoff.len().saturating_sub(1));
                    let atraso = self.cfg.backoff.get(i).copied().unwrap_or(Duration::from_secs(10));
                    self.attempt += 1;
                    match self.esperar(atraso).await {
                        Espera::Passou => {}
                        Espera::Agora => self.attempt = 0,
                        Espera::Saiu => {
                            self.session = None;
                            self.emit(SignalingEvent::Session(None));
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Fim de sala: apaga a sessão (menos em `replaced`) e avisa.
    fn terminar(&mut self, motivo: SignalingEnd) {
        if motivo != SignalingEnd::Replaced {
            self.session = None;
            self.emit(SignalingEvent::Session(None));
        }
        self.emit(SignalingEvent::End(motivo));
    }

    /// Espera o atraso da reconexão; `RetryNow` encurta, `Leave` encerra, o resto é descartado.
    async fn esperar(&mut self, atraso: Duration) -> Espera {
        let fim = sleep(atraso);
        tokio::pin!(fim);
        loop {
            tokio::select! {
                _ = &mut fim => return Espera::Passou,
                _ = self.ev.closed() => return Espera::Saiu,
                cmd = self.cmds.recv() => match cmd {
                    None | Some(SignalingCommand::Leave) => return Espera::Saiu,
                    Some(SignalingCommand::RetryNow) => return Espera::Agora,
                    Some(SignalingCommand::Send(_)) => {}
                },
            }
        }
    }

    async fn conectar(&mut self) -> Conexao {
        let url = ws_url(&self.cfg.base, &self.cfg.code);
        let origem = origin_of(&self.cfg.base);
        let tentativa = async {
            let mut req = url.into_client_request().ok()?;
            req.headers_mut().insert("Origin", HeaderValue::from_str(&origem).ok()?);
            let (ws, _) = connect_async(req).await.ok()?;
            Some(ws)
        };
        let tentativa = timeout(CONNECT_TIMEOUT, tentativa);
        tokio::pin!(tentativa);
        loop {
            tokio::select! {
                r = &mut tentativa => return match r {
                    Ok(Some(ws)) => Conexao::Pronta(Box::new(ws)),
                    _ => Conexao::Falhou,
                },
                _ = self.ev.closed() => return Conexao::Saiu,
                cmd = self.cmds.recv() => match cmd {
                    None | Some(SignalingCommand::Leave) => return Conexao::Saiu,
                    // Ainda sem conexão: mensagens perdidas não fazem falta; já está tentando.
                    Some(_) => {}
                },
            }
        }
    }

    fn primeira_mensagem(&self) -> ClientMessage {
        match &self.session {
            Some(s) => ClientMessage::Resume { peer_id: s.peer_id.clone(), resume_token: s.resume_token.clone(), media_protocol: MEDIA_PROTOCOL },
            None => ClientMessage::Join { nickname: self.cfg.nickname.clone(), media_protocol: MEDIA_PROTOCOL },
        }
    }

    async fn conexao(&mut self, mut ws: Socket) -> Saida {
        if enviar(&mut ws, &self.primeira_mensagem()).await.is_err() {
            return Saida::Caiu(1006);
        }
        let mut ping = interval_at(Instant::now() + self.cfg.ping_every, self.cfg.ping_every);
        ping.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut prazo_pong: Option<Instant> = None;
        loop {
            let limite = prazo_pong.unwrap_or_else(|| Instant::now() + Duration::from_secs(3600));
            tokio::select! {
                quadro = ws.next() => match quadro {
                    None | Some(Err(_)) => return Saida::Caiu(1006),
                    Some(Ok(Message::Close(f))) => return Saida::Caiu(f.map(|f| u16::from(f.code)).unwrap_or(1005)),
                    Some(Ok(Message::Text(t))) => match self.tratar(t.as_str()) {
                        Tratada::Segue => {}
                        Tratada::Pong => prazo_pong = None,
                        Tratada::Entrar => {
                            let join = ClientMessage::Join { nickname: self.cfg.nickname.clone(), media_protocol: MEDIA_PROTOCOL };
                            if enviar(&mut ws, &join).await.is_err() {
                                return Saida::Caiu(1006);
                            }
                        }
                        Tratada::Fim(motivo) => {
                            let _ = timeout(Duration::from_millis(500), ws.close(None)).await;
                            return Saida::Fim(motivo);
                        }
                    },
                    Some(Ok(_)) => {}
                },
                _ = ping.tick() => {
                    if ws.send(Message::text(PING)).await.is_err() {
                        return Saida::Caiu(1006);
                    }
                    // Mantém o prazo do primeiro ping sem resposta (o site reinicia; com 20 s e 8 s dá no mesmo).
                    prazo_pong.get_or_insert(Instant::now() + self.cfg.pong_timeout);
                }
                // Sem pong no prazo: abandona o socket (pode estar morto sem saber) e reconecta já.
                _ = sleep_until(limite), if prazo_pong.is_some() => return Saida::Caiu(1006),
                _ = self.ev.closed() => return self.sair(ws).await,
                cmd = self.cmds.recv() => match cmd {
                    None | Some(SignalingCommand::Leave) => return self.sair(ws).await,
                    Some(SignalingCommand::RetryNow) => {}
                    Some(SignalingCommand::Send(ClientMessage::TurnRefresh | ClientMessage::Join { .. } | ClientMessage::Resume { .. })) => {}
                    Some(SignalingCommand::Send(msg)) => {
                        if enviar(&mut ws, &msg).await.is_err() {
                            return Saida::Caiu(1006);
                        }
                    }
                },
            }
        }
    }

    /// "Sair da sala": fecha com 4001.
    async fn sair(&mut self, mut ws: Socket) -> Saida {
        let quadro = CloseFrame { code: CloseCode::from(close::LEAVE), reason: "saiu".into() };
        let _ = timeout(Duration::from_millis(500), ws.close(Some(quadro))).await;
        Saida::Saiu
    }

    /// Uma mensagem de texto do servidor (`Signaling.receive` do site).
    fn tratar(&mut self, texto: &str) -> Tratada {
        let msg = match serde_json::from_str::<ServerMessage>(texto) {
            Ok(m) => m,
            Err(_) => {
                // Welcome que não cabe no protocolo atual é servidor antigo: pede atualização.
                let e_welcome = serde_json::from_str::<serde_json::Value>(texto).ok().is_some_and(|v| v["t"] == "welcome");
                return if e_welcome { Tratada::Fim(SignalingEnd::Update) } else { Tratada::Segue };
            }
        };
        match msg {
            ServerMessage::Pong => Tratada::Pong,
            ServerMessage::Welcome { media_protocol, media_policy, you, resume_token, mut ice_servers, peers, broadcast_order, ice_expires_at, capacity, room } => {
                if media_protocol != MEDIA_PROTOCOL || !matches!(media_policy, MediaPolicy::DirectOnly { .. }) {
                    return Tratada::Fim(SignalingEnd::Update);
                }
                self.attempt = 0;
                let sessao = RoomSession { peer_id: you.id.clone(), resume_token: resume_token.clone() };
                self.session = Some(sessao.clone());
                self.emit(SignalingEvent::Session(Some(sessao)));
                self.emit(SignalingEvent::Status(SignalingStatus::Open));
                if self.was_down {
                    self.emit(SignalingEvent::Back);
                }
                self.was_down = false;
                ice_servers = direct_ice_servers(&ice_servers);
                self.emit(SignalingEvent::Message(ServerMessage::Welcome {
                    media_protocol,
                    media_policy,
                    broadcast_order,
                    you,
                    peers,
                    ice_servers,
                    ice_expires_at,
                    resume_token,
                    capacity,
                    room,
                }));
                Tratada::Segue
            }
            ServerMessage::ResumeFailed => {
                // Passou da tolerância: entra de novo como pessoa nova, na mesma conexão.
                self.session = None;
                self.emit(SignalingEvent::Session(None));
                Tratada::Entrar
            }
            ServerMessage::RoomNotFound => Tratada::Fim(SignalingEnd::NotFound),
            ServerMessage::RoomFull { .. } => Tratada::Fim(SignalingEnd::Full),
            ServerMessage::RoomDeleted => Tratada::Fim(SignalingEnd::Deleted),
            ServerMessage::Removed => Tratada::Fim(SignalingEnd::Removed),
            ServerMessage::IceServers { ice_servers, ice_expires_at } => {
                self.emit(SignalingEvent::Message(ServerMessage::IceServers { ice_servers: direct_ice_servers(&ice_servers), ice_expires_at }));
                Tratada::Segue
            }
            outra => {
                self.emit(SignalingEvent::Message(outra));
                Tratada::Segue
            }
        }
    }
}

async fn enviar(ws: &mut Socket, msg: &ClientMessage) -> Result<(), ()> {
    let texto = serde_json::to_string(msg).map_err(|_| ())?;
    ws.send(Message::text(texto)).await.map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_hdr_async;
    use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

    const COD: &str = "aaa-aaa";

    /// Conexão aceita pelo servidor falso: o que o cliente mandou no pedido e o socket para o teste dirigir.
    struct Conn {
        origin: Option<String>,
        caminho: String,
        ws: WebSocketStream<tokio::net::TcpStream>,
    }

    struct Falso {
        base: String,
        conns: UnboundedReceiver<Conn>,
    }

    async fn servidor() -> Falso {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (tx, conns) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            loop {
                let Ok((s, _)) = listener.accept().await else { return };
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut origin = None;
                    let mut caminho = String::new();
                    let cb = |req: &Request, res: Response| {
                        origin = req.headers().get("origin").and_then(|v| v.to_str().ok()).map(String::from);
                        caminho = req.uri().path().to_string();
                        Ok(res)
                    };
                    if let Ok(ws) = accept_hdr_async(s, cb).await {
                        let _ = tx.send(Conn { origin, caminho, ws });
                    }
                });
            }
        });
        Falso { base, conns }
    }

    fn config(base: &str, sessao: Option<RoomSession>) -> SignalingConfig {
        let mut c = SignalingConfig::new(base, COD, "Ana", sessao);
        c.ping_every = Duration::from_millis(60);
        c.pong_timeout = Duration::from_millis(150);
        c.backoff = vec![Duration::from_millis(30)];
        c
    }

    fn sessao() -> RoomSession {
        RoomSession { peer_id: "0123456789abcdef".into(), resume_token: "tok".into() }
    }

    const LIMITE: Duration = Duration::from_secs(5);

    async fn proxima(f: &mut Falso) -> Conn {
        timeout(LIMITE, f.conns.recv()).await.expect("conexão não chegou").expect("servidor fechou")
    }

    async fn texto(ws: &mut WebSocketStream<tokio::net::TcpStream>) -> String {
        loop {
            match timeout(LIMITE, ws.next()).await.expect("sem mensagem") {
                Some(Ok(Message::Text(t))) => return t.as_str().to_string(),
                Some(Ok(_)) => {}
                outro => panic!("socket acabou: {outro:?}"),
            }
        }
    }

    async fn json_de(ws: &mut WebSocketStream<tokio::net::TcpStream>) -> Value {
        serde_json::from_str(&texto(ws).await).unwrap()
    }

    fn welcome(ice: Value) -> String {
        json!({"t":"welcome","mediaProtocol":2,"mediaPolicy":{"kind":"direct-only","reason":"unverified-budget"},
            "broadcastOrder":null,"you":{"id":"0123456789abcdef","nickname":"Ana","tone":"deep","live":null},
            "peers":[],"iceServers":ice,"iceExpiresAt":99,"resumeToken":"tok","capacity":10})
        .to_string()
    }

    async fn evento(s: &mut Signaling) -> SignalingEvent {
        timeout(LIMITE, s.events.recv()).await.expect("sem evento").expect("canal fechou")
    }

    /// Consome eventos até o primeiro que casa; devolve tudo o que veio (o último é o que casou).
    async fn ate(s: &mut Signaling, casa: impl Fn(&SignalingEvent) -> bool) -> Vec<SignalingEvent> {
        let mut vistos = Vec::new();
        loop {
            let e = evento(s).await;
            let ok = casa(&e);
            vistos.push(e);
            if ok {
                return vistos;
            }
        }
    }

    async fn sem_conexao_nova(f: &mut Falso) {
        assert!(timeout(Duration::from_millis(400), f.conns.recv()).await.is_err(), "reconectou quando não devia");
    }

    fn fechar_com(codigo: u16) -> Message {
        Message::Close(Some(CloseFrame { code: CloseCode::from(codigo), reason: "".into() }))
    }

    #[tokio::test]
    async fn origin_caminho_e_join_com_protocolo_2() {
        let mut f = servidor().await;
        let _s = Signaling::start(config(&f.base, None));
        let mut c = proxima(&mut f).await;
        assert_eq!(c.origin.as_deref(), Some(f.base.as_str()));
        assert_eq!(c.caminho, format!("/ws/{COD}"));
        assert_eq!(texto(&mut c.ws).await, r#"{"t":"join","nickname":"Ana","mediaProtocol":2}"#);
    }

    #[tokio::test]
    async fn com_sessao_a_primeira_mensagem_e_resume() {
        let mut f = servidor().await;
        let _s = Signaling::start(config(&f.base, Some(sessao())));
        let mut c = proxima(&mut f).await;
        assert_eq!(
            json_de(&mut c.ws).await,
            json!({"t":"resume","peerId":"0123456789abcdef","resumeToken":"tok","mediaProtocol":2})
        );
    }

    #[tokio::test]
    async fn welcome_guarda_sessao_abre_e_filtra_so_stun() {
        let mut f = servidor().await;
        let mut s = Signaling::start(config(&f.base, None));
        let mut c = proxima(&mut f).await;
        let _ = texto(&mut c.ws).await;
        let ice = json!([{"urls":["stun:a.test:3478","turn:b.test"],"username":"u","credential":"c"},{"urls":"turn:c.test"},{"urls":"STUNS:d.test"}]);
        c.ws.send(Message::text(welcome(ice))).await.unwrap();
        let vistos = ate(&mut s, |e| matches!(e, SignalingEvent::Message(_))).await;
        assert_eq!(vistos[0], SignalingEvent::Status(SignalingStatus::Connecting));
        assert_eq!(vistos[1], SignalingEvent::Session(Some(sessao())));
        assert_eq!(vistos[2], SignalingEvent::Status(SignalingStatus::Open));
        match vistos.last().unwrap() {
            SignalingEvent::Message(ServerMessage::Welcome { ice_servers, you, .. }) => {
                let urls: Vec<Vec<String>> = ice_servers.iter().map(|i| i.urls.to_vec()).collect();
                assert_eq!(urls, vec![vec!["stun:a.test:3478".to_string()], vec!["STUNS:d.test".to_string()]]);
                assert!(ice_servers.iter().all(|i| i.username.is_none() && i.credential.is_none()));
                assert_eq!(you.id, "0123456789abcdef");
            }
            o => panic!("esperava welcome: {o:?}"),
        }
        // ice-servers também chega só com STUN
        let msg = json!({"t":"ice-servers","iceServers":[{"urls":"turn:x"},{"urls":"stun:y"}],"iceExpiresAt":5});
        c.ws.send(Message::text(msg.to_string())).await.unwrap();
        match evento(&mut s).await {
            SignalingEvent::Message(ServerMessage::IceServers { ice_servers, ice_expires_at }) => {
                assert_eq!(ice_expires_at, 5);
                assert_eq!(ice_servers.len(), 1);
                assert_eq!(ice_servers[0].urls.to_vec(), vec!["stun:y"]);
            }
            o => panic!("esperava ice-servers: {o:?}"),
        }
    }

    #[tokio::test]
    async fn ping_exato_e_pong_nao_derruba() {
        let mut f = servidor().await;
        let _s = Signaling::start(config(&f.base, None));
        let mut c = proxima(&mut f).await;
        let _ = texto(&mut c.ws).await;
        for _ in 0..4 {
            assert_eq!(texto(&mut c.ws).await, r#"{"t":"ping"}"#);
            c.ws.send(Message::text(espia_core::protocol::PONG)).await.unwrap();
        }
        // respondendo sempre, não houve segunda conexão (janela menor que ping + prazo do pong)
        assert!(timeout(Duration::from_millis(100), f.conns.recv()).await.is_err());
    }

    #[tokio::test]
    async fn sem_pong_abandona_e_retoma_com_resume() {
        let mut f = servidor().await;
        let mut s = Signaling::start(config(&f.base, None));
        let mut c = proxima(&mut f).await;
        let _ = texto(&mut c.ws).await;
        c.ws.send(Message::text(welcome(json!([])))).await.unwrap();
        let _ = ate(&mut s, |e| matches!(e, SignalingEvent::Message(_))).await;
        assert_eq!(texto(&mut c.ws).await, r#"{"t":"ping"}"#);
        // sem pong: o cliente reconecta sozinho
        let mut c2 = proxima(&mut f).await;
        assert_eq!(c2.origin.as_deref(), Some(f.base.as_str()));
        assert_eq!(
            json_de(&mut c2.ws).await,
            json!({"t":"resume","peerId":"0123456789abcdef","resumeToken":"tok","mediaProtocol":2})
        );
        c2.ws.send(Message::text(welcome(json!([])))).await.unwrap();
        let vistos = ate(&mut s, |e| matches!(e, SignalingEvent::Message(_))).await;
        assert!(vistos.contains(&SignalingEvent::Status(SignalingStatus::Reconnecting)));
        assert!(vistos.contains(&SignalingEvent::Back));
        assert!(vistos.contains(&SignalingEvent::Status(SignalingStatus::Open)));
    }

    #[tokio::test]
    async fn queda_sem_codigo_reconecta() {
        let mut f = servidor().await;
        let mut s = Signaling::start(config(&f.base, None));
        let mut c = proxima(&mut f).await;
        let _ = texto(&mut c.ws).await;
        c.ws.send(Message::text(welcome(json!([])))).await.unwrap();
        let _ = ate(&mut s, |e| matches!(e, SignalingEvent::Message(_))).await;
        drop(c); // queda abrupta (1006)
        let mut c2 = proxima(&mut f).await;
        assert_eq!(json_de(&mut c2.ws).await["t"], "resume");
    }

    #[tokio::test]
    async fn fechamento_fora_da_tabela_reconecta() {
        let mut f = servidor().await;
        let _s = Signaling::start(config(&f.base, Some(sessao())));
        let mut c = proxima(&mut f).await;
        let _ = texto(&mut c.ws).await;
        c.ws.send(fechar_com(1011)).await.unwrap();
        let mut c2 = proxima(&mut f).await;
        assert_eq!(json_de(&mut c2.ws).await["t"], "resume");
    }

    #[tokio::test]
    async fn cada_codigo_de_fechamento_vira_o_fim_do_site() {
        let tabela = [
            (4426u16, SignalingEnd::Update),
            (1008, SignalingEnd::Kicked),
            (4404, SignalingEnd::NotFound),
            (4409, SignalingEnd::Full),
            (4000, SignalingEnd::Replaced),
            (4410, SignalingEnd::Deleted),
            (4403, SignalingEnd::Removed),
        ];
        for (codigo, esperado) in tabela {
            let mut f = servidor().await;
            let mut s = Signaling::start(config(&f.base, Some(sessao())));
            let mut c = proxima(&mut f).await;
            let _ = texto(&mut c.ws).await;
            c.ws.send(fechar_com(codigo)).await.unwrap();
            let vistos = ate(&mut s, |e| matches!(e, SignalingEvent::End(_))).await;
            assert_eq!(vistos.last().unwrap(), &SignalingEvent::End(esperado), "código {codigo}");
            // só `replaced` guarda a sessão; os outros a apagam
            let apagou = vistos.contains(&SignalingEvent::Session(None));
            assert_eq!(apagou, esperado != SignalingEnd::Replaced, "sessão no código {codigo}");
            sem_conexao_nova(&mut f).await;
        }
    }

    #[tokio::test]
    async fn mensagens_de_fim_de_sala_nao_reconectam() {
        let tabela = [
            (r#"{"t":"room-not-found"}"#, SignalingEnd::NotFound),
            (r#"{"t":"room-full","capacity":10}"#, SignalingEnd::Full),
            (r#"{"t":"room-deleted"}"#, SignalingEnd::Deleted),
            (r#"{"t":"removed"}"#, SignalingEnd::Removed),
            (r#"{"t":"welcome"}"#, SignalingEnd::Update),
        ];
        for (msg, esperado) in tabela {
            let mut f = servidor().await;
            let mut s = Signaling::start(config(&f.base, None));
            let mut c = proxima(&mut f).await;
            let _ = texto(&mut c.ws).await;
            c.ws.send(Message::text(msg)).await.unwrap();
            let vistos = ate(&mut s, |e| matches!(e, SignalingEvent::End(_))).await;
            assert_eq!(vistos.last().unwrap(), &SignalingEvent::End(esperado), "mensagem {msg}");
            sem_conexao_nova(&mut f).await;
        }
    }

    #[tokio::test]
    async fn welcome_com_protocolo_ou_politica_errados_pede_atualizacao() {
        for ajuste in [json!({"mediaProtocol":1}), json!({"mediaPolicy":{"kind":"relay-allowed"}})] {
            let mut f = servidor().await;
            let mut s = Signaling::start(config(&f.base, None));
            let mut c = proxima(&mut f).await;
            let _ = texto(&mut c.ws).await;
            let mut w: Value = serde_json::from_str(&welcome(json!([]))).unwrap();
            w.as_object_mut().unwrap().extend(ajuste.as_object().unwrap().clone());
            c.ws.send(Message::text(w.to_string())).await.unwrap();
            let vistos = ate(&mut s, |e| matches!(e, SignalingEvent::End(_))).await;
            assert_eq!(vistos.last().unwrap(), &SignalingEvent::End(SignalingEnd::Update));
            assert!(!vistos.iter().any(|e| matches!(e, SignalingEvent::Message(_))));
            sem_conexao_nova(&mut f).await;
        }
    }

    #[tokio::test]
    async fn resume_failed_entra_como_pessoa_nova_na_mesma_conexao() {
        let mut f = servidor().await;
        let mut s = Signaling::start(config(&f.base, Some(sessao())));
        let mut c = proxima(&mut f).await;
        assert_eq!(json_de(&mut c.ws).await["t"], "resume");
        c.ws.send(Message::text(r#"{"t":"resume-failed"}"#)).await.unwrap();
        assert_eq!(texto(&mut c.ws).await, r#"{"t":"join","nickname":"Ana","mediaProtocol":2}"#);
        let vistos = ate(&mut s, |e| matches!(e, SignalingEvent::Session(None))).await;
        assert!(!vistos.iter().any(|e| matches!(e, SignalingEvent::End(_))));
    }

    #[tokio::test]
    async fn enviar_passa_mas_turn_refresh_nunca_sai() {
        let mut f = servidor().await;
        let s = Signaling::start(config(&f.base, None));
        let mut c = proxima(&mut f).await;
        let _ = texto(&mut c.ws).await;
        // o ping fica longe do teste: sem interferência
        s.send(ClientMessage::TurnRefresh);
        s.send(ClientMessage::Join { nickname: "Outro".into(), media_protocol: 2 });
        s.send(ClientMessage::Unlive { request_id: "r".into(), broadcast_seq: 3 });
        let m = loop {
            let t = texto(&mut c.ws).await;
            if t != PING {
                break t;
            }
        };
        assert_eq!(m, r#"{"t":"unlive","requestId":"r","broadcastSeq":3}"#);
    }

    #[tokio::test]
    async fn sair_fecha_com_4001_e_apaga_a_sessao() {
        let mut f = servidor().await;
        let mut s = Signaling::start(config(&f.base, Some(sessao())));
        let mut c = proxima(&mut f).await;
        let _ = texto(&mut c.ws).await;
        s.leave();
        let _ = ate(&mut s, |e| matches!(e, SignalingEvent::Session(None))).await;
        let codigo = loop {
            match timeout(LIMITE, c.ws.next()).await.expect("sem fechamento") {
                Some(Ok(Message::Close(Some(f)))) => break u16::from(f.code),
                Some(Ok(Message::Text(t))) if t.as_str() == PING => {}
                outro => panic!("esperava Close 4001: {outro:?}"),
            }
        };
        assert_eq!(codigo, 4001);
        sem_conexao_nova(&mut f).await;
    }

    #[tokio::test]
    async fn tentar_de_novo_nao_espera_a_espera_crescente() {
        let mut f = servidor().await;
        let mut cfg = config(&f.base, Some(sessao()));
        cfg.backoff = vec![Duration::from_secs(60)];
        let s = Signaling::start(cfg);
        let mut c = proxima(&mut f).await;
        let _ = texto(&mut c.ws).await;
        c.ws.send(fechar_com(1011)).await.unwrap();
        // espera de 60 s: sem ajuda, não volta
        assert!(timeout(Duration::from_millis(300), f.conns.recv()).await.is_err());
        s.retry_now();
        let mut c2 = proxima(&mut f).await;
        assert_eq!(json_de(&mut c2.ws).await["t"], "resume");
    }

    #[tokio::test]
    async fn servidor_fora_do_ar_tenta_de_novo_sem_panico() {
        // porta que ninguém escuta: fica reconectando e responde ao Leave
        let mut s = Signaling::start(config("http://127.0.0.1:9", Some(sessao())));
        let vistos = ate(&mut s, |e| matches!(e, SignalingEvent::Status(SignalingStatus::Reconnecting))).await;
        assert!(!vistos.is_empty());
        s.leave();
        let _ = ate(&mut s, |e| matches!(e, SignalingEvent::Session(None))).await;
    }
}
