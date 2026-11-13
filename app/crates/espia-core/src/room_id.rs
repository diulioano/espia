//! Código de sala curto no formato `xxx-xxx` (ex.: `k7f-q2x`). Porte de `src/lib/room-id.ts`.

/// Sem caracteres ambíguos: 0/o, 1/l/i. Espelha `ROOM_ID_ALPHABET` de `src/lib/room-id.ts`.
pub const ROOM_ID_ALPHABET: &str = "abcdefghjkmnpqrstuvwxyz23456789";

/// Espaço no sentido do `\s` do JavaScript (que o site usa): o `White_Space` do Unicode, menos
/// U+0085 (que o JS trata como controle) e mais U+FEFF.
pub(crate) fn is_js_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{FEFF}'
}

fn in_alphabet(c: char) -> bool {
    c.is_ascii() && ROOM_ID_ALPHABET.contains(c)
}

/// Vale o formato `xxx-xxx` com o alfabeto acima? É a expressão `ROOM_ID_RE` escrita à mão:
/// `/^[a-hjkmnp-z2-9]{3}-[a-hjkmnp-z2-9]{3}$/`.
pub fn is_valid_room_id(id: &str) -> bool {
    let b = id.as_bytes();
    b.len() == 7 && b[3] == b'-' && b.iter().enumerate().all(|(i, &c)| i == 3 || in_alphabet(c as char))
}

/// Monta um código a partir de bytes aleatórios. Rejeita os bytes acima do maior múltiplo do
/// tamanho do alfabeto, para não enviesar a distribuição. `None` se sobrarem menos de 6 aceitos.
pub fn room_id_from_bytes(bytes: &[u8]) -> Option<String> {
    let alphabet = ROOM_ID_ALPHABET.as_bytes();
    let n = alphabet.len();
    let limit = 256 - (256 % n);
    let chars: Vec<char> = bytes
        .iter()
        .filter(|&&b| (b as usize) < limit)
        .take(6)
        .map(|&b| alphabet[b as usize % n] as char)
        .collect();
    if chars.len() < 6 {
        return None;
    }
    let (a, b) = chars.split_at(3);
    Some(format!("{}-{}", a.iter().collect::<String>(), b.iter().collect::<String>()))
}

/// Sorteia um código novo com o gerador do sistema operacional.
pub fn generate_room_id() -> Result<String, getrandom::Error> {
    loop {
        let mut bytes = [0u8; 12];
        getrandom::fill(&mut bytes)?;
        if let Some(id) = room_id_from_bytes(&bytes) {
            return Ok(id);
        }
    }
}

/// Extrai o código de um link inteiro (`https://…/s/k7f-q2x`, `espia://s/k7f-q2x`), de um código
/// com ou sem hífen (`K7F Q2X`, `k7fq2x`), ou devolve `None` se não parecer uma sala.
pub fn parse_room_id(input: &str) -> Option<String> {
    let lower = input.trim_matches(is_js_space).to_lowercase();
    let mut text = lower.as_str();
    // Como `/\/s\/([^/?#\s]+)/`: o primeiro `/s/` seguido de ao menos um caractere válido.
    for (i, _) in lower.match_indices("/s/") {
        let rest = &lower[i + 3..];
        let end = rest.find(|c: char| c == '/' || c == '?' || c == '#' || is_js_space(c)).unwrap_or(rest.len());
        if end > 0 {
            text = &rest[..end];
            break;
        }
    }
    let chars: String = text.chars().filter(|&c| c != '-' && !is_js_space(c)).collect();
    if chars.chars().count() != 6 || !chars.chars().all(in_alphabet) {
        return None;
    }
    Some(format!("{}-{}", &chars[..3], &chars[3..]))
}

/// Caminho da sala no site (`/s/k7f-q2x`).
pub fn room_path(id: &str) -> String {
    format!("/s/{id}")
}

#[cfg(test)]
mod tests {
    //! Testes portados de `test/lib/room-id.test.ts`: o arquivo TS tem 3 casos (`it`) e os 3 foram
    //! portados (cada um com todas as suas asserções). Mais os casos novos de Rust: validação do
    //! formato, sorteio com rejeição de bytes e `room_path`.
    use super::*;

    #[test]
    fn aceita_codigo_com_e_sem_hifen() {
        assert_eq!(parse_room_id("k7f-q2x").as_deref(), Some("k7f-q2x"));
        assert_eq!(parse_room_id(" K7F Q2X ").as_deref(), Some("k7f-q2x"));
        assert_eq!(parse_room_id("k7fq2x").as_deref(), Some("k7f-q2x"));
    }

    #[test]
    fn aceita_o_link_do_site_e_o_link_do_app_desktop() {
        assert_eq!(parse_room_id("https://espia.diuliano.com/s/k7f-q2x").as_deref(), Some("k7f-q2x"));
        assert_eq!(parse_room_id("espia://s/k7f-q2x").as_deref(), Some("k7f-q2x"));
    }

    #[test]
    fn recusa_o_que_nao_e_sala() {
        assert_eq!(parse_room_id("espia://qualquer-coisa"), None);
        assert_eq!(parse_room_id("k7f-q2o"), None);
        assert_eq!(parse_room_id(""), None);
    }

    #[test]
    fn link_com_busca_fragmento_ou_barra_final() {
        assert_eq!(parse_room_id("https://x.com/s/k7f-q2x?a=1").as_deref(), Some("k7f-q2x"));
        assert_eq!(parse_room_id("https://x.com/s/k7f-q2x#mv=abc").as_deref(), Some("k7f-q2x"));
        assert_eq!(parse_room_id("https://x.com/s/k7f-q2x/").as_deref(), Some("k7f-q2x"));
        // Caminho `/s/` sem nada depois não vale; o texto inteiro vira "código" e falha.
        assert_eq!(parse_room_id("https://x.com/s/"), None);
        assert_eq!(parse_room_id("https://x.com/s/k7f-q2xz"), None);
    }

    #[test]
    fn formato_do_codigo() {
        assert!(is_valid_room_id("k7f-q2x"));
        assert!(is_valid_room_id("abc-234"));
        assert!(!is_valid_room_id("K7F-Q2X"));
        assert!(!is_valid_room_id("k7fq2x"));
        assert!(!is_valid_room_id("k7f-q2"));
        assert!(!is_valid_room_id("k7f-q2xx"));
        assert!(!is_valid_room_id("k7f_q2x"));
        // Caracteres ambíguos ficam de fora: 0, 1, i, l, o.
        for c in ['0', '1', 'i', 'l', 'o'] {
            assert!(!is_valid_room_id(&format!("k7f-q2{c}")), "{c}");
        }
        // Letra fora do ASCII nunca passa, mesmo com 7 bytes ou 6 caracteres.
        assert!(!is_valid_room_id("k7f-q2é"));
        assert!(!is_valid_room_id("k7f-qé"));
    }

    #[test]
    fn alfabeto_tem_31_caracteres_sem_ambiguos() {
        assert_eq!(ROOM_ID_ALPHABET, "abcdefghjkmnpqrstuvwxyz23456789");
        assert_eq!(ROOM_ID_ALPHABET.len(), 31);
    }

    #[test]
    fn sorteio_usa_os_bytes_e_rejeita_os_enviesados() {
        assert_eq!(room_id_from_bytes(&[0, 1, 2, 3, 4, 5]).as_deref(), Some("abc-def"));
        // 31 % 31 = 0 -> 'a'; 30 -> '9'.
        assert_eq!(room_id_from_bytes(&[31, 30, 0, 0, 0, 0]).as_deref(), Some("a9a-aaa"));
        // Acima de 247 (maior múltiplo de 31 até 256 é 248) é rejeitado.
        assert_eq!(room_id_from_bytes(&[248, 255, 0, 1, 2, 3, 4, 5]).as_deref(), Some("abc-def"));
        assert_eq!(room_id_from_bytes(&[247, 0, 0, 0, 0, 0]).as_deref(), Some("9aa-aaa"));
        // Bytes demais aceitos além de 6 não entram.
        assert_eq!(room_id_from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7]).as_deref(), Some("abc-def"));
        // Sobraram menos de 6 aceitos.
        assert_eq!(room_id_from_bytes(&[0, 1, 2, 3, 4, 255]), None);
        assert_eq!(room_id_from_bytes(&[]), None);
    }

    #[test]
    fn codigo_gerado_e_valido() {
        for _ in 0..200 {
            let id = generate_room_id().expect("o sistema deve dar bytes aleatórios");
            assert!(is_valid_room_id(&id), "{id}");
            assert_eq!(parse_room_id(&id).as_deref(), Some(id.as_str()));
        }
    }

    #[test]
    fn caminho_da_sala() {
        assert_eq!(room_path("k7f-q2x"), "/s/k7f-q2x");
    }
}
