//! Textos derivados da prévia da sala na tela de entrada (espelha `src/lib/join-view.ts`, D2 / M1).

use crate::protocol::Tone;

pub use crate::protocol::NICKNAME_MAX;

/// Uma pessoa da prévia (`JoinPreviewPerson`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinPreviewPerson {
    pub name: String,
    pub tone: Tone,
    pub live: bool,
}

/// "Lu", "Lu e Bia", "Lu, Bia e Tomás".
pub fn list_names<S: AsRef<str>>(names: &[S]) -> String {
    match names {
        [] => String::new(),
        [one] => one.as_ref().to_string(),
        [init @ .., last] => {
            let init: Vec<&str> = init.iter().map(AsRef::as_ref).collect();
            format!("{} e {}", init.join(", "), last.as_ref())
        }
    }
}

fn live_names(people: &[JoinPreviewPerson]) -> Vec<&str> {
    people.iter().filter(|p| p.live).map(|p| p.name.as_str()).collect()
}

/// "Lu ao vivo", "Lu e Bia ao vivo", "3 ao vivo". `None` quando ninguém transmite.
pub fn live_label(people: &[JoinPreviewPerson]) -> Option<String> {
    let live = live_names(people);
    match live.len() {
        0 => None,
        1 | 2 => Some(format!("{} ao vivo", list_names(&live))),
        n => Some(format!("{n} ao vivo")),
    }
}

/// "Lu, Bia e Tomás já estão lá" / "Lu já está lá".
pub fn who_is_there(people: &[JoinPreviewPerson]) -> String {
    let names: Vec<&str> = people.iter().map(|p| p.name.as_str()).collect();
    format!("{} {}", list_names(&names), if names.len() == 1 { "já está lá" } else { "já estão lá" })
}

/// Versão do celular: "Lu, Bia e Tomás já estão lá. Lu está ao vivo."
pub fn who_is_there_long(people: &[JoinPreviewPerson]) -> String {
    let live = live_names(people);
    let live_text = if live.is_empty() {
        String::new()
    } else {
        format!(" {} {} ao vivo.", list_names(&live), if live.len() == 1 { "está" } else { "estão" })
    };
    format!("{}.{}", who_is_there(people), live_text)
}

/// Apelido limpo: sem espaços nas pontas, até `NICKNAME_MAX` caracteres.
pub fn clean_nickname(value: &str) -> String {
    value
        .trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
        .chars()
        .take(NICKNAME_MAX)
        .collect()
}

#[cfg(test)]
mod tests {
    //! 0 casos no TS (`join-view.ts` não tem arquivo de teste), 0 portados. Os casos abaixo são novos e
    //! seguem os exemplos dos comentários do TS.
    use super::*;

    fn p(name: &str, live: bool) -> JoinPreviewPerson {
        JoinPreviewPerson { name: name.into(), tone: Tone::Sage, live }
    }

    #[test]
    fn lista_de_nomes() {
        assert_eq!(list_names::<&str>(&[]), "");
        assert_eq!(list_names(&["Lu"]), "Lu");
        assert_eq!(list_names(&["Lu", "Bia"]), "Lu e Bia");
        assert_eq!(list_names(&["Lu", "Bia", "Tomás"]), "Lu, Bia e Tomás");
    }

    #[test]
    fn rotulo_ao_vivo() {
        assert_eq!(live_label(&[p("Lu", false)]), None);
        assert_eq!(live_label(&[p("Lu", true), p("Bia", false)]).as_deref(), Some("Lu ao vivo"));
        assert_eq!(live_label(&[p("Lu", true), p("Bia", true)]).as_deref(), Some("Lu e Bia ao vivo"));
        assert_eq!(live_label(&[p("Lu", true), p("Bia", true), p("Tomás", true)]).as_deref(), Some("3 ao vivo"));
    }

    #[test]
    fn quem_esta_la() {
        assert_eq!(who_is_there(&[p("Lu", false)]), "Lu já está lá");
        assert_eq!(who_is_there(&[p("Lu", false), p("Bia", false), p("Tomás", false)]), "Lu, Bia e Tomás já estão lá");
    }

    #[test]
    fn quem_esta_la_versao_longa() {
        let gente = [p("Lu", true), p("Bia", false), p("Tomás", false)];
        assert_eq!(who_is_there_long(&gente), "Lu, Bia e Tomás já estão lá. Lu está ao vivo.");
        assert_eq!(who_is_there_long(&[p("Lu", false)]), "Lu já está lá.");
        assert_eq!(who_is_there_long(&[p("Lu", true), p("Bia", true)]), "Lu e Bia já estão lá. Lu e Bia estão ao vivo.");
    }

    #[test]
    fn apelido_limpo() {
        assert_eq!(clean_nickname("  Lu  "), "Lu");
        assert_eq!(clean_nickname(&"a".repeat(30)).chars().count(), 24);
        assert_eq!(clean_nickname(&"é".repeat(30)), "é".repeat(24));
        assert_eq!(NICKNAME_MAX, 24);
    }
}
