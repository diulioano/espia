//! Gera `ui/tokens.slint` a partir de `src/ui/tokens.css` (R18) e compila a interface Slint.

#[allow(dead_code)]
#[path = "src/tokens.rs"]
mod tokens;

use std::path::PathBuf;

fn main() {
    let manifesto = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    // app/crates/espia-ui -> raiz do repositório
    let css_path = manifesto.join("../../../src/ui/tokens.css");
    println!("cargo:rerun-if-changed={}", css_path.display());
    println!("cargo:rerun-if-changed=src/tokens.rs");

    let css = std::fs::read_to_string(&css_path)
        .unwrap_or_else(|e| panic!("não consegui ler {}: {e}", css_path.display()));
    let slint = tokens::generate_slint(&css).unwrap_or_else(|e| panic!("tokens.css: {e}"));

    // Só regrava quando mudou, para não acordar o cargo à toa.
    let destino = manifesto.join("ui/tokens.slint");
    if std::fs::read_to_string(&destino).ok().as_deref() != Some(slint.as_str()) {
        std::fs::write(&destino, slint).expect("não consegui gravar ui/tokens.slint");
    }

    slint_build::compile("ui/app.slint").expect("falha ao compilar a interface Slint");
}
