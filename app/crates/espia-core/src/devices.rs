//! Câmeras, placas de captura e entradas de som (espelha `src/lib/devices.ts`, specs/006 research R3-R5).
//! Funções puras. Sem a crate `regex`: as expressões do TS viram buscas de trecho em texto minúsculo.
//! `audioConstraints` e `videoConstraints` não portam (são de navegador).

use std::collections::HashSet;

pub use crate::device::AudioRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: String,
    pub group_id: String,
    pub label: String,
}

/// Dispositivo candidato a prévia (`busy`: em uso por outra coisa).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewDevice {
    pub id: String,
    pub busy: bool,
}

/// Até quantas prévias ao vivo ao mesmo tempo (FR-603).
pub const PREVIEW_LIMIT: usize = 4;

/// `cam ?link`
const CAM_LINK: [&str; 2] = ["camlink", "cam link"];

fn has_any(lower: &str, words: &[&str]) -> bool {
    words.iter().any(|w| lower.contains(w))
}

/// `usb ?3?\.?0? ?video`: depois de "usb", espaço, "3", ponto, "0" e espaço, todos opcionais, e "video".
fn has_usb_video(lower: &str) -> bool {
    lower.match_indices("usb").any(|(i, _)| {
        let rest = &lower[i + 3..];
        (0u8..32).any(|mask| {
            let mut prefix = String::new();
            for (bit, piece) in [" ", "3", ".", "0", " "].iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    prefix.push_str(piece);
                }
            }
            rest.strip_prefix(prefix.as_str()).is_some_and(|r| r.starts_with("video"))
        })
    })
}

/// O nome parece de placa de captura (ícone e sugestão de som).
pub fn is_capture_card(label: &str) -> bool {
    let l = label.to_lowercase();
    has_any(&l, &["capture", "hdmi", "elgato", "avermedia", "game", "live gamer"]) || has_any(&l, &CAM_LINK) || has_usb_video(&l)
}

/// Voz (com cancelamento de eco) ou jogo (cru)? Na dúvida, voz: protege contra eco.
/// `paired_with_capture`: a entrada é do mesmo aparelho de uma placa de captura.
pub fn audio_role(label: &str, paired_with_capture: bool) -> AudioRole {
    let l = label.to_lowercase();
    if has_any(&l, &["mic", "microfone", "headset", "fone", "headphone"]) {
        return AudioRole::Voice;
    }
    let game = has_any(&l, &["capture", "hdmi", "line", "elgato", "avermedia", "game", "digital audio interface", "live gamer"])
        || has_any(&l, &CAM_LINK);
    if paired_with_capture || game {
        AudioRole::Game
    } else {
        AudioRole::Voice
    }
}

const IGNORED_TOKENS: [&str; 8] = ["default", "padrão", "usb", "audio", "áudio", "device", "video", "vídeo"];

/// Tira "(046d:0825)" (ids USB, hexadecimal minúsculo) trocando por espaço.
fn strip_usb_ids(lower: &str) -> String {
    let chars: Vec<char> = lower.chars().collect();
    let is_id = |w: &[char]| {
        w.len() == 11
            && w[0] == '('
            && w[5] == ':'
            && w[10] == ')'
            && w[1..5].iter().chain(&w[6..10]).all(|c| c.is_ascii_digit() || ('a'..='f').contains(c))
    };
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if i + 11 <= chars.len() && is_id(&chars[i..i + 11]) {
            out.push(' ');
            i += 11;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Palavras do nome: minúsculas, `[a-z0-9à-ú]`, 3 letras ou mais, sem as genéricas.
fn tokens(label: &str) -> HashSet<String> {
    strip_usb_ids(&label.to_lowercase())
        .split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || ('à'..='ú').contains(&c)))
        .filter(|t| t.chars().count() >= 3 && !IGNORED_TOKENS.contains(t))
        .map(str::to_string)
        .collect()
}

fn is_generic(id: &str) -> bool {
    id == "default" || id == "communications"
}

/// A entrada de som do mesmo aparelho (`group_id`) ou de nome parecido; senão nenhuma.
pub fn suggest_audio<'a>(video: &DeviceInfo, audios: &'a [DeviceInfo]) -> Option<&'a DeviceInfo> {
    if !video.group_id.is_empty() {
        if let Some(same) = audios.iter().find(|a| a.group_id == video.group_id && !is_generic(&a.id)) {
            return Some(same);
        }
    }
    let want = tokens(&video.label);
    let mut best = None;
    let mut score = 0;
    for a in audios {
        if is_generic(&a.id) {
            continue;
        }
        let s = tokens(&a.label).iter().filter(|t| want.contains(*t)).count();
        if s > score {
            score = s;
            best = Some(a);
        }
    }
    best
}

/// Quais dispositivos ganham prévia ao vivo: a escolhida primeiro, depois os livres, até `PREVIEW_LIMIT`.
pub fn preview_ids(devices: &[PreviewDevice], selected_id: Option<&str>) -> Vec<String> {
    preview_ids_with_limit(devices, selected_id, PREVIEW_LIMIT)
}

/// Igual a `preview_ids`, com limite explícito.
pub fn preview_ids_with_limit(devices: &[PreviewDevice], selected_id: Option<&str>, limit: usize) -> Vec<String> {
    let free: Vec<&PreviewDevice> = devices.iter().filter(|d| !d.busy).collect();
    let first = free.iter().find(|d| Some(d.id.as_str()) == selected_id);
    let rest = free.iter().filter(|d| Some(d.id.as_str()) != selected_id);
    first.into_iter().chain(rest).take(limit).map(|d| d.id.clone()).collect()
}

#[cfg(test)]
mod tests {
    //! 14 casos no TS, 11 portados. Não portados: os 3 de `constraints` (`audioConstraints` e
    //! `videoConstraints` são de navegador). Mais 2 casos extras sem par no TS (regex à mão e limite).
    use super::*;

    fn dev(id: &str, label: &str, group_id: &str) -> DeviceInfo {
        DeviceInfo { id: id.into(), label: label.into(), group_id: group_id.into() }
    }

    #[test]
    fn placas_conhecidas() {
        for l in ["Elgato HD60 X", "Cam Link 4K", "AVerMedia Live Gamer MINI", "USB3 Video", "HDMI Capture"] {
            assert!(is_capture_card(l), "{l}");
        }
    }

    #[test]
    fn webcams_e_cameras_virtuais_nao() {
        for l in ["HD Pro Webcam C920 (046d:082d)", "OBS Virtual Camera", "Integrated Camera"] {
            assert!(!is_capture_card(l), "{l}");
        }
    }

    #[test]
    fn microfones_sao_voz() {
        assert_eq!(audio_role("Microfone (HD Pro Webcam C920)", false), AudioRole::Voice);
        assert_eq!(audio_role("Headset Microphone (HyperX)", false), AudioRole::Voice);
    }

    #[test]
    fn entrada_da_placa_e_jogo() {
        assert_eq!(audio_role("Digital Audio Interface (HD60 X)", false), AudioRole::Game);
        assert_eq!(audio_role("Line In (Realtek)", false), AudioRole::Game);
        assert_eq!(audio_role("Qualquer coisa", true), AudioRole::Game);
    }

    #[test]
    fn na_duvida_voz() {
        assert_eq!(audio_role("Entrada desconhecida", false), AudioRole::Voice);
    }

    fn audios() -> Vec<DeviceInfo> {
        vec![
            dev("default", "Padrão - Microfone (Realtek)", "g0"),
            dev("a1", "Microfone (Realtek Audio)", "g0"),
            dev("a2", "Digital Audio Interface (HD60 X)", "g-elgato"),
            dev("a3", "Microfone (HD Pro Webcam C920)", "g-c920"),
        ]
    }

    #[test]
    fn mesmo_aparelho_group_id_primeiro() {
        let lista = audios();
        let s = suggest_audio(&dev("v1", "Elgato HD60 X", "g-elgato"), &lista);
        assert_eq!(s.map(|d| d.id.as_str()), Some("a2"));
    }

    #[test]
    fn sem_group_id_igual_nome_parecido() {
        let lista = audios();
        let s = suggest_audio(&dev("v2", "HD Pro Webcam C920 (046d:082d)", "outro"), &lista);
        assert_eq!(s.map(|d| d.id.as_str()), Some("a3"));
    }

    #[test]
    fn nada_parecido_nenhuma() {
        let lista = audios();
        assert_eq!(suggest_audio(&dev("v3", "OBS Virtual Camera", "g-obs"), &lista), None);
    }

    #[test]
    fn nunca_sugere_as_entradas_genericas_default_e_communications() {
        let lista = [dev("default", "Padrão", "g0"), dev("communications", "Comunicações", "g0")];
        assert_eq!(suggest_audio(&dev("v4", "Webcam", "g0"), &lista), None);
    }

    fn lista_previa() -> Vec<PreviewDevice> {
        ["a", "b", "c", "d", "e", "f"].iter().map(|id| PreviewDevice { id: (*id).into(), busy: *id == "b" }).collect()
    }

    #[test]
    fn no_maximo_4_sem_os_ocupados() {
        assert_eq!(preview_ids(&lista_previa(), None), ["a", "c", "d", "e"]);
    }

    #[test]
    fn a_escolhida_sempre_entra_primeiro() {
        assert_eq!(preview_ids(&lista_previa(), Some("f")), ["f", "a", "c", "d"]);
    }

    #[test]
    fn variacoes_do_nome_de_placa_e_limite_explicito() {
        // Sem equivalente no TS: `cam ?link` e `usb ?3?\.?0? ?video` escritos à mão, e o limite.
        for l in ["CamLink", "USB 3.0 Video", "usb video", "USB3.0 VIDEO"] {
            assert!(is_capture_card(l), "{l}");
        }
        assert!(!is_capture_card("USB Webcam"));
        assert_eq!(preview_ids_with_limit(&lista_previa(), None, 2), ["a", "c"]);
        // Selecionada ocupada não entra (só vale entre os livres).
        assert_eq!(preview_ids(&lista_previa(), Some("b")), ["a", "c", "d", "e"]);
    }
}
