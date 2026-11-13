//! Regras puras do caminho pela placa (specs/020): quem recebe o quê, ações por canal, quadro
//! inteiro a pedido, internet de cada espectador e caminho de compressão.
//! Sem rede, sem placa, sem relógio: quem chama passa o instante (ms) ou chama uma vez por segundo,
//! como em `cpu_governor.rs`. Contratos: `specs/020-codificar-pela-placa/contracts/qualidades.md`
//! e `contracts/caminho.md`. Valem só no caminho pela placa; no caminho pelo processador a
//! biblioteca comprime por espectador e vale `quality::send_params`.
//!
//! # Desenho (T004, escrito pelo principal)
//!
//! Quem chama é sempre `espia-media::publish` (o publicador). Nada aqui conhece ligação, faixa ou
//! codificador: o publicador traduz as respostas em chamadas.
//!
//! ## 1. Quem recebe o quê (T004)
//!
//! - `Tier { Full, Reduced }`: uma qualidade comprimida (cheia ou reduzida).
//! - "O que um espectador recebe" é `Option<Tier>`; `None` é nada.
//! - `Viewer { mode: Option<ViewMode>, weak: bool }`: `mode` vem do `ViewMessage` (sem modo = não
//!   clicou em Assistir); `weak` vem de `BandwidthRule::tick` (seção 4).
//! - `receives(Viewer) -> Option<Tier>`: tabela da seção 1 do contrato.
//! - `ActiveTiers { full: bool, reduced: bool }` e `active_tiers(&[Viewer])`: seção 2.
//! - `ReducedTuning { Thumb, WeakLink }` e `reduced_tuning(&[Viewer]) -> Option<ReducedTuning>`:
//!   `WeakLink` se há pelo menos um `Full` fraco; `Thumb` se só miniaturas; `None` se ninguém
//!   recebe a reduzida.
//! - `full_quality(&GoLive, &[Viewer], cpu_steps) -> Quality`: `effective_quality` com
//!   `count_full_viewers` pelo MODO (o `Full` fraco conta no teto).
//! - `reduced_quality(ReducedTuning, &Quality /* a cheia */) -> Quality`:
//!   `Thumb` = `REDUCED_THUMB` (360p, 15, 500 kbps); `WeakLink` = `REDUCED_WEAK` (720p, 30,
//!   1,5 Mbps), salvo quando a cheia já é 720p30 ou menor: aí o tamanho e os quadros por segundo
//!   são os da cheia e a taxa é o menor entre 1,5 Mbps e metade da taxa da cheia.
//!
//! O publicador chama `active_tiers`, `full_quality`, `reduced_tuning` e `reduced_quality` a cada
//! mudança de espectador para configurar os (no máximo dois) codificadores.
//!
//! ## 2. Ações por canal (T005)
//!
//! - `Action<Id>` (`Id: Copy + Eq` identifica a ligação). Variantes, e só estas:
//!   `ActivateTier(Tier)`, `StopTier(Tier)`, `SetVideoActive { viewer, active }`,
//!   `PointTo { viewer, tier }`, `RequestKeyframe(Tier)`, `SetSound { viewer, on }`.
//!   **Não existe variante que tire a faixa de um canal** (`set_track(None)` derrubou a sonda
//!   do G1). Parar de enviar é `SetVideoActive { active: false }`; a faixa fica no canal.
//! - `plan_actions(before: &[(Id, Viewer)], after: &[(Id, Viewer)]) -> Vec<Action<Id>>`.
//!   Chamado pelo publicador a cada `ViewMessage`, a cada resultado de `BandwidthRule::tick` e
//!   quando alguém entra ou sai. Só emite o que mudou.
//!   - Ordem da lista: primeiro os `ActivateTier`, depois as ações por espectador (na ordem de
//!     `after`), depois os `RequestKeyframe` (no máximo um por qualidade) e por fim os `StopTier`.
//!   - Espectador novo (está em `after`, não em `before`): parte de "nada, canal desligado, som
//!     ainda não decidido"; recebe sempre um `SetSound` com `wants_sound`.
//!   - Espectador que saiu (não está em `after`): nenhuma ação por canal (a ligação acabou);
//!     só conta para parar a qualidade.
//!   - Passou a receber uma qualidade `t` (de nada ou da outra): `PointTo { tier: t }` e
//!     `RequestKeyframe(t)`; se vinha de nada, também `SetVideoActive { active: true }`. O
//!     `PointTo` sai também na volta de `Hidden` (a função não sabe para onde o canal apontava
//!     antes); o publicador ignora se o canal já aponta para lá.
//!   - Passou a nada: `SetVideoActive { active: false }`, sem `PointTo`.
//!   - Som: `SetSound` quando `wants_sound(mode)` mudou.
//!   - `ActivateTier(t)` quando `t` não estava ativa em `before` e está em `after`; `StopTier(t)`
//!     no inverso.
//!
//! ## 3. Quadro inteiro a pedido (T006)
//!
//! - `KeyframeGate`: uma conta por qualidade. `KeyframeGate::new()` já nasce com o primeiro quadro
//!   devendo sair inteiro. `next_frame(&mut self, requested: bool, now_ms: u64) -> bool`: chamado
//!   pelo publicador a cada quadro da qualidade, com o resultado de `take_keyframe_request()`.
//!   Devolve se ESTE quadro sai inteiro. Pedido dentro de `KEYFRAME_MIN_INTERVAL_MS` (300) do
//!   último quadro inteiro fica pendente e sai no primeiro quadro em que a janela já passou;
//!   nunca é descartado. Sem quadro inteiro periódico.
//! - `KeyframeGates { full, reduced }` com `gate(&mut self, Tier) -> &mut KeyframeGate` e
//!   `activate(&mut self, Tier)` (qualidade que acabou de ficar ativa: conta nova, primeiro
//!   quadro inteiro).
//!
//! ## 4. Internet de cada espectador (T007)
//!
//! - `LinkSample<Id> { id, mode: Option<ViewMode>, available_bps: u32 }`: a leitura de um segundo.
//! - `BandwidthRule<Id>`: guarda, por espectador em `Full`, os contadores (segundos seguidos
//!   abaixo de 60%, abaixo de 90%, em 90% ou mais, e segundos na reduzida).
//!   `tick(&mut self, samples: &[LinkSample<Id>], nominal_bps: u32) -> BandOutcome<Id>`, uma vez
//!   por segundo, com `nominal_bps` = `max_bitrate` da cheia.
//! - `BandOutcome<Id> { weak: Vec<Id>, full_bitrate: u32 }`: `weak` vira `Viewer.weak`;
//!   `full_bitrate` é o alvo do codificador da cheia.
//! - Passo de um segundo, nesta ordem:
//!   1. Quem não está em `Full` (ou sumiu das leituras) perde o estado: voltar a `Full` recomeça
//!      do zero, com banda boa. `Thumb` não entra em nada.
//!   2. Quem está na reduzida por banda: soma um segundo na reduzida; banda em 90% ou mais soma
//!      nos "segundos bons", senão zera. Com `RETURN_SECS` (15) bons seguidos, ou com
//!      `RETRY_SECS` (30) na reduzida (tentativa), volta à cheia com os contadores zerados; a
//!      partir do segundo seguinte vale de novo a regra de descer.
//!   3. Quem já recebia a cheia no começo do passo: atualiza "abaixo de 60%" e "abaixo de 90%"
//!      (zera o que não vale). `todos_abaixo` = há alguém e todos estão abaixo de 90%.
//!   4. Se `todos_abaixo`: ninguém desce (um sozinho nunca desce) e
//!      `full_bitrate = min(nominal, 90% da menor banda entre eles)`. Senão: desce quem tem
//!      `DROP_FAST_SECS` (5) seguidos abaixo de `DROP_FAST_PCT` (60) ou `DROP_SLOW_SECS` (15)
//!      seguidos abaixo de `DROP_SLOW_PCT` (90), e `full_bitrate = nominal`.
//!   A base das porcentagens é sempre a nominal. Contas em inteiro (`u64`), sem ponto flutuante.
//!
//! ## 5. Caminho de compressão (T008)
//!
//! - `EncodePath { Choosing, Gpu, Cpu }` e `PathEvent { Started, NotFound, StartFailed, Error,
//!   FrameOut, SecondWithoutFrame, NoActiveTier }` (os fatos que `espia-media` informa).
//! - `PathMachine::new()` (em `Choosing`), `state()`, `on(&mut self, PathEvent) -> EncodePath`:
//!   a máquina de `contracts/caminho.md`. `MAX_ERRORS_IN_A_ROW` = 3, `NO_FRAME_SECS` = 2. `Cpu` é
//!   final nesta transmissão. Evento sem linha na tabela não muda nada.
//! - `ReopenCounter::reopened(&mut self, now_s: u64) -> bool`: o codificador reabriu por recusar
//!   a taxa ao vivo; devolve `true` quando é a segunda (ou mais) dentro de `REOPEN_WINDOW_SECS`
//!   (60), e aí o publicador manda um `PathEvent::Error`.
//! - `path_quality(EncodePath, &GoLive, &[Viewer], cpu_steps) -> Quality`: nos dois caminhos é
//!   `full_quality` (o processador não tem teto próprio; quem baixa é o `CpuGovernor`).
//! - `encoder_limited(late_frames: u32) -> bool`: o segundo conta como "limitado" para
//!   `CpuGovernor::tick` quando algum quadro não saiu a tempo.

use crate::device::GoLive;
use crate::quality::{Quality, ViewMode, count_full_viewers, effective_quality, wants_sound};

/// Miniatura (os números de `send_params` para `Thumb`): 360p, 15 quadros por segundo, 500 kbps.
pub const REDUCED_THUMB: Quality = Quality { resolution: 360, fps: 15, max_bitrate: 500_000 };
/// Reduzida para quem está com internet fraca: 720p, 30 quadros por segundo, 1,5 Mbps.
pub const REDUCED_WEAK: Quality = Quality { resolution: 720, fps: 30, max_bitrate: 1_500_000 };

/// Uma qualidade comprimida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Full,
    Reduced,
}

/// As duas qualidades, na ordem em que as ações saem.
const TIERS: [Tier; 2] = [Tier::Full, Tier::Reduced];

/// Um espectador como as regras o veem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewer {
    /// Sem modo = não clicou em Assistir.
    pub mode: Option<ViewMode>,
    /// Vem de `BandwidthRule::tick`.
    pub weak: bool,
}

/// O que o espectador recebe; `None` é nada.
pub fn receives(v: Viewer) -> Option<Tier> {
    match v.mode {
        Some(ViewMode::Full) if v.weak => Some(Tier::Reduced),
        Some(ViewMode::Full) => Some(Tier::Full),
        // A banda não muda nada para miniatura.
        Some(ViewMode::Thumb) => Some(Tier::Reduced),
        Some(ViewMode::Hidden | ViewMode::Off) | None => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveTiers {
    pub full: bool,
    pub reduced: bool,
}

impl ActiveTiers {
    fn has(self, tier: Tier) -> bool {
        match tier {
            Tier::Full => self.full,
            Tier::Reduced => self.reduced,
        }
    }
}

/// Uma qualidade fica ativa se pelo menos um espectador a recebe.
pub fn active_tiers(viewers: &[Viewer]) -> ActiveTiers {
    let mut on = ActiveTiers { full: false, reduced: false };
    for tier in viewers.iter().filter_map(|v| receives(*v)) {
        match tier {
            Tier::Full => on.full = true,
            Tier::Reduced => on.reduced = true,
        }
    }
    on
}

/// Só existe uma reduzida: este é o ajuste que todos os que a recebem compartilham.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReducedTuning {
    Thumb,
    WeakLink,
}

/// `WeakLink` se algum `Full` está fraco; `Thumb` se só há miniaturas; `None` se ninguém a recebe.
pub fn reduced_tuning(viewers: &[Viewer]) -> Option<ReducedTuning> {
    if viewers.iter().any(|v| v.mode == Some(ViewMode::Full) && v.weak) {
        Some(ReducedTuning::WeakLink)
    } else if viewers.iter().any(|v| v.mode == Some(ViewMode::Thumb)) {
        Some(ReducedTuning::Thumb)
    } else {
        None
    }
}

/// A cheia é sempre a efetiva. O `Full` fraco ainda conta no teto: a contagem é pelo modo.
pub fn full_quality(chosen: &GoLive, viewers: &[Viewer], cpu_steps: u32) -> Quality {
    let full_viewers = count_full_viewers(viewers.iter().filter_map(|v| v.mode));
    effective_quality(chosen, full_viewers, cpu_steps)
}

/// `full` é a qualidade cheia em vigor. Com a cheia já em 720p30 ou menos, a reduzida de internet
/// fraca fica do mesmo tamanho e com os mesmos quadros por segundo, e a taxa é a menor entre 1,5 Mbps
/// e metade da cheia (R18): subir a resolução sobre a cheia não faria sentido.
pub fn reduced_quality(tuning: ReducedTuning, full: &Quality) -> Quality {
    match tuning {
        ReducedTuning::Thumb => REDUCED_THUMB,
        ReducedTuning::WeakLink if full.resolution <= 720 && full.fps <= 30 => Quality {
            max_bitrate: REDUCED_WEAK.max_bitrate.min(full.max_bitrate / 2),
            ..*full
        },
        ReducedTuning::WeakLink => REDUCED_WEAK,
    }
}


/// O que o publicador faz com os canais. Não há variante que tire a faixa de um canal: isso
/// derrubou a sonda do G1. Parar de enviar é `SetVideoActive { active: false }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<Id> {
    ActivateTier(Tier),
    StopTier(Tier),
    SetVideoActive { viewer: Id, active: bool },
    PointTo { viewer: Id, tier: Tier },
    RequestKeyframe(Tier),
    SetSound { viewer: Id, on: bool },
}

fn tiers_of<Id>(list: &[(Id, Viewer)]) -> ActiveTiers {
    let viewers: Vec<Viewer> = list.iter().map(|(_, v)| *v).collect();
    active_tiers(&viewers)
}

/// As ações que levam o publicador de `before` a `after`; só o que mudou. Ordem: ativar
/// qualidades, ações por espectador (na ordem de `after`), quadros inteiros (um por qualidade) e
/// por fim parar qualidades, para o destino já estar comprimindo quando o canal apontar para ele.
pub fn plan_actions<Id: Copy + Eq>(
    before: &[(Id, Viewer)],
    after: &[(Id, Viewer)],
) -> Vec<Action<Id>> {
    let (was_on, is_on) = (tiers_of(before), tiers_of(after));
    let mut out: Vec<Action<Id>> = TIERS
        .into_iter()
        .filter(|t| !was_on.has(*t) && is_on.has(*t))
        .map(Action::ActivateTier)
        .collect();

    let mut keyframe = [false; 2];
    for (id, now) in after {
        let old = before.iter().find(|(b, _)| b == id).map(|(_, v)| *v);
        let (was, is) = (old.and_then(receives), receives(*now));
        if was != is {
            match is {
                Some(tier) => {
                    // Aponta também na volta de `Hidden`: aqui não se sabe para onde apontava.
                    out.push(Action::PointTo { viewer: *id, tier });
                    if was.is_none() {
                        out.push(Action::SetVideoActive { viewer: *id, active: true });
                    }
                    keyframe[tier as usize] = true;
                }
                None => out.push(Action::SetVideoActive { viewer: *id, active: false }),
            }
        }
        // Espectador novo: o som ainda não foi decidido.
        let sound_changed = match old {
            Some(o) => wants_sound(o.mode) != wants_sound(now.mode),
            None => true,
        };
        if sound_changed {
            out.push(Action::SetSound { viewer: *id, on: wants_sound(now.mode) });
        }
    }

    out.extend(TIERS.into_iter().filter(|t| keyframe[*t as usize]).map(Action::RequestKeyframe));
    out.extend(
        TIERS.into_iter().filter(|t| was_on.has(*t) && !is_on.has(*t)).map(Action::StopTier),
    );
    out
}


/// No máximo um quadro inteiro a cada tanto por qualidade (juntam-se pedidos próximos).
pub const KEYFRAME_MIN_INTERVAL_MS: u64 = 300;

/// A conta de quadro inteiro de uma qualidade. Sem quadro inteiro periódico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyframeGate {
    pending: bool,
    last_ms: Option<u64>,
}

impl KeyframeGate {
    /// O primeiro quadro de uma qualidade nova sai inteiro.
    pub fn new() -> Self {
        Self { pending: true, last_ms: None }
    }

    /// Chamado a cada quadro da qualidade. Pedido dentro da janela fica pendente, nunca é
    /// descartado: descartar deixaria sem imagem quem pediu.
    pub fn next_frame(&mut self, requested: bool, now_ms: u64) -> bool {
        self.pending |= requested;
        let in_window =
            self.last_ms.is_some_and(|l| now_ms.saturating_sub(l) < KEYFRAME_MIN_INTERVAL_MS);
        if !self.pending || in_window {
            return false;
        }
        self.pending = false;
        self.last_ms = Some(now_ms);
        true
    }
}

impl Default for KeyframeGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Uma conta por qualidade: um pedido da cheia não faz a reduzida emitir.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyframeGates {
    pub full: KeyframeGate,
    pub reduced: KeyframeGate,
}

impl KeyframeGates {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gate(&mut self, tier: Tier) -> &mut KeyframeGate {
        match tier {
            Tier::Full => &mut self.full,
            Tier::Reduced => &mut self.reduced,
        }
    }

    /// A qualidade acabou de ficar ativa: conta nova, primeiro quadro inteiro.
    pub fn activate(&mut self, tier: Tier) {
        *self.gate(tier) = KeyframeGate::new();
    }
}


/// Desce com banda abaixo de `DROP_FAST_PCT`% da taxa nominal por `DROP_FAST_SECS` segundos seguidos...
pub const DROP_FAST_PCT: u64 = 60;
pub const DROP_FAST_SECS: u32 = 5;
/// ...ou abaixo de `DROP_SLOW_PCT`% por `DROP_SLOW_SECS` segundos seguidos.
pub const DROP_SLOW_PCT: u64 = 90;
pub const DROP_SLOW_SECS: u32 = 15;
/// Volta à cheia com banda em `DROP_SLOW_PCT`% ou mais por tanto tempo seguido.
pub const RETURN_SECS: u32 = 15;
/// Tentativa de volta: depois de tanto tempo na reduzida, volta à cheia para testar.
pub const RETRY_SECS: u32 = 30;
/// A taxa da cheia, quando todos estão abaixo, fica nesta porcentagem da menor banda.
const ALL_BELOW_RATE_PCT: u64 = 90;

/// A leitura de um segundo da banda de saída de um espectador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkSample<Id> {
    pub id: Id,
    pub mode: Option<ViewMode>,
    pub available_bps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandOutcome<Id> {
    /// Espectadores `Full` que recebem a reduzida por banda (viram `Viewer.weak`).
    pub weak: Vec<Id>,
    /// Alvo do codificador da cheia.
    pub full_bitrate: u32,
}

/// Contadores de um espectador em `Full`.
#[derive(Debug, Clone, Default)]
struct LinkState {
    below_fast: u32,
    below_slow: u32,
    good_secs: u32,
    on_reduced: bool,
    reduced_secs: u32,
}

/// A regra de internet de cada espectador (R9). Uma chamada por segundo.
#[derive(Debug, Clone)]
pub struct BandwidthRule<Id> {
    states: Vec<(Id, LinkState)>,
}

impl<Id: Copy + Eq> BandwidthRule<Id> {
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    /// `nominal_bps` é o `max_bitrate` da cheia: a base de todas as porcentagens, mesmo com a
    /// cheia já baixada. Contas em inteiro.
    pub fn tick(&mut self, samples: &[LinkSample<Id>], nominal_bps: u32) -> BandOutcome<Id> {
        let nominal = u64::from(nominal_bps);
        let full: Vec<&LinkSample<Id>> =
            samples.iter().filter(|s| s.mode == Some(ViewMode::Full)).collect();

        // Quem saiu de `Full` (ou sumiu) perde o estado; quem chega começa do zero, com banda boa.
        self.states.retain(|(id, _)| full.iter().any(|s| s.id == *id));
        for s in &full {
            if !self.states.iter().any(|(id, _)| *id == s.id) {
                self.states.push((s.id, LinkState::default()));
            }
        }

        // Quem recebia a cheia no começo do passo; quem volta agora só conta a partir do próximo.
        let mut on_full: Vec<(Id, u64)> = Vec::new();
        for s in &full {
            let Some((_, st)) = self.states.iter_mut().find(|(id, _)| *id == s.id) else { continue };
            let bps = u64::from(s.available_bps);
            if st.on_reduced {
                st.reduced_secs += 1;
                st.good_secs = if bps * 100 >= nominal * DROP_SLOW_PCT { st.good_secs + 1 } else { 0 };
                if st.good_secs >= RETURN_SECS || st.reduced_secs >= RETRY_SECS {
                    *st = LinkState::default();
                }
            } else {
                st.below_fast = if bps * 100 < nominal * DROP_FAST_PCT { st.below_fast + 1 } else { 0 };
                st.below_slow = if bps * 100 < nominal * DROP_SLOW_PCT { st.below_slow + 1 } else { 0 };
                on_full.push((s.id, bps));
            }
        }

        // Todos abaixo: ninguém desce (sozinho, nunca) e a taxa da cheia acompanha a menor banda.
        let all_below = !on_full.is_empty()
            && on_full.iter().all(|(id, _)| self.state(*id).is_some_and(|st| st.below_slow > 0));
        let full_bitrate = if all_below {
            let lowest = on_full.iter().map(|(_, bps)| *bps).min().unwrap_or(nominal);
            let rate = (lowest * ALL_BELOW_RATE_PCT / 100).min(nominal);
            u32::try_from(rate).unwrap_or(nominal_bps)
        } else {
            for (id, _) in &on_full {
                if let Some((_, st)) = self.states.iter_mut().find(|(i, _)| i == id) {
                    if st.below_fast >= DROP_FAST_SECS || st.below_slow >= DROP_SLOW_SECS {
                        *st = LinkState { on_reduced: true, ..LinkState::default() };
                    }
                }
            }
            nominal_bps
        };

        let weak = full
            .iter()
            .filter(|s| self.state(s.id).is_some_and(|st| st.on_reduced))
            .map(|s| s.id)
            .collect();
        BandOutcome { weak, full_bitrate }
    }

    fn state(&self, id: Id) -> Option<&LinkState> {
        self.states.iter().find(|(i, _)| *i == id).map(|(_, st)| st)
    }
}

impl<Id: Copy + Eq> Default for BandwidthRule<Id> {
    fn default() -> Self {
        Self::new()
    }
}


/// Erros seguidos da placa que levam ao processador.
pub const MAX_ERRORS_IN_A_ROW: u32 = 3;
/// Segundos sem quadro comprimido (com alguma qualidade ativa) que levam ao processador.
pub const NO_FRAME_SECS: u32 = 2;
/// Janela da segunda reabertura do codificador por recusar a taxa ao vivo.
pub const REOPEN_WINDOW_SECS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodePath {
    Choosing,
    Gpu,
    Cpu,
}

/// Os fatos que `espia-media` informa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathEvent {
    Started,
    NotFound,
    StartFailed,
    Error,
    FrameOut,
    SecondWithoutFrame,
    NoActiveTier,
}

/// A máquina de `contracts/caminho.md`. Só vai da placa para o processador, nunca volta.
#[derive(Debug, Clone)]
pub struct PathMachine {
    state: EncodePath,
    errors_in_a_row: u32,
    secs_without_frame: u32,
}

impl PathMachine {
    pub fn new() -> Self {
        Self { state: EncodePath::Choosing, errors_in_a_row: 0, secs_without_frame: 0 }
    }

    pub fn state(&self) -> EncodePath {
        self.state
    }

    /// Evento sem linha na tabela não muda nada.
    pub fn on(&mut self, event: PathEvent) -> EncodePath {
        match (self.state, event) {
            (EncodePath::Choosing, PathEvent::Started) => self.state = EncodePath::Gpu,
            (EncodePath::Choosing, PathEvent::NotFound | PathEvent::StartFailed) => {
                self.state = EncodePath::Cpu;
            }
            (EncodePath::Gpu, PathEvent::Error) => {
                self.errors_in_a_row += 1;
                if self.errors_in_a_row >= MAX_ERRORS_IN_A_ROW {
                    self.state = EncodePath::Cpu;
                }
            }
            (EncodePath::Gpu, PathEvent::FrameOut) => {
                self.errors_in_a_row = 0;
                self.secs_without_frame = 0;
            }
            (EncodePath::Gpu, PathEvent::SecondWithoutFrame) => {
                self.secs_without_frame += 1;
                if self.secs_without_frame >= NO_FRAME_SECS {
                    self.state = EncodePath::Cpu;
                }
            }
            // Sem qualidade ativa nada é comprimido: não há o que esperar.
            (EncodePath::Gpu, PathEvent::NoActiveTier) => self.secs_without_frame = 0,
            _ => {}
        }
        self.state
    }
}

impl Default for PathMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Conta as reaberturas do codificador por recusar a taxa ao vivo.
#[derive(Debug, Clone, Default)]
pub struct ReopenCounter {
    last_s: Option<u64>,
}

impl ReopenCounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` na segunda (ou mais) reabertura dentro de `REOPEN_WINDOW_SECS`: o publicador
    /// manda então um `PathEvent::Error`.
    pub fn reopened(&mut self, now_s: u64) -> bool {
        let again = self.last_s.is_some_and(|l| now_s.saturating_sub(l) < REOPEN_WINDOW_SECS);
        self.last_s = Some(now_s);
        again
    }
}

/// Nos dois caminhos vale `full_quality`: o processador não tem teto próprio; quem baixa a
/// qualidade quando o computador não dá conta é o `CpuGovernor`.
pub fn path_quality(
    _path: EncodePath,
    chosen: &GoLive,
    viewers: &[Viewer],
    cpu_steps: u32,
) -> Quality {
    full_quality(chosen, viewers, cpu_steps)
}

/// O segundo conta como "limitado" para `CpuGovernor::tick` se algum quadro não saiu a tempo.
pub fn encoder_limited(late_frames: u32) -> bool {
    late_frames > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: Viewer = Viewer { mode: Some(ViewMode::Full), weak: false };
    const WEAK: Viewer = Viewer { mode: Some(ViewMode::Full), weak: true };
    const THUMB: Viewer = Viewer { mode: Some(ViewMode::Thumb), weak: false };
    const HIDDEN: Viewer = Viewer { mode: Some(ViewMode::Hidden), weak: false };
    const OFF: Viewer = Viewer { mode: Some(ViewMode::Off), weak: false };
    const NO_MODE: Viewer = Viewer { mode: None, weak: false };

    fn chosen(resolution: u16, fps: u16) -> GoLive {
        GoLive { resolution, fps, ..GoLive::default() }
    }
    const fn q(resolution: u16, fps: u16, max_bitrate: u32) -> Quality {
        Quality { resolution, fps, max_bitrate }
    }

    /// Seção 1 do contrato: quem recebe o quê (7 casos).
    mod secao1_quem_recebe {
        use super::*;

        #[test]
        fn tabela() {
            // caso 1: Full, banda boa
            assert_eq!(receives(GOOD), Some(Tier::Full));
            // caso 2: Full, banda fraca
            assert_eq!(receives(WEAK), Some(Tier::Reduced));
            // caso 3: Thumb, banda boa
            assert_eq!(receives(THUMB), Some(Tier::Reduced));
            // caso 4: Thumb, banda fraca (a banda não muda nada)
            assert_eq!(receives(Viewer { weak: true, ..THUMB }), Some(Tier::Reduced));
            // caso 5: Hidden
            assert_eq!(receives(HIDDEN), None);
            // caso 6: Off
            assert_eq!(receives(OFF), None);
            // caso 7: sem modo
            assert_eq!(receives(NO_MODE), None);
        }
    }

    /// Seção 2: qualidades ativas (6 casos, mais uma linha extra da tabela).
    mod secao2_ativas {
        use super::*;

        fn on(full: bool, reduced: bool) -> ActiveTiers {
            ActiveTiers { full, reduced }
        }

        #[test]
        fn tabela() {
            // caso 1: 0 espectadores
            assert_eq!(active_tiers(&[]), on(false, false));
            // caso 2: 2 Thumb
            assert_eq!(active_tiers(&[THUMB, THUMB]), on(false, true));
            // caso 3: 3 Full com banda boa
            assert_eq!(active_tiers(&[GOOD, GOOD, GOOD]), on(true, false));
            // caso 4: 1 Full e 1 Thumb
            assert_eq!(active_tiers(&[GOOD, THUMB]), on(true, true));
            // caso 5: o único Full passa a Hidden
            assert_eq!(active_tiers(&[HIDDEN]), on(false, false));
            // caso 6: 1 Thumb e 1 Hidden
            assert_eq!(active_tiers(&[THUMB, HIDDEN]), on(false, true));
        }

        #[test]
        fn linhas_da_tabela_de_regras_fora_dos_casos() {
            assert_eq!(active_tiers(&[HIDDEN, OFF, NO_MODE]), on(false, false));
            assert_eq!(active_tiers(&[WEAK, GOOD]), on(true, true));
        }
    }

    /// Seção 3: ajuste da reduzida e cheia efetiva (8 casos).
    mod secao3_ajuste {
        use super::*;

        const THUMB_Q: Quality = q(360, 15, 500_000);
        const WEAK_Q: Quality = q(720, 30, 1_500_000);

        fn reduced_for(c: &GoLive, viewers: &[Viewer]) -> Option<Quality> {
            let full = full_quality(c, viewers, 0);
            reduced_tuning(viewers).map(|t| reduced_quality(t, &full))
        }

        #[test]
        fn reduzida() {
            let c = chosen(1080, 60);
            // caso 1: 2 Thumb
            assert_eq!(reduced_for(&c, &[THUMB, THUMB]), Some(THUMB_Q));
            // caso 2: 1 Full fraco
            assert_eq!(reduced_for(&c, &[WEAK]), Some(WEAK_Q));
            // caso 3: 1 Full fraco e 1 Thumb (a miniatura recebe os 720p30)
            assert_eq!(reduced_for(&c, &[WEAK, THUMB]), Some(WEAK_Q));
            // caso 4: o Full fraco volta à cheia, 2 Thumb ficam
            assert_eq!(reduced_for(&c, &[GOOD, THUMB, THUMB]), Some(THUMB_Q));
        }

        #[test]
        fn cheia_efetiva() {
            let c = chosen(1080, 60);
            // caso 5: 4 Full
            assert_eq!(full_quality(&c, &[GOOD; 4], 0), q(1080, 60, 6_000_000));
            // caso 6: 5 Full
            assert_eq!(full_quality(&c, &[GOOD; 5], 0), q(720, 60, 4_000_000));
            // caso 7: 7 Full
            assert_eq!(full_quality(&c, &[GOOD; 7], 0), q(720, 30, 2_500_000));
        }

        #[test]
        fn caso8_escolhida_720p30_com_um_full_fraco() {
            let c = chosen(720, 30);
            let full = full_quality(&c, &[WEAK], 0);
            assert_eq!(full, q(720, 30, 2_500_000));
            // mesmo tamanho; metade da taxa da cheia (R18)
            assert_eq!(reduced_quality(ReducedTuning::WeakLink, &full), q(720, 30, 1_250_000));
        }
    }

    /// Seções 4 e 5: ações por canal (5 + 4 casos, mais a varredura das transições).
    mod secoes4_e_5_acoes {
        use super::*;
        use Action::*;

        type A = Action<u8>;

        const ESTADOS: [Viewer; 6] = [GOOD, WEAK, THUMB, HIDDEN, OFF, NO_MODE];

        fn um(antes: Viewer, depois: Viewer) -> Vec<A> {
            plan_actions(&[(1, antes)], &[(1, depois)])
        }

        /// `match` exaustivo e sem `_`: quebra se alguém acrescentar uma variante (por exemplo,
        /// uma que tire a faixa do canal).
        fn nome(a: &A) -> &'static str {
            match a {
                ActivateTier(_) => "ativar",
                StopTier(_) => "parar",
                SetVideoActive { .. } => "ativo",
                PointTo { .. } => "apontar",
                RequestKeyframe(_) => "quadro inteiro",
                SetSound { .. } => "som",
            }
        }

        /// Parar de enviar nunca vem com `PointTo`; a lista só tem variantes previstas.
        fn confere(acoes: &[A]) {
            let para = acoes.iter().any(|a| matches!(a, SetVideoActive { active: false, .. }));
            let aponta = acoes.iter().any(|a| matches!(a, PointTo { .. }));
            assert!(!(para && aponta), "parou e apontou: {acoes:?}");
            assert!(acoes.iter().all(|a| !nome(a).is_empty()));
        }

        // seção 4

        #[test]
        fn caso1_full_vira_hidden_para_o_video_e_o_som_continua() {
            assert_eq!(
                um(GOOD, HIDDEN),
                vec![SetVideoActive { viewer: 1, active: false }, StopTier(Tier::Full)]
            );
        }

        #[test]
        fn caso2_full_vira_off_para_o_video_e_corta_o_som() {
            assert_eq!(
                um(GOOD, OFF),
                vec![
                    SetVideoActive { viewer: 1, active: false },
                    SetSound { viewer: 1, on: false },
                    StopTier(Tier::Full),
                ]
            );
        }

        #[test]
        fn caso3_hidden_vira_full_liga_e_pede_quadro_inteiro_da_cheia() {
            assert_eq!(
                um(HIDDEN, GOOD),
                vec![
                    ActivateTier(Tier::Full),
                    PointTo { viewer: 1, tier: Tier::Full },
                    SetVideoActive { viewer: 1, active: true },
                    RequestKeyframe(Tier::Full),
                ]
            );
        }

        #[test]
        fn caso4_thumb_vira_off_e_a_reduzida_para_se_era_o_ultimo() {
            assert_eq!(
                um(THUMB, OFF),
                vec![
                    SetVideoActive { viewer: 1, active: false },
                    SetSound { viewer: 1, on: false },
                    StopTier(Tier::Reduced),
                ]
            );
            // Com outro Thumb ainda assistindo, a reduzida continua.
            let acoes = plan_actions(&[(1, THUMB), (2, THUMB)], &[(1, OFF), (2, THUMB)]);
            assert!(!acoes.contains(&StopTier(Tier::Reduced)));
        }

        #[test]
        fn caso5_nenhuma_saida_tira_a_faixa_nem_para_com_apontar() {
            for antes in ESTADOS {
                for depois in ESTADOS {
                    confere(&um(antes, depois));
                }
            }
        }

        // seção 5

        #[test]
        fn caso6_so_o_canal_de_a_muda_de_faixa() {
            // A passa à reduzida; B e C ficam na cheia; a reduzida já estava ativa (D).
            let antes = [(1, GOOD), (2, GOOD), (3, GOOD), (4, THUMB)];
            let depois = [(1, WEAK), (2, GOOD), (3, GOOD), (4, THUMB)];
            assert_eq!(
                plan_actions(&antes, &depois),
                vec![PointTo { viewer: 1, tier: Tier::Reduced }, RequestKeyframe(Tier::Reduced)]
            );
        }

        #[test]
        fn caso7_a_reduzida_inativa_passa_a_ativa() {
            let antes = [(1, GOOD), (2, GOOD)];
            let depois = [(1, WEAK), (2, GOOD)];
            assert_eq!(
                plan_actions(&antes, &depois),
                vec![
                    ActivateTier(Tier::Reduced),
                    PointTo { viewer: 1, tier: Tier::Reduced },
                    RequestKeyframe(Tier::Reduced),
                ]
            );
        }

        #[test]
        fn caso8_o_unico_da_reduzida_volta_e_ela_para() {
            let antes = [(1, WEAK), (2, GOOD)];
            let depois = [(1, GOOD), (2, GOOD)];
            assert_eq!(
                plan_actions(&antes, &depois),
                vec![
                    PointTo { viewer: 1, tier: Tier::Full },
                    RequestKeyframe(Tier::Full),
                    StopTier(Tier::Reduced),
                ]
            );
        }

        #[test]
        fn caso9_o_unico_da_cheia_desce_e_a_cheia_para() {
            assert_eq!(
                um(GOOD, WEAK),
                vec![
                    ActivateTier(Tier::Reduced),
                    PointTo { viewer: 1, tier: Tier::Reduced },
                    RequestKeyframe(Tier::Reduced),
                    StopTier(Tier::Full),
                ]
            );
        }

        // varredura

        #[test]
        fn todas_as_transicoes_entre_os_seis_estados_entrando_e_saindo() {
            for antes in ESTADOS {
                for depois in ESTADOS {
                    let acoes = um(antes, depois);
                    confere(&acoes);
                    let (a, d) = (receives(antes), receives(depois));
                    // Quem passa a nada para o canal, sem apontar.
                    if a.is_some() && d.is_none() {
                        assert!(acoes.contains(&SetVideoActive { viewer: 1, active: false }));
                    }
                    // Quem passa a receber é apontado e pede quadro inteiro.
                    if let (true, Some(t)) = (a != d, d) {
                        assert!(acoes.contains(&PointTo { viewer: 1, tier: t }));
                        assert!(acoes.contains(&RequestKeyframe(t)));
                    }
                    // Só `wants_sound` mexe no som.
                    let som = acoes.iter().any(|x| matches!(x, SetSound { .. }));
                    assert_eq!(som, wants_sound(antes.mode) != wants_sound(depois.mode));
                    // Sem mudança, nada.
                    if antes == depois {
                        assert!(acoes.is_empty());
                    }
                }
            }
            for estado in ESTADOS {
                // Entrar: sempre `SetSound` com `wants_sound`.
                let entra = plan_actions(&[], &[(1, estado)]);
                confere(&entra);
                assert!(entra.contains(&SetSound { viewer: 1, on: wants_sound(estado.mode) }));
                // Sair: nenhuma ação por canal (a ligação acabou).
                let sai = plan_actions(&[(1, estado)], &[]);
                confere(&sai);
                assert!(sai.iter().all(|a| matches!(a, StopTier(_))));
            }
        }
    }

    /// Seção 6: quadro inteiro a pedido (7 casos).
    mod secao6_quadro_inteiro {
        use super::*;

        /// Um quadro a cada 100 ms; `pedidos` são os instantes com pedido. Devolve os instantes
        /// dos quadros inteiros.
        fn rodar(gate: &mut KeyframeGate, pedidos: &[u64], ate_ms: u64) -> Vec<u64> {
            (0..=ate_ms / 100)
                .map(|i| i * 100)
                .filter(|t| gate.next_frame(pedidos.contains(t), *t))
                .collect()
        }

        #[test]
        fn caso1_pedido_em_0_sai_no_proximo_quadro() {
            let mut g = KeyframeGate::new();
            assert!(g.next_frame(true, 0));
        }

        #[test]
        fn caso2_pedidos_em_0_100_e_200_saem_em_0_e_300() {
            let mut g = KeyframeGate::new();
            assert_eq!(rodar(&mut g, &[0, 100, 200], 600), vec![0, 300]);
        }

        #[test]
        fn caso3_pedidos_em_0_e_300_saem_em_0_e_300() {
            let mut g = KeyframeGate::new();
            assert_eq!(rodar(&mut g, &[0, 300], 600), vec![0, 300]);
        }

        #[test]
        fn caso4_pedido_em_299_sai_em_300_nao_em_299() {
            let mut g = KeyframeGate::new();
            assert!(g.next_frame(true, 0));
            assert!(!g.next_frame(true, 299));
            assert!(g.next_frame(false, 300));
        }

        #[test]
        fn caso5_60_s_sem_pedido_nao_dao_nenhum_alem_do_primeiro() {
            let mut g = KeyframeGate::new();
            assert_eq!(rodar(&mut g, &[], 60_000), vec![0]);
        }

        #[test]
        fn caso6_pedido_na_cheia_nao_faz_a_reduzida_emitir() {
            let mut gates = KeyframeGates::new();
            assert!(gates.gate(Tier::Full).next_frame(false, 0));
            assert!(gates.gate(Tier::Reduced).next_frame(false, 0));
            assert!(gates.gate(Tier::Full).next_frame(true, 500));
            assert!(!gates.gate(Tier::Reduced).next_frame(false, 500));
        }

        #[test]
        fn caso7_reduzida_que_acaba_de_ficar_ativa_sai_inteira() {
            let mut gates = KeyframeGates::new();
            assert!(gates.gate(Tier::Reduced).next_frame(false, 0));
            assert!(!gates.gate(Tier::Reduced).next_frame(false, 100));
            gates.activate(Tier::Reduced);
            assert!(gates.gate(Tier::Reduced).next_frame(false, 200));
        }
    }

    /// Seção 7: internet de cada espectador (11 casos + 4 "a mais").
    mod secao7_internet {
        use super::*;

        const NOMINAL: u32 = 6_000_000;
        const FULL_MODE: Option<ViewMode> = Some(ViewMode::Full);

        /// Décimos de Mbps em bps (35 = 3,5 Mbps).
        const fn mbps(tenths: u32) -> u32 {
            tenths * 100_000
        }

        fn s(id: u8, mode: Option<ViewMode>, bps: u32) -> LinkSample<u8> {
            LinkSample { id, mode, available_bps: bps }
        }

        /// Um passo por segundo, `secs` vezes, com as mesmas leituras (id, bps), todos `Full`.
        fn passos(rule: &mut BandwidthRule<u8>, secs: u32, bandas: &[(u8, u32)]) -> Vec<BandOutcome<u8>> {
            let amostras: Vec<_> = bandas.iter().map(|(id, bps)| s(*id, FULL_MODE, *bps)).collect();
            (0..secs).map(|_| rule.tick(&amostras, NOMINAL)).collect()
        }

        /// Leva A (id 1) à reduzida com banda ruim e B (id 2) com banda boa.
        fn a_na_reduzida() -> BandwidthRule<u8> {
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 15, &[(1, mbps(40)), (2, mbps(60))]);
            assert_eq!(saidas[14].weak, vec![1]);
            rule
        }

        #[test]
        fn caso1_4_mbps_por_60_s_desce_aos_15_s() {
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 15, &[(1, mbps(40)), (2, mbps(60))]);
            assert!(saidas[..14].iter().all(|o| o.weak.is_empty()));
            assert_eq!(saidas[14].weak, vec![1]);
        }

        #[test]
        fn caso2_3_5_mbps_por_4_s_e_depois_6_continua_na_cheia() {
            let mut rule = BandwidthRule::new();
            let mut saidas = passos(&mut rule, 4, &[(1, mbps(35)), (2, mbps(60))]);
            saidas.extend(passos(&mut rule, 30, &[(1, mbps(60)), (2, mbps(60))]));
            assert!(saidas.iter().all(|o| o.weak.is_empty()));
        }

        #[test]
        fn caso3_3_5_mbps_por_5_s_desce() {
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 5, &[(1, mbps(35)), (2, mbps(60))]);
            assert!(saidas[..4].iter().all(|o| o.weak.is_empty()));
            assert_eq!(saidas[4].weak, vec![1]);
        }

        #[test]
        fn caso4_na_reduzida_com_5_4_mbps_por_14_s_continua() {
            let mut rule = a_na_reduzida();
            let saidas = passos(&mut rule, 14, &[(1, mbps(54)), (2, mbps(60))]);
            assert!(saidas.iter().all(|o| o.weak == vec![1]));
        }

        #[test]
        fn caso5_na_reduzida_com_5_4_mbps_por_15_s_volta() {
            let mut rule = a_na_reduzida();
            let saidas = passos(&mut rule, 15, &[(1, mbps(54)), (2, mbps(60))]);
            assert!(saidas[..14].iter().all(|o| o.weak == vec![1]));
            assert!(saidas[14].weak.is_empty());
        }

        #[test]
        fn caso6_5_0_mbps_por_60_s_sem_tentativa_antes_dos_30_s() {
            let mut rule = a_na_reduzida();
            let saidas = passos(&mut rule, 60, &[(1, mbps(50)), (2, mbps(60))]);
            // Até o segundo 29 continua na reduzida; no 30 há a tentativa.
            assert!(saidas[..29].iter().all(|o| o.weak == vec![1]));
            assert!(saidas[29].weak.is_empty());
        }

        #[test]
        fn caso7_banda_sem_subir_por_30_s_tenta_a_cheia() {
            let mut rule = a_na_reduzida();
            let saidas = passos(&mut rule, 30, &[(1, mbps(40)), (2, mbps(60))]);
            assert!(saidas[..29].iter().all(|o| o.weak == vec![1]));
            assert!(saidas[29].weak.is_empty());
            // A tentativa que não dá certo: vale de novo a regra de descer (15 s abaixo de 90%).
            let depois = passos(&mut rule, 15, &[(1, mbps(40)), (2, mbps(60))]);
            assert!(depois[..14].iter().all(|o| o.weak.is_empty()));
            assert_eq!(depois[14].weak, vec![1]);
        }

        #[test]
        fn caso8_tres_com_3_mbps_ninguem_desce_e_a_taxa_baixa_para_2_7() {
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 20, &[(1, mbps(30)), (2, mbps(30)), (3, mbps(30))]);
            assert!(saidas.iter().all(|o| o.weak.is_empty()));
            assert_eq!(saidas[4].full_bitrate, 2_700_000);
        }

        #[test]
        fn caso9_sozinho_com_3_mbps_nunca_desce_e_a_taxa_baixa_para_2_7() {
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 20, &[(1, mbps(30))]);
            assert!(saidas.iter().all(|o| o.weak.is_empty()));
            assert_eq!(saidas[4].full_bitrate, 2_700_000);
        }

        #[test]
        fn caso10_so_a_desce_e_os_outros_ficam_na_taxa_cheia() {
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 5, &[(1, mbps(30)), (2, mbps(60)), (3, mbps(60))]);
            assert_eq!(saidas[4].weak, vec![1]);
            assert!(saidas.iter().all(|o| o.full_bitrate == NOMINAL));
        }

        #[test]
        fn caso11_thumb_com_0_2_mbps_fica_fora_da_conta_de_todos_abaixo() {
            let mut rule = BandwidthRule::new();
            let amostras = [s(1, Some(ViewMode::Thumb), mbps(2)), s(2, FULL_MODE, mbps(30))];
            for _ in 0..20 {
                let o = rule.tick(&amostras, NOMINAL);
                // A não entra em nada; só B conta, e sozinho nunca desce.
                assert!(o.weak.is_empty());
                assert_eq!(o.full_bitrate, 2_700_000);
            }
        }

        #[test]
        fn mais1_um_full_com_40_por_cento_por_10_s_nao_desce_e_a_taxa_e_90_da_banda() {
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 10, &[(1, mbps(24))]);
            assert!(saidas.iter().all(|o| o.weak.is_empty()));
            assert_eq!(saidas[9].full_bitrate, 2_160_000);
        }

        #[test]
        fn mais2_bandas_5_4_e_8_a_taxa_e_a_nominal_e_5_e_4_descem_aos_15_s() {
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 15, &[(1, mbps(50)), (2, mbps(40)), (3, mbps(80))]);
            assert!(saidas.iter().all(|o| o.full_bitrate == NOMINAL));
            assert!(saidas[..14].iter().all(|o| o.weak.is_empty()));
            assert_eq!(saidas[14].weak, vec![1, 2]);
        }

        #[test]
        fn mais3_bandas_9_10_e_8_a_taxa_e_a_nominal() {
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 20, &[(1, mbps(90)), (2, mbps(100)), (3, mbps(80))]);
            assert!(saidas.iter().all(|o| o.full_bitrate == NOMINAL && o.weak.is_empty()));
        }

        #[test]
        fn mais4_o_limite_e_sempre_da_taxa_nominal_mesmo_com_a_cheia_ja_baixada() {
            // Cheia já baixada para 3 Mbps, mas `nominal_bps` segue 6 Mbps: 3,2 Mbps é abaixo de 60%.
            let mut rule = BandwidthRule::new();
            let saidas = passos(&mut rule, 5, &[(1, mbps(32)), (2, mbps(60))]);
            assert_eq!(saidas[4].weak, vec![1]);
        }

        #[test]
        fn extra_voltar_a_full_recomeca_do_zero() {
            let mut rule = a_na_reduzida();
            // Sai do modo Full (miniatura) e volta: recomeça na cheia, com banda boa.
            rule.tick(&[s(1, Some(ViewMode::Thumb), mbps(40)), s(2, FULL_MODE, mbps(60))], NOMINAL);
            let saidas = passos(&mut rule, 14, &[(1, mbps(40)), (2, mbps(60))]);
            assert!(saidas.iter().all(|o| o.weak.is_empty()));
        }
    }

    /// Caminho de compressão: máquina de estados (14), qualidade no caminho (4), seção 8 (5).
    mod caminho {
        use super::*;
        use crate::cpu_governor::CpuGovernor;
        use PathEvent::*;

        const TODOS: [PathEvent; 7] =
            [Started, NotFound, StartFailed, Error, FrameOut, SecondWithoutFrame, NoActiveTier];

        fn rodar(m: &mut PathMachine, eventos: &[PathEvent]) -> EncodePath {
            for e in eventos {
                m.on(*e);
            }
            m.state()
        }

        /// Máquina já em `Gpu`.
        fn gpu() -> PathMachine {
            let mut m = PathMachine::new();
            assert_eq!(m.on(Started), EncodePath::Gpu);
            m
        }

        /// Máquina já em `Cpu`.
        fn cpu() -> PathMachine {
            let mut m = PathMachine::new();
            assert_eq!(m.on(NotFound), EncodePath::Cpu);
            m
        }

        // máquina de estados

        #[test]
        fn linha01_escolhendo_iniciou_vai_para_a_placa() {
            assert_eq!(PathMachine::new().state(), EncodePath::Choosing);
            assert_eq!(rodar(&mut PathMachine::new(), &[Started]), EncodePath::Gpu);
        }

        #[test]
        fn linha02_escolhendo_nao_achou_vai_para_o_processador() {
            assert_eq!(rodar(&mut PathMachine::new(), &[NotFound]), EncodePath::Cpu);
        }

        #[test]
        fn linha03_escolhendo_nao_iniciou_vai_para_o_processador() {
            assert_eq!(rodar(&mut PathMachine::new(), &[StartFailed]), EncodePath::Cpu);
        }

        #[test]
        fn linha04_dois_erros_ficam_na_placa() {
            assert_eq!(rodar(&mut gpu(), &[Error, Error]), EncodePath::Gpu);
        }

        #[test]
        fn linha05_tres_erros_seguidos_vao_para_o_processador() {
            assert_eq!(rodar(&mut gpu(), &[Error, Error, Error]), EncodePath::Cpu);
        }

        #[test]
        fn linha06_quadro_que_sai_zera_a_contagem_de_erros() {
            assert_eq!(rodar(&mut gpu(), &[Error, Error, FrameOut, Error, Error]), EncodePath::Gpu);
        }

        #[test]
        fn linha07_erros_alternados_com_quadros_ficam_na_placa() {
            let eventos = [Error, FrameOut, Error, FrameOut, Error];
            assert_eq!(rodar(&mut gpu(), &eventos), EncodePath::Gpu);
        }

        #[test]
        fn linha08_um_segundo_sem_quadro_fica_na_placa() {
            assert_eq!(rodar(&mut gpu(), &[SecondWithoutFrame]), EncodePath::Gpu);
        }

        #[test]
        fn linha09_dois_segundos_sem_quadro_vao_para_o_processador() {
            assert_eq!(rodar(&mut gpu(), &[SecondWithoutFrame, SecondWithoutFrame]), EncodePath::Cpu);
        }

        #[test]
        fn linha10_quadro_que_sai_zera_o_tempo_sem_quadro() {
            let eventos = [SecondWithoutFrame, FrameOut, SecondWithoutFrame];
            assert_eq!(rodar(&mut gpu(), &eventos), EncodePath::Gpu);
        }

        #[test]
        fn linha11_sem_qualidade_ativa_por_10_s_nao_conta() {
            assert_eq!(rodar(&mut gpu(), &[NoActiveTier; 10]), EncodePath::Gpu);
        }

        #[test]
        fn linha12_sem_qualidade_ativa_zera_o_tempo_sem_quadro() {
            let eventos = [SecondWithoutFrame, NoActiveTier, SecondWithoutFrame];
            assert_eq!(rodar(&mut gpu(), &eventos), EncodePath::Gpu);
        }

        #[test]
        fn linha13_no_processador_nao_volta_para_a_placa() {
            let eventos = [Started, FrameOut, Error, Error, Error];
            assert_eq!(rodar(&mut cpu(), &eventos), EncodePath::Cpu);
        }

        #[test]
        fn linha14_no_processador_qualquer_sequencia_fica_no_processador() {
            for e in TODOS {
                assert_eq!(rodar(&mut cpu(), &[e]), EncodePath::Cpu);
            }
            // Todas as combinações de dois eventos.
            for a in TODOS {
                for b in TODOS {
                    assert_eq!(rodar(&mut cpu(), &[a, b, Started, FrameOut]), EncodePath::Cpu);
                }
            }
        }

        #[test]
        fn extra_evento_sem_linha_na_tabela_nao_muda_nada() {
            assert_eq!(rodar(&mut PathMachine::new(), &[Error, FrameOut, SecondWithoutFrame]), EncodePath::Choosing);
            assert_eq!(rodar(&mut gpu(), &[Started, NotFound, StartFailed]), EncodePath::Gpu);
        }

        #[test]
        fn extra_codificador_reaberto_duas_vezes_em_60_s_conta_como_erro() {
            let mut r = ReopenCounter::new();
            assert!(!r.reopened(100));
            assert!(r.reopened(159));
            // A janela anda a partir da última reabertura.
            assert!(!ReopenCounter::new().reopened(0));
            let mut r = ReopenCounter::new();
            assert!(!r.reopened(0));
            assert!(!r.reopened(61));
        }

        #[test]
        fn extra_quadro_atrasado_conta_o_segundo_como_limitado() {
            assert!(!encoder_limited(0));
            assert!(encoder_limited(1));
        }

        // qualidade no caminho pelo processador

        #[test]
        fn qualidade1_processador_sem_teto_proprio_de_1_a_4_full() {
            let c = chosen(1080, 60);
            for n in 1..=4 {
                let viewers = vec![GOOD; n];
                assert_eq!(path_quality(EncodePath::Cpu, &c, &viewers, 0), q(1080, 60, 6_000_000));
            }
        }

        #[test]
        fn qualidade2_processador_com_720p30_escolhido() {
            let c = chosen(720, 30);
            assert_eq!(path_quality(EncodePath::Cpu, &c, &[GOOD], 0), q(720, 30, 2_500_000));
        }

        #[test]
        fn qualidade3_placa_e_a_efetiva() {
            let c = chosen(1080, 60);
            let viewers = [GOOD; 5];
            assert_eq!(
                path_quality(EncodePath::Gpu, &c, &viewers, 0),
                effective_quality(&c, 5, 0)
            );
        }

        #[test]
        fn qualidade4_processador_limitado_por_10_s_perde_um_degrau() {
            let c = chosen(1080, 60);
            let mut governor = CpuGovernor::new();
            for _ in 0..10 {
                governor.tick(encoder_limited(2), true);
            }
            assert_eq!(governor.steps, 1);
            assert_eq!(path_quality(EncodePath::Cpu, &c, &[GOOD], governor.steps), q(1080, 30, 4_500_000));
        }

        // seção 8 de qualidades.md

        #[test]
        fn secao8_caso1_3_full_com_1_degrau() {
            let c = chosen(1080, 60);
            assert_eq!(full_quality(&c, &[GOOD; 3], 1), q(1080, 30, 4_500_000));
        }

        #[test]
        fn secao8_caso2_3_full_com_2_degraus() {
            let c = chosen(1080, 60);
            assert_eq!(full_quality(&c, &[GOOD; 3], 2), q(720, 30, 2_500_000));
        }

        #[test]
        fn secao8_caso3_nunca_sobe_acima_do_escolhido() {
            let c = chosen(720, 30);
            assert_eq!(full_quality(&c, &[GOOD; 5], 0), q(720, 30, 2_500_000));
        }

        #[test]
        fn secao8_caso4_so_os_full_contam_para_o_teto() {
            let c = chosen(1080, 60);
            let viewers = [THUMB, THUMB, GOOD, GOOD, GOOD, GOOD];
            // Com os Thumb contando seriam 6 espectadores (720p60); contando só os 4 Full, não muda.
            assert_eq!(full_quality(&c, &viewers, 0), q(1080, 60, 6_000_000));
        }

        #[test]
        fn secao8_caso5_placa_limitada_por_10_s_baixa_um_degrau_na_cheia() {
            let c = chosen(1080, 60);
            let viewers = [GOOD; 3];
            let antes = full_quality(&c, &viewers, 0);
            let mut governor = CpuGovernor::new();
            for _ in 0..10 {
                governor.tick(encoder_limited(1), true);
            }
            let depois = full_quality(&c, &viewers, governor.steps);
            assert_eq!(antes, q(1080, 60, 6_000_000));
            assert_eq!(depois, q(1080, 30, 4_500_000));
        }
    }
}
