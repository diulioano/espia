//! Cenas da tela `join` do mock. Origem: `src/mocks/join.ts` (5 cenas portadas; `mobile` e `fixedmobile` ficam só no site).

use super::Scene;
use espia_core::protocol::Tone;
use espia_core::view_model::{JoinPreviewPerson, JoinStatus, JoinView};

/// `MOCK_ROOM_ID` de `src/mocks/room.ts`.
const SALA: &str = "k7f-q2x";

fn gente() -> Vec<JoinPreviewPerson> {
    let p = |name: &str, tone, live| JoinPreviewPerson { name: name.into(), tone, live };
    vec![p("Lu", Tone::Accent, true), p("Bia", Tone::Sage, false), p("Tomás", Tone::Sand, false)]
}

fn base() -> JoinView {
    JoinView {
        room_id: SALA.into(),
        status: JoinStatus::Ready,
        people: Some(gente()),
        capacity: 10,
        nickname: "Rafa".into(),
        can_share: true,
        fixed_name: None,
        replaced: None,
    }
}

pub fn scenes() -> Vec<Scene<JoinView>> {
    vec![
        Scene::new("default", "D2", "Entrar por link", base()),
        Scene::new("nopreview", "D2", "Sem prévia da sala", JoinView { people: None, ..base() }),
        Scene::new("full", "E2*", "Sala cheia (10 de 10)", JoinView { status: JoinStatus::Full, ..base() }),
        Scene::new("fixed", "F8", "Sala fixa (specs/011)", JoinView { fixed_name: Some("CS da galera".into()), ..base() }),
        Scene::new("replaced", "D2·aba", "Aberta em outra aba", JoinView { replaced: Some(true), ..base() }),
    ]
}
