//! Liga o redutor da sala ao resto (T034): traduz os eventos da sinalização para `room::Event` e executa cada
//! `room::Effect` como a tabela do desenho em `app.rs` manda.

use std::collections::VecDeque;

use espia_core::device::{Session, remember_room, rename_saved, forget_room};
use espia_core::room::{
    self, ApiCall, ApiEvent, CaptureEvent, DeviceChange, Destination, Effect, EndReason, Event, LinkEvent,
    OwnerFailure as RoomFailure, StartFailure,
};
use espia_net::api::{self, OwnerFailure};
use espia_net::signaling::{SignalingCommand, SignalingEnd, SignalingEvent, SignalingStatus};

use crate::app::{App, Entrada, Resposta, Tempo, depurar};

/// O que chegou da sinalização, na língua do redutor.
pub(crate) fn traduzir(evento: SignalingEvent) -> Event {
    match evento {
        SignalingEvent::Message(m) => Event::Server(m),
        SignalingEvent::Status(SignalingStatus::Connecting) => Event::Link(LinkEvent::Connecting),
        SignalingEvent::Status(SignalingStatus::Open) => Event::Link(LinkEvent::Open),
        SignalingEvent::Status(SignalingStatus::Reconnecting) => Event::Link(LinkEvent::Reconnecting),
        SignalingEvent::Session(s) => Event::Link(LinkEvent::Session(s.map(|s| room::RoomSession {
            peer_id: s.peer_id,
            resume_token: s.resume_token,
        }))),
        SignalingEvent::Back => Event::Link(LinkEvent::Back),
        SignalingEvent::End(fim) => Event::Link(LinkEvent::Ended(match fim {
            SignalingEnd::NotFound => EndReason::NotFound,
            SignalingEnd::Full => EndReason::Full,
            SignalingEnd::Replaced => EndReason::Replaced,
            SignalingEnd::Kicked => EndReason::Kicked,
            SignalingEnd::Deleted => EndReason::Deleted,
            SignalingEnd::Removed => EndReason::Removed,
            SignalingEnd::Update => EndReason::Update,
        })),
    }
}

fn falha_do_dono(f: OwnerFailure) -> RoomFailure {
    match f {
        OwnerFailure::Forbidden => RoomFailure::Forbidden,
        OwnerFailure::Gone => RoomFailure::Gone,
        OwnerFailure::Name => RoomFailure::Name,
        OwnerFailure::RateLimited => RoomFailure::RateLimited,
        OwnerFailure::Network => RoomFailure::Network,
    }
}

impl App {
    pub(crate) fn sinalizacao(&mut self, evento: SignalingEvent) {
        self.evento_da_sala(traduzir(evento));
    }

    /// Entrega um evento ao redutor e executa os efeitos, na ordem. Eventos que os efeitos geram entram numa fila e são
    /// tratados antes de voltar ao laço.
    pub(crate) fn evento_da_sala(&mut self, evento: Event) {
        let mut fila = VecDeque::from([evento]);
        while let Some(evento) = fila.pop_front() {
            let agora = self.agora_ms;
            let Some(sala) = self.sala.as_mut() else { return };
            let efeitos = sala.reduzir(evento, agora);
            for efeito in efeitos {
                self.executar(efeito, &mut fila);
            }
        }
    }

    fn executar(&mut self, efeito: Effect, fila: &mut VecDeque<Event>) {
        match efeito {
            // ---- Servidor ----
            Effect::Send(mensagem) => self.comando(SignalingCommand::Send(mensagem)),
            Effect::Leave => self.comando(SignalingCommand::Leave),
            Effect::RetryNow => self.comando(SignalingCommand::RetryNow),
            Effect::Api(chamada) => self.chamar_api(chamada),

            // ---- Tempo ----
            Effect::Schedule { timer, after_ms } => self.agendar(Tempo::Sala(timer), after_ms),
            Effect::Cancel { timer } => self.cancelar(&Tempo::Sala(timer)),

            // ---- Aparelho ----
            Effect::Save(mudanca) => self.guardar(mudanca),
            Effect::CopyInvite(url) => self.copiar(url),

            // ---- Rota ----
            Effect::Navigate(Destination::Home) => self.ir_para_inicio(None),
            Effect::Navigate(Destination::Ended(motivo)) => self.fim_de_sala(motivo),

            // ---- Captura: sem captura ainda; pedir para transmitir devolve a falha para a tela não travar ----
            Effect::StartCapture { .. } => fila.push_back(Event::Capture(CaptureEvent::StartFailed(StartFailure::Other))),

            // ---- Ligações (T037), captura (fase 4) e o resto: ignorados, com log de depuração ----
            outro => depurar!("efeito ainda sem destino, ignorado: {outro:?}"),
        }
    }

    fn comando(&mut self, comando: SignalingCommand) {
        if let Some(sala) = &self.sala {
            let _ = sala.comandos.send(comando);
        }
    }

    /// `Api(..)` do redutor: uma tarefa do tokio; a resposta volta como `Entrada` e vira `Event::Api`.
    fn chamar_api(&mut self, chamada: ApiCall) {
        let (base, geracao, tx) = (self.cfg.base.clone(), self.geracao, self.auto.clone());
        tokio::spawn(async move {
            let evento = match chamada {
                ApiCall::Rename { code, key, name } => {
                    ApiEvent::Rename(api::rename_room(&base, &code, &key, &name).await.map(|_| ()).map_err(falha_do_dono))
                }
                ApiCall::Delete { code, key } => {
                    ApiEvent::Delete(api::delete_room(&base, &code, &key).await.map_err(falha_do_dono))
                }
                ApiCall::Remove { code, key, peer_id } => {
                    let result = api::remove_person(&base, &code, &key, &peer_id).await.map_err(falha_do_dono);
                    ApiEvent::Remove { peer_id, result }
                }
            };
            let _ = tx.send(Entrada::Resposta { geracao, resposta: Resposta::Sala(evento) });
        });
    }

    /// `Save(..)`: muda o `DeviceData` em memória e grava o `device.json`.
    fn guardar(&mut self, mudanca: DeviceChange) {
        let agora = self.agora_ms as i64;
        let d = &mut self.device;
        match mudanca {
            DeviceChange::GoLivePrefs(prefs) => d.golive = Some(prefs),
            DeviceChange::Volume { nickname, volume } => {
                d.volumes.insert(nickname.to_lowercase(), volume);
            }
            DeviceChange::Session { code, session: Some(room::RoomSession { peer_id, resume_token }) } => {
                d.sessions.insert(code, Session { peer_id, resume_token, at: agora });
            }
            DeviceChange::Session { code, session: None } => {
                d.sessions.remove(&code);
            }
            DeviceChange::RememberRoom { code, name, at_ms } => {
                d.rooms = remember_room(std::mem::take(&mut d.rooms), &code, &name, at_ms as i64);
            }
            DeviceChange::RenameRoom { code, name } => {
                d.rooms = rename_saved(std::mem::take(&mut d.rooms), &code, &name);
            }
            DeviceChange::ForgetRoom { code } => {
                d.rooms = forget_room(std::mem::take(&mut d.rooms), &code);
            }
            DeviceChange::ForgetOwnerKey { code } => {
                d.owners.remove(&code);
            }
        }
        self.salvar();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use espia_net::signaling::RoomSession;

    #[test]
    fn fim_da_sinalizacao_vira_fim_do_redutor() {
        let pares = [
            (SignalingEnd::NotFound, EndReason::NotFound),
            (SignalingEnd::Full, EndReason::Full),
            (SignalingEnd::Replaced, EndReason::Replaced),
            (SignalingEnd::Kicked, EndReason::Kicked),
            (SignalingEnd::Deleted, EndReason::Deleted),
            (SignalingEnd::Removed, EndReason::Removed),
            (SignalingEnd::Update, EndReason::Update),
        ];
        for (de, para) in pares {
            assert_eq!(traduzir(SignalingEvent::End(de)), Event::Link(LinkEvent::Ended(para)));
        }
    }

    #[test]
    fn estados_e_sessao_viram_eventos_de_ligacao() {
        assert_eq!(traduzir(SignalingEvent::Status(SignalingStatus::Reconnecting)), Event::Link(LinkEvent::Reconnecting));
        assert_eq!(traduzir(SignalingEvent::Back), Event::Link(LinkEvent::Back));
        let s = RoomSession { peer_id: "0123456789abcdef".into(), resume_token: "t".into() };
        assert_eq!(
            traduzir(SignalingEvent::Session(Some(s))),
            Event::Link(LinkEvent::Session(Some(room::RoomSession { peer_id: "0123456789abcdef".into(), resume_token: "t".into() })))
        );
        assert_eq!(traduzir(SignalingEvent::Session(None)), Event::Link(LinkEvent::Session(None)));
    }

    #[test]
    fn falhas_do_dono_mantem_o_motivo() {
        assert_eq!(falha_do_dono(OwnerFailure::Forbidden), RoomFailure::Forbidden);
        assert_eq!(falha_do_dono(OwnerFailure::Gone), RoomFailure::Gone);
        assert_eq!(falha_do_dono(OwnerFailure::Name), RoomFailure::Name);
        assert_eq!(falha_do_dono(OwnerFailure::RateLimited), RoomFailure::RateLimited);
        assert_eq!(falha_do_dono(OwnerFailure::Network), RoomFailure::Network);
    }
}
