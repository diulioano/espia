//! Cenas da tela `home` do mock. Origem: `src/mocks/home.ts` (13 cenas portadas, mais update-available e update-minimum na T096).
//! Todas as cenas não marcadas `device: 'mobile'`, com os mesmos `id`, `code` e `label` do site (a paridade é testada na T033).

use super::Scene;
use espia_core::protocol::Tone;
use espia_core::view_model::{
    AppInvite, HomeView, NameDialogMode, NameDialogView, SavedRoomGroup, SavedRoomState, SavedRoomView, SavedRoomsView,
};

/// `LINK_ERROR` de `src/mocks/home.ts`.
const LINK_ERROR: &str = "Esse link não parece de uma sala.";

/// `row()` de `src/mocks/home.ts`: sala do mock (de outra pessoa, sálvia, com números e "usada ontem").
fn row(code: &str, name: &str, tone: Tone) -> SavedRoomView {
    SavedRoomView {
        code: code.into(),
        name: name.into(),
        own: false,
        tone,
        state: SavedRoomState::Ok,
        people: 0,
        streaming: 0,
        recent: "usada ontem".into(),
        menu_open: None,
    }
}

fn com(mut r: SavedRoomView, people: u32, streaming: u32) -> SavedRoomView {
    r.people = people;
    r.streaming = streaming;
    r
}

fn sua(mut r: SavedRoomView) -> SavedRoomView {
    r.own = true;
    r
}

/// As três salas de `FEW`.
fn few() -> Vec<SavedRoomView> {
    vec![
        sua(com(row("k7f-q2x", "CS da galera", Tone::Accent), 3, 1)),
        com(row("m4p-t8a", "Trampo do TCC", Tone::Sand), 2, 0),
        row("r2d-h7k", "Filme de sexta", Tone::Sage),
    ]
}

fn uma_lista(rooms: Vec<SavedRoomView>) -> SavedRoomsView {
    SavedRoomsView { groups: vec![SavedRoomGroup { label: None, rooms }], note: "neste aparelho".into() }
}

fn base() -> HomeView {
    HomeView {
        nickname: "Rafa".into(),
        link: String::new(),
        link_error: None,
        create_error: None,
        creating: None,
        app_invite: None,
        notice: None,
        // No celular o app não existe (specs/013); no site de computador o botão está sempre à mão.
        download: Some(true),
        rooms: None,
        fixed_dialog: None,
        delete_dialog: None,
    }
}

fn muitas() -> SavedRoomsView {
    SavedRoomsView {
        groups: vec![
            SavedRoomGroup {
                label: Some("Com gente agora".into()),
                rooms: vec![
                    sua(com(row("k7f-q2x", "CS da galera", Tone::Accent), 4, 2)),
                    com(row("h3k-q9w", "Lu joga Hollow Knight", Tone::Sage), 3, 1),
                    com(row("m4p-t8a", "Trampo do TCC", Tone::Sand), 2, 0),
                ],
            },
            SavedRoomGroup {
                label: Some("Usadas recentemente".into()),
                rooms: vec![
                    row("r2d-h7k", "Filme de sexta", Tone::Sage),
                    sua(SavedRoomView { recent: "há 3 dias".into(), ..row("c8v-n2m", "Revisão de código", Tone::Deep) }),
                ],
            },
        ],
        note: "5 salas neste aparelho".into(),
    }
}

pub fn scenes() -> Vec<Scene<HomeView>> {
    let carregando = few().into_iter().map(|r| SavedRoomView { state: SavedRoomState::Loading, ..r }).collect();
    let menu = few().into_iter().enumerate().map(|(i, r)| SavedRoomView { menu_open: Some(i == 0), ..r }).collect();
    let sumiu = {
        let f = few();
        vec![
            f[0].clone(),
            f[1].clone(),
            SavedRoomView { state: SavedRoomState::Gone, ..row("w9c-b3n", "Minecraft do Caio", Tone::Deep) },
            f[2].clone(),
        ]
    };
    vec![
        Scene::new("default", "D1", "Página inicial", base()),
        Scene::new("empty", "D1", "Primeira visita (sem apelido)", HomeView { nickname: String::new(), ..base() }),
        Scene::new(
            "error",
            "D1",
            "Link inválido",
            HomeView { link: "https://youtu.be/dQw4w9WgXcQ".into(), link_error: Some(LINK_ERROR.into()), ..base() },
        ),
        Scene::new("invite", "D1", "Convite do app (Windows)", HomeView { app_invite: Some(AppInvite::Offer), ..base() }),
        Scene::new("invitehint", "D1", "Convite: baixando", HomeView { app_invite: Some(AppInvite::Hint), ..base() }),
        // Salas fixas (specs/011).
        Scene::new("first", "F1", "Sem salas fixas · convite", base()),
        Scene::new(
            "create",
            "F7",
            "Modal Criar sala fixa",
            HomeView {
                fixed_dialog: Some(NameDialogView {
                    mode: NameDialogMode::Create,
                    value: "CS da galera".into(),
                    busy: false,
                    error: None,
                }),
                ..base()
            },
        ),
        Scene::new("few", "F2", "Suas salas · 3 salas", HomeView { rooms: Some(uma_lista(few())), ..base() }),
        Scene::new("menu", "F3", "Menu do item (dono)", HomeView { rooms: Some(uma_lista(menu)), ..base() }),
        Scene::new("many", "F4", "Suas salas · 5, agrupadas", HomeView { rooms: Some(muitas()), ..base() }),
        Scene::new("loading", "F5", "Carregando quem está em cada sala", HomeView { rooms: Some(uma_lista(carregando)), ..base() }),
        Scene::new("gone", "F6", "Sala que deixou de existir", HomeView { rooms: Some(uma_lista(sumiu)), ..base() }),
        Scene::new(
            "removed",
            "R3",
            "Tirado da sala pelo dono (specs/014)",
            HomeView { rooms: Some(uma_lista(few())), notice: Some("O dono te tirou da sala".into()), ..base() },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treze_cenas_com_os_ids_do_site() {
        let ids: Vec<_> = scenes().into_iter().map(|c| c.id).collect();
        assert_eq!(
            ids,
            ["default", "empty", "error", "invite", "invitehint", "first", "create", "few", "menu", "many", "loading", "gone", "removed"]
        );
    }

    #[test]
    fn suas_salas_tem_no_maximo_5() {
        for c in scenes() {
            if let Some(r) = c.view.rooms {
                let total: usize = r.groups.iter().map(|g| g.rooms.len()).sum();
                assert!(total <= 5, "{} tem {total} salas", c.id);
            }
        }
    }
}
