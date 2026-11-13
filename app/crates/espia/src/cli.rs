//! Linha de comando do `Espia.exe` (contracts/app-nativo.md, "Linha de comando"):
//!
//! ```text
//! Espia.exe [<link>] [--mock [<tela>]] [--scene <cena>] [--exit-after <segundos>] [--screenshot <arquivo.png>]
//! ```
//!
//! `--exit-after` e `--screenshot` são para conferência automática (fecham a janela sozinhas).

use espia_core::room_id::parse_room_id;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

/// `--mock [<tela>] [--scene <cena>]`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mock {
    /// `None` = o índice.
    pub screen: Option<String>,
    pub scene: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cli {
    pub mock: Option<Mock>,
    /// Código da sala do link (`espia://s/<código>`). Sempre `None` com `--mock`: o mock vale e o link é ignorado.
    pub link: Option<String>,
    pub exit_after: Option<Duration>,
    pub screenshot: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// A opção precisa de um valor e ele não veio.
    FaltaValor(&'static str),
    /// `--exit-after` pede um número de segundos (0 ou mais).
    NumeroInvalido { opcao: &'static str, valor: String },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::FaltaValor(o) => write!(f, "a opção {o} precisa de um valor"),
            CliError::NumeroInvalido { opcao, valor } => {
                write!(f, "a opção {opcao} pede um número de segundos, veio “{valor}”")
            }
        }
    }
}

impl std::error::Error for CliError {}

/// Um nome de tela: não começa com `-` e não parece link (`espia://s/…`).
fn parece_tela(s: &str) -> bool {
    !s.is_empty() && !s.starts_with('-') && !s.contains('/') && !s.contains(':')
}

/// Lê os argumentos (sem o caminho do executável). Argumentos desconhecidos são ignorados;
/// o primeiro que for um link de sala válido vira `link`. `--scene` sem `--mock` é ignorada.
pub fn parse<I, S>(args: I) -> Result<Cli, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let mut mock: Option<Mock> = None;
    let mut scene: Option<String> = None;
    let mut cli = Cli::default();
    let mut posicionais = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        // Aceita `--opcao=valor` além de `--opcao valor`.
        let (nome, junto) = match a.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n, Some(v.to_string())),
            _ => (a.as_str(), None),
        };
        match nome {
            "--mock" => {
                let tela = match junto {
                    Some(v) => Some(v),
                    None => match args.get(i + 1) {
                        Some(p) if parece_tela(p) => {
                            i += 1;
                            Some(p.clone())
                        }
                        _ => None,
                    },
                };
                mock = Some(Mock { screen: tela.filter(|t| !t.is_empty()), scene: None });
            }
            "--scene" => {
                scene = Some(valor("--scene", junto, &args, &mut i)?);
            }
            "--exit-after" => {
                let v = valor("--exit-after", junto, &args, &mut i)?;
                let s: f64 = v
                    .trim()
                    .parse()
                    .ok()
                    .filter(|s: &f64| s.is_finite() && *s >= 0.0)
                    .ok_or(CliError::NumeroInvalido { opcao: "--exit-after", valor: v.clone() })?;
                cli.exit_after = Some(Duration::from_secs_f64(s));
            }
            "--screenshot" => {
                cli.screenshot = Some(PathBuf::from(valor("--screenshot", junto, &args, &mut i)?));
            }
            _ => posicionais.push(a.clone()),
        }
        i += 1;
    }

    match mock {
        Some(mut m) => {
            m.scene = scene;
            cli.mock = Some(m);
        }
        None => cli.link = posicionais.iter().find_map(|p| parse_room_id(p)),
    }
    Ok(cli)
}

/// O valor de uma opção: o que veio depois do `=` ou o próximo argumento.
fn valor(opcao: &'static str, junto: Option<String>, args: &[String], i: &mut usize) -> Result<String, CliError> {
    if let Some(v) = junto {
        return Ok(v);
    }
    match args.get(*i + 1) {
        Some(v) => {
            *i += 1;
            Ok(v.clone())
        }
        None => Err(CliError::FaltaValor(opcao)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ler(args: &[&str]) -> Cli {
        parse(args.iter().copied()).expect("argumentos válidos")
    }

    #[test]
    fn sem_argumentos_abre_o_app() {
        assert_eq!(ler(&[]), Cli::default());
    }

    #[test]
    fn mock_sozinho_abre_o_indice() {
        let c = ler(&["--mock"]);
        assert_eq!(c.mock, Some(Mock { screen: None, scene: None }));
    }

    #[test]
    fn mock_com_tela_usa_a_cena_padrao() {
        let c = ler(&["--mock", "home"]);
        assert_eq!(c.mock, Some(Mock { screen: Some("home".into()), scene: None }));
    }

    #[test]
    fn mock_com_tela_e_cena() {
        let c = ler(&["--mock", "room", "--scene", "grid"]);
        assert_eq!(c.mock, Some(Mock { screen: Some("room".into()), scene: Some("grid".into()) }));
    }

    #[test]
    fn cena_pode_vir_antes_da_tela() {
        let c = ler(&["--scene", "grid", "--mock", "room"]);
        assert_eq!(c.mock, Some(Mock { screen: Some("room".into()), scene: Some("grid".into()) }));
    }

    #[test]
    fn forma_com_igual() {
        let c = ler(&["--mock=join", "--scene=full"]);
        assert_eq!(c.mock, Some(Mock { screen: Some("join".into()), scene: Some("full".into()) }));
    }

    #[test]
    fn tela_que_nao_existe_passa_adiante_para_o_indice_avisar() {
        let c = ler(&["--mock", "naoexiste"]);
        assert_eq!(c.mock.unwrap().screen.as_deref(), Some("naoexiste"));
    }

    #[test]
    fn cena_sem_mock_e_ignorada() {
        let c = ler(&["--scene", "grid"]);
        assert_eq!(c.mock, None);
        assert_eq!(c.link, None);
    }

    #[test]
    fn link_espia_vira_o_codigo_da_sala() {
        let c = ler(&["espia://s/k7f-q2x"]);
        assert_eq!(c.link.as_deref(), Some("k7f-q2x"));
        assert_eq!(c.mock, None);
    }

    #[test]
    fn link_https_tambem_vale() {
        let c = ler(&["https://espia.diuliano.com/s/k7f-q2x"]);
        assert_eq!(c.link.as_deref(), Some("k7f-q2x"));
    }

    #[test]
    fn argumentos_que_nao_sao_link_sao_ignorados() {
        let c = ler(&["--qualquer-coisa", "lixo", "espia://s/k7f-q2x"]);
        assert_eq!(c.link.as_deref(), Some("k7f-q2x"));
    }

    #[test]
    fn link_invalido_nao_vira_sala() {
        assert_eq!(ler(&["espia://qualquer-coisa"]).link, None);
    }

    #[test]
    fn mock_junto_de_link_vale_o_mock_e_o_link_e_ignorado() {
        let c = ler(&["espia://s/k7f-q2x", "--mock", "home"]);
        assert_eq!(c.link, None);
        assert_eq!(c.mock.unwrap().screen.as_deref(), Some("home"));
    }

    #[test]
    fn link_logo_depois_de_mock_nao_vira_nome_de_tela() {
        let c = ler(&["--mock", "espia://s/k7f-q2x"]);
        assert_eq!(c.link, None);
        assert_eq!(c.mock, Some(Mock { screen: None, scene: None }));
    }

    #[test]
    fn exit_after_em_segundos() {
        assert_eq!(ler(&["--mock", "--exit-after", "5"]).exit_after, Some(Duration::from_secs(5)));
        assert_eq!(ler(&["--exit-after", "0.5"]).exit_after, Some(Duration::from_millis(500)));
    }

    #[test]
    fn exit_after_invalido_e_erro() {
        assert!(matches!(parse(["--exit-after", "abc"]), Err(CliError::NumeroInvalido { .. })));
        assert!(matches!(parse(["--exit-after", "-1"]), Err(CliError::NumeroInvalido { .. })));
        assert_eq!(parse(["--exit-after"]), Err(CliError::FaltaValor("--exit-after")));
    }

    #[test]
    fn screenshot_pede_o_arquivo() {
        assert_eq!(ler(&["--screenshot", "a.png"]).screenshot, Some(PathBuf::from("a.png")));
        assert_eq!(parse(["--screenshot"]), Err(CliError::FaltaValor("--screenshot")));
    }

    #[test]
    fn scene_sem_valor_e_erro() {
        assert_eq!(parse(["--mock", "--scene"]), Err(CliError::FaltaValor("--scene")));
    }
}
