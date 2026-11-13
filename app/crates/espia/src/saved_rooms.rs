//! Suas salas (specs/011 US3): textos e ordem do painel. Porte de `buildSavedRooms`, `recentLabel` e `roomTone` de
//! `src/lib/saved-rooms.ts`; lembrar, esquecer e renomear já estão em `espia_core::device`.

use espia_core::device::SavedRoom;
use espia_core::protocol::Tone;
use espia_core::view_model::{SavedRoomGroup, SavedRoomState, SavedRoomView, SavedRoomsView};
use espia_net::api::{RoomStatus, RoomSummary};

const DIA_MS: i64 = 24 * 60 * 60_000;

/// Cor da inicial: sempre a mesma para a mesma sala (`TONES = deep, accent, sage, sand`).
pub fn room_tone(code: &str) -> Tone {
    let mut h: u32 = 0;
    for unidade in code.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(u32::from(unidade));
    }
    [Tone::Deep, Tone::Accent, Tone::Sage, Tone::Sand][(h % 4) as usize]
}

/// "usada hoje", "usada ontem", "há 3 dias", "há 1 semana", "há 2 semanas", "há mais de um mês".
/// `fuso_ms`: diferença do fuso local para UTC (os dias contam à meia-noite local, como no site).
pub fn recent_label(last_used: i64, now: i64, fuso_ms: i64) -> String {
    let inicio = |t: i64| (t + fuso_ms).div_euclid(DIA_MS) * DIA_MS;
    let dias = (inicio(now) - inicio(last_used)) / DIA_MS;
    match dias {
        d if d <= 0 => "usada hoje".into(),
        1 => "usada ontem".into(),
        d if d < 7 => format!("há {d} dias"),
        d if d < 14 => "há 1 semana".into(),
        d if d < 30 => format!("há {} semanas", d / 7),
        _ => "há mais de um mês".into(),
    }
}

/// Diferença do fuso horário local para UTC, em milissegundos (0 se o Windows não disser).
pub fn fuso_local_ms() -> i64 {
    time::UtcOffset::current_local_offset().map(|o| i64::from(o.whole_seconds()) * 1000).unwrap_or(0)
}

/// Monta o painel. `statuses` ausente = ainda perguntando (todas "carregando"). Ordem: com gente agora primeiro (mais
/// gente antes), depois as usadas mais recentemente.
pub fn build_saved_rooms(
    list: &[SavedRoom],
    statuses: Option<&[RoomStatus]>,
    owned: impl Fn(&str) -> bool,
    now: i64,
    fuso_ms: i64,
    menu_code: Option<&str>,
) -> Option<SavedRoomsView> {
    if list.is_empty() {
        return None;
    }
    let achar = |code: &str| statuses.and_then(|s| s.iter().rev().find(|x| x.code == code));
    let mut salas: Vec<(SavedRoomView, i64)> = list
        .iter()
        .map(|r| {
            let s = achar(&r.code);
            let (state, name, people, streaming) = match s.map(|s| &s.summary) {
                None => (SavedRoomState::Loading, r.name.clone(), 0, 0),
                Some(RoomSummary::Ok { name, people, streaming }) => (
                    SavedRoomState::Ok,
                    name.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| r.name.clone()),
                    *people,
                    *streaming,
                ),
                Some(_) => (SavedRoomState::Gone, r.name.clone(), 0, 0),
            };
            let view = SavedRoomView {
                code: r.code.clone(),
                name,
                own: state != SavedRoomState::Gone && owned(&r.code),
                tone: room_tone(&r.code),
                state,
                people,
                streaming,
                recent: recent_label(r.last_used, now, fuso_ms),
                menu_open: (menu_code == Some(r.code.as_str())).then_some(true),
            };
            (view, r.last_used)
        })
        .collect();
    salas.sort_by(|a, b| b.0.people.cmp(&a.0.people).then(b.1.cmp(&a.1)));
    let salas: Vec<SavedRoomView> = salas.into_iter().map(|(v, _)| v).collect();

    let (com_gente, vazias): (Vec<_>, Vec<_>) = salas.iter().cloned().partition(|r| r.people > 0);
    if salas.len() >= 4 && !com_gente.is_empty() && !vazias.is_empty() {
        return Some(SavedRoomsView {
            groups: vec![
                SavedRoomGroup { label: Some("Com gente agora".into()), rooms: com_gente },
                SavedRoomGroup { label: Some("Usadas recentemente".into()), rooms: vazias },
            ],
            note: format!("{} salas neste aparelho", salas.len()),
        });
    }
    Some(SavedRoomsView { groups: vec![SavedRoomGroup { label: None, rooms: salas }], note: "neste aparelho".into() })
}

#[cfg(test)]
mod tests {
    //! Portados de `test/lib/saved-rooms.test.ts` (recentLabel, buildSavedRooms, roomTone). Os de rememberRoom,
    //! forgetRoom e renameSaved moram em `espia_core::device`. O fuso dos testes é UTC, e `NOW` cai às 15h.
    use super::*;

    const NOW: i64 = 1_793_372_400_000; // 2026-10-30 15:00 UTC

    fn sala(code: &str, name: &str, dias: i64) -> SavedRoom {
        SavedRoom { code: code.into(), name: name.into(), last_used: NOW - dias * DIA_MS }
    }
    fn ok(code: &str, people: u32, streaming: u32, name: Option<&str>) -> RoomStatus {
        RoomStatus { code: code.into(), summary: RoomSummary::Ok { name: name.map(String::from), people, streaming } }
    }
    fn codigos(g: &SavedRoomGroup) -> Vec<&str> {
        g.rooms.iter().map(|r| r.code.as_str()).collect()
    }

    #[test]
    fn rotulos_de_uso() {
        assert_eq!(recent_label(NOW - 60_000, NOW, 0), "usada hoje");
        assert_eq!(recent_label(NOW - DIA_MS, NOW, 0), "usada ontem");
        assert_eq!(recent_label(NOW - 3 * DIA_MS, NOW, 0), "há 3 dias");
        assert_eq!(recent_label(NOW - 8 * DIA_MS, NOW, 0), "há 1 semana");
        assert_eq!(recent_label(NOW - 15 * DIA_MS, NOW, 0), "há 2 semanas");
        assert_eq!(recent_label(NOW - 40 * DIA_MS, NOW, 0), "há mais de um mês");
        // Futuro (relógio mudado) vale hoje.
        assert_eq!(recent_label(NOW + 5 * DIA_MS, NOW, 0), "usada hoje");
    }

    #[test]
    fn o_dia_conta_a_meia_noite_local() {
        let tres = -3 * 60 * 60_000; // UTC-3
        // 00:30 UTC de hoje é 21:30 de ontem em UTC-3.
        let uso_cedo = NOW - 14 * 60 * 60_000 - 30 * 60_000;
        assert_eq!(recent_label(uso_cedo, NOW, 0), "usada hoje");
        assert_eq!(recent_label(uso_cedo, NOW, tres), "usada ontem");
    }

    #[test]
    fn cor_estavel() {
        assert_eq!(room_tone("k7f-q2x"), room_tone("k7f-q2x"));
        // h("a") = 97 e 97 % 4 = 1: accent.
        assert_eq!(room_tone("a"), Tone::Accent);
    }

    #[test]
    fn lista_vazia_nao_tem_painel() {
        assert!(build_saved_rooms(&[], None, |_| false, NOW, 0, None).is_none());
    }

    #[test]
    fn sem_numeros_todas_carregando_na_ordem_de_uso() {
        let lista = [sala("aaa-aaa", "Filme", 1), sala("bbb-bbb", "CS", 2), sala("ccc-ccc", "TCC", 3)];
        let v = build_saved_rooms(&lista, None, |_| false, NOW, 0, None).unwrap();
        assert_eq!(codigos(&v.groups[0]), ["aaa-aaa", "bbb-bbb", "ccc-ccc"]);
        assert!(v.groups[0].rooms.iter().all(|r| r.state == SavedRoomState::Loading));
    }

    #[test]
    fn com_gente_primeiro_e_sua_so_para_o_dono() {
        let lista = [sala("aaa-aaa", "Filme", 1), sala("bbb-bbb", "CS", 2), sala("ccc-ccc", "TCC", 3)];
        let st = [ok("aaa-aaa", 0, 0, None), ok("bbb-bbb", 3, 1, None), ok("ccc-ccc", 2, 0, None)];
        let v = build_saved_rooms(&lista, Some(&st), |c| c == "bbb-bbb", NOW, 0, None).unwrap();
        assert_eq!(v.note, "neste aparelho");
        assert_eq!(v.groups.len(), 1);
        assert_eq!(codigos(&v.groups[0]), ["bbb-bbb", "ccc-ccc", "aaa-aaa"]);
        assert_eq!(v.groups[0].rooms.iter().map(|r| r.own).collect::<Vec<_>>(), [true, false, false]);
    }

    #[test]
    fn nome_novo_do_servidor_vale_e_menu_aberto_marca() {
        let lista = [sala("aaa-aaa", "Filme", 1)];
        let st = [ok("aaa-aaa", 0, 0, Some("Filme de sábado"))];
        let v = build_saved_rooms(&lista, Some(&st), |_| false, NOW, 0, Some("aaa-aaa")).unwrap();
        assert_eq!(v.groups[0].rooms[0].name, "Filme de sábado");
        assert_eq!(v.groups[0].rooms[0].menu_open, Some(true));
    }

    #[test]
    fn apagada_ou_inexistente_vira_gone_sem_sua() {
        let lista = [sala("aaa-aaa", "A", 1), sala("bbb-bbb", "B", 2)];
        let st = [
            RoomStatus { code: "aaa-aaa".into(), summary: RoomSummary::Gone { reason: espia_net::api::GoneReason::Deleted } },
            RoomStatus { code: "bbb-bbb".into(), summary: RoomSummary::NotFound },
        ];
        let v = build_saved_rooms(&lista, Some(&st), |_| true, NOW, 0, None).unwrap();
        assert!(v.groups[0].rooms.iter().all(|r| r.state == SavedRoomState::Gone && !r.own));
    }

    #[test]
    fn quatro_ou_mais_com_gente_e_vazias_viram_dois_grupos() {
        let lista = [
            sala("aaa-aaa", "A", 1),
            sala("bbb-bbb", "B", 2),
            sala("ccc-ccc", "C", 3),
            sala("ddd-ddd", "D", 4),
            sala("eee-eee", "E", 5),
        ];
        let st = [
            ok("aaa-aaa", 0, 0, None),
            ok("bbb-bbb", 4, 2, None),
            ok("ccc-ccc", 2, 0, None),
            ok("ddd-ddd", 0, 0, None),
            ok("eee-eee", 3, 1, None),
        ];
        let v = build_saved_rooms(&lista, Some(&st), |_| false, NOW, 0, None).unwrap();
        assert_eq!(v.note, "5 salas neste aparelho");
        assert_eq!(v.groups[0].label.as_deref(), Some("Com gente agora"));
        assert_eq!(codigos(&v.groups[0]), ["bbb-bbb", "eee-eee", "ccc-ccc"]);
        assert_eq!(v.groups[1].label.as_deref(), Some("Usadas recentemente"));
        assert_eq!(codigos(&v.groups[1]), ["aaa-aaa", "ddd-ddd"]);
    }

    #[test]
    fn cinco_vazias_ficam_num_grupo_so() {
        let lista: Vec<SavedRoom> = ["aaa-aaa", "bbb-bbb", "ccc-ccc", "ddd-ddd", "eee-eee"]
            .iter()
            .enumerate()
            .map(|(i, c)| sala(c, c, i as i64))
            .collect();
        let st: Vec<RoomStatus> = lista.iter().map(|r| ok(&r.code, 0, 0, None)).collect();
        let v = build_saved_rooms(&lista, Some(&st), |_| false, NOW, 0, None).unwrap();
        assert_eq!(v.groups.len(), 1);
        assert_eq!(v.groups[0].label, None);
    }
}
