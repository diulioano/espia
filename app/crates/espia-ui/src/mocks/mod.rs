//! Dados falsos (R17, contrato `contracts/dados-falsos.md`): registro de telas e cenas e o laço do
//! modo `--mock`. Nada aqui abre rede, câmera, captura ou arquivo do aparelho.

pub mod components;
pub mod home;
pub mod join;
pub mod notfound;
pub mod offline;
pub mod registry;
pub mod room;

pub use registry::{MockScene, MockScreen, catalogo, registry, resolve};

use crate::{MockAction, Screen, Ui, UiAction, UiError};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

/// Uma cena de uma tela: um view-model pronto mais o texto do índice.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene<V> {
    pub id: String,
    /// Código curto do índice (o `code` do site, ex.: "D1").
    pub code: String,
    pub label: String,
    pub view: V,
}

impl<V> Scene<V> {
    pub fn new(id: impl Into<String>, code: impl Into<String>, label: impl Into<String>, view: V) -> Self {
        Scene { id: id.into(), code: code.into(), label: label.into(), view }
    }
}

/// Opções de conferência automática (`--exit-after`, `--screenshot`).
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub exit_after: Option<Duration>,
    /// Salva a janela neste PNG depois de 1 s e fecha.
    pub screenshot: Option<PathBuf>,
}

/// Abre a janela no modo mock. `tela` ausente = índice; `aviso` é uma linha extra no alto do índice
/// (se `tela` não existir, vale a mensagem do nome que não existe).
pub fn run(tela: Option<&str>, cena: Option<&str>, aviso: Option<String>, opts: RunOptions) -> Result<(), UiError> {
    let registro = Rc::new(registry());
    let ui = Ui::new()?;
    ui.set_mock_catalog(catalogo(&registro));
    ui.set_mock_bar_collapsed(opts.screenshot.is_some());

    navegar(&ui, &registro, tela, cena, aviso);

    let ui2 = ui.clone();
    let reg2 = registro.clone();
    ui.on_action(move |acao| match acao {
        UiAction::Mock(MockAction::Open { screen, scene }) => {
            navegar(&ui2, &reg2, Some(&screen), scene.as_deref(), None)
        }
        UiAction::Mock(MockAction::Index) => navegar(&ui2, &reg2, None, None, None),
        // As ações das telas não fazem nada no mock: não há servidor, câmera nem captura.
        _ => {}
    });

    if let Some(d) = opts.exit_after {
        ui.quit_after(d);
    }
    if let Some(arq) = opts.screenshot {
        ui.screenshot_after(Duration::from_secs(1), arq);
    }
    ui.run()
}

/// Mostra a tela/cena pedida, ou o índice (com a linha de aviso) se não houver ou não existir.
fn navegar(ui: &Ui, registro: &[MockScreen], tela: Option<&str>, cena: Option<&str>, aviso: Option<String>) {
    match tela {
        None => {
            ui.set_mock_current(None);
            ui.show(&Screen::MockIndex { erro: aviso });
        }
        Some(t) => match resolve(registro, t, cena) {
            Ok((tela, cena)) => {
                ui.set_mock_current(Some((tela.id, cena.id.as_str())));
                ui.show(&cena.screen);
            }
            Err(msg) => {
                ui.set_mock_current(None);
                ui.show(&Screen::MockIndex { erro: Some(msg) });
            }
        },
    }
}
