//! Conversão view-model -> propriedades da Slint, uma tela por arquivo.
//! Cada arquivo tem `apply` (view-model para propriedades) e `connect` (callbacks para ações).

pub(crate) mod home;
pub(crate) mod join;
pub(crate) mod notfound;
pub(crate) mod offline;
pub(crate) mod room;

/// Inicial em maiúscula para o avatar (a Slint não tem "primeira letra"): "bia" vira "B".
pub fn initial(nome: &str) -> String {
    nome.trim().chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default()
}

/// Tom do view-model (`Tone`) para o da Slint.
#[allow(dead_code)] // usado pelas telas (T028 em diante)
pub(crate) fn tone(t: espia_core::protocol::Tone) -> crate::Tone {
    use espia_core::protocol::Tone as T;
    match t {
        T::Accent => crate::Tone::Accent,
        T::Sage => crate::Tone::Sage,
        T::Sand => crate::Tone::Sand,
        T::Deep => crate::Tone::Deep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inicial_em_maiuscula() {
        assert_eq!(initial("bia"), "B");
        assert_eq!(initial("  émerson"), "É");
        assert_eq!(initial(""), "");
    }
}
