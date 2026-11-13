//! Telas do Espia em Slint (T025 a T027). Depende só de `espia-core` (constituição IV):
//! recebe view-models prontos (`Screen`) e devolve ações (`UiAction`). Nunca fala com rede,
//! captura ou mídia.
//!
//! Uso: `let ui = Ui::new()?; ui.on_action(|a| ...); ui.show(&Screen::Home(view)); ui.run()?;`.

mod bind;
mod png;
mod screen;
mod ui;

pub mod mocks;
pub mod tokens;

pub use bind::initial;
pub use screen::{ComponentsScene, MockAction, OfflineAction, Screen, UiAction};
pub use ui::{MockSceneInfo, MockScreenInfo, Ui, UiError, UiRemote};

/// Código gerado da Slint (janela, globais das telas e tipos). Só a crate enxerga.
#[allow(clippy::all)]
mod generated {
    slint::include_modules!();
}
pub(crate) use generated::*;

/// Callback que as telas usam para devolver ações.
pub(crate) type Emit = std::rc::Rc<dyn Fn(UiAction)>;
