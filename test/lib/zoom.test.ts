import { describe, expect, it } from 'vitest';
import { clampZoom, isDoubleTap, MAX_ZOOM, pan, pinch, ZOOM_NONE } from '../../src/lib/zoom';

const box = { width: 400, height: 800 };

describe('zoom da tela cheia (specs/018 US3)', () => {
  it('clampZoom: escala entre 1 e 4', () => {
    expect(clampZoom({ scale: 0.5, x: -10, y: -10 }, box)).toEqual(ZOOM_NONE);
    expect(clampZoom({ scale: 9, x: 0, y: 0 }, box).scale).toBe(MAX_ZOOM);
  });
  it('clampZoom: a imagem não descola das bordas', () => {
    expect(clampZoom({ scale: 2, x: 50, y: 50 }, box)).toEqual({ scale: 2, x: 0, y: 0 });
    expect(clampZoom({ scale: 2, x: -9999, y: -9999 }, box)).toEqual({ scale: 2, x: -400, y: -800 });
  });
  it('pinch: dedos de 100 para 200 px dobram a escala e o meio fica parado', () => {
    const s = pinch(ZOOM_NONE, { x: 150, y: 400 }, { x: 250, y: 400 }, { x: 100, y: 400 }, { x: 300, y: 400 }, box);
    expect(s.scale).toBeCloseTo(2);
    // O ponto (200, 400) da imagem continua sob o meio dos dedos.
    expect(200 * s.scale + s.x).toBeCloseTo(200);
    expect(400 * s.scale + s.y).toBeCloseTo(400);
  });
  it('pinch: não passa de 4×', () => {
    const s = pinch({ scale: 3, x: -400, y: -800 }, { x: 190, y: 400 }, { x: 210, y: 400 }, { x: 0, y: 400 }, { x: 400, y: 400 }, box);
    expect(s.scale).toBe(MAX_ZOOM);
  });
  it('pan: sem zoom não move; com zoom, não passa das bordas', () => {
    expect(pan(ZOOM_NONE, 50, 50, box)).toEqual(ZOOM_NONE);
    expect(pan({ scale: 2, x: -100, y: -100 }, -50, 30, box)).toEqual({ scale: 2, x: -150, y: -70 });
    expect(pan({ scale: 2, x: -100, y: -100 }, 500, -2000, box)).toEqual({ scale: 2, x: 0, y: -800 });
  });
  it('isDoubleTap: 300 ms e 24 px', () => {
    const a = { t: 1000, x: 100, y: 100 };
    expect(isDoubleTap(a, { t: 1250, x: 110, y: 100 })).toBe(true);
    expect(isDoubleTap(a, { t: 1350, x: 100, y: 100 })).toBe(false);
    expect(isDoubleTap(a, { t: 1100, x: 130, y: 100 })).toBe(false);
    expect(isDoubleTap(null, a)).toBe(false);
  });
});
