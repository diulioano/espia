//! Laço principal do app (specs/019, T034).
//!
//! # Desenho
//!
//! ## Duas threads, dois canais
//!
//! ```text
//!  thread principal (Slint)                    thread do tokio (runtime de uma thread só)
//!  ───────────────────────                     ──────────────────────────────────────────
//!  Ui: mostra Screen, devolve UiAction  ──▶  entrada: mpsc<Entrada>
//!                                                 │
//!                                                 ▼
//!                                            App::tratar(entrada)   ← único dono do estado
//!                                                 │
//!  slint::invoke_from_event_loop(show)  ◀──  saída: Screen novo, quando mudou
//! ```
//!
//! - A Slint roda na thread principal e não guarda estado do app: só mostra o último `Screen`
//!   (`espia_ui::Screen`, feito de view-models de `espia-core`) e manda cada `UiAction` para o canal
//!   de entrada. Ela nunca vê tipo de rede, de arquivo nem de mídia.
//! - O `tokio` roda em outra thread. Lá vive o `App`, o **único dono do estado**: a rota, o estado
//!   da tela inicial, o estado da entrada, o `RoomState` do redutor e os dados do aparelho
//!   (`DeviceData`). Ninguém mais muda esses valores, então não há trava nem estado partilhado.
//! - Tudo o que acontece vira uma `Entrada` na mesma fila: ação da tela, mensagem da sinalização,
//!   resposta da API, temporizador vencido, link `espia://` recebido. O `App` trata uma por vez,
//!   até o fim, e só então olha a próxima. Depois de cada uma ele monta o `Screen` e, se mudou,
//!   manda para a thread da Slint.
//!
//! ## Rotas (`router.rs`)
//!
//! `Rota::Home`, `Rota::Join { code }` (prévia e apelido), `Rota::Room { code }` (dentro da sala),
//! `Rota::NotFound { code, reason }` e `Rota::Offline`. Trocar de rota larga o que a rota anterior
//! tinha aberto (sinalização, temporizadores, pedidos em andamento: a resposta que chegar depois é
//! descartada por um número de geração).
//!
//! ## Como os efeitos do redutor viram chamadas (`bind.rs`)
//!
//! Dentro da sala, cada `Entrada` relevante vira um `room::Event`; o `App` chama
//! `room::reduce(estado, evento, agora_ms)`, guarda o estado novo e executa os `Effect` na ordem:
//!
//! | Efeito | Quem executa |
//! |---|---|
//! | `Send`, `Leave`, `RetryNow` | `espia-net::signaling` (comandos do `Signaling` da sala) |
//! | `Api(..)` | `espia-net::api`, numa tarefa do tokio; a resposta volta como `Entrada` e vira `Event::Api` |
//! | `Schedule`, `Cancel` | tabela de temporizadores do `App` (`tokio::time::sleep` numa tarefa; ao vencer manda `Entrada::Temporizador`; agendar de novo o mesmo id cancela o anterior) |
//! | `Save(DeviceChange)` | muda o `DeviceData` em memória e grava `device.json` com `espia-shell::storage` |
//! | `CopyInvite` | área de transferência (pela thread da Slint) |
//! | `Navigate` | troca a rota |
//! | `OpenPeer`, `ClosePeer`, `RetryPeer`, `Signal`, `SetIceServers`, `SendView`, `RequestView`, `SetSendParams`, `SetCaptureQuality` | `espia-media` (T037). Até lá: registrados em log de depuração e ignorados |
//! | captura e câmeras (`StartCapture`, `StopCapture`, listar fontes, prévias…) | `espia-capture` (fase 4). Até lá: ignorados; pedir para transmitir devolve ao redutor a falha `StartFailure::Other` |
//!
//! O relógio (`agora_ms`) é lido uma vez por `Entrada`, na borda, e passado ao redutor.
//!
//! ## Servidor
//!
//! A base é `espia_net::server::server_base()`: `https://espia.diuliano.com`, e só em build de
//! depuração a variável `ESPIA_SERVER` (por exemplo `http://127.0.0.1:8787`) a troca. O `Origin`
//! segue a base em uso (isso já é da `espia-net`).
//!
//! ## Modo mock
//!
//! `--mock` não passa por este módulo: `main.rs` chama `espia_ui::mocks::run` e não cria o runtime
//! do tokio. O modo mock continua sem rede.
//!
//! ## Testes
//!
//! O `App` não conhece a Slint: recebe `Entrada` e devolve `Screen` por um canal. Os testes criam o
//! `App` com pasta temporária e servidor local falso, mandam entradas e conferem os `Screen`.
//!
//! ## Notas de implementação
//!
//! O que a implementação decidiu onde o desenho acima não disse (T034 a T036):
//!
//! - **`Saida` em vez de só `Screen`.** O `App` entrega `Saida::Tela(Screen)` e `Saida::Copiar(texto)` a uma função
//!   (`Box<dyn Fn(Saida) + Send>`), não a um canal: em `main.rs` ela chama `UiRemote::show_live` (a Slint enfileira) e
//!   a área de transferência; nos testes ela empurra num canal. `Screen` só é entregue se mudou.
//! - **Rotas.** `Rota::Join` cobre o que o site chama de `RoomRoute` sem sala aberta: a prévia, o apelido e a tela
//!   de sala não encontrada. `Rota::Room` é a sala com sinalização. Não existe `Rota::Offline`: nada dispara essa tela
//!   ainda (a tela inicial e a prévia lidam com a falta de rede com os mesmos textos do site).
//! - **Geração.** Toda troca de rota soma 1 a `geracao`, aborta os temporizadores e solta a sala. Resposta de API,
//!   evento da sinalização e temporizador carregam a geração em que nasceram e, se não for a atual, são descartados.
//!   O temporizador também carrega um número de agendamento, para o que venceu e foi cancelado em seguida não valer.
//! - **Estado do redutor entra e sai por valor** (`reduce(estado, evento, agora)`), então a `Sala` guarda um
//!   `Option<RoomState>` que só está vazio durante a chamada.
//! - **Efeitos que geram eventos.** `StartCapture` devolve `Event::Capture(StartFailed(Other))` ao redutor (a tela não
//!   trava); eles entram numa fila e são tratados, um a um, antes de a `Entrada` seguinte ser olhada.
//! - **A tela não apaga o que a pessoa digita.** O `Screen` novo chega a cada resposta de rede; por isso a `Ui` ganhou
//!   `UiRemote::show_live`, que na mesma tela não sobrescreve apelido, link e nomes de diálogo.
//! - **Tempo.** `agora_ms` é lido uma vez por `Entrada` (e uma vez na abertura), como no desenho.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use espia_core::device::{DeviceData, SESSION_MAX_AGE_MS};
use espia_net::api::{CreateFixedResult, CreateResult, OwnerFailure, PreviewResult, RoomStatus};
use espia_net::server::server_base;
use espia_net::signaling::{RoomSession, SignalingConfig, SignalingEvent};
use espia_core::room::{ApiEvent, TimerId};
use espia_shell::storage::{self, NOME_DEVICE};
use espia_ui::{Screen, UiAction};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use crate::home::HomeEstado;
use crate::room_screen::{EntradaEstado, Sala};
use crate::router::Rota;
use crate::saved_rooms::fuso_local_ms;

/// Escreve no erro padrão só em build de depuração (o "log de depuração" do desenho).
macro_rules! depurar {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!("espia: {}", format_args!($($arg)*));
        }
    };
}
pub(crate) use depurar;

/// Tudo que o `App` precisa saber do ambiente.
#[derive(Clone)]
pub struct Config {
    /// Base do servidor (`https://espia.diuliano.com`, ou a de `ESPIA_SERVER` em depuração).
    pub base: String,
    /// Pasta dos arquivos do aparelho. `None`: nada é lido nem gravado.
    pub pasta: Option<PathBuf>,
    /// Relógio, em milissegundos desde 1970.
    pub agora: fn() -> u64,
    /// Diferença do fuso local para UTC, em milissegundos.
    pub fuso_ms: fn() -> i64,
    /// Ajuste da sinalização (os testes encurtam o ping e a espera da reconexão).
    pub ajuste_sinalizacao: fn(&mut SignalingConfig),
}

fn agora_do_sistema() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn sem_ajuste(_: &mut SignalingConfig) {}

impl Config {
    /// A base dada e a pasta dada; relógio e fuso do sistema.
    pub fn nova(base: impl Into<String>, pasta: Option<PathBuf>) -> Config {
        Config {
            base: base.into(),
            pasta,
            agora: agora_do_sistema,
            fuso_ms: fuso_local_ms,
            ajuste_sinalizacao: sem_ajuste,
        }
    }

    /// A do app de verdade: `server_base()` e `%APPDATA%\dev.espia.app\`. Só em build de depuração, a variável
    /// `ESPIA_DATA_DIR` troca a pasta (para testes e conferências nunca tocarem nos dados reais).
    pub fn do_ambiente() -> Config {
        Config::nova(server_base(), pasta_dos_dados())
    }
}

/// A pasta dos dados: `ESPIA_DATA_DIR` (só em depuração) ou a pasta do app.
pub fn pasta_dos_dados() -> Option<PathBuf> {
    pasta_com(cfg!(debug_assertions), std::env::var_os("ESPIA_DATA_DIR"))
}

fn pasta_com(depuracao: bool, variavel: Option<std::ffi::OsString>) -> Option<PathBuf> {
    if depuracao && let Some(v) = variavel.filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(v));
    }
    storage::pasta_do_app()
}

/// O que acontece e o `App` trata, uma por vez.
#[derive(Debug)]
pub enum Entrada {
    /// A pessoa fez algo na tela.
    Ui(UiAction),
    /// Algo chegou da sinalização da sala aberta na geração `geracao`.
    Sinalizacao { geracao: u64, evento: SignalingEvent },
    /// Uma resposta da API HTTP.
    Resposta { geracao: u64, resposta: Resposta },
    /// Um temporizador venceu.
    Tempo { geracao: u64, tempo: Tempo, seq: u64 },
    /// Chegou um link (`espia://s/<código>`, ou o do site): abre a sala.
    Link(String),
    /// Fecha o laço.
    Encerrar,
}

/// Respostas da API, já no formato que o `App` usa.
#[derive(Debug)]
pub enum Resposta {
    SalaRapida { nickname: String, resultado: CreateResult },
    SalaFixa { nickname: String, nome: String, resultado: CreateFixedResult },
    Numeros(Option<Vec<RoomStatus>>),
    Renomeada(Result<String, OwnerFailure>),
    Apagada(Result<(), OwnerFailure>),
    Previa(PreviewResult),
    /// Resposta do dono dentro da sala (renomear, apagar, tirar): vai ao redutor.
    Sala(ApiEvent),
}

/// Os temporizadores do `App`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tempo {
    /// Um temporizador do redutor da sala.
    Sala(TimerId),
    /// O aviso "O dono te tirou da sala" some da tela inicial (8 s, como no site).
    AvisoDoInicio,
}

/// O que o `App` entrega para fora.
#[derive(Debug, Clone, PartialEq)]
pub enum Saida {
    Tela(Screen),
    /// Copiar para a área de transferência.
    Copiar(String),
}

/// Cria o canal de entrada do laço.
pub fn canal() -> (UnboundedSender<Entrada>, UnboundedReceiver<Entrada>) {
    mpsc::unbounded_channel()
}

/// O dono do estado do app (ver o desenho acima).
pub struct App {
    pub(crate) cfg: Config,
    /// Para as tarefas do tokio devolverem o que acharam.
    pub(crate) auto: UnboundedSender<Entrada>,
    saida: Box<dyn Fn(Saida) + Send>,
    pub(crate) device: DeviceData,
    pub(crate) rota: Rota,
    pub(crate) geracao: u64,
    pub(crate) home: HomeEstado,
    pub(crate) entrada: EntradaEstado,
    pub(crate) sala: Option<Sala>,
    tempos: HashMap<Tempo, (u64, JoinHandle<()>)>,
    seq_tempo: u64,
    ultima: Option<Screen>,
    /// Lido uma vez por `Entrada`.
    pub(crate) agora_ms: u64,
}

impl App {
    /// Lê o `device.json` (se houver pasta) e prepara o estado. Não abre nada ainda: veja `iniciar`.
    pub fn novo(cfg: Config, auto: UnboundedSender<Entrada>, saida: impl Fn(Saida) + Send + 'static) -> App {
        let agora = (cfg.agora)();
        let texto = cfg
            .pasta
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p.join(NOME_DEVICE)).ok())
            .unwrap_or_default();
        App {
            device: DeviceData::from_json(&texto, agora as i64),
            cfg,
            auto,
            saida: Box::new(saida),
            rota: Rota::Home,
            geracao: 0,
            home: HomeEstado::default(),
            entrada: EntradaEstado::vazia(),
            sala: None,
            tempos: HashMap::new(),
            seq_tempo: 0,
            ultima: None,
            agora_ms: agora,
        }
    }

    /// Abre a tela de partida: a sala do link, ou a tela inicial. Precisa de um runtime do tokio.
    pub fn iniciar(&mut self, link: Option<String>) {
        self.agora_ms = (self.cfg.agora)();
        match link.as_deref().and_then(crate::router::codigo_do_texto) {
            Some(code) => self.abrir_codigo(code, None, false),
            None => self.ir_para_inicio(None),
        }
        self.publicar();
    }

    /// Trata uma `Entrada` até o fim e entrega o `Screen` novo, se mudou.
    pub fn tratar(&mut self, entrada: Entrada) {
        self.agora_ms = (self.cfg.agora)();
        match entrada {
            Entrada::Ui(acao) => self.acao_da_tela(acao),
            Entrada::Sinalizacao { geracao, evento } if geracao == self.geracao => self.sinalizacao(evento),
            Entrada::Resposta { geracao, resposta } if geracao == self.geracao => self.resposta(resposta),
            Entrada::Tempo { geracao, tempo, seq } if geracao == self.geracao => self.tempo_venceu(tempo, seq),
            Entrada::Link(texto) => {
                if let Some(code) = crate::router::codigo_do_texto(&texto) {
                    self.abrir_codigo(code, None, false);
                }
            }
            Entrada::Sinalizacao { .. } | Entrada::Resposta { .. } | Entrada::Tempo { .. } | Entrada::Encerrar => {}
        }
        self.publicar();
    }

    fn acao_da_tela(&mut self, acao: UiAction) {
        match (acao, &self.rota) {
            (UiAction::Home(a), Rota::Home) => self.acao_home(a),
            (UiAction::Join(a), Rota::Join { .. }) => self.acao_join(a),
            (UiAction::NotFound(a), Rota::Join { .. }) => self.acao_nao_encontrada(a),
            (UiAction::Room(a), Rota::Room { .. }) => self.acao_sala(a),
            (UiAction::Reload, Rota::Room { .. }) => self.recarregar_sala(),
            (acao, rota) => depurar!("ação fora de hora ({rota:?}): {acao:?}"),
        }
    }

    fn resposta(&mut self, resposta: Resposta) {
        match resposta {
            Resposta::Sala(evento) => self.evento_da_sala(espia_core::room::Event::Api(evento)),
            Resposta::Previa(r) => self.previa_chegou(r),
            outra => self.resposta_home(outra),
        }
    }

    // ---- Tela ----

    fn tela(&self) -> Screen {
        match &self.rota {
            Rota::Home => Screen::Home(self.tela_home()),
            Rota::Join { .. } => self.tela_entrada(),
            Rota::Room { .. } => self.tela_sala(),
        }
    }

    fn publicar(&mut self) {
        let tela = self.tela();
        if self.ultima.as_ref() != Some(&tela) {
            self.ultima = Some(tela.clone());
            (self.saida)(Saida::Tela(tela));
        }
    }

    pub(crate) fn copiar(&self, texto: String) {
        (self.saida)(Saida::Copiar(texto));
    }

    // ---- Rotas ----

    /// Larga o que a rota anterior tinha aberto: respostas e temporizadores antigos, e a sala.
    pub(crate) fn nova_geracao(&mut self) {
        self.geracao += 1;
        for (_, (_, tarefa)) in self.tempos.drain() {
            tarefa.abort();
        }
        self.sala = None;
    }

    /// A tela inicial. `aviso`: texto no alto (o dono tirou você da sala).
    pub(crate) fn ir_para_inicio(&mut self, aviso: Option<String>) {
        self.nova_geracao();
        self.rota = Rota::Home;
        self.home = HomeEstado::default();
        if aviso.is_some() {
            self.agendar(Tempo::AvisoDoInicio, 8_000);
        }
        self.home.aviso = aviso;
        self.atualizar_numeros();
    }

    /// `/s/<código>` do site: entra direto se há apelido combinado ou sessão guardada; senão mostra a entrada.
    pub(crate) fn abrir_codigo(&mut self, code: String, nickname: Option<String>, criada: bool) {
        let direto = nickname
            .filter(|n| !n.is_empty())
            .or_else(|| self.sessao_valida(&code).and(self.device.nickname.clone()).filter(|n| !n.is_empty()));
        match direto {
            Some(nick) => self.entrar_na_sala(code, nick, criada),
            None => self.ir_para_entrada(code, true),
        }
    }

    // ---- Dados do aparelho ----

    /// A sessão guardada da sala, se ainda vale (24 h).
    pub(crate) fn sessao_valida(&self, code: &str) -> Option<RoomSession> {
        let s = self.device.sessions.get(code)?;
        (self.agora_ms as i64 - s.at < SESSION_MAX_AGE_MS)
            .then(|| RoomSession { peer_id: s.peer_id.clone(), resume_token: s.resume_token.clone() })
    }

    /// Grava o `device.json` (limpo). Sem pasta, não grava.
    pub(crate) fn salvar(&self) {
        let Some(pasta) = &self.cfg.pasta else { return };
        let limpo = self.device.clone().sanitize(self.agora_ms as i64);
        if let Err(e) = storage::gravar(pasta, NOME_DEVICE, &limpo) {
            depurar!("não deu para gravar o device.json: {e}");
        }
    }

    pub(crate) fn guardar_apelido(&mut self, apelido: &str) {
        self.device.nickname = Some(apelido.to_string());
        self.salvar();
    }

    // ---- Temporizadores ----

    /// Agenda `tempo`; agendar de novo o mesmo cancela o anterior.
    pub(crate) fn agendar(&mut self, tempo: Tempo, ms: u64) {
        self.cancelar(&tempo);
        self.seq_tempo += 1;
        let (seq, geracao, tx, t) = (self.seq_tempo, self.geracao, self.auto.clone(), tempo.clone());
        let tarefa = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            let _ = tx.send(Entrada::Tempo { geracao, tempo: t, seq });
        });
        self.tempos.insert(tempo, (seq, tarefa));
    }

    pub(crate) fn cancelar(&mut self, tempo: &Tempo) {
        if let Some((_, tarefa)) = self.tempos.remove(tempo) {
            tarefa.abort();
        }
    }

    fn tempo_venceu(&mut self, tempo: Tempo, seq: u64) {
        if self.tempos.get(&tempo).map(|(s, _)| *s) != Some(seq) {
            return; // cancelado ou reagendado depois de vencer
        }
        self.tempos.remove(&tempo);
        match tempo {
            Tempo::Sala(id) => self.evento_da_sala(espia_core::room::Event::Timer(id)),
            Tempo::AvisoDoInicio => self.home.aviso = None,
        }
    }
}

/// O laço do tokio: trata uma `Entrada` por vez até `Encerrar` (ou até o canal fechar).
pub async fn laco(mut app: App, mut entradas: UnboundedReceiver<Entrada>, link: Option<String>) {
    app.iniciar(link);
    while let Some(entrada) = entradas.recv().await {
        if matches!(entrada, Entrada::Encerrar) {
            break;
        }
        app.tratar(entrada);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pasta_so_troca_em_depuracao() {
        let outra = Some(std::ffi::OsString::from("C:\\temp\\teste"));
        assert_eq!(pasta_com(true, outra.clone()), Some(PathBuf::from("C:\\temp\\teste")));
        // Em release a variável é ignorada: vale a pasta do app (ou nenhuma, se APPDATA não existir).
        assert_eq!(pasta_com(false, outra), storage::pasta_do_app());
        assert_eq!(pasta_com(true, Some(std::ffi::OsString::new())), storage::pasta_do_app());
    }
}
