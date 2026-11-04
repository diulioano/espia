import { describe, expect, it } from 'vitest';
import { CaptureTiming } from '../../src/lib/capture-timing';

describe('CaptureTiming', () => {
  it.each([2000, 10000])('não incorpora startup de %i ms na época do vídeo', (startup) => {
    let now = startup * 1000;
    const t = new CaptureTiming(() => now);
    expect(t.video(0, 0)).toBe(now);
    expect(t.audio(480)).toBe(now);
    now += 10000;
    expect(t.video(10000, 1)).toBe(now);
  });
  it('descarta timestamp velho, inválido e regressivo sem reancorar cada frame', () => {
    let now = 1_000_000;
    const t = new CaptureTiming(() => now);
    expect(t.video(0, 0)).toBe(now);
    now += 1_000_000;
    expect(t.video(10000, 1)).toBeNull();
    expect(t.video(1_000_000, 100)).toBe(now);
    expect(t.video(500_000, 101)).toBeNull();
    expect(t.video(NaN, 102)).toBeNull();
    expect(t.video(1_100_000, Infinity)).toBeNull();
  });
  it('áudio não agenda segundos no futuro por burst, reancora após lacuna e recriação', () => {
    let now = 1_000_000;
    const t = new CaptureTiming(() => now);
    for (let i = 0; i < 20; i++) expect(t.audio(480)).toBe(now + i * 10000);
    expect(t.audio(480)).toBeNull();
    now += 10_000_000;
    expect(t.audio(480)).toBe(now);
    t.resetAudio();
    now += 10_000_000;
    expect(t.audio(480)).toBe(now);
    expect(t.audio(0)).toBeNull();
    expect(t.audio(480.5)).toBeNull();
  });
  it('stop é terminal', () => {
    const t = new CaptureTiming(() => 1000);
    t.stop();
    t.resetAudio();
    expect(t.video(0, 0)).toBeNull();
    expect(t.audio(480)).toBeNull();
  });
});
