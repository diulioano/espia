//! `Ui`: a janela do Espia. Recebe `Screen`, devolve `UiAction`.

use crate::{
    AppWindow, ComponentsState, Emit, MockAction, MockSceneData, MockScreenData, Screen, ScreenId, UiAction, bind, png,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::{Cell, RefCell};
use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

/// Erro ao abrir ou rodar a janela.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiError {
    /// A Slint não conseguiu criar a janela ou o laço de eventos.
    Platform(String),
    /// Não deu para capturar ou gravar a imagem da janela (`--screenshot`).
    Screenshot(String),
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiError::Platform(m) => write!(f, "falha na janela: {m}"),
            UiError::Screenshot(m) => write!(f, "falha na captura da janela: {m}"),
        }
    }
}

impl std::error::Error for UiError {}

impl From<slint::PlatformError> for UiError {
    fn from(e: slint::PlatformError) -> Self {
        UiError::Platform(e.to_string())
    }
}

/// Uma cena como o índice e o seletor do mock a enxergam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockSceneInfo {
    pub id: String,
    pub code: String,
    pub label: String,
}

/// Uma tela do mock com as cenas dela.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockScreenInfo {
    pub id: String,
    pub label: String,
    pub scenes: Vec<MockSceneInfo>,
}

type Handler = Rc<RefCell<Option<Rc<dyn Fn(UiAction)>>>>;

struct Inner {
    app: AppWindow,
    handler: Handler,
    catalog: RefCell<Vec<MockScreenInfo>>,
    screenshot_error: Rc<RefCell<Option<String>>>,
}

/// A janela. É barata de clonar (um `Rc`): os clones falam com a mesma janela. Só na thread da interface.
#[derive(Clone)]
pub struct Ui {
    inner: Rc<Inner>,
}

fn modelo<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(v))
}

impl Ui {
    /// Cria a janela (ainda não visível até `run`). Mostra o índice do mock até o primeiro `show`.
    pub fn new() -> Result<Ui, UiError> {
        let app = AppWindow::new()?;
        let handler: Handler = Rc::new(RefCell::new(None));
        let h = handler.clone();
        let emit: Emit = Rc::new(move |acao| {
            // Solta o empréstimo antes de chamar: o callback pode trocar o callback.
            let cb = h.borrow().clone();
            if let Some(cb) = cb {
                cb(acao);
            }
        });

        bind::home::connect(&app, &emit);
        bind::join::connect(&app, &emit);
        bind::room::connect(&app, &emit);
        bind::notfound::connect(&app, &emit);
        bind::offline::connect(&app, &emit);

        let e = emit.clone();
        app.on_mock_open(move |tela, cena| {
            let scene = (!cena.is_empty()).then(|| cena.to_string());
            e(UiAction::Mock(MockAction::Open { screen: tela.to_string(), scene }));
        });
        let e = emit.clone();
        app.on_mock_index(move || e(UiAction::Mock(MockAction::Index)));

        Ok(Ui {
            inner: Rc::new(Inner {
                app,
                handler,
                catalog: RefCell::new(Vec::new()),
                screenshot_error: Rc::new(RefCell::new(None)),
            }),
        })
    }

    /// Define quem recebe as ações da tela. Substitui o anterior.
    pub fn on_action(&self, callback: impl Fn(UiAction) + 'static) {
        *self.inner.handler.borrow_mut() = Some(Rc::new(callback));
    }

    /// Troca o que a janela mostra (mock): o view-model vale por inteiro, inclusive os textos dos campos.
    pub fn show(&self, screen: &Screen) {
        mostrar(&self.inner.app, screen, false);
    }

    /// Como `show`, para o app de verdade (ver `UiRemote::show_live`).
    pub fn show_live(&self, screen: &Screen) {
        mostrar_vivo(&self.inner.app, screen);
    }

    /// Alça que outras threads usam para falar com esta janela (`Send`): o app roda a lógica numa thread à parte.
    pub fn remote(&self) -> UiRemote {
        UiRemote { app: self.inner.app.as_weak() }
    }

    /// Entrega um quadro de vídeo (RGBA, 4 bytes por pixel, `largura` x `altura`) ao tile da transmissão `id` na sala
    /// (palco, grade ou faixa; com o id da própria pessoa, a prévia "Você está ao vivo"). Pode ser chamada a cada quadro:
    /// só troca a imagem daquele tile. O quadro fica guardado e volta a cada nova `show` da sala. Devolve `false` e
    /// ignora o quadro se o tamanho de `rgba` não for `largura * altura * 4`.
    pub fn set_stream_frame(&self, id: &str, largura: u32, altura: u32, rgba: &[u8]) -> bool {
        bind::room::set_frame(&self.inner.app, id, largura, altura, rgba)
    }

    /// A transmissão `id` acabou: esquece o último quadro dela (o tile volta ao placeholder).
    pub fn clear_stream_frame(&self, id: &str) {
        bind::room::clear_frame(&self.inner.app, id);
    }

    /// Modo mock: as telas e cenas do índice e do seletor.
    pub fn set_mock_catalog(&self, catalogo: Vec<MockScreenInfo>) {
        let telas: Vec<MockScreenData> = catalogo
            .iter()
            .map(|t| MockScreenData {
                id: t.id.as_str().into(),
                label: t.label.as_str().into(),
                scenes: modelo(t.scenes.iter().map(cena_data).collect()),
            })
            .collect();
        self.inner.app.set_mock_screens(modelo(telas));
        *self.inner.catalog.borrow_mut() = catalogo;
    }

    /// Modo mock: a barra do canto começa recolhida (usado nas capturas).
    pub fn set_mock_bar_collapsed(&self, recolhida: bool) {
        self.inner.app.set_mock_recolhida(recolhida);
    }

    /// Modo mock: a tela e a cena abertas (liga o seletor no canto); `None` esconde o seletor.
    pub fn set_mock_current(&self, atual: Option<(&str, &str)>) {
        let app = &self.inner.app;
        let catalogo = self.inner.catalog.borrow();
        let achado = atual.and_then(|(t, c)| {
            let tela = catalogo.iter().find(|x| x.id == t)?;
            let cena = tela.scenes.iter().find(|x| x.id == c)?;
            Some((tela, cena))
        });
        match achado {
            Some((tela, cena)) => {
                app.set_mock_scenes(modelo(tela.scenes.iter().map(cena_data).collect()));
                app.set_mock_screen_id(SharedString::from(tela.id.as_str()));
                app.set_mock_scene_id(SharedString::from(cena.id.as_str()));
                app.set_mock_label(format!("{} · {}", tela.label, cena.label).into());
                app.set_mock_bar(true);
            }
            None => app.set_mock_bar(false),
        }
    }

    /// Fecha a janela depois de `d` (conferência automática).
    pub fn quit_after(&self, d: Duration) {
        slint::Timer::single_shot(d, || {
            let _ = slint::quit_event_loop();
        });
    }

    /// Salva a janela em PNG depois de `d` e fecha. Erros voltam em `run`.
    ///
    /// A captura do FemtoVG lê o buffer de trás do OpenGL, que depois de trocar de quadro (swap) fica
    /// com conteúdo indefinido: no Windows, dependendo do quadro, vinha tudo branco. Por isso a leitura
    /// acontece dentro do `AfterRendering`, depois de desenhar e antes de apresentar o quadro.
    pub fn screenshot_after(&self, d: Duration, arquivo: PathBuf) {
        let pedido = Rc::new(Cell::new(false));
        let app = self.inner.app.as_weak();
        let erro = self.inner.screenshot_error.clone();
        let pedido2 = pedido.clone();
        let app2 = app.clone();
        let notificador = self.inner.app.window().set_rendering_notifier(move |estado, _| {
            if !matches!(estado, slint::RenderingState::AfterRendering) || !pedido2.replace(false) {
                return;
            }
            let r = (|| -> Result<(), String> {
                let app = app2.upgrade().ok_or("a janela já tinha fechado")?;
                let img = app.window().take_snapshot().map_err(|e| e.to_string())?;
                let bytes = png::encode_rgba(img.width(), img.height(), img.as_bytes());
                std::fs::write(&arquivo, bytes).map_err(|e| format!("{}: {e}", arquivo.display()))
            })();
            if let Err(e) = r {
                *erro.borrow_mut() = Some(e);
            }
            let _ = slint::quit_event_loop();
        });
        if let Err(e) = notificador {
            *self.inner.screenshot_error.borrow_mut() = Some(format!("sem notificador de desenho: {e}"));
            let _ = slint::quit_event_loop();
            return;
        }
        slint::Timer::single_shot(d, move || {
            pedido.set(true);
            if let Some(app) = app.upgrade() {
                app.window().request_redraw();
            }
        });
    }

    /// Mostra a janela e roda até ela fechar.
    pub fn run(&self) -> Result<(), UiError> {
        self.inner.app.run()?;
        match self.inner.screenshot_error.borrow_mut().take() {
            Some(e) => Err(UiError::Screenshot(e)),
            None => Ok(()),
        }
    }
}

fn cena_data(c: &MockSceneInfo) -> MockSceneData {
    MockSceneData { id: c.id.as_str().into(), code: c.code.as_str().into(), label: c.label.as_str().into() }
}

/// Mostra a tela. `manter_texto`: o que a pessoa está digitando não é sobrescrito (ver `mostrar_vivo`).
fn mostrar(app: &AppWindow, screen: &Screen, manter_texto: bool) {
    let id = match screen {
        Screen::MockIndex { erro } => {
            app.set_mock_error(erro.clone().unwrap_or_default().into());
            ScreenId::MockIndex
        }
        Screen::Home(v) => {
            bind::home::apply(app, v, manter_texto);
            ScreenId::Home
        }
        Screen::Join(v) => {
            bind::join::apply(app, v, manter_texto);
            ScreenId::Join
        }
        Screen::Room(v) => {
            bind::room::apply(app, v, manter_texto);
            ScreenId::Room
        }
        Screen::NotFound(v) => {
            bind::notfound::apply(app, v);
            ScreenId::Notfound
        }
        Screen::Offline => ScreenId::Offline,
        Screen::Components(c) => {
            app.global::<ComponentsState>().set_scene(c.0.as_str().into());
            ScreenId::Components
        }
    };
    app.set_screen(id);
}

/// Se a janela já está nesta mesma tela, o que a pessoa está digitando (apelido, link, nome da sala) não é apagado
/// quando chega um view-model novo (por exemplo, os números de Suas salas). Ao trocar de tela vale o view-model.
fn mostrar_vivo(app: &AppWindow, screen: &Screen) {
    let atual = app.get_screen();
    let mesma = match screen {
        Screen::Home(_) => atual == ScreenId::Home,
        Screen::Join(_) => atual == ScreenId::Join,
        Screen::Room(_) => atual == ScreenId::Room,
        _ => false,
    };
    mostrar(app, screen, mesma);
}

/// A janela vista de outra thread. Cada chamada é enfileirada no laço de eventos da Slint, na ordem em que foi feita.
#[derive(Clone)]
pub struct UiRemote {
    app: slint::Weak<AppWindow>,
}

impl UiRemote {
    /// Troca o que a janela mostra, sem apagar o que a pessoa está digitando na mesma tela. Janela já fechada: ignora.
    pub fn show_live(&self, screen: Screen) {
        let _ = self.app.upgrade_in_event_loop(move |app| mostrar_vivo(&app, &screen));
    }

    /// Roda `f` na thread da janela (a área de transferência, por exemplo, só se mexe de lá).
    pub fn run_in_ui(&self, f: impl FnOnce() + Send + 'static) {
        let _ = slint::invoke_from_event_loop(f);
    }
}
