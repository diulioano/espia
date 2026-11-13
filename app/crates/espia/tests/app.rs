//! O `App` sem Slint (T034 a T036): entradas de um lado, `Screen` do outro, servidor falso no meio.

mod comum;

use comum::*;
use espia_core::view_model::{
    HomeAction, HomeView, JoinAction, JoinStatus, JoinView, LinkState, NotFoundAction, NotFoundReason, NotFoundView,
    RoomAction, RoomKind, RoomView, SavedRoomState,
};
use espia_ui::{Screen, UiAction};
use serde_json::json;

fn home(t: &Screen) -> Option<&HomeView> {
    if let Screen::Home(v) = t { Some(v) } else { None }
}
fn join(t: &Screen) -> Option<&JoinView> {
    if let Screen::Join(v) = t { Some(v) } else { None }
}
fn sala(t: &Screen) -> Option<&RoomView> {
    if let Screen::Room(v) = t { Some(v) } else { None }
}
fn nao_achada(t: &Screen) -> Option<&NotFoundView> {
    if let Screen::NotFound(v) = t { Some(v) } else { None }
}
fn nomes(v: &RoomView) -> Vec<&str> {
    v.participants.iter().map(|p| p.name.as_str()).collect()
}

/// Servidor que sabe criar salas e responder a prévia de `abc-def`.
fn site() -> impl Fn(&Req) -> (u16, String) + Send + Sync + 'static {
    |r| match (r.method.as_str(), r.path.as_str()) {
        ("POST", "/api/rooms") if r.body.contains("fixed") => resp(201, json!({ "code": "k7f-q2x", "ownerKey": CHAVE })),
        ("POST", "/api/rooms") => resp(201, json!({ "code": "abc-def" })),
        ("GET", "/api/rooms/abc-def") => resp(
            200,
            json!({ "code": "abc-def", "capacity": 10, "people": [{ "nickname": "Beto", "tone": "sage", "live": true }] }),
        ),
        ("GET", "/api/rooms/zzz-zzz") => resp(404, json!({ "error": "not-found" })),
        _ => resp(404, json!({})),
    }
}

/// Entra na sala `abc-def` pela tela de entrada e responde ao `join` com um `welcome`.
async fn entrar(falso: &mut Falso, j: &mut Janela, outros: Vec<serde_json::Value>) -> Conn {
    j.tela(|t| join(t).is_some_and(|v| v.people.is_some())).await;
    j.acao(UiAction::Join(JoinAction::Join { nickname: "Ana".into() }));
    let mut c = falso.conexao().await;
    assert_eq!(c.json().await["t"], "join");
    let w = welcome("Ana", outros, None);
    confere_protocolo(&w);
    c.mandar(w).await;
    j.tela(|t| sala(t).is_some_and(|v| v.status == espia_core::view_model::RoomStatus::Ready)).await;
    c
}

fn hoje_menos_dias(dias: i64) -> i64 {
    agora_ms() - dias * 24 * 60 * 60_000
}

// ---------------------------------------------------------------- tela inicial

#[tokio::test]
async fn abrir_cai_na_tela_inicial_com_as_salas_guardadas() {
    let falso = servidor(|r| match r.path.as_str() {
        "/api/rooms/status" => resp(
            200,
            json!({ "rooms": [{ "code": "aaa-aaa", "state": "ok", "name": "Filme de sábado", "people": 2, "streaming": 1 }] }),
        ),
        _ => resp(404, json!({})),
    })
    .await;
    let disco = json!({
        "version": 1, "nickname": "Ana",
        "rooms": [{ "code": "aaa-aaa", "name": "Filme", "lastUsed": hoje_menos_dias(1) }],
        "owners": { "aaa-aaa": CHAVE }
    })
    .to_string();
    let mut j = abrir(&falso.base, Some(&disco), None);

    // Primeiro com os números ainda carregando.
    let t = j.tela(|t| home(t).is_some()).await;
    let h = home(&t).unwrap();
    assert_eq!(h.nickname, "Ana");
    let sala0 = &h.rooms.as_ref().unwrap().groups[0].rooms[0];
    assert_eq!((sala0.code.as_str(), sala0.state, sala0.own), ("aaa-aaa", SavedRoomState::Loading, true));
    assert_eq!(sala0.recent, "usada ontem");

    // Depois com os números do servidor, e o nome novo vai para o disco.
    let t = j.tela(|t| home(t).is_some_and(|h| h.rooms.as_ref().is_some_and(|r| r.groups[0].rooms[0].state == SavedRoomState::Ok))).await;
    let linha = &home(&t).unwrap().rooms.as_ref().unwrap().groups[0].rooms[0];
    assert_eq!((linha.people, linha.streaming, linha.name.as_str()), (2, 1, "Filme de sábado"));
    assert_eq!(j.device().rooms[0].name, "Filme de sábado");

    let pedido = &falso.pedidos_de("POST", "/api/rooms/status")[0];
    assert_eq!(pedido.header("origin"), Some(falso.base.as_str()));
    assert_eq!(pedido.json(), json!({ "codes": ["aaa-aaa"] }));
}

#[tokio::test]
async fn sala_que_acabou_perde_a_chave_de_dono() {
    let falso = servidor(|_| resp(200, json!({ "rooms": [{ "code": "aaa-aaa", "state": "gone", "reason": "deleted" }] }))).await;
    let disco = json!({
        "version": 1, "nickname": "Ana",
        "rooms": [{ "code": "aaa-aaa", "name": "Filme", "lastUsed": hoje_menos_dias(1) }],
        "owners": { "aaa-aaa": CHAVE }
    })
    .to_string();
    let mut j = abrir(&falso.base, Some(&disco), None);
    let t = j.tela(|t| home(t).is_some_and(|h| h.rooms.as_ref().is_some_and(|r| r.groups[0].rooms[0].state == SavedRoomState::Gone))).await;
    assert!(!home(&t).unwrap().rooms.as_ref().unwrap().groups[0].rooms[0].own);
    assert!(j.device().owners.is_empty());
    assert_eq!(j.device().rooms.len(), 1, "a sala continua em Suas salas (aparece como apagada)");
}

#[tokio::test]
async fn sem_salas_guardadas_nao_pergunta_ao_servidor() {
    let falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, None);
    let t = j.tela(|t| home(t).is_some()).await;
    assert!(home(&t).unwrap().rooms.is_none());
    assert_eq!(home(&t).unwrap().nickname, "");
    assert!(falso.pedidos().is_empty());
}

#[tokio::test]
async fn criar_sala_rapida_navega_para_a_sala() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, None);
    j.tela(|t| home(t).is_some()).await;

    j.acao(UiAction::Home(HomeAction::CreateRoom { nickname: "  Ana  ".into() }));
    j.tela(|t| home(t).is_some_and(|h| h.creating == Some(true))).await;
    let mut c = falso.conexao().await;
    assert_eq!(c.caminho, "/ws/abc-def");
    assert_eq!(c.origin.as_deref(), Some(falso.base.as_str()));
    assert_eq!(c.json().await, json!({ "t": "join", "nickname": "Ana", "mediaProtocol": 2 }));
    assert_eq!(falso.pedidos_de("POST", "/api/rooms")[0].body, "");

    c.mandar(welcome("Ana", vec![], None)).await;
    let t = j.tela(|t| sala(t).is_some_and(|v| v.status == espia_core::view_model::RoomStatus::Ready)).await;
    let v = sala(&t).unwrap();
    assert_eq!((v.room_id.as_str(), v.room.clone()), ("abc-def", RoomKind::Quick));
    assert_eq!(nomes(v), ["Ana"]);
    assert_eq!(j.device().nickname.as_deref(), Some("Ana"));
    assert!(j.copias.is_empty(), "sala rápida não copia o convite sozinha");
}

#[tokio::test]
async fn criar_sala_rapida_mostra_o_erro_do_site() {
    let falso = servidor(|_| resp(429, json!({ "error": "rate-limited" }))).await;
    let mut j = abrir(&falso.base, None, None);
    j.tela(|t| home(t).is_some()).await;
    j.acao(UiAction::Home(HomeAction::CreateRoom { nickname: "Ana".into() }));
    let t = j.tela(|t| home(t).is_some_and(|h| h.create_error.is_some())).await;
    let h = home(&t).unwrap();
    assert_eq!(h.create_error.as_deref(), Some("Muitas salas criadas daqui agora. Tenta de novo em um minuto."));
    assert_eq!(h.creating, Some(false));
}

#[tokio::test]
async fn criar_sala_fixa_guarda_a_chave_lembra_a_sala_e_copia_o_convite_oficial() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, None);
    j.tela(|t| home(t).is_some()).await;

    j.acao(UiAction::Home(HomeAction::OpenFixedDialog));
    j.tela(|t| home(t).is_some_and(|h| h.fixed_dialog.is_some())).await;
    j.acao(UiAction::Home(HomeAction::CreateFixedRoom { nickname: "Ana".into(), name: "Noite de jogo".into() }));
    let t = j.tela(|t| home(t).is_some_and(|h| h.fixed_dialog.as_ref().is_some_and(|d| d.busy))).await;
    assert_eq!(home(&t).unwrap().fixed_dialog.as_ref().unwrap().value, "Noite de jogo");

    let mut c = falso.conexao().await;
    assert_eq!(c.caminho, "/ws/k7f-q2x");
    assert_eq!(c.json().await["t"], "join");
    assert_eq!(falso.pedidos_de("POST", "/api/rooms")[0].json(), json!({ "kind": "fixed", "name": "Noite de jogo" }));

    // A chave e a sala já estão no disco antes mesmo de conectar.
    let d = j.device();
    assert_eq!(d.owners.get("k7f-q2x").map(String::as_str), Some(CHAVE));
    assert_eq!(d.rooms[0].code, "k7f-q2x");
    assert_eq!(d.rooms[0].name, "Noite de jogo");

    c.mandar(welcome("Ana", vec![], Some(json!({ "kind": "fixed", "name": "Noite de jogo" })))).await;
    let t = j.tela(|t| sala(t).is_some_and(|v| v.status == espia_core::view_model::RoomStatus::Ready)).await;
    assert_eq!(sala(&t).unwrap().room, RoomKind::Fixed { name: "Noite de jogo".into(), own: true });
    assert_eq!(j.copia().await, "https://espia.diuliano.com/s/k7f-q2x", "o convite usa o endereço oficial, não o servidor local");
}

#[tokio::test]
async fn nome_de_sala_fixa_recusado_mostra_o_erro_no_dialogo() {
    let falso = servidor(|_| resp(400, json!({ "error": "name" }))).await;
    let mut j = abrir(&falso.base, None, None);
    j.tela(|t| home(t).is_some()).await;
    j.acao(UiAction::Home(HomeAction::OpenFixedDialog));
    j.acao(UiAction::Home(HomeAction::CreateFixedRoom { nickname: "Ana".into(), name: "Sala".into() }));
    let t = j.tela(|t| home(t).is_some_and(|h| h.fixed_dialog.as_ref().is_some_and(|d| d.error.is_some()))).await;
    let d = home(&t).unwrap().fixed_dialog.clone().unwrap();
    assert_eq!((d.busy, d.error.as_deref()), (false, Some("Esse nome não dá. Use letras, números e espaços.")));
}

#[tokio::test]
async fn colar_link_ou_codigo_entra_na_previa() {
    for texto in ["https://espia.diuliano.com/s/abc-def", "ABC DEF"] {
        let falso = servidor(site()).await;
        let mut j = abrir(&falso.base, None, None);
        j.tela(|t| home(t).is_some()).await;
        j.acao(UiAction::Home(HomeAction::OpenLink { text: texto.into() }));
        let t = j.tela(|t| join(t).is_some_and(|v| v.people.is_some())).await;
        let v = join(&t).unwrap();
        assert_eq!(v.room_id, "abc-def");
        assert_eq!(v.status, JoinStatus::Ready);
        let gente = v.people.as_ref().unwrap();
        assert_eq!((gente[0].name.as_str(), gente[0].live), ("Beto", true));
        assert_eq!(falso.pedidos_de("GET", "/api/rooms/abc-def")[0].header("origin"), Some(falso.base.as_str()));
    }
}

#[tokio::test]
async fn codigo_invalido_mostra_o_erro_do_site_e_nao_vai_a_rede() {
    let falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, None);
    j.tela(|t| home(t).is_some()).await;
    j.acao(UiAction::Home(HomeAction::OpenLink { text: "isso não é sala".into() }));
    let t = j.tela(|t| home(t).is_some_and(|h| h.link_error.is_some())).await;
    assert_eq!(home(&t).unwrap().link_error.as_deref(), Some("Esse link não parece de uma sala."));
    j.acao(UiAction::Home(HomeAction::ClearLinkError));
    j.tela(|t| home(t).is_some_and(|h| h.link_error.is_none())).await;
    assert!(falso.pedidos().is_empty());
}

#[tokio::test]
async fn entrar_pela_sala_guardada_com_apelido_vai_direto_e_sem_apelido_pede() {
    let mut falso = servidor(site()).await;
    let disco = json!({
        "version": 1, "rooms": [{ "code": "abc-def", "name": "Sala", "lastUsed": hoje_menos_dias(0) }]
    })
    .to_string();
    let mut j = abrir(&falso.base, Some(&disco), None);
    j.tela(|t| home(t).is_some()).await;
    // Sem apelido: a tela de entrada pergunta.
    j.acao(UiAction::Home(HomeAction::EnterSaved { code: "abc-def".into(), nickname: String::new() }));
    j.tela(|t| join(t).is_some()).await;
    j.acao(UiAction::Join(JoinAction::GoHome));
    j.tela(|t| home(t).is_some()).await;
    // Com apelido: entra direto.
    j.acao(UiAction::Home(HomeAction::EnterSaved { code: "abc-def".into(), nickname: "Ana".into() }));
    let mut c = falso.conexao().await;
    assert_eq!(c.json().await, json!({ "t": "join", "nickname": "Ana", "mediaProtocol": 2 }));
    assert_eq!(j.device().nickname.as_deref(), Some("Ana"));
}

#[tokio::test]
async fn apagar_e_renomear_sala_guardada_pelo_menu_do_dono() {
    let falso = servidor(|r| match (r.method.as_str(), r.path.as_str()) {
        ("POST", "/api/rooms/status") => resp(200, json!({ "rooms": [{ "code": "aaa-aaa", "state": "ok", "name": null, "people": 3, "streaming": 0 }] })),
        ("PATCH", "/api/rooms/aaa-aaa") => resp(200, json!({ "name": "Cinema" })),
        ("DELETE", "/api/rooms/aaa-aaa") => resp(200, json!({})),
        _ => resp(404, json!({})),
    })
    .await;
    let disco = json!({
        "version": 1, "nickname": "Ana",
        "rooms": [{ "code": "aaa-aaa", "name": "Filme", "lastUsed": hoje_menos_dias(1) }],
        "owners": { "aaa-aaa": CHAVE }
    })
    .to_string();
    let mut j = abrir(&falso.base, Some(&disco), None);
    j.tela(|t| home(t).is_some_and(|h| h.rooms.as_ref().is_some_and(|r| r.groups[0].rooms[0].state == SavedRoomState::Ok))).await;

    j.acao(UiAction::Home(HomeAction::ToggleSavedMenu { code: Some("aaa-aaa".into()) }));
    j.tela(|t| home(t).is_some_and(|h| h.rooms.as_ref().is_some_and(|r| r.groups[0].rooms[0].menu_open == Some(true)))).await;
    j.acao(UiAction::Home(HomeAction::RenameSaved { code: "aaa-aaa".into() }));
    let t = j.tela(|t| home(t).is_some_and(|h| h.fixed_dialog.is_some())).await;
    assert_eq!(home(&t).unwrap().fixed_dialog.as_ref().unwrap().value, "Filme");
    j.acao(UiAction::Home(HomeAction::SubmitRename { name: "Cinema".into() }));
    j.tela(|t| home(t).is_some_and(|h| h.fixed_dialog.is_none() && h.rooms.as_ref().is_some_and(|r| r.groups[0].rooms[0].name == "Cinema"))).await;
    let patch = &falso.pedidos_de("PATCH", "/api/rooms/aaa-aaa")[0];
    assert_eq!(patch.header("authorization"), Some(format!("Bearer {CHAVE}").as_str()));
    assert_eq!(j.device().rooms[0].name, "Cinema");

    j.acao(UiAction::Home(HomeAction::DeleteSaved { code: "aaa-aaa".into() }));
    let t = j.tela(|t| home(t).is_some_and(|h| h.delete_dialog.is_some())).await;
    assert_eq!(home(&t).unwrap().delete_dialog.as_ref().unwrap().people, 3);
    j.acao(UiAction::Home(HomeAction::ConfirmDelete));
    j.tela(|t| home(t).is_some_and(|h| h.delete_dialog.is_none() && h.rooms.is_none())).await;
    assert!(j.device().rooms.is_empty() && j.device().owners.is_empty());
}

// ---------------------------------------------------------------- entrada

#[tokio::test]
async fn previa_de_sala_inexistente_vai_para_nao_encontrada() {
    let falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("espia://s/zzz-zzz"));
    let t = j.tela(|t| nao_achada(t).is_some()).await;
    assert_eq!(*nao_achada(&t).unwrap(), NotFoundView { reason: NotFoundReason::Quick, name: None, saved: false });
}

#[tokio::test]
async fn sala_fixa_apagada_ou_vencida_mostra_o_nome_guardado_e_tira_de_suas_salas() {
    let falso = servidor(|r| match r.path.as_str() {
        "/api/rooms/aaa-aaa" => resp(410, json!({ "reason": "expired" })),
        _ => resp(404, json!({})),
    })
    .await;
    let disco = json!({
        "version": 1, "rooms": [{ "code": "aaa-aaa", "name": "Filme", "lastUsed": hoje_menos_dias(2) }],
        "owners": { "aaa-aaa": CHAVE }
    })
    .to_string();
    let mut j = abrir(&falso.base, Some(&disco), Some("aaa-aaa"));
    let t = j.tela(|t| nao_achada(t).is_some()).await;
    assert_eq!(
        *nao_achada(&t).unwrap(),
        NotFoundView { reason: NotFoundReason::Expired, name: Some("Filme".into()), saved: true }
    );
    assert!(j.device().owners.is_empty(), "a chave de dono não serve mais");
    j.acao(UiAction::NotFound(NotFoundAction::Forget));
    j.tela(|t| home(t).is_some_and(|h| h.rooms.is_none())).await;
    assert!(j.device().rooms.is_empty());
}

#[tokio::test]
async fn sem_resposta_da_previa_a_entrada_segue_sem_a_pilha_de_avatares() {
    let falso = servidor(|_| resp(500, json!({}))).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    let t = j.tela(|t| join(t).is_some()).await;
    // A primeira tela já é a de entrada e continua assim quando a prévia falha.
    assert!(join(&t).unwrap().people.is_none());
    j.esperar(300).await; // dá tempo à resposta; nada muda
    assert_eq!(j.ultima.as_ref().and_then(join).map(|v| v.status), Some(JoinStatus::Ready));
}

#[tokio::test]
async fn entrar_manda_join_com_protocolo_2_e_mostra_as_pessoas_do_welcome() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("espia://s/abc-def"));
    let mut c = entrar(&mut falso, &mut j, vec![pessoa(BETO, "Beto", "sage")]).await;
    assert_eq!(c.caminho, "/ws/abc-def");
    assert_eq!(c.origin.as_deref(), Some(falso.base.as_str()));
    let t = j.ultima.clone().unwrap();
    let v = sala(&t).unwrap();
    assert_eq!(nomes(v), ["Ana", "Beto"]);
    assert_eq!(v.capacity, 10);
    assert_eq!(j.device().nickname.as_deref(), Some("Ana"));
    assert!(j.device().sessions.contains_key("abc-def"), "a sessão é guardada para retomar o lugar");
    // Nenhuma conversa a mais: só o `join`.
    let _ = &mut c;
}

#[tokio::test]
async fn peer_joined_e_peer_left_atualizam_a_lista() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    let mut c = entrar(&mut falso, &mut j, vec![]).await;

    c.mandar(json!({ "t": "peer-joined", "peer": pessoa(BETO, "Beto", "sage") }).to_string()).await;
    let t = j.tela(|t| sala(t).is_some_and(|v| v.participants.len() == 2)).await;
    assert_eq!(nomes(sala(&t).unwrap()), ["Ana", "Beto"]);
    assert!(sala(&t).unwrap().toasts.iter().any(|x| x.text == "Beto entrou na sala"));

    c.mandar(json!({ "t": "peer-left", "id": BETO }).to_string()).await;
    let t = j.tela(|t| sala(t).is_some_and(|v| v.participants.len() == 1)).await;
    assert_eq!(nomes(sala(&t).unwrap()), ["Ana"]);
}

#[tokio::test]
async fn sair_volta_para_o_inicio_e_avisa_o_servidor() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    let mut c = entrar(&mut falso, &mut j, vec![]).await;
    j.acao(UiAction::Room(RoomAction::Leave));
    j.tela(|t| home(t).is_some()).await;
    assert_eq!(c.codigo_de_fechamento().await, Some(4001));
    assert!(!j.device().sessions.contains_key("abc-def"), "quem sai de propósito não retoma o lugar");
}

#[tokio::test]
async fn queda_mostra_reconectando_e_a_volta_retoma_com_resume() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    let c = entrar(&mut falso, &mut j, vec![]).await;

    drop(c); // a internet caiu
    let t = j.tela(|t| sala(t).is_some_and(|v| v.link == LinkState::Reconnecting)).await;
    assert_eq!(nomes(sala(&t).unwrap()), ["Ana"], "a sala continua na tela");

    let mut c2 = falso.conexao().await;
    assert_eq!(
        c2.json().await,
        json!({ "t": "resume", "peerId": EU, "resumeToken": "tok-1", "mediaProtocol": 2 })
    );
    c2.mandar(welcome("Ana", vec![], None)).await;
    let t = j.tela(|t| sala(t).is_some_and(|v| v.link == LinkState::Ok && v.toasts.iter().any(|x| x.text == "De volta!"))).await;
    assert_eq!(nomes(sala(&t).unwrap()), ["Ana"]);
}

#[tokio::test]
async fn sessao_guardada_ha_menos_de_24_horas_retoma_sem_pedir_apelido() {
    let mut falso = servidor(site()).await;
    let disco = json!({
        "version": 1, "nickname": "Ana",
        "sessions": { "abc-def": { "peerId": EU, "resumeToken": "tok-velho", "at": agora_ms() - 3_600_000 } }
    })
    .to_string();
    let _j = abrir(&falso.base, Some(&disco), Some("https://espia.diuliano.com/s/abc-def"));
    let mut c = falso.conexao().await;
    assert_eq!(c.json().await, json!({ "t": "resume", "peerId": EU, "resumeToken": "tok-velho", "mediaProtocol": 2 }));
}

#[tokio::test]
async fn sessao_de_mais_de_24_horas_nao_vale_e_a_entrada_pede_o_apelido() {
    let falso = servidor(site()).await;
    let disco = json!({
        "version": 1, "nickname": "Ana",
        "sessions": { "abc-def": { "peerId": EU, "resumeToken": "tok-velho", "at": agora_ms() - 25 * 3_600_000 } }
    })
    .to_string();
    let mut j = abrir(&falso.base, Some(&disco), Some("abc-def"));
    let t = j.tela(|t| join(t).is_some_and(|v| v.people.is_some())).await;
    assert_eq!(join(&t).unwrap().nickname, "Ana", "o apelido lembrado já vem preenchido");
}

// ---------------------------------------------------------------- fechamentos

#[tokio::test]
async fn sala_cheia_volta_para_a_entrada_com_o_aviso() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    j.tela(|t| join(t).is_some_and(|v| v.people.is_some())).await;
    j.acao(UiAction::Join(JoinAction::Join { nickname: "Ana".into() }));
    let mut c = falso.conexao().await;
    c.json().await;
    c.fechar_com(4409).await;
    let t = j.tela(|t| join(t).is_some_and(|v| v.status == JoinStatus::Full)).await;
    assert_eq!(join(&t).unwrap().room_id, "abc-def");
}

#[tokio::test]
async fn sala_apagada_com_voce_dentro_vai_para_nao_encontrada_apagada() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    let mut c = entrar(&mut falso, &mut j, vec![]).await;
    c.mandar(json!({ "t": "room-deleted" }).to_string()).await;
    let t = j.tela(|t| nao_achada(t).is_some()).await;
    assert_eq!(nao_achada(&t).unwrap().reason, NotFoundReason::Kicked);
}

#[tokio::test]
async fn removido_pelo_dono_volta_ao_inicio_com_o_aviso() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    let mut c = entrar(&mut falso, &mut j, vec![]).await;
    c.mandar(json!({ "t": "removed" }).to_string()).await;
    let t = j.tela(|t| home(t).is_some()).await;
    assert_eq!(home(&t).unwrap().notice.as_deref(), Some("O dono te tirou da sala"));
    j.acao(UiAction::Home(HomeAction::DismissNotice));
    j.tela(|t| home(t).is_some_and(|h| h.notice.is_none())).await;
}

#[tokio::test]
async fn aberta_em_outra_janela_cede_o_lugar_e_explica() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    let mut c = entrar(&mut falso, &mut j, vec![]).await;
    c.fechar_com(4000).await;
    let t = j.tela(|t| join(t).is_some_and(|v| v.replaced == Some(true))).await;
    assert_eq!(join(&t).unwrap().room_id, "abc-def");
    // E a prévia é buscada de novo.
    j.tela(|t| join(t).is_some_and(|v| v.replaced == Some(true) && v.people.is_some())).await;
}

#[tokio::test]
async fn servidor_mais_novo_pede_para_atualizar_e_atualizar_abre_a_sala_de_novo() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    j.tela(|t| join(t).is_some_and(|v| v.people.is_some())).await;
    j.acao(UiAction::Join(JoinAction::Join { nickname: "Ana".into() }));
    let mut c = falso.conexao().await;
    c.json().await;
    c.mandar(welcome("Ana", vec![], None).replace("\"mediaProtocol\":2", "\"mediaProtocol\":3")).await;
    let t = j.tela(|t| sala(t).is_some_and(|v| v.requires_update == Some(true))).await;
    assert_eq!(sala(&t).unwrap().room_id, "abc-def");

    j.acao(UiAction::Reload);
    let mut c2 = falso.conexao().await;
    assert_eq!(c2.json().await["t"], "join");
}

#[tokio::test]
async fn pedir_para_transmitir_nao_trava_a_tela() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    let _c = entrar(&mut falso, &mut j, vec![]).await;
    j.acao(UiAction::Room(RoomAction::OpenGoLive));
    let t = j.tela(|t| sala(t).is_some_and(|v| v.go_live_open)).await;
    assert!(sala(&t).unwrap().picker.sources.is_empty());
    // Ainda dá para fechar o seletor.
    j.acao(UiAction::Room(RoomAction::CloseGoLive));
    j.tela(|t| sala(t).is_some_and(|v| !v.go_live_open)).await;
}

#[tokio::test]
async fn copiar_convite_na_sala_usa_o_endereco_oficial() {
    let mut falso = servidor(site()).await;
    let mut j = abrir(&falso.base, None, Some("abc-def"));
    let _c = entrar(&mut falso, &mut j, vec![]).await;
    j.acao(UiAction::Room(RoomAction::CopyInvite));
    assert_eq!(j.copia().await, "https://espia.diuliano.com/s/abc-def");
    j.tela(|t| sala(t).is_some_and(|v| v.copied)).await;
}
