// Quadro parado da própria transmissão (specs/014 US3, research R6): a prévia "Você está ao vivo" troca a
// imagem a cada poucos segundos em vez de desenhar um vídeo ao vivo o tempo todo.

/** De quanto em quanto tempo a prévia troca de imagem (FR-010: 5 s ± 2). */
export const SNAPSHOT_MS = 5_000;
/** Largura da prévia guardada: a barra mostra ~250 px. */
export const SNAPSHOT_WIDTH = 320;

interface ImageCaptureLike {
  grabFrame(): Promise<ImageBitmap>;
}
type ImageCaptureCtor = new (track: MediaStreamTrack) => ImageCaptureLike;
type ProcessorCtor = new (init: { track: MediaStreamTrack }) => { readable: ReadableStream<VideoFrame> };

/** Tamanho reduzido mantendo a proporção. */
export function fitWidth(w: number, h: number, width: number): { w: number; h: number } {
  if (w <= 0 || h <= 0) return { w: width, h: Math.round((width * 9) / 16) };
  const scale = Math.min(1, width / w);
  return { w: Math.max(1, Math.round(w * scale)), h: Math.max(1, Math.round(h * scale)) };
}

/** Um quadro do track, reduzido para `width` px. Qualquer problema: null (a prévia fica no placeholder). */
export async function grabSnapshot(track: MediaStreamTrack, width = SNAPSHOT_WIDTH): Promise<ImageBitmap | null> {
  if (track.readyState !== 'live' || typeof OffscreenCanvas === 'undefined') return null;
  try {
    const source = await grabRaw(track);
    if (!source) return null;
    const sw = 'displayWidth' in source ? source.displayWidth : source.width;
    const sh = 'displayHeight' in source ? source.displayHeight : source.height;
    const { w, h } = fitWidth(sw, sh, width);
    const canvas = new OffscreenCanvas(w, h);
    canvas.getContext('2d')?.drawImage(source, 0, 0, w, h);
    source.close();
    return canvas.transferToImageBitmap();
  } catch {
    return null;
  }
}

async function grabRaw(track: MediaStreamTrack): Promise<ImageBitmap | VideoFrame | null> {
  const IC = (globalThis as { ImageCapture?: ImageCaptureCtor }).ImageCapture;
  if (IC) {
    try {
      return await new IC(track).grabFrame();
    } catch {
      // Alguns tracks (gerados) recusam: tenta o leitor de quadros.
    }
  }
  const MP = (globalThis as { MediaStreamTrackProcessor?: ProcessorCtor }).MediaStreamTrackProcessor;
  if (!MP) return null;
  const clone = track.clone();
  const reader = new MP({ track: clone }).readable.getReader();
  try {
    const { value } = await Promise.race([
      reader.read(),
      new Promise<{ value: undefined }>((r) => setTimeout(() => r({ value: undefined }), 2000)),
    ]);
    return value ?? null;
  } finally {
    void reader.cancel().catch(() => {});
    clone.stop();
  }
}
