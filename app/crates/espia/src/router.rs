//! Rotas do app (porte de `src/lib/router.svelte.ts`, que só tem `home` e `room`).
//!
//! No site, `/s/:código` é uma rota só, e o `RoomRoute` mostra a tela de entrada enquanto não há sala aberta.
//! Aqui são duas, para o `App` saber o que está aberto: `Join` (prévia, apelido, sala não encontrada) e `Room`
//! (dentro da sala, com sinalização).

use espia_core::room_id::parse_room_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rota {
    /// A tela inicial.
    Home,
    /// Tela de entrada de `/s/:código` (ou a de sala não encontrada), sem sala aberta.
    Join { code: String },
    /// Dentro da sala.
    Room { code: String },
}

impl Rota {
    pub fn code(&self) -> Option<&str> {
        match self {
            Rota::Home => None,
            Rota::Join { code } | Rota::Room { code } => Some(code),
        }
    }
}

/// O código de sala de um link ou código colado (`parseRoomId` do site), ou `None`.
pub fn codigo_do_texto(texto: &str) -> Option<String> {
    parse_room_id(texto)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rota_diz_o_codigo() {
        assert_eq!(Rota::Home.code(), None);
        assert_eq!(Rota::Join { code: "aaa-aaa".into() }.code(), Some("aaa-aaa"));
        assert_eq!(Rota::Room { code: "bbb-bbb".into() }.code(), Some("bbb-bbb"));
    }

    #[test]
    fn link_e_codigo_colados() {
        assert_eq!(codigo_do_texto("https://espia.diuliano.com/s/k7f-q2x"), Some("k7f-q2x".into()));
        assert_eq!(codigo_do_texto(" K7F Q2X "), Some("k7f-q2x".into()));
        assert_eq!(codigo_do_texto("lixo"), None);
    }
}
