//! Cena da tela `offline` (sem internet). Não existe no site: porte de `src-tauri/www/index.html` (T030).

use super::Scene;
use crate::Screen;

pub fn scenes() -> Vec<Scene<()>> {
    vec![Scene::new("default", "O1", "Sem internet", ())]
}

pub(super) fn tela(_: ()) -> Screen {
    Screen::Offline
}
