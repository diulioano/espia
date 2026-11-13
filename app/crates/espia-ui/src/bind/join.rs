//! Liga `JoinView` à tela `ui/pages/join.slint`. Os textos derivados da prévia (quem está lá, "N ao vivo")
//! vêm de `espia_core::join_view`; aqui só se montam as frases da tela.

use crate::{AppWindow, Emit, JoinState, UiAction, bind::{initial, tone}};
use espia_core::join_view::{self as jv, clean_nickname, live_label, who_is_there, who_is_there_long, NICKNAME_MAX};
use espia_core::view_model::{JoinAction, JoinStatus, JoinView};
use slint::{ComponentHandle, ModelRc, VecModel};

/// O view-model e `join_view` têm cada um o seu `JoinPreviewPerson` (mesmos campos): converte para os textos.
fn gente(view: &JoinView) -> Vec<jv::JoinPreviewPerson> {
    view.people
        .iter()
        .flatten()
        .map(|p| jv::JoinPreviewPerson { name: p.name.clone(), tone: p.tone, live: p.live })
        .collect()
}

/// Subtítulo do cartão (porte de `subtitle` em `JoinPage.svelte`).
pub(crate) fn subtitle(view: &JoinView) -> String {
    let people = gente(view);
    let people = people.as_slice();
    if view.fixed_name.is_some() {
        // Sala fixa: o nome já é o título.
        return if people.is_empty() {
            "Te chamaram pra cá.".into()
        } else {
            format!("Te chamaram pra cá. {}.", who_is_there(people))
        };
    }
    if people.is_empty() {
        return format!("Sala {} · até {} pessoas", view.room_id, view.capacity);
    }
    if view.can_share {
        format!("{} · sala {}", who_is_there(people), view.room_id)
    } else {
        who_is_there_long(people)
    }
}

/// Ajuda do campo de apelido: só quem pode transmitir e já tinha apelido.
pub(crate) fn hint(view: &JoinView) -> &'static str {
    if view.can_share && !view.nickname.is_empty() { "Lembramos da última vez. Fica só neste computador." } else { "" }
}

pub(crate) fn title_cheia(capacity: u32) -> String {
    format!("Essa sala está cheia ({capacity} de {capacity}).")
}

pub(crate) fn corpo_cheia(capacity: u32) -> String {
    format!("Cabem até {capacity} pessoas pra transmissão não pesar pra ninguém. Quando alguém sair, dá pra entrar.")
}

/// Copia o view-model para as propriedades de `JoinState`. Com `manter_texto`, o apelido que a pessoa está digitando
/// não é sobrescrito.
pub(crate) fn apply(app: &AppWindow, view: &JoinView, manter_texto: bool) {
    let s = app.global::<JoinState>();
    let people = view.people.clone().unwrap_or_default();
    let textos = gente(view);
    let avatars: Vec<crate::AvatarData> =
        people.iter().map(|p| crate::AvatarData { letter: initial(&p.name).into(), tone: tone(p.tone) }).collect();
    s.set_full(view.status == JoinStatus::Full);
    s.set_people(ModelRc::new(VecModel::from(avatars)));
    // O selo é escrito em maiúsculas (a Slint não tem text-transform).
    s.set_badge(live_label(&textos).map(|t| t.to_uppercase()).unwrap_or_default().into());
    s.set_fixed_name(view.fixed_name.clone().unwrap_or_default().into());
    s.set_replaced(view.replaced.unwrap_or(false));
    s.set_can_share(view.can_share);
    s.set_hint(hint(view).into());
    if !manter_texto {
        s.set_nickname(view.nickname.as_str().into());
    }
    if view.status == JoinStatus::Full {
        s.set_title(title_cheia(view.capacity).into());
        s.set_full_body(corpo_cheia(view.capacity).into());
    } else {
        s.set_title("Te chamaram pra uma sala.".into());
        s.set_subtitle(subtitle(view).into());
    }
}

/// Liga os callbacks de `JoinState` às ações devolvidas por `emit`.
pub(crate) fn connect(app: &AppWindow, emit: &Emit) {
    let s = app.global::<JoinState>();
    s.on_limit_nickname(|t| t.chars().take(NICKNAME_MAX).collect::<String>().into());
    s.on_clean_nickname(|t| clean_nickname(&t).into());
    let e = emit.clone();
    s.on_go_home(move || e(UiAction::Join(JoinAction::GoHome)));
    let e = emit.clone();
    s.on_join(move |apelido| e(UiAction::Join(JoinAction::Join { nickname: clean_nickname(&apelido) })));
    let e = emit.clone();
    s.on_create_room(move || e(UiAction::Join(JoinAction::CreateRoom)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use espia_core::protocol::Tone;
    use espia_core::view_model::JoinPreviewPerson;

    fn pessoa(nome: &str, live: bool) -> JoinPreviewPerson {
        JoinPreviewPerson { name: nome.into(), tone: Tone::Sage, live }
    }

    fn view(people: Option<Vec<JoinPreviewPerson>>, fixed: Option<&str>, can_share: bool) -> JoinView {
        JoinView {
            room_id: "k7f-q2x".into(),
            status: JoinStatus::Ready,
            people,
            capacity: 10,
            nickname: "Rafa".into(),
            can_share,
            fixed_name: fixed.map(String::from),
            replaced: None,
        }
    }

    #[test]
    fn subtitulos_do_site() {
        let tres = vec![pessoa("Lu", true), pessoa("Bia", false), pessoa("Tomás", false)];
        assert_eq!(subtitle(&view(Some(tres.clone()), None, true)), "Lu, Bia e Tomás já estão lá · sala k7f-q2x");
        assert_eq!(subtitle(&view(None, None, true)), "Sala k7f-q2x · até 10 pessoas");
        assert_eq!(subtitle(&view(Some(vec![]), None, true)), "Sala k7f-q2x · até 10 pessoas");
        assert_eq!(subtitle(&view(Some(tres), Some("CS da galera"), true)), "Te chamaram pra cá. Lu, Bia e Tomás já estão lá.");
        assert_eq!(subtitle(&view(None, Some("CS da galera"), true)), "Te chamaram pra cá.");
    }

    #[test]
    fn ajuda_so_com_apelido_lembrado() {
        assert!(hint(&view(None, None, true)).starts_with("Lembramos"));
        assert_eq!(hint(&view(None, None, false)), "");
        let mut v = view(None, None, true);
        v.nickname.clear();
        assert_eq!(hint(&v), "");
    }

    #[test]
    fn sala_cheia_diz_a_capacidade() {
        assert_eq!(title_cheia(10), "Essa sala está cheia (10 de 10).");
        assert!(corpo_cheia(10).starts_with("Cabem até 10 pessoas"));
    }
}
