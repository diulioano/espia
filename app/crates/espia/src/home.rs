//! Tela inicial real (T035). Porte de `src/routes/HomeRoute.svelte`: criar sala rápida e fixa, entrar por código ou
//! link colado, Suas salas com os números do servidor, renomear e apagar pelo "⋯".

use espia_core::device::{forget_room, remember_room, rename_saved};
use espia_core::join_view::clean_nickname;
use espia_core::room_name::normalize_room_name;
use espia_core::view_model::{
    DeleteDialogView, HomeAction, HomeView, NameDialogMode, NameDialogView, SavedRoomsView,
};
use espia_net::api::{self, CreateFailure, CreateFixedFailure, RoomStatus, RoomSummary};

use crate::app::{App, Resposta};
use crate::saved_rooms::build_saved_rooms;

const ERRO_LIMITE: &str = "Muitas salas criadas daqui agora. Tenta de novo em um minuto.";
const ERRO_REDE: &str = "Não deu pra falar com o servidor. Confere a internet e tenta de novo.";
const ERRO_NOME: &str = "Esse nome não dá. Use letras, números e espaços.";
const ERRO_LINK: &str = "Esse link não parece de uma sala.";
/// Aviso quando o dono tira você da sala (specs/014 US4).
pub(crate) const AVISO_REMOVIDO: &str = "O dono te tirou da sala";

/// O que a tela inicial guarda além do `device.json`.
#[derive(Default)]
pub struct HomeEstado {
    pub(crate) link_error: Option<String>,
    pub(crate) create_error: Option<String>,
    pub(crate) creating: bool,
    pub(crate) aviso: Option<String>,
    pub(crate) fixed_dialog: Option<NameDialogView>,
    pub(crate) delete_dialog: Option<DeleteDialogView>,
    /// Números de `fetchStatuses`. `None`: ainda perguntando (ou sem rede).
    pub(crate) numeros: Option<Vec<RoomStatus>>,
    /// Sala do "⋯" aberto.
    pub(crate) menu_code: Option<String>,
    /// Sala do "⋯" que está sendo renomeada ou apagada.
    pub(crate) owner_code: Option<String>,
}

impl App {
    pub(crate) fn tela_home(&self) -> HomeView {
        let h = &self.home;
        let owners = &self.device.owners;
        let rooms: Option<SavedRoomsView> = build_saved_rooms(
            &self.device.rooms,
            h.numeros.as_deref(),
            |code| owners.contains_key(code),
            self.agora_ms as i64,
            (self.cfg.fuso_ms)(),
            h.menu_code.as_deref(),
        );
        HomeView {
            nickname: self.device.nickname.clone().unwrap_or_default(),
            link: String::new(),
            link_error: h.link_error.clone(),
            create_error: h.create_error.clone(),
            creating: Some(h.creating),
            // O app nativo não oferece baixar o app (data-model.md, mapeamento de `espia:appinvite`).
            app_invite: None,
            notice: h.aviso.clone(),
            download: None,
            rooms,
            fixed_dialog: h.fixed_dialog.clone(),
            delete_dialog: h.delete_dialog.clone(),
        }
    }

    /// `refreshRooms`: pergunta ao servidor quantas pessoas há em cada sala guardada.
    pub(crate) fn atualizar_numeros(&mut self) {
        if self.device.rooms.is_empty() {
            return;
        }
        let codes: Vec<String> = self.device.rooms.iter().map(|r| r.code.clone()).collect();
        let (base, geracao, tx) = (self.cfg.base.clone(), self.geracao, self.auto.clone());
        tokio::spawn(async move {
            let numeros = api::fetch_statuses(&base, &codes).await;
            let _ = tx.send(crate::app::Entrada::Resposta { geracao, resposta: Resposta::Numeros(numeros) });
        });
    }

    pub(crate) fn acao_home(&mut self, acao: HomeAction) {
        match acao {
            HomeAction::CreateRoom { nickname } => self.criar_sala_rapida(&nickname),
            HomeAction::OpenLink { text } => match crate::router::codigo_do_texto(&text) {
                Some(code) => self.abrir_codigo(code, None, false),
                None => self.home.link_error = Some(ERRO_LINK.into()),
            },
            HomeAction::ClearLinkError => self.home.link_error = None,
            // O app nativo não tem o convite de baixar o app.
            HomeAction::DownloadApp | HomeAction::DismissAppInvite => {}
            HomeAction::DismissNotice => self.home.aviso = None,
            HomeAction::OpenFixedDialog => {
                self.home.fixed_dialog =
                    Some(NameDialogView { mode: NameDialogMode::Create, value: String::new(), busy: false, error: None });
            }
            HomeAction::CloseFixedDialog => {
                self.home.fixed_dialog = None;
                self.home.owner_code = None;
            }
            HomeAction::CreateFixedRoom { nickname, name } => self.criar_sala_fixa(&nickname, &name),
            HomeAction::EnterSaved { code, nickname } => {
                // Com apelido, entra direto; sem, a tela de entrada pede.
                let apelido = clean_nickname(&nickname);
                if apelido.is_empty() {
                    self.abrir_codigo(code, None, false);
                } else {
                    self.guardar_apelido(&apelido);
                    self.abrir_codigo(code, Some(apelido), false);
                }
            }
            HomeAction::ForgetSaved { code } => {
                // Só deste aparelho; a chave de dono fica (abrir o link de novo devolve "Sua").
                self.device.rooms = forget_room(std::mem::take(&mut self.device.rooms), &code);
                self.salvar();
                self.home.menu_code = None;
            }
            HomeAction::ToggleSavedMenu { code } => self.home.menu_code = code,
            HomeAction::RenameSaved { code } => {
                self.home.menu_code = None;
                let Some(sala) = self.device.rooms.iter().find(|r| r.code == code) else { return };
                let valor = sala.name.clone();
                self.home.owner_code = Some(code);
                self.home.fixed_dialog =
                    Some(NameDialogView { mode: NameDialogMode::Rename, value: valor, busy: false, error: None });
            }
            HomeAction::SubmitRename { name } => self.renomear_guardada(name),
            HomeAction::DeleteSaved { code } => {
                self.home.menu_code = None;
                let Some(sala) = self.device.rooms.iter().find(|r| r.code == code) else { return };
                let nome = sala.name.clone();
                let pessoas = self
                    .home
                    .numeros
                    .iter()
                    .flatten()
                    .find(|s| s.code == code)
                    .map_or(0, |s| match s.summary {
                        RoomSummary::Ok { people, .. } => people,
                        _ => 0,
                    });
                self.home.owner_code = Some(code);
                self.home.delete_dialog = Some(DeleteDialogView { name: nome, people: pessoas, busy: false, error: None });
            }
            HomeAction::ConfirmDelete => self.apagar_guardada(),
            HomeAction::CloseDeleteDialog => {
                self.home.delete_dialog = None;
                self.home.owner_code = None;
            }
        }
    }

    fn criar_sala_rapida(&mut self, nickname: &str) {
        let apelido = clean_nickname(nickname);
        if apelido.is_empty() {
            return;
        }
        self.guardar_apelido(&apelido);
        self.home.creating = true;
        self.home.create_error = None;
        let (base, geracao, tx) = (self.cfg.base.clone(), self.geracao, self.auto.clone());
        tokio::spawn(async move {
            let resultado = api::create_room(&base).await;
            let resposta = Resposta::SalaRapida { nickname: apelido, resultado };
            let _ = tx.send(crate::app::Entrada::Resposta { geracao, resposta });
        });
    }

    fn criar_sala_fixa(&mut self, nickname: &str, nome: &str) {
        let apelido = clean_nickname(nickname);
        let Some(dialogo) = self.home.fixed_dialog.as_mut() else { return };
        if apelido.is_empty() {
            return;
        }
        // O servidor refaz a conta; mandar o nome já limpo evita um "Esse nome não dá" à toa.
        let nome = normalize_room_name(nome).unwrap_or_else(|| nome.to_string());
        *dialogo = NameDialogView { value: nome.clone(), busy: true, error: None, ..dialogo.clone() };
        self.guardar_apelido(&apelido);
        let (base, geracao, tx) = (self.cfg.base.clone(), self.geracao, self.auto.clone());
        tokio::spawn(async move {
            let resultado = api::create_fixed_room(&base, &nome).await;
            let resposta = Resposta::SalaFixa { nickname: apelido, nome, resultado };
            let _ = tx.send(crate::app::Entrada::Resposta { geracao, resposta });
        });
    }

    fn renomear_guardada(&mut self, nome: String) {
        let Some(code) = self.home.owner_code.clone() else { return };
        let Some(chave) = self.device.owners.get(&code).cloned() else { return };
        let Some(dialogo) = self.home.fixed_dialog.as_mut() else { return };
        *dialogo = NameDialogView { value: nome.clone(), busy: true, error: None, ..dialogo.clone() };
        let (base, geracao, tx) = (self.cfg.base.clone(), self.geracao, self.auto.clone());
        tokio::spawn(async move {
            let resultado = api::rename_room(&base, &code, &chave, &nome).await;
            let _ = tx.send(crate::app::Entrada::Resposta { geracao, resposta: Resposta::Renomeada(resultado) });
        });
    }

    fn apagar_guardada(&mut self) {
        let Some(code) = self.home.owner_code.clone() else { return };
        let Some(chave) = self.device.owners.get(&code).cloned() else { return };
        let Some(dialogo) = self.home.delete_dialog.as_mut() else { return };
        *dialogo = DeleteDialogView { busy: true, error: None, ..dialogo.clone() };
        let (base, geracao, tx) = (self.cfg.base.clone(), self.geracao, self.auto.clone());
        tokio::spawn(async move {
            let resultado = api::delete_room(&base, &code, &chave).await;
            let _ = tx.send(crate::app::Entrada::Resposta { geracao, resposta: Resposta::Apagada(resultado) });
        });
    }

    /// As respostas da API que pertencem à tela inicial.
    pub(crate) fn resposta_home(&mut self, resposta: Resposta) {
        match resposta {
            Resposta::SalaRapida { nickname, resultado } => {
                self.home.creating = false;
                match resultado {
                    Ok(code) => self.entrar_na_sala(code, nickname, false),
                    Err(CreateFailure::RateLimited) => self.home.create_error = Some(ERRO_LIMITE.into()),
                    Err(CreateFailure::Network) => self.home.create_error = Some(ERRO_REDE.into()),
                }
            }
            Resposta::SalaFixa { nickname, nome, resultado } => match resultado {
                Ok(criada) => {
                    // A chave de dono vem só nesta resposta. Já fica em Suas salas, mesmo se sair antes de conectar.
                    self.device.owners.insert(criada.code.clone(), criada.owner_key);
                    self.device.rooms =
                        remember_room(std::mem::take(&mut self.device.rooms), &criada.code, &nome, self.agora_ms as i64);
                    self.salvar();
                    self.entrar_na_sala(criada.code, nickname, true);
                }
                Err(falha) => {
                    let erro = match falha {
                        CreateFixedFailure::RateLimited => ERRO_LIMITE,
                        CreateFixedFailure::Name => ERRO_NOME,
                        CreateFixedFailure::Network => ERRO_REDE,
                    };
                    if let Some(d) = self.home.fixed_dialog.as_mut() {
                        d.busy = false;
                        d.error = Some(erro.into());
                    }
                }
            },
            Resposta::Numeros(Some(numeros)) => self.numeros_chegaram(numeros),
            // Sem rede ou no limite: fica sem os números; "Entrar" continua funcionando.
            Resposta::Numeros(None) => {}
            Resposta::Renomeada(Ok(novo)) => {
                if let Some(code) = self.home.owner_code.take() {
                    self.device.rooms = rename_saved(std::mem::take(&mut self.device.rooms), &code, &novo);
                    self.salvar();
                }
                self.home.fixed_dialog = None;
                self.atualizar_numeros();
            }
            Resposta::Renomeada(Err(motivo)) => {
                if let Some(d) = self.home.fixed_dialog.as_mut() {
                    d.busy = false;
                    d.error = Some(api::owner_error_text(motivo).into());
                }
            }
            Resposta::Apagada(Err(motivo)) if motivo != api::OwnerFailure::Gone => {
                if let Some(d) = self.home.delete_dialog.as_mut() {
                    d.busy = false;
                    d.error = Some(api::owner_error_text(motivo).into());
                }
            }
            // Apagou (ou já não existia): sai da lista e a chave não serve mais.
            Resposta::Apagada(_) => {
                if let Some(code) = self.home.owner_code.take() {
                    self.device.rooms = forget_room(std::mem::take(&mut self.device.rooms), &code);
                    self.device.owners.remove(&code);
                    self.salvar();
                }
                self.home.delete_dialog = None;
            }
            Resposta::Previa(_) | Resposta::Sala(_) => {}
        }
    }

    fn numeros_chegaram(&mut self, numeros: Vec<RoomStatus>) {
        let mut mudou = false;
        for s in &numeros {
            match &s.summary {
                // Nome mudou (o dono renomeou): guarda o novo.
                RoomSummary::Ok { name: Some(nome), .. } => {
                    if self.device.rooms.iter().any(|r| r.code == s.code && &r.name != nome) {
                        self.device.rooms = rename_saved(std::mem::take(&mut self.device.rooms), &s.code, nome);
                        mudou = true;
                    }
                }
                RoomSummary::Ok { name: None, .. } => {}
                // A sala acabou: a chave de dono não serve mais.
                _ => mudou |= self.device.owners.remove(&s.code).is_some(),
            }
        }
        if mudou {
            self.salvar();
        }
        self.home.numeros = Some(numeros);
    }
}
