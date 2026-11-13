//! Conferência contra o site local de verdade (`npm run dev:worker`), não contra o servidor falso.
//! Fica `#[ignore]`: roda só com `ESPIA_SERVER=http://127.0.0.1:8787 cargo test -p espia --test site_local -- --ignored <nome>`.
//! Um teste por execução: o cliente HTTP da `espia-net` é um `static` que nasce no runtime do primeiro teste, e o site
//! de verdade mantém a conexão aberta (o servidor falso não), então um segundo runtime no mesmo processo perde a conexão.

mod comum;

use comum::*;
use espia_core::view_model::{HomeAction, JoinAction, RoomAction, RoomKind, RoomStatus};
use espia_ui::{Screen, UiAction};

fn pronta(t: &Screen) -> bool {
    matches!(t, Screen::Room(v) if v.status == RoomStatus::Ready)
}

#[tokio::test]
#[ignore = "precisa do site local no ar (ESPIA_SERVER)"]
async fn ana_cria_sala_fixa_e_beto_entra_pelo_link() {
    let base = std::env::var("ESPIA_SERVER").expect("defina ESPIA_SERVER=http://127.0.0.1:8787");
    let mut ana = abrir(&base, None, None);
    ana.tela(|t| matches!(t, Screen::Home(_))).await;
    ana.acao(UiAction::Home(HomeAction::OpenFixedDialog));
    ana.acao(UiAction::Home(HomeAction::CreateFixedRoom { nickname: "Ana".into(), name: "Sala do teste".into() }));
    let t = ana.tela(pronta).await;
    let Screen::Room(v) = &t else { unreachable!() };
    assert_eq!(v.room, RoomKind::Fixed { name: "Sala do teste".into(), own: true });
    let code = v.room_id.clone();
    assert_eq!(ana.copia().await, format!("https://espia.diuliano.com/s/{code}"));
    assert_eq!(ana.device().owners.get(&code).map(String::len), Some(64));

    // Beto abre o link: vê Ana na prévia, entra, e Ana vê Beto chegar.
    let mut beto = abrir(&base, None, Some(&format!("espia://s/{code}")));
    let t = beto.tela(|t| matches!(t, Screen::Join(v) if v.people.is_some())).await;
    let Screen::Join(j) = &t else { unreachable!() };
    assert_eq!(j.fixed_name.as_deref(), Some("Sala do teste"));
    assert_eq!(j.people.as_ref().unwrap()[0].name, "Ana");
    beto.acao(UiAction::Join(JoinAction::Join { nickname: "Beto".into() }));
    beto.tela(pronta).await;
    let t = ana.tela(|t| matches!(t, Screen::Room(v) if v.participants.len() == 2)).await;
    let Screen::Room(v) = &t else { unreachable!() };
    assert_eq!(v.participants[1].name, "Beto");

    beto.acao(UiAction::Room(RoomAction::Leave));
    beto.tela(|t| matches!(t, Screen::Home(_))).await;
    ana.tela(|t| matches!(t, Screen::Room(v) if v.participants.len() == 1)).await;

    // Ana sai e a sala fica em Suas salas com os números do servidor.
    ana.acao(UiAction::Room(RoomAction::Leave));
    let t = ana.tela(|t| matches!(t, Screen::Home(h) if h.rooms.as_ref().is_some_and(|r| r.groups[0].rooms[0].state != espia_core::view_model::SavedRoomState::Loading))).await;
    let Screen::Home(h) = &t else { unreachable!() };
    let linha = &h.rooms.as_ref().unwrap().groups[0].rooms[0];
    assert_eq!((linha.code.as_str(), linha.name.as_str(), linha.own), (code.as_str(), "Sala do teste", true));
}

#[tokio::test]
#[ignore = "precisa do site local no ar (ESPIA_SERVER)"]
async fn sala_rapida_e_sala_que_nao_existe() {
    let base = std::env::var("ESPIA_SERVER").expect("defina ESPIA_SERVER=http://127.0.0.1:8787");
    let mut ana = abrir(&base, None, None);
    ana.tela(|t| matches!(t, Screen::Home(_))).await;
    ana.acao(UiAction::Home(HomeAction::CreateRoom { nickname: "Ana".into() }));
    let t = ana.tela(pronta).await;
    let Screen::Room(v) = &t else { unreachable!() };
    assert_eq!(v.room, RoomKind::Quick);
    assert_eq!(v.participants.len(), 1);

    let mut outro = abrir(&base, None, Some("zzz-zzz"));
    outro.tela(|t| matches!(t, Screen::NotFound(_))).await;
}
