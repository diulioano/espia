//! Que som pode sair na transmissão (espelha `src/lib/audio-source.ts`, specs/002 data-model.md).
//! Função pura: sem navegador, testável.

use crate::protocol::AudioKind;

/// Superfície capturada. Não é `device::Surface`: aqui existe `Browser` (aba) e não existe `Camera`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureSurface {
    Browser,
    Window,
    Monitor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioDecision {
    pub kind: AudioKind,
    /// A faixa de áudio recebida tem que ser parada e removida antes de enviar.
    pub drop: bool,
    /// Mostrar o aviso "Tela inteira vai sem som...".
    pub warn: bool,
}

const fn decision(kind: AudioKind, drop: bool, warn: bool) -> AudioDecision {
    AudioDecision { kind, drop, warn }
}

pub struct AudioInput<'a> {
    pub wanted: bool,
    /// `None`: superfície desconhecida.
    pub surface: Option<CaptureSurface>,
    /// Rótulo da faixa de áudio recebida; `None` = veio sem faixa.
    pub audio_label: Option<&'a str>,
}

/// O Chrome/Edge dá ao som do PC inteiro o rótulo "System Audio" (research.md R3): `/system/i`.
fn is_system_label(label: &str) -> bool {
    label.to_lowercase().contains("system")
}

pub fn decide_audio(input: &AudioInput<'_>) -> AudioDecision {
    let has = input.audio_label.is_some();
    if !input.wanted {
        return decision(AudioKind::None, has, false);
    }
    if input.surface == Some(CaptureSurface::Monitor) {
        return decision(AudioKind::None, has, true);
    }
    let Some(label) = input.audio_label else {
        return decision(AudioKind::None, false, false);
    };
    match input.surface {
        Some(CaptureSurface::Browser) => decision(AudioKind::Tab, false, false),
        Some(CaptureSurface::Window) if is_system_label(label) => decision(AudioKind::None, true, true),
        Some(CaptureSurface::Window) => decision(AudioKind::Window, false, false),
        // Superfície desconhecida: na dúvida, nada de som.
        _ => decision(AudioKind::None, true, false),
    }
}

/// Motivo de estar sem som (só a tela inteira tem texto próprio).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoSoundReason {
    Monitor,
}

/// Texto do card "Você está ao vivo" (FR-207).
pub fn audio_label_text(kind: AudioKind, reason: Option<NoSoundReason>) -> &'static str {
    match kind {
        AudioKind::Tab => "com som da aba",
        AudioKind::Window => "com som da janela",
        AudioKind::Pc => "com som do PC (sem Discord)",
        AudioKind::Device => "com som do dispositivo",
        AudioKind::None if reason == Some(NoSoundReason::Monitor) => "sem som (tela inteira)",
        AudioKind::None => "sem som",
    }
}

/// Texto sob "Compartilhar áudio" no seletor do app desktop (specs/005 data-model.md).
/// `selected_title` vazio vale como ausente (como no TS).
pub fn native_audio_text(surface: CaptureSurface, audio: bool, selected_title: Option<&str>) -> String {
    if !audio {
        return "Só a imagem vai pra sala.".into();
    }
    if surface == CaptureSurface::Monitor {
        return "Todo o som do computador vai junto — menos o Discord.".into();
    }
    match selected_title.filter(|t| !t.is_empty()) {
        Some(title) => format!("Só o som de {title} — nada de notificação vazando."),
        None => "Só o som da janela que você escolher — Discord e notificações ficam de fora.".into(),
    }
}

#[cfg(test)]
mod tests {
    //! 15 casos no TS (10 linhas da tabela de `decideAudio` + 1 + 4), 15 portados.
    //! Os 10 da tabela viram 10 `#[test]` separados, um por linha, como no `it` por linha do TS.
    use super::*;
    use AudioKind::{None as Nenhum, Tab, Window};
    use CaptureSurface::{Browser, Monitor, Window as Janela};

    fn caso(wanted: bool, surface: Option<CaptureSurface>, audio_label: Option<&str>, esperado: AudioDecision) {
        assert_eq!(decide_audio(&AudioInput { wanted, surface, audio_label }), esperado);
    }

    #[test]
    fn desligado_sem_faixa() {
        caso(false, Some(Janela), None, decision(Nenhum, false, false));
    }
    #[test]
    fn desligado_veio_faixa() {
        caso(false, Some(Browser), Some("Tab audio"), decision(Nenhum, true, false));
    }
    #[test]
    fn aba_com_som() {
        caso(true, Some(Browser), Some("Tab audio"), decision(Tab, false, false));
    }
    #[test]
    fn janela_com_som_da_janela() {
        caso(true, Some(Janela), Some("Window audio"), decision(Window, false, false));
    }
    #[test]
    fn janela_com_som_do_sistema() {
        caso(true, Some(Janela), Some("System Audio"), decision(Nenhum, true, true));
    }
    #[test]
    fn tela_inteira_veio_som() {
        caso(true, Some(Monitor), Some("System Audio"), decision(Nenhum, true, true));
    }
    #[test]
    fn tela_inteira_sem_som() {
        caso(true, Some(Monitor), None, decision(Nenhum, false, true));
    }
    #[test]
    fn aba_sem_faixa() {
        caso(true, Some(Browser), None, decision(Nenhum, false, false));
    }
    #[test]
    fn janela_sem_faixa() {
        caso(true, Some(Janela), None, decision(Nenhum, false, false));
    }
    #[test]
    fn superficie_desconhecida_com_faixa() {
        caso(true, None, Some("Audio"), decision(Nenhum, true, false));
    }

    #[test]
    fn textos_do_card() {
        assert_eq!(audio_label_text(Tab, None), "com som da aba");
        assert_eq!(audio_label_text(Window, None), "com som da janela");
        assert_eq!(audio_label_text(Nenhum, None), "sem som");
        assert_eq!(audio_label_text(Nenhum, Some(NoSoundReason::Monitor)), "sem som (tela inteira)");
        assert_eq!(audio_label_text(AudioKind::Pc, None), "com som do PC (sem Discord)");
        assert_eq!(audio_label_text(AudioKind::Device, None), "com som do dispositivo");
    }

    #[test]
    fn janela_escolhida_com_som_cita_o_titulo() {
        assert_eq!(
            native_audio_text(Janela, true, Some("Hollow Knight")),
            "Só o som de Hollow Knight — nada de notificação vazando."
        );
    }

    #[test]
    fn janela_ainda_nao_escolhida() {
        assert_eq!(
            native_audio_text(Janela, true, None),
            "Só o som da janela que você escolher — Discord e notificações ficam de fora."
        );
    }

    #[test]
    fn tela_inteira_com_som_pc_menos_o_discord() {
        assert_eq!(
            native_audio_text(Monitor, true, Some("Tela 1")),
            "Todo o som do computador vai junto — menos o Discord."
        );
    }

    #[test]
    fn som_desligado() {
        assert_eq!(native_audio_text(Monitor, false, None), "Só a imagem vai pra sala.");
        assert_eq!(native_audio_text(Janela, false, Some("X")), "Só a imagem vai pra sala.");
    }
}
