//! Entrada e sala sem mídia (T036). Porte de `src/routes/RoomRoute.svelte`: prévia de quem está na sala, apelido,
//! entrar, sala não encontrada, e a sala em si (o redutor do T021 dirige tudo o que acontece lá dentro).

use espia_core::join_view::clean_nickname;
use espia_core::room::{self, EndReason, Event, RoomInit, RoomState};
use espia_core::view_model::{
    JoinAction, JoinPreviewPerson, JoinStatus, JoinView, NotFoundAction, NotFoundReason, NotFoundView, RoomAction,
};
use espia_net::api::{self, GoneReason, PreviewResult};
use espia_net::signaling::{Signaling, SignalingCommand, SignalingConfig};
use espia_ui::Screen;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::{App, Entrada, Resposta};
use crate::home::AVISO_REMOVIDO;
use crate::router::Rota;

/// A sala aberta: o estado do redutor e a ponta da sinalização.
pub struct Sala {
    pub(crate) code: String,
    pub(crate) nickname: String,
    /// Só está vazio durante `reduzir` (o redutor recebe e devolve o estado por valor).
    estado: Option<RoomState>,
    pub(crate) comandos: UnboundedSender<SignalingCommand>,
    /// A sala usa uma versão nova da transmissão: "Atualize o Espia para continuar".
    pub(crate) precisa_atualizar: bool,
}

impl Sala {
    pub(crate) fn estado(&self) -> &RoomState {
        self.estado.as_ref().expect("o estado da sala só falta durante reduzir")
    }

    pub(crate) fn reduzir(&mut self, evento: Event, agora_ms: u64) -> Vec<room::Effect> {
        let estado = self.estado.take().expect("o estado da sala só falta durante reduzir");
        let (novo, efeitos) = room::reduce(estado, evento, agora_ms);
        self.estado = Some(novo);
        efeitos
    }
}

/// A tela de `/s/<código>` sem sala aberta: a entrada (prévia e apelido) ou a de sala não encontrada.
pub struct EntradaEstado {
    pub(crate) view: JoinView,
    pub(crate) fim: Option<NotFoundView>,
}

impl EntradaEstado {
    pub(crate) fn vazia() -> Self {
        EntradaEstado { view: nova_entrada("", ""), fim: None }
    }
}

fn nova_entrada(code: &str, apelido: &str) -> JoinView {
    JoinView {
        room_id: code.to_string(),
        status: JoinStatus::Ready,
        people: None,
        capacity: espia_core::protocol::CAPACITY,
        nickname: apelido.to_string(),
        // No app, transmitir tela e câmera sempre existem (o `RoomView` diz o mesmo).
        can_share: true,
        fixed_name: None,
        replaced: None,
    }
}

impl App {
    // ---- Entrada (prévia) ----

    pub(crate) fn ir_para_entrada(&mut self, code: String, buscar_previa: bool) {
        self.nova_geracao();
        let apelido = self.device.nickname.clone().unwrap_or_default();
        self.entrada = EntradaEstado { view: nova_entrada(&code, &apelido), fim: None };
        self.rota = Rota::Join { code: code.clone() };
        if buscar_previa {
            self.buscar_previa(code);
        }
    }

    fn buscar_previa(&self, code: String) {
        let (base, geracao, tx) = (self.cfg.base.clone(), self.geracao, self.auto.clone());
        tokio::spawn(async move {
            let previa = api::fetch_preview(&base, &code).await;
            let _ = tx.send(Entrada::Resposta { geracao, resposta: Resposta::Previa(previa) });
        });
    }

    pub(crate) fn previa_chegou(&mut self, previa: PreviewResult) {
        if !matches!(self.rota, Rota::Join { .. }) {
            return;
        }
        match previa {
            PreviewResult::NotFound => self.nao_encontrada(NotFoundReason::Quick),
            PreviewResult::Gone(GoneReason::Deleted) => self.nao_encontrada(NotFoundReason::Deleted),
            PreviewResult::Gone(GoneReason::Expired) => self.nao_encontrada(NotFoundReason::Expired),
            PreviewResult::Unknown => {
                self.entrada.view.status = JoinStatus::Ready;
                self.entrada.view.people = None;
            }
            PreviewResult::Ready(p) | PreviewResult::Full(p) => {
                let cheia = p.people.len() as u64 >= u64::from(p.capacity);
                let v = &mut self.entrada.view;
                v.status = if cheia { JoinStatus::Full } else { JoinStatus::Ready };
                v.capacity = p.capacity;
                v.people = Some(
                    p.people
                        .into_iter()
                        .map(|x| JoinPreviewPerson { name: x.nickname, tone: x.tone, live: x.live })
                        .collect(),
                );
                v.fixed_name = p.fixed.map(|f| f.name);
            }
        }
    }

    /// A sala não existe (mais): rápida que acabou, fixa apagada ou vencida (specs/011).
    fn nao_encontrada(&mut self, motivo: NotFoundReason) {
        let Some(code) = self.rota.code().map(str::to_string) else { return };
        // O nome vem de Suas salas, se este aparelho conhecia a sala; o servidor não guarda (research R5).
        let guardada = self.device.rooms.iter().find(|r| r.code == code);
        self.entrada.fim = Some(NotFoundView {
            reason: motivo,
            name: guardada.map(|r| r.name.clone()),
            saved: guardada.is_some(),
        });
        // A sala acabou: a chave de dono não serve mais para nada.
        if self.device.owners.remove(&code).is_some() {
            self.salvar();
        }
    }

    pub(crate) fn tela_entrada(&self) -> Screen {
        match &self.entrada.fim {
            Some(fim) => Screen::NotFound(fim.clone()),
            None => Screen::Join(self.entrada.view.clone()),
        }
    }

    pub(crate) fn acao_join(&mut self, acao: JoinAction) {
        match acao {
            JoinAction::GoHome | JoinAction::CreateRoom | JoinAction::PasteOtherLink => self.ir_para_inicio(None),
            JoinAction::Join { nickname } => {
                let apelido = clean_nickname(&nickname);
                let Some(code) = self.rota.code().map(str::to_string) else { return };
                if apelido.is_empty() {
                    return;
                }
                self.guardar_apelido(&apelido);
                self.entrar_na_sala(code, apelido, false);
            }
        }
    }

    pub(crate) fn acao_nao_encontrada(&mut self, acao: NotFoundAction) {
        match acao {
            NotFoundAction::GoHome | NotFoundAction::CreateRoom | NotFoundAction::PasteOtherLink => {
                self.ir_para_inicio(None)
            }
            NotFoundAction::Forget => {
                if let Some(code) = self.rota.code().map(str::to_string) {
                    self.device.rooms = espia_core::device::forget_room(std::mem::take(&mut self.device.rooms), &code);
                    self.salvar();
                }
                self.ir_para_inicio(None);
            }
        }
    }

    // ---- Sala ----

    /// Abre a sala: o redutor nasce com o que o aparelho sabe e a sinalização começa a conectar (com a sessão guardada,
    /// se ainda vale, retoma o lugar).
    pub(crate) fn entrar_na_sala(&mut self, code: String, apelido: String, criada: bool) {
        self.nova_geracao();
        self.rota = Rota::Room { code: code.clone() };
        let init = RoomInit {
            code: code.clone(),
            go_live_prefs: self.device.golive_or_default(),
            owner_key: self.device.owners.get(&code).cloned(),
            volumes: self.device.volumes.clone(),
            just_created: criada,
        };
        let mut cfg = SignalingConfig::new(&self.cfg.base, &code, &apelido, self.sessao_valida(&code));
        (self.cfg.ajuste_sinalizacao)(&mut cfg);
        let Signaling { commands, mut events } = Signaling::start(cfg);
        let (geracao, tx) = (self.geracao, self.auto.clone());
        tokio::spawn(async move {
            while let Some(evento) = events.recv().await {
                if tx.send(Entrada::Sinalizacao { geracao, evento }).is_err() {
                    break;
                }
            }
        });
        self.sala = Some(Sala {
            code,
            nickname: apelido,
            estado: Some(RoomState::new(init)),
            comandos: commands,
            precisa_atualizar: false,
        });
    }

    pub(crate) fn tela_sala(&self) -> Screen {
        let Some(sala) = &self.sala else { return Screen::Offline };
        let mut view = room::view(sala.estado(), self.agora_ms);
        if sala.precisa_atualizar {
            view.requires_update = Some(true);
        }
        Screen::Room(view)
    }

    pub(crate) fn acao_sala(&mut self, acao: RoomAction) {
        self.evento_da_sala(Event::Action(acao));
    }

    /// "Atualizar" da tela "Atualize o Espia": o site recarrega a página; o app abre a sala de novo.
    pub(crate) fn recarregar_sala(&mut self) {
        let Some(sala) = self.sala.as_ref().filter(|s| s.precisa_atualizar) else { return };
        let (code, apelido) = (sala.code.clone(), sala.nickname.clone());
        self.entrar_na_sala(code, apelido, false);
    }

    /// A sala acabou de vez (o redutor mandou `Navigate(Ended(..))`): para onde ir, como em `RoomRoute.svelte`.
    pub(crate) fn fim_de_sala(&mut self, motivo: EndReason) {
        let Some(code) = self.sala.as_ref().map(|s| s.code.clone()) else { return };
        match motivo {
            // A sala usa uma versão nova da transmissão: a tela fica, pedindo para atualizar.
            EndReason::Update => {
                if let Some(sala) = self.sala.as_mut() {
                    sala.precisa_atualizar = true;
                }
            }
            // O dono apagou com você dentro.
            EndReason::Deleted => {
                self.ir_para_entrada(code, false);
                self.nao_encontrada(NotFoundReason::Kicked);
            }
            // O dono tirou você: início com o aviso; o link continua valendo.
            EndReason::Removed => self.ir_para_inicio(Some(AVISO_REMOVIDO.into())),
            EndReason::Full => {
                self.ir_para_entrada(code, false);
                self.entrada.view.status = JoinStatus::Full;
            }
            // A mesma pessoa abriu a sala em outra janela: esta cede o lugar e explica.
            EndReason::Replaced => {
                self.ir_para_entrada(code, true);
                self.entrada.view.replaced = Some(true);
            }
            // Não existe: a prévia diz se era rápida (404) ou fixa apagada ou vencida (410).
            EndReason::NotFound | EndReason::Kicked => self.ir_para_entrada(code, true),
        }
    }
}
