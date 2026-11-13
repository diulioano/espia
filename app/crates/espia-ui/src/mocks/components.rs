//! Cenas da vitrine dos componentes base (T026): uma por componente, mais `all` com todos.
//! Só existe no mock, para conferir o visual; o site não tem.

use super::Scene;
use crate::ComponentsScene;

pub fn scenes() -> Vec<Scene<ComponentsScene>> {
    let c = |id: &str, label: &str| Scene::new(id, "C1", label, ComponentsScene(id.to_string()));
    vec![
        c("all", "Todos os componentes"),
        c("logo", "Logo"),
        c("mascot", "Castor"),
        c("buttons", "Botões"),
        c("avatars", "Avatares"),
        c("spinner", "Spinner"),
        c("fields", "Campo de texto"),
        c("toasts", "Avisos"),
    ]
}
