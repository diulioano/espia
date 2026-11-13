//! O que cada pessoa está transmitindo (espelha `src/lib/live-label.ts`, specs/012, research R1).
//! Nunca o título de uma janela: no app vai só o nome do programa; no site, só o tipo que o navegador
//! entregou; na câmera, o nome dela.

use crate::protocol::LIVE_LABEL_MAX;

/// Lado da câmera do celular.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    User,
    Environment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelInput {
    NativeWindow { app: String },
    NativeMonitor { title: String, monitors: u32 },
    /// `surface`: o que o navegador entregou (`monitor`, `window`, `browser`), se entregou.
    Browser { surface: Option<String> },
    Camera { title: Option<String>, facing: Option<Facing> },
}

fn is_js_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

/// `String.prototype.trim` do JavaScript (também tira o BOM, que o `trim` do Rust não tira).
fn trim_js(text: &str) -> &str {
    text.trim_matches(is_js_space)
}

pub fn live_label(input: &LabelInput) -> Option<String> {
    match input {
        LabelInput::NativeWindow { app } => {
            let app = trim_js(app);
            Some(cut(if app.is_empty() { "Janela" } else { app }))
        }
        LabelInput::NativeMonitor { title, monitors } => {
            let title = trim_js(title);
            Some(cut(if *monitors > 1 && !title.is_empty() { title } else { "Tela inteira" }))
        }
        LabelInput::Browser { surface } => match surface.as_deref() {
            Some("monitor") => Some("Tela inteira".into()),
            Some("window") => Some("Janela".into()),
            Some("browser") => Some("Aba do navegador".into()),
            _ => None,
        },
        LabelInput::Camera { title, facing } => {
            let title = title.as_deref().map(trim_js).unwrap_or("");
            if !title.is_empty() {
                return Some(cut(&format!("Câmera · {title}")));
            }
            Some(
                match facing {
                    Some(Facing::User) => "Câmera frontal",
                    Some(Facing::Environment) => "Câmera traseira",
                    None => "Câmera",
                }
                .into(),
            )
        }
    }
}

/// Corta em `LIVE_LABEL_MAX` caracteres, com reticências no fim.
fn cut(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() > LIVE_LABEL_MAX {
        let head: String = chars[..LIVE_LABEL_MAX - 1].iter().collect();
        format!("{}…", head.trim_end_matches(is_js_space))
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    //! 5 casos no TS, 5 portados. Mais 1 caso extra sem par no TS (câmera sem nada, corte por caractere).
    use super::*;

    fn janela(app: &str) -> LabelInput {
        LabelInput::NativeWindow { app: app.into() }
    }
    fn tela(title: &str, monitors: u32) -> LabelInput {
        LabelInput::NativeMonitor { title: title.into(), monitors }
    }
    fn site(surface: Option<&str>) -> LabelInput {
        LabelInput::Browser { surface: surface.map(Into::into) }
    }
    fn cam(title: Option<&str>, facing: Option<Facing>) -> LabelInput {
        LabelInput::Camera { title: title.map(Into::into), facing }
    }

    #[test]
    fn app_janela_so_o_nome_do_programa() {
        assert_eq!(live_label(&janela("Hollow Knight")).as_deref(), Some("Hollow Knight"));
        assert_eq!(live_label(&janela("  ")).as_deref(), Some("Janela"));
    }

    #[test]
    fn app_tela_tela_inteira_ou_o_nome_com_mais_de_um_monitor() {
        assert_eq!(live_label(&tela("Tela 1", 1)).as_deref(), Some("Tela inteira"));
        assert_eq!(live_label(&tela("Tela 2", 2)).as_deref(), Some("Tela 2"));
    }

    #[test]
    fn site_o_tipo_que_o_navegador_entregou() {
        assert_eq!(live_label(&site(Some("monitor"))).as_deref(), Some("Tela inteira"));
        assert_eq!(live_label(&site(Some("window"))).as_deref(), Some("Janela"));
        assert_eq!(live_label(&site(Some("browser"))).as_deref(), Some("Aba do navegador"));
        assert_eq!(live_label(&site(None)), None);
    }

    #[test]
    fn camera() {
        assert_eq!(live_label(&cam(Some("HD Pro Webcam C920"), None)).as_deref(), Some("Câmera · HD Pro Webcam C920"));
        assert_eq!(live_label(&cam(None, Some(Facing::User))).as_deref(), Some("Câmera frontal"));
        assert_eq!(live_label(&cam(Some(""), Some(Facing::Environment))).as_deref(), Some("Câmera traseira"));
    }

    #[test]
    fn corta_em_60_com_reticencias() {
        let rotulo = live_label(&janela(&"x".repeat(80))).unwrap();
        assert_eq!(rotulo.chars().count(), 60);
        assert!(rotulo.ends_with('…'));
    }

    #[test]
    fn camera_sem_nada_e_corte_conta_caracteres() {
        assert_eq!(live_label(&cam(None, None)).as_deref(), Some("Câmera"));
        let rotulo = live_label(&janela(&"é".repeat(80))).unwrap();
        assert_eq!(rotulo.chars().count(), 60);
    }
}
