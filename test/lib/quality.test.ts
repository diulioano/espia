import { describe, expect, it } from 'vitest';
import { countFullViewers, effectiveQuality, isAdjusted, sendParams, stepDown, stepUp } from '../../src/lib/quality';

describe('effectiveQuality', () => {
  const p1080_60 = { resolution: 1080, fps: 60, audio: true, surface: 'window' } as const;
  const p1080_30 = { resolution: 1080, fps: 30, audio: true, surface: 'window' } as const;
  const p720_30 = { resolution: 720, fps: 30, audio: false, surface: 'window' } as const;

  it('1–4 espectadores: o que foi escolhido', () => {
    expect(effectiveQuality(p1080_60, 4)).toEqual({ resolution: 1080, fps: 60, maxBitrate: 6_000_000 });
    expect(effectiveQuality(p720_30, 1)).toEqual({ resolution: 720, fps: 30, maxBitrate: 2_500_000 });
  });

  it('5–6: 720p60 se escolheu 60 fps; senão 1080p30; até 4 Mbps', () => {
    expect(effectiveQuality(p1080_60, 5)).toEqual({ resolution: 720, fps: 60, maxBitrate: 4_000_000 });
    expect(effectiveQuality(p1080_30, 6)).toEqual({ resolution: 1080, fps: 30, maxBitrate: 4_000_000 });
  });

  it('7+: 720p30', () => {
    expect(effectiveQuality(p1080_60, 7)).toEqual({ resolution: 720, fps: 30, maxBitrate: 2_500_000 });
    expect(effectiveQuality(p1080_30, 9)).toEqual({ resolution: 720, fps: 30, maxBitrate: 2_500_000 });
  });

  it('nunca sobe acima do escolhido', () => {
    expect(effectiveQuality(p720_30, 5)).toEqual({ resolution: 720, fps: 30, maxBitrate: 2_500_000 });
  });

  it('isAdjusted só quando resolução ou fps mudaram', () => {
    expect(isAdjusted(p1080_60, effectiveQuality(p1080_60, 4))).toBe(false);
    expect(isAdjusted(p1080_60, effectiveQuality(p1080_60, 5))).toBe(true);
    expect(isAdjusted(p1080_30, effectiveQuality(p1080_30, 5))).toBe(false);
  });
});

describe('sendParams', () => {
  const q = { resolution: 1080, fps: 60, maxBitrate: 6_000_000 } as const;
  it('cheia: qualidade efetiva', () => {
    expect(sendParams(q, 'full')).toEqual({ active: true, scaleResolutionDownBy: 1, maxFramerate: 60, maxBitrate: 6_000_000 });
  });
  it('miniatura: ~360p, 15 fps, 0,5 Mbps', () => {
    expect(sendParams(q, 'thumb')).toEqual({ active: true, scaleResolutionDownBy: 3, maxFramerate: 15, maxBitrate: 500_000 });
  });
  it('escondida: pausa', () => {
    expect(sendParams(q, 'hidden').active).toBe(false);
  });
});

it('countFullViewers conta só "full"', () => {
  expect(countFullViewers(['full', 'thumb', 'hidden', 'full'])).toBe(2);
});

describe('degraus por CPU (specs/010)', () => {
  const q = (resolution: 720 | 1080, fps: 30 | 60) => ({ resolution, fps });

  it('stepDown: primeiro o fps, depois a resolução; mínimo 720p30', () => {
    expect(stepDown(q(1080, 60))).toEqual(q(1080, 30));
    expect(stepDown(q(1080, 30))).toEqual(q(720, 30));
    expect(stepDown(q(720, 60))).toEqual(q(720, 30));
    expect(stepDown(q(720, 30))).toEqual(q(720, 30));
  });

  it('stepUp: o inverso, sem passar do escolhido', () => {
    expect(stepUp(q(720, 30), q(1080, 60))).toEqual(q(1080, 30));
    expect(stepUp(q(1080, 30), q(1080, 60))).toEqual(q(1080, 60));
    expect(stepUp(q(720, 30), q(720, 60))).toEqual(q(720, 60));
    expect(stepUp(q(1080, 60), q(1080, 60))).toEqual(q(1080, 60));
  });

  it('effectiveQuality com degraus de CPU: vale o mais baixo junto com espectadores', () => {
    const chosen = { resolution: 1080, fps: 60, audio: true, surface: 'window' } as const;
    expect(effectiveQuality(chosen, 3, 1)).toEqual({ resolution: 1080, fps: 30, maxBitrate: 4_500_000 });
    expect(effectiveQuality(chosen, 3, 2)).toEqual({ resolution: 720, fps: 30, maxBitrate: 2_500_000 });
    // 5 espectadores já levam a 720p60 (até 4 Mbps); 1 degrau de CPU → 720p30.
    expect(effectiveQuality(chosen, 5, 1)).toEqual({ resolution: 720, fps: 30, maxBitrate: 2_500_000 });
    expect(isAdjusted(chosen, effectiveQuality(chosen, 3, 1))).toBe(true);
  });
});
