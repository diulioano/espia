//! Área de transferência do Windows (copiar o convite). Só a thread da Slint chama.

/// Copia o texto. `false` se o Windows não deixou (outro programa segurando a área de transferência).
pub fn copiar(texto: &str) -> bool {
    match arboard::Clipboard::new().and_then(|mut c| c.set_text(texto.to_string())) {
        Ok(()) => true,
        Err(e) => {
            if cfg!(debug_assertions) {
                eprintln!("espia: não deu para copiar o convite: {e}");
            }
            false
        }
    }
}
