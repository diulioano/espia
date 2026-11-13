//! Cenas da tela `notfound` do mock. Origem: `src/mocks/notfound.ts` (4 cenas portadas; a `mobile` fica só no site).

use super::Scene;
use espia_core::view_model::{NotFoundReason, NotFoundView};

fn cena(id: &str, code: &str, label: &str, reason: NotFoundReason) -> Scene<NotFoundView> {
    // Como no site: nome só fora da sala rápida; `saved` só vem de `&saved=1` (aqui, sempre falso).
    let name = (reason != NotFoundReason::Quick).then(|| "CS da galera".to_string());
    Scene::new(id, code, label, NotFoundView { reason, name, saved: false })
}

pub fn scenes() -> Vec<Scene<NotFoundView>> {
    vec![
        cena("quick", "N4", "Sala rápida que acabou", NotFoundReason::Quick),
        cena("deleted", "N1", "Apagada pelo dono", NotFoundReason::Deleted),
        cena("kicked", "N2", "Apagada com você dentro", NotFoundReason::Kicked),
        cena("expired", "N3", "Expirada (30 dias)", NotFoundReason::Expired),
    ]
}
