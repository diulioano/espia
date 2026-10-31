// Qualidade de quem transmite (architecture.md §4.1b e §4.2). Funções puras: sem WebRTC aqui.
import type { Fps, GoLivePrefs, Resolution } from './types';

/** Como alguém está vendo a SUA tela. Vai pelo canal `ctl` de quem assiste para quem transmite. */
export type ViewMode = 'full' | 'thumb' | 'hidden';

export interface Quality {
  resolution: Resolution;
  fps: Fps;
  /** bps para cada espectador em tela cheia/grade. */
  maxBitrate: number;
}

/** Bitrate máximo por espectador (research.md R10). */
const BITRATE: Record<`${Resolution}-${Fps}`, number> = {
  '720-30': 2_500_000,
  '720-60': 4_000_000,
  '1080-30': 4_500_000,
  '1080-60': 6_000_000,
};

/**
 * Teto pelo número de pessoas vendo em tela cheia/grade (miniaturas e escondidas não contam).
 * Nunca sobe acima do que foi escolhido no modal.
 *  1–4: o escolhido · 5–6: 720p60 (se escolheu 60) ou 1080p30, até 4 Mbps · 7+: 720p30.
 */
export function effectiveQuality(chosen: GoLivePrefs, fullViewers: number, cpuSteps = 0): Quality {
  let { resolution, fps } = chosen;
  let cap = Infinity;
  if (fullViewers >= 7) {
    resolution = 720;
    fps = 30;
  } else if (fullViewers >= 5) {
    if (fps === 60) resolution = 720;
    cap = 4_000_000;
  }
  // PC no limite (specs/010): degraus a menos, depois da regra de espectadores. Vale o mais baixo.
  for (let i = 0; i < cpuSteps; i++) ({ resolution, fps } = stepDown({ resolution, fps }));
  return { resolution, fps, maxBitrate: Math.min(BITRATE[`${resolution}-${fps}`], cap) };
}

type Step = { resolution: Resolution; fps: Fps };

/** Um degrau abaixo: primeiro o fps, depois a resolução. Mínimo 720p30. */
export function stepDown(q: Step): Step {
  if (q.fps === 60) return { resolution: q.resolution, fps: 30 };
  return { resolution: 720, fps: 30 };
}

/** Um degrau acima, sem passar do escolhido. */
export function stepUp(q: Step, chosen: Step): Step {
  if (q.resolution < chosen.resolution) return { resolution: chosen.resolution, fps: 30 };
  if (q.fps < chosen.fps) return { resolution: q.resolution, fps: chosen.fps };
  return q;
}

export function isAdjusted(chosen: GoLivePrefs, q: Quality) {
  return q.resolution !== chosen.resolution || q.fps !== chosen.fps;
}

export interface SendParams {
  active: boolean;
  scaleResolutionDownBy: number;
  maxFramerate: number;
  maxBitrate: number;
}

/** Miniatura: ~360p · 15 fps · 0,5 Mbps. Escondida: pausa o vídeo (o som continua, specs/014). */
export function sendParams(q: Quality, mode: ViewMode): SendParams {
  if (mode === 'hidden') return { active: false, scaleResolutionDownBy: 1, maxFramerate: q.fps, maxBitrate: q.maxBitrate };
  if (mode === 'thumb') return { active: true, scaleResolutionDownBy: q.resolution / 360, maxFramerate: 15, maxBitrate: 500_000 };
  return { active: true, scaleResolutionDownBy: 1, maxFramerate: q.fps, maxBitrate: q.maxBitrate };
}

export function countFullViewers(modes: Iterable<ViewMode>) {
  let n = 0;
  for (const m of modes) if (m === 'full') n++;
  return n;
}
