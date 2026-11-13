//! Nome de sala fixa: mesma regra do navegador e do Worker. Porte de `src/shared/room-name.ts`.
use crate::room_id::is_js_space;

// espelha src/shared/protocol.ts
pub const ROOM_NAME_MAX: usize = 40;

/// `\p{Cc}` e `\p{Cf}` do Unicode, menos o ZWJ (U+200D): sem ele, emojis compostos (família,
/// profissões) seriam recusados.
fn is_control_or_invisible(c: char) -> bool {
    let cp = c as u32;
    if cp == 0x200D {
        return false;
    }
    matches!(cp,
        0x0000..=0x001F | 0x007F..=0x009F
        // Cf (Unicode 15)
        | 0x00AD | 0x0600..=0x0605 | 0x061C | 0x06DD | 0x070F | 0x0890..=0x0891 | 0x08E2
        | 0x180E | 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2064 | 0x2066..=0x206F
        | 0xFEFF | 0xFFF9..=0xFFFB | 0x110BD | 0x110CD | 0x13430..=0x1343F | 0x1BCA0..=0x1BCA3
        | 0x1D173..=0x1D17A | 0xE0001 | 0xE0020..=0xE007F)
}

/// Nome pronto para guardar, ou `None` se não dá (vazio, longo demais, caractere invisível ou de
/// controle). Espaços repetidos viram um só e as pontas são aparadas. Sem normalização NFC (não há
/// dependência para isso na crate): a entrada vem de campos de texto que já entregam composto.
pub fn normalize_room_name(raw: &str) -> Option<String> {
    let mut name = String::with_capacity(raw.len());
    let mut pending_space = false;
    for c in raw.chars() {
        if is_js_space(c) {
            pending_space = !name.is_empty();
        } else {
            if pending_space {
                name.push(' ');
                pending_space = false;
            }
            name.push(c);
        }
    }
    if name.chars().any(is_control_or_invisible) {
        return None;
    }
    let length = name.chars().count();
    if length == 0 || length > ROOM_NAME_MAX {
        None
    } else {
        Some(name)
    }
}

#[cfg(test)]
mod tests {
    //! Testes portados de `test/lib/room-name.test.ts`: o arquivo TS tem 6 casos (`it`) e os 6 foram
    //! portados, com as mesmas asserções (os caracteres invisíveis vão escritos como escapes
    //! `\u{...}` para ficarem legíveis). Mais casos novos: ponta dos 40, controles C1 e ZWJ solto.
    use super::*;

    #[test]
    fn tira_espacos_das_pontas_e_junta_os_repetidos() {
        assert_eq!(normalize_room_name("  CS   da galera ").as_deref(), Some("CS da galera"));
    }

    #[test]
    fn tab_e_quebra_de_linha_viram_espaco() {
        assert_eq!(normalize_room_name("CS\tda\ngalera").as_deref(), Some("CS da galera"));
    }

    #[test]
    fn recusa_vazio() {
        assert_eq!(normalize_room_name(""), None);
        assert_eq!(normalize_room_name("   "), None);
    }

    #[test]
    fn conta_caracteres_nao_unidades_utf16() {
        let quarenta = "\u{1F3AE}".repeat(40);
        assert_eq!(normalize_room_name(&quarenta), Some(quarenta));
        assert_eq!(normalize_room_name(&"\u{1F3AE}".repeat(41)), None);
        assert_eq!(normalize_room_name(&"a".repeat(41)), None);
    }

    #[test]
    fn recusa_caracteres_invisiveis_e_de_controle() {
        assert_eq!(normalize_room_name("a\u{202E}b"), None);
        assert_eq!(normalize_room_name("a\u{200B}b"), None);
        assert_eq!(normalize_room_name("a\u{0007}b"), None);
    }

    #[test]
    fn aceita_emoji_composto_zwj() {
        let familia = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
        let nome = format!("Família {familia}");
        assert_eq!(normalize_room_name(&nome), Some(nome.clone()));
    }

    #[test]
    fn limite_exato_de_40_passa() {
        assert_eq!(normalize_room_name(&"a".repeat(40)), Some("a".repeat(40)));
    }

    #[test]
    fn recusa_controles_c1_e_marcas_de_formato() {
        assert_eq!(normalize_room_name("a\u{009F}b"), None);
        assert_eq!(normalize_room_name("a\u{00AD}b"), None);
        assert_eq!(normalize_room_name("a\u{2066}b"), None);
        assert_eq!(normalize_room_name("a\u{E0041}b"), None);
    }

    #[test]
    fn u0085_e_controle_e_feff_e_espaco_como_no_js() {
        assert_eq!(normalize_room_name("a\u{0085}b"), None);
        assert_eq!(normalize_room_name("a\u{FEFF}b").as_deref(), Some("a b"));
    }

    #[test]
    fn espacos_unicode_tambem_juntam() {
        assert_eq!(normalize_room_name("a\u{00A0}\u{2003}b").as_deref(), Some("a b"));
    }
}
