import { describe, expect, it } from 'vitest';
import { fitWidth, SNAPSHOT_MS } from '../../src/lib/snapshot';

describe('snapshot (specs/014 US3)', () => {
  it('reduz pra largura mantendo a proporção', () => {
    expect(fitWidth(1920, 1080, 320)).toEqual({ w: 320, h: 180 });
    expect(fitWidth(1080, 1920, 320)).toEqual({ w: 320, h: 569 });
  });
  it('não aumenta imagem pequena', () => {
    expect(fitWidth(200, 100, 320)).toEqual({ w: 200, h: 100 });
  });
  it('sem tamanho ainda: 16:9', () => {
    expect(fitWidth(0, 0, 320)).toEqual({ w: 320, h: 180 });
  });
  it('5 s ± 2 (FR-010)', () => {
    expect(SNAPSHOT_MS).toBeGreaterThanOrEqual(3000);
    expect(SNAPSHOT_MS).toBeLessThanOrEqual(7000);
  });
});
