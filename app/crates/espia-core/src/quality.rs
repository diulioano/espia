//! Qualidade de quem transmite (espelha `src/lib/quality.ts`). Funções puras: sem WebRTC aqui.
//! Só `audioConstraints` e `videoConstraints` (de `devices.ts`) ficam de fora: são do navegador.

use crate::device::GoLive;

/// `RTCDegradationPreference`, só os dois valores que o site usa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationPreference {
    MaintainFramerate,
    MaintainResolution,
}

/// A dica de conteúdo manda; sem dica, câmera mantém a fluidez e tela mantém a nitidez.
/// `camera`: a fonte é a câmera (senão, tela).
pub fn content_preference(hint: &str, camera: bool) -> DegradationPreference {
    match hint {
        "motion" => DegradationPreference::MaintainFramerate,
        "detail" | "text" => DegradationPreference::MaintainResolution,
        _ if camera => DegradationPreference::MaintainFramerate,
        _ => DegradationPreference::MaintainResolution,
    }
}

/// Como alguém está vendo a SUA tela (vai pelo canal `ctl`).
/// - `Hidden`: janela em segundo plano. Sem vídeo, o som continua.
/// - `Off`: parou de assistir. Sem vídeo e sem som.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Full,
    Thumb,
    Hidden,
    Off,
}

/// O vídeo vai pra essa pessoa?
pub fn wants_video(mode: Option<ViewMode>) -> bool {
    matches!(mode, Some(ViewMode::Full | ViewMode::Thumb))
}

/// O som vai pra essa pessoa? (Só parar de assistir corta o som; sem modo, vai.)
pub fn wants_sound(mode: Option<ViewMode>) -> bool {
    mode != Some(ViewMode::Off)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quality {
    pub resolution: u16,
    pub fps: u16,
    /// bps para cada espectador em tela cheia/grade.
    pub max_bitrate: u32,
}

/// Resolução e fps, sem bitrate (`Step` de `quality.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub resolution: u16,
    pub fps: u16,
}

/// Bitrate máximo por espectador (research.md R10). Combinação fora da tabela: usa a faixa da resolução.
fn bitrate(resolution: u16, fps: u16) -> u32 {
    match (resolution >= 1080, fps >= 60) {
        (false, false) => 2_500_000,
        (false, true) => 4_000_000,
        (true, false) => 4_500_000,
        (true, true) => 6_000_000,
    }
}

/// Teto pelo número de pessoas vendo em tela cheia/grade (miniaturas e escondidas não contam).
/// Nunca sobe acima do que foi escolhido.
/// 1-4: o escolhido · 5-6: 720p60 (se escolheu 60) ou 1080p30, até 4 Mbps · 7+: 720p30.
/// `cpu_steps`: degraus a menos por PC no limite (specs/010), aplicados depois da regra de espectadores.
pub fn effective_quality(chosen: &GoLive, full_viewers: u32, cpu_steps: u32) -> Quality {
    let mut resolution = chosen.resolution;
    let mut fps = chosen.fps;
    let mut cap = u32::MAX;
    if full_viewers >= 7 {
        resolution = 720;
        fps = 30;
    } else if full_viewers >= 5 {
        if fps == 60 {
            resolution = 720;
        }
        cap = 4_000_000;
    }
    for _ in 0..cpu_steps {
        let s = step_down(Step { resolution, fps });
        resolution = s.resolution;
        fps = s.fps;
    }
    Quality { resolution, fps, max_bitrate: bitrate(resolution, fps).min(cap) }
}

/// Um degrau abaixo: primeiro o fps, depois a resolução. Mínimo 720p30.
pub fn step_down(q: Step) -> Step {
    if q.fps == 60 {
        return Step { resolution: q.resolution, fps: 30 };
    }
    Step { resolution: 720, fps: 30 }
}

/// Um degrau acima, sem passar do escolhido.
pub fn step_up(q: Step, chosen: Step) -> Step {
    if q.resolution < chosen.resolution {
        return Step { resolution: chosen.resolution, fps: 30 };
    }
    if q.fps < chosen.fps {
        return Step { resolution: q.resolution, fps: chosen.fps };
    }
    q
}

pub fn is_adjusted(chosen: &GoLive, q: &Quality) -> bool {
    q.resolution != chosen.resolution || q.fps != chosen.fps
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SendParams {
    pub active: bool,
    pub scale_resolution_down_by: f64,
    pub max_framerate: u16,
    pub max_bitrate: u32,
}

/// Miniatura: ~360p · 15 fps · 0,5 Mbps. Escondida e parada: pausa o vídeo (o som continua, specs/014).
pub fn send_params(q: &Quality, mode: ViewMode) -> SendParams {
    match mode {
        ViewMode::Hidden | ViewMode::Off => SendParams {
            active: false,
            scale_resolution_down_by: 1.0,
            max_framerate: q.fps,
            max_bitrate: q.max_bitrate,
        },
        ViewMode::Thumb => SendParams {
            active: true,
            scale_resolution_down_by: f64::from(q.resolution) / 360.0,
            max_framerate: 15,
            max_bitrate: 500_000,
        },
        ViewMode::Full => SendParams {
            active: true,
            scale_resolution_down_by: 1.0,
            max_framerate: q.fps,
            max_bitrate: q.max_bitrate,
        },
    }
}

pub fn count_full_viewers(modes: impl IntoIterator<Item = ViewMode>) -> u32 {
    modes.into_iter().filter(|m| *m == ViewMode::Full).count() as u32
}

#[cfg(test)]
mod tests {
    //! 13 casos no TS (12 de `quality.test.ts` + 1 de `quality-content.test.ts`), 13 portados.
    //! Mais 1 caso extra sem par no TS (`view_mode_quer_video_e_som`, para `wantsVideo`/`wantsSound`).
    use super::*;

    fn escolha(resolution: u16, fps: u16) -> GoLive {
        GoLive { resolution, fps, ..GoLive::default() }
    }
    fn q(resolution: u16, fps: u16, max_bitrate: u32) -> Quality {
        Quality { resolution, fps, max_bitrate }
    }
    fn s(resolution: u16, fps: u16) -> Step {
        Step { resolution, fps }
    }

    // effectiveQuality
    #[test]
    fn de_1_a_4_espectadores_o_que_foi_escolhido() {
        assert_eq!(effective_quality(&escolha(1080, 60), 4, 0), q(1080, 60, 6_000_000));
        assert_eq!(effective_quality(&escolha(720, 30), 1, 0), q(720, 30, 2_500_000));
    }

    #[test]
    fn de_5_a_6_720p60_se_escolheu_60_senao_1080p30_ate_4_mbps() {
        assert_eq!(effective_quality(&escolha(1080, 60), 5, 0), q(720, 60, 4_000_000));
        assert_eq!(effective_quality(&escolha(1080, 30), 6, 0), q(1080, 30, 4_000_000));
    }

    #[test]
    fn de_7_em_diante_720p30() {
        assert_eq!(effective_quality(&escolha(1080, 60), 7, 0), q(720, 30, 2_500_000));
        assert_eq!(effective_quality(&escolha(1080, 30), 9, 0), q(720, 30, 2_500_000));
    }

    #[test]
    fn nunca_sobe_acima_do_escolhido() {
        assert_eq!(effective_quality(&escolha(720, 30), 5, 0), q(720, 30, 2_500_000));
    }

    #[test]
    fn is_adjusted_so_quando_resolucao_ou_fps_mudaram() {
        let (a, b) = (escolha(1080, 60), escolha(1080, 30));
        assert!(!is_adjusted(&a, &effective_quality(&a, 4, 0)));
        assert!(is_adjusted(&a, &effective_quality(&a, 5, 0)));
        assert!(!is_adjusted(&b, &effective_quality(&b, 5, 0)));
    }

    // sendParams
    #[test]
    fn cheia_qualidade_efetiva() {
        let p = send_params(&q(1080, 60, 6_000_000), ViewMode::Full);
        assert_eq!(p, SendParams { active: true, scale_resolution_down_by: 1.0, max_framerate: 60, max_bitrate: 6_000_000 });
    }

    #[test]
    fn miniatura_360p_15_fps_meio_mbps() {
        let p = send_params(&q(1080, 60, 6_000_000), ViewMode::Thumb);
        assert_eq!(p, SendParams { active: true, scale_resolution_down_by: 3.0, max_framerate: 15, max_bitrate: 500_000 });
    }

    #[test]
    fn escondida_pausa() {
        let base = q(1080, 60, 6_000_000);
        assert!(!send_params(&base, ViewMode::Hidden).active);
        // O TS só testa `hidden`; `off` tem a mesma regra e fica coberto na mesma asserção.
        assert!(!send_params(&base, ViewMode::Off).active);
    }

    #[test]
    fn count_full_viewers_conta_so_full() {
        let modos = [ViewMode::Full, ViewMode::Thumb, ViewMode::Hidden, ViewMode::Full];
        assert_eq!(count_full_viewers(modos), 2);
    }

    // degraus por CPU (specs/010)
    #[test]
    fn step_down_primeiro_o_fps_depois_a_resolucao_minimo_720p30() {
        assert_eq!(step_down(s(1080, 60)), s(1080, 30));
        assert_eq!(step_down(s(1080, 30)), s(720, 30));
        assert_eq!(step_down(s(720, 60)), s(720, 30));
        assert_eq!(step_down(s(720, 30)), s(720, 30));
    }

    #[test]
    fn step_up_o_inverso_sem_passar_do_escolhido() {
        assert_eq!(step_up(s(720, 30), s(1080, 60)), s(1080, 30));
        assert_eq!(step_up(s(1080, 30), s(1080, 60)), s(1080, 60));
        assert_eq!(step_up(s(720, 30), s(720, 60)), s(720, 60));
        assert_eq!(step_up(s(1080, 60), s(1080, 60)), s(1080, 60));
    }

    #[test]
    fn degraus_de_cpu_valem_o_mais_baixo_junto_com_espectadores() {
        let c = escolha(1080, 60);
        assert_eq!(effective_quality(&c, 3, 1), q(1080, 30, 4_500_000));
        assert_eq!(effective_quality(&c, 3, 2), q(720, 30, 2_500_000));
        // 5 espectadores já levam a 720p60 (até 4 Mbps); 1 degrau de CPU leva a 720p30.
        assert_eq!(effective_quality(&c, 5, 1), q(720, 30, 2_500_000));
        assert!(is_adjusted(&c, &effective_quality(&c, 3, 1)));
    }

    // quality-content.test.ts
    #[test]
    fn dica_de_conteudo_vem_antes_da_fonte() {
        use DegradationPreference::*;
        assert_eq!(content_preference("motion", false), MaintainFramerate);
        assert_eq!(content_preference("text", true), MaintainResolution);
        assert_eq!(content_preference("", true), MaintainFramerate);
        assert_eq!(content_preference("", false), MaintainResolution);
    }

    #[test]
    fn view_mode_quer_video_e_som() {
        assert!(wants_video(Some(ViewMode::Full)) && wants_video(Some(ViewMode::Thumb)));
        assert!(!wants_video(Some(ViewMode::Hidden)) && !wants_video(Some(ViewMode::Off)) && !wants_video(None));
        assert!(wants_sound(Some(ViewMode::Hidden)) && wants_sound(None));
        assert!(!wants_sound(Some(ViewMode::Off)));
    }
}
