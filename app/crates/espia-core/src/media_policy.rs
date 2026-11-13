//! Política de mídia (espelha `src/shared/media.ts`): só ligação direta, ordem de transmissão e ICE só STUN.

use serde::{Deserialize, Serialize};

use crate::protocol::{IceServer, Urls};

/// Versão do protocolo de mídia que o app fala (`MEDIA_PROTOCOL`).
pub const MEDIA_PROTOCOL: u32 = 2;
/// Maior inteiro seguro do JavaScript (`Number.MAX_SAFE_INTEGER`).
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransmissionKey {
    pub peer_id: String,
    pub started_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BroadcastStatus {
    Live,
    Ended,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastOrder {
    pub highest_broadcast_seq: u64,
    pub request_id: String,
    pub last_started_at: u64,
    pub status: BroadcastStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DirectOnlyReason {
    UnverifiedBudget,
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum MediaPolicy {
    DirectOnly { reason: DirectOnlyReason },
}

/// Valor de `DIRECT_ONLY` do site.
pub const DIRECT_ONLY: MediaPolicy = MediaPolicy::DirectOnly { reason: DirectOnlyReason::UnverifiedBudget };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BroadcastKind {
    Live,
    Unlive,
}

/// O que `nextBroadcastOrder` lê de um `live`/`unlive`.
#[derive(Debug, Clone, Copy)]
pub struct BroadcastMsg<'a> {
    pub kind: BroadcastKind,
    pub request_id: &'a str,
    pub broadcast_seq: u64,
}

/// Inteiro não negativo que cabe com exatidão num número do JavaScript (`isSafeSequence`).
pub fn is_safe_sequence(v: u64) -> bool {
    v <= MAX_SAFE_INTEGER
}

/// UUID v4 em minúsculas, `8-4-4-4-12` (`isRequestId`).
pub fn is_request_id(v: &str) -> bool {
    let partes: Vec<&str> = v.split('-').collect();
    let tamanhos = [8usize, 4, 4, 4, 12];
    if partes.len() != 5 || partes.iter().zip(tamanhos).any(|(p, n)| p.len() != n) {
        return false;
    }
    if !partes.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))) {
        return false;
    }
    partes[2].starts_with('4') && matches!(partes[3].as_bytes()[0], b'8' | b'9' | b'a' | b'b')
}

/// Duas chaves iguais (mesma pessoa e mesmo início); `None` nunca é igual a nada (`sameTransmission`).
pub fn same_transmission(a: Option<&TransmissionKey>, b: Option<&TransmissionKey>) -> bool {
    matches!((a, b), (Some(a), Some(b)) if a == b)
}

/// Fica só com URLs `stun:`/`stuns:` (sem diferenciar maiúsculas), sem usuário nem credencial; descarta
/// o servidor que ficar sem URL (`directIceServers`). A saída sempre usa a forma de lista.
pub fn direct_ice_servers(servers: &[IceServer]) -> Vec<IceServer> {
    servers
        .iter()
        .filter_map(|s| {
            let urls: Vec<String> = s.urls.to_vec().into_iter().filter(|u| is_stun_url(u)).collect();
            (!urls.is_empty()).then(|| IceServer { urls: Urls::Many(urls), username: None, credential: None })
        })
        .collect()
}

fn is_stun_url(url: &str) -> bool {
    let inicio = |p: &str| url.len() >= p.len() && url.is_char_boundary(p.len()) && url[..p.len()].eq_ignore_ascii_case(p);
    inicio("stun:") || inicio("stuns:")
}

/// Próxima ordem de transmissão da sala, ou `None` se a mensagem não vale (`nextBroadcastOrder`).
pub fn next_broadcast_order(current: Option<&BroadcastOrder>, msg: &BroadcastMsg<'_>, now: u64) -> Option<BroadcastOrder> {
    if !is_request_id(msg.request_id) || !is_safe_sequence(msg.broadcast_seq) || !is_safe_sequence(now) {
        return None;
    }
    let Some(cur) = current else {
        // Primeira da sala: só `live` com sequência 0.
        if msg.broadcast_seq != 0 || msg.kind == BroadcastKind::Unlive {
            return None;
        }
        return Some(BroadcastOrder {
            highest_broadcast_seq: 0,
            request_id: msg.request_id.to_string(),
            last_started_at: now,
            status: BroadcastStatus::Live,
        });
    };
    if msg.broadcast_seq < cur.highest_broadcast_seq {
        return None;
    }
    if msg.broadcast_seq == cur.highest_broadcast_seq {
        if msg.request_id != cur.request_id || (msg.kind == BroadcastKind::Live && cur.status == BroadcastStatus::Ended) {
            return None;
        }
        let status = if msg.kind == BroadcastKind::Unlive { BroadcastStatus::Ended } else { BroadcastStatus::Live };
        return Some(BroadcastOrder { status, ..cur.clone() });
    }
    if msg.kind == BroadcastKind::Unlive || msg.request_id == cur.request_id {
        return None;
    }
    if msg.broadcast_seq != cur.highest_broadcast_seq + 1 {
        return None;
    }
    let last_started_at = now.max(cur.last_started_at + 1);
    is_safe_sequence(last_started_at).then(|| BroadcastOrder {
        highest_broadcast_seq: msg.broadcast_seq,
        request_id: msg.request_id.to_string(),
        last_started_at,
        status: BroadcastStatus::Live,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Urls;

    const A: &str = "00000000-0000-4000-8000-000000000001";
    const B: &str = "00000000-0000-4000-8000-000000000002";

    fn msg(kind: BroadcastKind, request_id: &str, broadcast_seq: u64) -> BroadcastMsg<'_> {
        BroadcastMsg { kind, request_id, broadcast_seq }
    }

    fn ice(urls: Urls, username: Option<&str>, credential: Option<&str>) -> IceServer {
        IceServer { urls, username: username.map(Into::into), credential: credential.map(Into::into) }
    }

    #[test]
    fn constantes() {
        assert_eq!(MEDIA_PROTOCOL, 2);
        assert_eq!(MAX_SAFE_INTEGER, (1u64 << 53) - 1);
        assert_eq!(
            serde_json::to_string(&DIRECT_ONLY).unwrap(),
            r#"{"kind":"direct-only","reason":"unverified-budget"}"#
        );
    }

    #[test]
    fn ice_direto_nao_guarda_turn_nem_credencial() {
        let entrada = vec![
            ice(Urls::Many(vec!["turn:relay".into(), "stun:stun.example".into()]), Some("old"), Some("old")),
            ice(Urls::One("turns:relay".into()), None, None),
        ];
        assert_eq!(direct_ice_servers(&entrada), vec![ice(Urls::Many(vec!["stun:stun.example".into()]), None, None)]);
        let https = vec![ice(Urls::One("https://invalid".into()), None, None)];
        assert!(direct_ice_servers(&https).is_empty());
    }

    #[test]
    fn ice_direto_aceita_stuns_e_maiusculas() {
        let entrada = vec![ice(Urls::One("STUNS:a.example:5349".into()), None, None)];
        assert_eq!(direct_ice_servers(&entrada).len(), 1);
    }

    #[test]
    fn sequencia_segura() {
        assert!(is_safe_sequence(0));
        assert!(is_safe_sequence(MAX_SAFE_INTEGER));
        assert!(!is_safe_sequence(MAX_SAFE_INTEGER + 1));
    }

    #[test]
    fn request_id_so_uuid_v4_minusculo() {
        assert!(is_request_id(A));
        assert!(is_request_id("3f2b8c1e-5d4a-4e7b-9a6c-1d2e3f4a5b6c"));
        assert!(!is_request_id("old"));
        assert!(!is_request_id(""));
        assert!(!is_request_id("00000000-0000-1000-8000-000000000001")); // versão 1
        assert!(!is_request_id("00000000-0000-4000-7000-000000000001")); // variante
        assert!(!is_request_id("00000000-0000-4000-8000-00000000000G"));
        assert!(!is_request_id("00000000-0000-4000-8000-00000000000A")); // maiúscula
        assert!(!is_request_id("00000000-0000-4000-8000-0000000000011")); // longo
    }

    #[test]
    fn mesma_transmissao() {
        let k = |p: &str, t| TransmissionKey { peer_id: p.into(), started_at: t };
        assert!(same_transmission(Some(&k("a", 1)), Some(&k("a", 1))));
        assert!(!same_transmission(Some(&k("a", 1)), Some(&k("a", 2))));
        assert!(!same_transmission(Some(&k("a", 1)), Some(&k("b", 1))));
        assert!(!same_transmission(None, Some(&k("a", 1))));
        assert!(!same_transmission(None, None));
    }

    #[test]
    fn rejeita_sequencia_estourada_e_inicial_pulada() {
        assert!(next_broadcast_order(None, &msg(BroadcastKind::Live, A, MAX_SAFE_INTEGER + 1), 100).is_none());
        assert!(next_broadcast_order(None, &msg(BroadcastKind::Live, A, 1), 100).is_none());
        assert!(next_broadcast_order(None, &msg(BroadcastKind::Live, "old", 0), 100).is_none());
        assert!(next_broadcast_order(None, &msg(BroadcastKind::Live, A, 0), MAX_SAFE_INTEGER + 1).is_none());
        assert!(next_broadcast_order(None, &msg(BroadcastKind::Unlive, A, 0), 100).is_none());
    }

    #[test]
    fn nunca_reabre_pedido_encerrado_ou_substituido_e_guarda_o_tempo() {
        let first = next_broadcast_order(None, &msg(BroadcastKind::Live, A, 0), 100).unwrap();
        assert_eq!(first.status, BroadcastStatus::Live);
        assert_eq!(first.last_started_at, 100);
        assert_eq!(next_broadcast_order(Some(&first), &msg(BroadcastKind::Live, A, 0), 200).unwrap().last_started_at, 100);

        let ended = next_broadcast_order(Some(&first), &msg(BroadcastKind::Unlive, A, 0), 200).unwrap();
        assert_eq!(ended.status, BroadcastStatus::Ended);
        assert!(next_broadcast_order(Some(&ended), &msg(BroadcastKind::Live, A, 0), 300).is_none());

        let newer = next_broadcast_order(Some(&ended), &msg(BroadcastKind::Live, B, 1), 1).unwrap();
        assert_eq!(newer.last_started_at, 101);
        assert_eq!(newer.highest_broadcast_seq, 1);
        assert!(next_broadcast_order(Some(&newer), &msg(BroadcastKind::Unlive, A, 0), 300).is_none());
        // cópia via JSON, como o `welcome` traz na retomada
        let copia: BroadcastOrder = serde_json::from_str(&serde_json::to_string(&newer).unwrap()).unwrap();
        assert!(next_broadcast_order(Some(&copia), &msg(BroadcastKind::Live, A, 0), 300).is_none());
    }

    #[test]
    fn sequencia_seguinte_e_pedido_repetido() {
        let first = next_broadcast_order(None, &msg(BroadcastKind::Live, A, 0), 100).unwrap();
        // pula sequência
        assert!(next_broadcast_order(Some(&first), &msg(BroadcastKind::Live, B, 2), 200).is_none());
        // mesmo pedido em sequência nova
        assert!(next_broadcast_order(Some(&first), &msg(BroadcastKind::Live, A, 1), 200).is_none());
        // unlive em sequência nova
        assert!(next_broadcast_order(Some(&first), &msg(BroadcastKind::Unlive, B, 1), 200).is_none());
        let second = next_broadcast_order(Some(&first), &msg(BroadcastKind::Live, B, 1), 200).unwrap();
        // sequência velha
        assert!(next_broadcast_order(Some(&second), &msg(BroadcastKind::Live, A, 0), 300).is_none());
        // mesma sequência com outro pedido
        assert!(next_broadcast_order(Some(&second), &msg(BroadcastKind::Unlive, A, 1), 300).is_none());
        // relógio atrás do último início: o encerramento mantém o tempo
        let third = next_broadcast_order(Some(&second), &msg(BroadcastKind::Unlive, B, 1), 5).unwrap();
        assert_eq!(third.last_started_at, 200);
    }
}
