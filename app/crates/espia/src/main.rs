//! Espia.exe: junta todas as crates. `--mock` abre as telas com dados falsos (T027); sem `--mock` roda o app de
//! verdade (T034): a Slint na thread principal e o tokio em outra, ligados por canais (ver `espia::app`).

use std::process::ExitCode;

use espia::app::{self, App, Config, Saida};
use espia::cli::{self, Cli};
use espia::{Modo, clipboard, modo};
use espia_ui::mocks::{self, RunOptions};
use espia_ui::{Ui, UiAction, UiError};

fn main() -> ExitCode {
    // O primeiro argumento é o caminho do executável.
    let cli = match cli::parse(std::env::args().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("espia: {e}");
            return ExitCode::from(2);
        }
    };

    let resultado = match modo(&cli) {
        Modo::Mock => rodar_mock(&cli),
        Modo::App => rodar_app(cli),
    };
    match resultado {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("espia: {e}");
            ExitCode::FAILURE
        }
    }
}

/// O modo mock: sem tokio, sem rede e sem arquivos do aparelho.
fn rodar_mock(cli: &Cli) -> Result<(), UiError> {
    let opts = RunOptions { exit_after: cli.exit_after, screenshot: cli.screenshot.clone() };
    let mock = cli.mock.as_ref().expect("Modo::Mock só sai de um --mock");
    mocks::run(mock.screen.as_deref(), mock.scene.as_deref(), None, opts)
}

/// O app de verdade. A thread principal é da Slint (a tela só vê view-models); o `App`, dono do estado, vive numa
/// thread própria com um runtime do tokio de uma thread só.
fn rodar_app(cli: Cli) -> Result<(), UiError> {
    let ui = Ui::new()?;
    let remoto = ui.remote();
    let (entradas, recebe) = app::canal();

    // A tela devolve ações; o `App` devolve telas e pedidos de cópia.
    let para_o_app = entradas.clone();
    ui.on_action(move |acao: UiAction| {
        let _ = para_o_app.send(app::Entrada::Ui(acao));
    });
    let saida = {
        let remoto = remoto.clone();
        move |s: Saida| match s {
            Saida::Tela(tela) => remoto.show_live(tela),
            Saida::Copiar(texto) => remoto.run_in_ui(move || {
                clipboard::copiar(&texto);
            }),
        }
    };

    let config = Config::do_ambiente();
    let link = cli.link.clone();
    let tokio_thread = std::thread::Builder::new()
        .name("espia-tokio".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime do tokio");
            let app = App::novo(config, entradas, saida);
            runtime.block_on(app::laco(app, recebe, link));
        })
        .map_err(|e| UiError::Platform(format!("não deu para criar a thread de rede: {e}")))?;
    let _ = tokio_thread; // termina junto com o processo

    if let Some(d) = cli.exit_after {
        ui.quit_after(d);
    }
    if let Some(arquivo) = cli.screenshot {
        ui.screenshot_after(std::time::Duration::from_secs(1), arquivo);
    }
    ui.run()
}
