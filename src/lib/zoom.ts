// Zoom com dois dedos na tela cheia do celular (specs/018 US3, research R4). Só contas: o Viewer liga os gestos.

export interface ZoomState {
  scale: number;
  /** translate em px, aplicado antes do scale (transform-origin 0 0). */
  x: number;
  y: number;
}
export interface Box {
  width: number;
  height: number;
}
export const ZOOM_NONE: ZoomState = { scale: 1, x: 0, y: 0 };
export const MAX_ZOOM = 4;

export interface Point {
  x: number;
  y: number;
}

/** Escala entre 1 e MAX_ZOOM, e a imagem nunca descola das bordas do visualizador. */
export function clampZoom(s: ZoomState, box: Box): ZoomState {
  const scale = Math.min(MAX_ZOOM, Math.max(1, s.scale));
  if (scale === 1) return ZOOM_NONE;
  const clamp = (v: number, size: number) => Math.min(0, Math.max(size * (1 - scale), v));
  return { scale, x: clamp(s.x, box.width), y: clamp(s.y, box.height) };
}

const mid = (a: Point, b: Point): Point => ({ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 });
const dist = (a: Point, b: Point) => Math.hypot(a.x - b.x, a.y - b.y);

/**
 * Dois dedos: de (a0, b0) para (a1, b1). O ponto da imagem que estava no meio dos dedos
 * acompanha o novo meio (aproximar e arrastar no mesmo gesto).
 */
export function pinch(s: ZoomState, a0: Point, b0: Point, a1: Point, b1: Point, box: Box): ZoomState {
  const d0 = dist(a0, b0);
  if (d0 < 1) return s;
  const scale = Math.min(MAX_ZOOM, Math.max(1, s.scale * (dist(a1, b1) / d0)));
  const m0 = mid(a0, b0);
  const m1 = mid(a1, b1);
  // Ponto da imagem (sem zoom) sob o meio antigo.
  const ix = (m0.x - s.x) / s.scale;
  const iy = (m0.y - s.y) / s.scale;
  return clampZoom({ scale, x: m1.x - ix * scale, y: m1.y - iy * scale }, box);
}

/** Um dedo arrastando a imagem aproximada. Sem zoom não mexe. */
export function pan(s: ZoomState, dx: number, dy: number, box: Box): ZoomState {
  if (s.scale <= 1) return s;
  return clampZoom({ scale: s.scale, x: s.x + dx, y: s.y + dy }, box);
}

export interface Tap {
  t: number;
  x: number;
  y: number;
}

/** Toque duplo: até 300 ms e 24 px do anterior. */
export function isDoubleTap(prev: Tap | null, now: Tap): boolean {
  return !!prev && now.t - prev.t <= 300 && Math.hypot(now.x - prev.x, now.y - prev.y) <= 24;
}
