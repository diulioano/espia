//! Liga `HomeView` à tela `ui/pages/home.slint`. Os textos derivados (linha de baixo de cada sala, aviso do diálogo de
//! apagar) e as regras de corte do apelido (24) e do nome da sala (40) moram aqui, a partir de `espia-core`.

use crate::{
    AppWindow, Emit, HomeState, InviteKind, SavedGroupData, SavedRoomData, SavedState, ToastData, ToastKind, UiAction,
    bind::{initial, tone},
};
use espia_core::device::{NICKNAME_MAX, ROOM_NAME_MAX};
use espia_core::join_view::clean_nickname;
use espia_core::room_name::normalize_room_name;
use espia_core::view_model::{
    AppInvite, DeleteDialogView, HomeAction, HomeView, NameDialogMode, SavedRoomState, SavedRoomView, SavedRoomsView,
};
use slint::{ComponentHandle, ModelRc, VecModel};

/// Linha de baixo da sala: "3 pessoas agora · 1 transmitindo" ou "Vazia · usada ontem" (`savedRoomSub`).
pub(crate) fn saved_room_sub(room: &SavedRoomView) -> String {
    if room.people > 0 {
        let quem = format!("{} {} agora", room.people, if room.people == 1 { "pessoa" } else { "pessoas" });
        if room.streaming > 0 { format!("{quem} · {} transmitindo", room.streaming) } else { quem }
    } else {
        format!("Vazia · {}", room.recent)
    }
}

/// Título do diálogo de apagar: `Apagar "nome"?`.
pub(crate) fn delete_title(name: &str) -> String {
    format!("Apagar \"{name}\"?")
}

/// Aviso de quem está na sala; vazio quando ninguém está.
pub(crate) fn delete_warn(people: u32) -> String {
    match people {
        0 => String::new(),
        1 => "Tem 1 pessoa aqui agora. Ela sai da sala e vê um aviso.".into(),
        n => format!("Tem {n} pessoas aqui agora. Elas saem da sala e veem um aviso."),
    }
}

/// Corta em `max` caracteres (emoji conta 1, como o `[...value]` do site).
pub(crate) fn cortar(texto: &str, max: usize) -> String {
    texto.chars().take(max).collect()
}

/// A sala com o menu "⋯" aberto, se houver (o menu é desenhado uma vez, pela página).
fn sala_com_menu(view: &HomeView) -> Option<&SavedRoomView> {
    view.rooms.as_ref()?.groups.iter().flat_map(|g| &g.rooms).find(|r| r.menu_open.unwrap_or(false))
}

fn sala(r: &SavedRoomView) -> SavedRoomData {
    SavedRoomData {
        code: r.code.as_str().into(),
        name: r.name.as_str().into(),
        letter: initial(&r.name).into(),
        tone: tone(r.tone),
        own: r.own,
        state: match r.state {
            SavedRoomState::Loading => SavedState::Loading,
            SavedRoomState::Ok => SavedState::Ok,
            SavedRoomState::Gone => SavedState::Gone,
        },
        sub: saved_room_sub(r).into(),
        live: r.state == SavedRoomState::Ok && r.streaming > 0,
        menu_open: r.menu_open.unwrap_or(false),
    }
}

fn grupos(rooms: &SavedRoomsView) -> Vec<SavedGroupData> {
    rooms
        .groups
        .iter()
        .map(|g| SavedGroupData {
            // O título é escrito em maiúsculas (a Slint não tem text-transform).
            label: g.label.as_deref().unwrap_or("").to_uppercase().into(),
            rooms: ModelRc::new(VecModel::from(g.rooms.iter().map(sala).collect::<Vec<_>>())),
        })
        .collect()
}

/// Copia o view-model para as propriedades de `HomeState`. Com `manter_texto`, o que a pessoa está digitando (apelido,
/// link, nome da sala fixa) não é sobrescrito: só vale o que o view-model traz quando a tela abre ou o diálogo abre.
pub(crate) fn apply(app: &AppWindow, view: &HomeView, manter_texto: bool) {
    let s = app.global::<HomeState>();
    if !manter_texto {
        s.set_nickname(view.nickname.as_str().into());
        s.set_link(view.link.as_str().into());
    }
    s.set_link_error(view.link_error.clone().unwrap_or_default().into());
    s.set_create_error(view.create_error.clone().unwrap_or_default().into());
    s.set_creating(view.creating.unwrap_or(false));
    s.set_invite(match view.app_invite {
        None => InviteKind::None,
        Some(AppInvite::Offer) => InviteKind::Offer,
        Some(AppInvite::Hint) => InviteKind::Hint,
    });
    let avisos: Vec<ToastData> = view
        .notice
        .iter()
        .map(|t| ToastData { id: "notice".into(), kind: ToastKind::Leave, text: t.as_str().into(), action: "".into() })
        .collect();
    s.set_notices(ModelRc::new(VecModel::from(avisos)));
    s.set_download(view.download.unwrap_or(false));

    s.set_has_rooms(view.rooms.is_some());
    let (nota, lista) = match &view.rooms {
        Some(r) => (r.note.clone(), grupos(r)),
        None => (String::new(), Vec::new()),
    };
    s.set_rooms_note(nota.into());
    s.set_groups(ModelRc::new(VecModel::from(lista)));
    let aberta = sala_com_menu(view);
    s.set_menu_open(aberta.is_some());
    s.set_menu_code(aberta.map(|r| r.code.as_str()).unwrap_or("").into());
    s.set_menu_own(aberta.map(|r| r.own).unwrap_or(false));

    match &view.fixed_dialog {
        Some(d) => {
            if !(manter_texto && s.get_fixed_open()) {
                s.set_fixed_value(d.value.as_str().into());
            }
            s.set_fixed_open(true);
            s.set_fixed_create(d.mode == NameDialogMode::Create);
            s.set_fixed_busy(d.busy);
            s.set_fixed_error(d.error.clone().unwrap_or_default().into());
        }
        None => s.set_fixed_open(false),
    }
    match &view.delete_dialog {
        Some(DeleteDialogView { name, people, busy, error }) => {
            s.set_delete_open(true);
            s.set_delete_title(delete_title(name).into());
            s.set_delete_warn(delete_warn(*people).into());
            s.set_delete_busy(*busy);
            s.set_delete_error(error.clone().unwrap_or_default().into());
        }
        None => s.set_delete_open(false),
    }
}

/// Liga os callbacks de `HomeState` às ações devolvidas por `emit`.
pub(crate) fn connect(app: &AppWindow, emit: &Emit) {
    let s = app.global::<HomeState>();
    s.on_limit_nickname(|t| cortar(&t, NICKNAME_MAX).into());
    s.on_clean_nickname(|t| clean_nickname(&t).into());
    s.on_limit_room_name(|t| cortar(&t, ROOM_NAME_MAX).into());
    s.on_normalize_room_name(|t| normalize_room_name(&t).unwrap_or_default().into());
    s.on_non_blank(|t| !t.trim().is_empty());

    // Cada callback abaixo faz o mesmo: monta a ação e entrega a `emit`.
    macro_rules! acao {
        ($on:ident, || $a:expr) => {{
            let e = emit.clone();
            s.$on(move || e(UiAction::Home($a)));
        }};
        ($on:ident, |$($p:ident),+| $a:expr) => {{
            let e = emit.clone();
            s.$on(move |$($p),*| e(UiAction::Home($a)));
        }};
    }
    acao!(on_create_room, |nick| HomeAction::CreateRoom { nickname: nick.to_string() });
    acao!(on_open_link, |text| HomeAction::OpenLink { text: text.to_string() });
    acao!(on_clear_link_error, || HomeAction::ClearLinkError);
    acao!(on_download_app, || HomeAction::DownloadApp);
    acao!(on_dismiss_app_invite, || HomeAction::DismissAppInvite);
    acao!(on_dismiss_notice, || HomeAction::DismissNotice);
    acao!(on_open_fixed_dialog, || HomeAction::OpenFixedDialog);
    acao!(on_close_fixed_dialog, || HomeAction::CloseFixedDialog);
    acao!(on_submit_fixed, |criar, nick, nome| {
        if criar {
            HomeAction::CreateFixedRoom { nickname: nick.to_string(), name: nome.to_string() }
        } else {
            HomeAction::SubmitRename { name: nome.to_string() }
        }
    });
    acao!(on_enter_saved, |code, nick| HomeAction::EnterSaved { code: code.to_string(), nickname: nick.to_string() });
    acao!(on_forget_saved, |code| HomeAction::ForgetSaved { code: code.to_string() });
    acao!(on_toggle_saved_menu, |code, aberto| HomeAction::ToggleSavedMenu { code: aberto.then(|| code.to_string()) });
    acao!(on_close_saved_menu, || HomeAction::ToggleSavedMenu { code: None });
    acao!(on_rename_saved, |code| HomeAction::RenameSaved { code: code.to_string() });
    acao!(on_delete_saved, |code| HomeAction::DeleteSaved { code: code.to_string() });
    acao!(on_confirm_delete, || HomeAction::ConfirmDelete);
    acao!(on_close_delete_dialog, || HomeAction::CloseDeleteDialog);
}

#[cfg(test)]
mod tests {
    use super::*;
    use espia_core::protocol::Tone;

    fn sala_de(people: u32, streaming: u32) -> SavedRoomView {
        SavedRoomView {
            code: "k7f-q2x".into(),
            name: "CS da galera".into(),
            own: true,
            tone: Tone::Accent,
            state: SavedRoomState::Ok,
            people,
            streaming,
            recent: "usada ontem".into(),
            menu_open: None,
        }
    }

    #[test]
    fn linha_de_baixo_como_no_site() {
        assert_eq!(saved_room_sub(&sala_de(3, 1)), "3 pessoas agora · 1 transmitindo");
        assert_eq!(saved_room_sub(&sala_de(1, 0)), "1 pessoa agora");
        assert_eq!(saved_room_sub(&sala_de(2, 0)), "2 pessoas agora");
        assert_eq!(saved_room_sub(&sala_de(0, 0)), "Vazia · usada ontem");
    }

    #[test]
    fn aviso_de_apagar_concorda_no_plural() {
        assert_eq!(delete_warn(0), "");
        assert_eq!(delete_warn(1), "Tem 1 pessoa aqui agora. Ela sai da sala e vê um aviso.");
        assert_eq!(delete_warn(2), "Tem 2 pessoas aqui agora. Elas saem da sala e veem um aviso.");
        assert_eq!(delete_title("CS da galera"), "Apagar \"CS da galera\"?");
    }

    #[test]
    fn apelido_ate_24_e_nome_de_sala_ate_40() {
        assert_eq!(cortar(&"a".repeat(30), NICKNAME_MAX).chars().count(), 24);
        assert_eq!(cortar(&"é".repeat(50), ROOM_NAME_MAX).chars().count(), 40);
        // Emoji conta 1 caractere.
        assert_eq!(cortar("😀😀😀", 2), "😀😀");
        assert_eq!(cortar("ok", 24), "ok");
    }

    #[test]
    fn linha_viva_e_menu_aberto_vem_do_view_model() {
        let mut r = sala_de(2, 1);
        r.menu_open = Some(true);
        let d = sala(&r);
        assert!(d.live && d.menu_open && d.own);
        assert_eq!(d.letter.as_str(), "C");
        let mut gone = sala_de(0, 0);
        gone.state = SavedRoomState::Gone;
        assert!(!sala(&gone).live);
    }
}
