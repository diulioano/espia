//! Liga `NotFoundView` à tela `ui/pages/notfound.slint`. Os textos de cada motivo moram aqui
//! (porte do `COPY` de `src/ui/pages/NotFoundPage.svelte`).

use crate::{AppWindow, Emit, MascotMood, NotFoundState, UiAction};
use espia_core::view_model::{NotFoundAction, NotFoundReason, NotFoundView};
use slint::ComponentHandle;

/// Título e texto de apoio do motivo.
pub(crate) fn copy(reason: NotFoundReason) -> (&'static str, &'static str) {
    match reason {
        NotFoundReason::Quick => (
            "Essa sala não existe mais.",
            "O link pode estar incompleto, ou todo mundo saiu — salas rápidas somem quando ficam vazias.",
        ),
        NotFoundReason::Deleted => ("Esta sala foi apagada.", "Quem criou a sala apagou. O link não funciona mais."),
        NotFoundReason::Kicked => {
            ("Esta sala foi apagada.", "Quem criou a sala apagou agora há pouco, e todo mundo saiu junto.")
        }
        NotFoundReason::Expired => (
            "Esta sala ficou 30 dias sem uso e deixou de existir.",
            "Salas fixas somem sozinhas depois de um mês vazias. Dá pra criar outra com o mesmo nome.",
        ),
    }
}

/// O nome só aparece fora da sala rápida; "Tirar de Suas salas" também.
fn nome(view: &NotFoundView) -> &str {
    if view.reason == NotFoundReason::Quick { "" } else { view.name.as_deref().unwrap_or("") }
}

fn guardada(view: &NotFoundView) -> bool {
    view.saved && view.reason != NotFoundReason::Quick
}

/// Copia o view-model para as propriedades de `NotFoundState`.
pub(crate) fn apply(app: &AppWindow, view: &NotFoundView) {
    let s = app.global::<NotFoundState>();
    let (titulo, corpo) = copy(view.reason);
    s.set_name(nome(view).into());
    s.set_title(titulo.into());
    s.set_body(corpo.into());
    s.set_mood(if view.reason == NotFoundReason::Expired { MascotMood::Sleepy } else { MascotMood::Sad });
    s.set_saved(guardada(view));
}

/// Liga os callbacks de `NotFoundState` às ações devolvidas por `emit`.
pub(crate) fn connect(app: &AppWindow, emit: &Emit) {
    let s = app.global::<NotFoundState>();
    let e = emit.clone();
    s.on_go_home(move || e(UiAction::NotFound(NotFoundAction::GoHome)));
    let e = emit.clone();
    s.on_create_room(move || e(UiAction::NotFound(NotFoundAction::CreateRoom)));
    let e = emit.clone();
    s.on_paste_other_link(move || e(UiAction::NotFound(NotFoundAction::PasteOtherLink)));
    let e = emit.clone();
    s.on_forget(move || e(UiAction::NotFound(NotFoundAction::Forget)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(reason: NotFoundReason, name: Option<&str>, saved: bool) -> NotFoundView {
        NotFoundView { reason, name: name.map(String::from), saved }
    }

    #[test]
    fn textos_do_site_por_motivo() {
        assert_eq!(copy(NotFoundReason::Quick).0, "Essa sala não existe mais.");
        assert_eq!(copy(NotFoundReason::Deleted).1, "Quem criou a sala apagou. O link não funciona mais.");
        assert_eq!(copy(NotFoundReason::Kicked).0, copy(NotFoundReason::Deleted).0);
        assert!(copy(NotFoundReason::Expired).0.contains("30 dias"));
    }

    #[test]
    fn sala_rapida_nao_mostra_nome_nem_tirar_de_suas_salas() {
        let v = view(NotFoundReason::Quick, Some("CS da galera"), true);
        assert_eq!(nome(&v), "");
        assert!(!guardada(&v));
        let v = view(NotFoundReason::Deleted, Some("CS da galera"), true);
        assert_eq!(nome(&v), "CS da galera");
        assert!(guardada(&v));
    }
}
