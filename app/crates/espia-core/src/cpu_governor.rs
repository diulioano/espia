//! Ajuste automático por CPU de quem transmite (espelha `src/lib/cpu-governor.ts`, specs/010).
//! Função pura: quem chama lê o `qualityLimitationReason` do motor uma vez por segundo e aplica os
//! degraus em `effective_quality`. Sem relógio aqui: cada `tick` é um segundo.

/// Segundos seguidos no limite para baixar um degrau.
pub const DOWN_AFTER: u32 = 10;
/// Segundos seguidos com folga para subir um degrau.
pub const UP_AFTER: u32 = 60;
/// 1080p60 -> 1080p30 -> 720p30.
pub const MAX_STEPS: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuTick {
    Down,
    Up,
}

#[derive(Debug, Clone, Default)]
pub struct CpuGovernor {
    /// Degraus abaixo do escolhido (0 a `MAX_STEPS`).
    pub steps: u32,
    limited_for: u32,
    free_for: u32,
}

impl CpuGovernor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Um segundo a mais. `can_go_down`: ainda há degrau abaixo da qualidade em vigor.
    pub fn tick(&mut self, limited: bool, can_go_down: bool) -> Option<CpuTick> {
        if limited {
            self.free_for = 0;
            self.limited_for += 1;
            if self.limited_for >= DOWN_AFTER && can_go_down && self.steps < MAX_STEPS {
                self.steps += 1;
                self.limited_for = 0;
                return Some(CpuTick::Down);
            }
            return None;
        }
        self.limited_for = 0;
        self.free_for += 1;
        if self.free_for >= UP_AFTER && self.steps > 0 {
            self.steps -= 1;
            self.free_for = 0;
            return Some(CpuTick::Up);
        }
        None
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    //! 6 casos no TS, 6 portados.
    use super::*;

    fn rodar(g: &mut CpuGovernor, n: u32, limited: bool, can_go_down: bool) -> Vec<Option<CpuTick>> {
        (0..n).map(|_| g.tick(limited, can_go_down)).collect()
    }
    fn tem(v: &[Option<CpuTick>], t: CpuTick) -> bool {
        v.contains(&Some(t))
    }

    #[test]
    fn baixa_depois_de_10_s_seguidos_no_limite_nao_antes() {
        let mut g = CpuGovernor::new();
        assert!(rodar(&mut g, DOWN_AFTER - 1, true, true).iter().all(Option::is_none));
        assert_eq!(g.tick(true, true), Some(CpuTick::Down));
        assert_eq!(g.steps, 1);
    }

    #[test]
    fn um_segundo_com_folga_zera_a_contagem_do_limite() {
        let mut g = CpuGovernor::new();
        rodar(&mut g, DOWN_AFTER - 1, true, true);
        g.tick(false, true);
        assert!(!tem(&rodar(&mut g, DOWN_AFTER - 1, true, true), CpuTick::Down));
    }

    #[test]
    fn sobe_depois_de_60_s_seguidos_com_folga() {
        let mut g = CpuGovernor::new();
        rodar(&mut g, DOWN_AFTER, true, true);
        assert!(!tem(&rodar(&mut g, UP_AFTER - 1, false, true), CpuTick::Up));
        assert_eq!(g.tick(false, true), Some(CpuTick::Up));
        assert_eq!(g.steps, 0);
    }

    #[test]
    fn nao_passa_de_2_degraus_nem_baixa_sem_degrau_disponivel() {
        let mut g = CpuGovernor::new();
        rodar(&mut g, DOWN_AFTER * 5, true, true);
        assert_eq!(g.steps, 2);
        let mut h = CpuGovernor::new();
        rodar(&mut h, DOWN_AFTER * 2, true, false);
        assert_eq!(h.steps, 0);
    }

    #[test]
    fn sem_degrau_baixado_folga_nao_sobe_nada() {
        let mut g = CpuGovernor::new();
        assert!(!tem(&rodar(&mut g, UP_AFTER * 2, false, true), CpuTick::Up));
    }

    #[test]
    fn reset_volta_ao_comeco() {
        let mut g = CpuGovernor::new();
        rodar(&mut g, DOWN_AFTER * 2, true, true);
        g.reset();
        assert_eq!(g.steps, 0);
        assert!(!tem(&rodar(&mut g, DOWN_AFTER - 1, true, true), CpuTick::Down));
    }
}
