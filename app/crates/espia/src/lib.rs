//! Espia.exe: junta todas as crates. O laço principal (T034) está em `app`; as telas reais, em `home` e
//! `room_screen`. A biblioteca existe para os testes enxergarem o `App` sem a Slint.

pub mod app;
pub mod cli;
pub mod clipboard;
pub mod router;
pub mod saved_rooms;

mod bind;
mod home;
mod room_screen;

/// O que o programa vai fazer, decidido só pelos argumentos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modo {
    /// `--mock`: telas com dados falsos. Não cria runtime do tokio, não abre rede nem lê arquivos do aparelho.
    Mock,
    /// O app de verdade.
    App,
}

pub fn modo(cli: &cli::Cli) -> Modo {
    if cli.mock.is_some() { Modo::Mock } else { Modo::App }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_nao_e_o_app() {
        let mock = cli::parse(["--mock", "home"]).unwrap();
        assert_eq!(modo(&mock), Modo::Mock);
        // O link é ignorado com --mock (contrato): continua sendo o mock.
        let mock_com_link = cli::parse(["espia://s/k7f-q2x", "--mock"]).unwrap();
        assert_eq!(modo(&mock_com_link), Modo::Mock);
        assert_eq!(mock_com_link.link, None);
        assert_eq!(modo(&cli::parse(Vec::<String>::new()).unwrap()), Modo::App);
        assert_eq!(modo(&cli::parse(["espia://s/k7f-q2x"]).unwrap()), Modo::App);
    }
}
