//! Quem o usuário escolheu assistir (espelha `src/lib/watch-state.ts`). Só a escolha explícita autoriza
//! a transmissão atual; identidade velha não sobrevive (`prune_watching`).

use std::collections::BTreeMap;

use crate::media_policy::TransmissionKey;
use crate::protocol::Peer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchStatus {
    Opening,
    Playing,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchChoice {
    pub transmission: TransmissionKey,
    pub revision: u64,
    pub status: WatchStatus,
}

/// Escolhas por id de quem transmite (ordenado por id, para saída estável).
pub type Choices = BTreeMap<String, WatchChoice>;

/// Liga ou desliga o Assistir de `peer`. No celular, ligar troca a escolha anterior (só uma por vez).
/// A revisão sobe a cada nova escolha da mesma pessoa.
pub fn choose_watch(choices: &Choices, peer: &Peer, on: bool, mobile: bool) -> Choices {
    let mut next = if mobile && on { Choices::new() } else { choices.clone() };
    match (&peer.live, on) {
        (Some(live), true) => {
            let revision = choices.get(&peer.id).map_or(0, |c| c.revision) + 1;
            next.insert(
                peer.id.clone(),
                WatchChoice {
                    transmission: TransmissionKey { peer_id: peer.id.clone(), started_at: live.started_at },
                    revision,
                    status: WatchStatus::Opening,
                },
            );
        }
        _ => {
            next.remove(&peer.id);
        }
    }
    next
}

/// Mantém só as escolhas cuja transmissão ainda é a mesma (mesmo id e mesmo `started_at`).
pub fn prune_watching(choices: &Choices, peers: &[Peer]) -> Choices {
    choices
        .iter()
        .filter(|(id, choice)| {
            peers.iter().any(|p| {
                p.id == **id && p.live.as_ref().is_some_and(|l| l.started_at == choice.transmission.started_at)
            })
        })
        .map(|(id, c)| (id.clone(), c.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    //! 2 casos no TS, 2 portados. Mais 1 caso extra sem par no TS (revisão e quem não transmite).
    use super::*;
    use crate::protocol::{AudioKind, LiveInfo, LiveSource, Tone};

    fn pessoa(id: &str, started_at: Option<u64>) -> Peer {
        Peer {
            id: id.into(),
            nickname: id.into(),
            tone: Tone::Accent,
            live: started_at.map(|started_at| LiveInfo {
                resolution: 1080,
                fps: 30,
                audio: AudioKind::None,
                source: LiveSource::Screen,
                started_at,
                label: None,
            }),
        }
    }

    #[test]
    fn so_a_escolha_explicita_autoriza_a_transmissao_atual() {
        let a = pessoa("a", Some(10));
        let selecionada = choose_watch(&Choices::new(), &a, true, false);
        assert_eq!(selecionada["a"].transmission.started_at, 10);
        assert_eq!(prune_watching(&selecionada, &[a.clone()]).len(), 1);
        assert_eq!(prune_watching(&selecionada, &[pessoa("a", Some(11))]).len(), 0);
        assert_eq!(choose_watch(&selecionada, &a, false, false).len(), 0);
    }

    #[test]
    fn no_celular_a_escolha_troca_a_anterior_e_identidade_velha_nao_sobrevive() {
        let a = pessoa("a", Some(10));
        let primeira = choose_watch(&Choices::new(), &a, true, false);
        let selecionada = choose_watch(&primeira, &pessoa("b", Some(20)), true, true);
        assert_eq!(selecionada.keys().collect::<Vec<_>>(), ["b"]);
        assert_eq!(prune_watching(&selecionada, &[]).len(), 0);
    }

    #[test]
    fn revisao_sobe_a_cada_escolha_e_quem_nao_transmite_nao_entra() {
        let a = pessoa("a", Some(10));
        let um = choose_watch(&Choices::new(), &a, true, false);
        let dois = choose_watch(&um, &a, true, false);
        assert_eq!(um["a"].revision, 1);
        assert_eq!(dois["a"].revision, 2);
        assert_eq!(dois["a"].status, WatchStatus::Opening);
        assert!(choose_watch(&Choices::new(), &pessoa("z", None), true, false).is_empty());
    }
}
