// Câmeras, placas de captura e entradas de som (specs/006 research R3–R5). Funções puras: sem
// navegador, testáveis. Quem fala com o navegador é camera.ts.
import type { Fps, Resolution } from './types';

export type AudioRole = 'voice' | 'game';

export interface DeviceInfo {
  id: string;
  groupId: string;
  label: string;
}

/** Até quantas prévias ao vivo ao mesmo tempo (FR-603). */
export const PREVIEW_LIMIT = 4;

const CAPTURE_RE = /capture|hdmi|elgato|avermedia|cam ?link|game|usb ?3?\.?0? ?video|live gamer/i;
const GAME_AUDIO_RE = /capture|hdmi|line|elgato|avermedia|cam ?link|game|digital audio interface|live gamer/i;
const VOICE_RE = /mic|microfone|headset|fone|headphone/i;

/** O nome parece de placa de captura (ícone e sugestão de som). */
export function isCaptureCard(label: string): boolean {
  return CAPTURE_RE.test(label);
}

/**
 * Voz (com cancelamento de eco) ou jogo (cru)? Na dúvida, voz: protege contra eco.
 * `pairedWithCapture`: a entrada é do mesmo aparelho de uma placa de captura.
 */
export function audioRole(label: string, pairedWithCapture = false): AudioRole {
  if (VOICE_RE.test(label)) return 'voice';
  if (pairedWithCapture || GAME_AUDIO_RE.test(label)) return 'game';
  return 'voice';
}

function tokens(label: string): Set<string> {
  return new Set(
    label
      .toLowerCase()
      // Tira o "(046d:0825)" de ids USB e pontuação.
      .replace(/\([0-9a-f]{4}:[0-9a-f]{4}\)/g, ' ')
      .split(/[^a-z0-9à-ú]+/)
      .filter((t) => t.length >= 3 && !['default', 'padrão', 'usb', 'audio', 'áudio', 'device', 'video', 'vídeo'].includes(t)),
  );
}

/** A entrada de som do mesmo aparelho (groupId) ou de nome parecido; senão nenhuma. */
export function suggestAudio(video: DeviceInfo, audios: DeviceInfo[]): DeviceInfo | null {
  const same = video.groupId ? audios.find((a) => a.groupId === video.groupId && a.id !== 'default' && a.id !== 'communications') : undefined;
  if (same) return same;
  const want = tokens(video.label);
  let best: DeviceInfo | null = null;
  let score = 0;
  for (const a of audios) {
    if (a.id === 'default' || a.id === 'communications') continue;
    let s = 0;
    for (const t of tokens(a.label)) if (want.has(t)) s++;
    if (s > score) {
      score = s;
      best = a;
    }
  }
  return best;
}

/** Constraints do som por tipo (data-model.md "Constraints por tipo de som"). id '' = entrada padrão (celular). */
export function audioConstraints(id: string, role: AudioRole): MediaTrackConstraints {
  const voice = role === 'voice';
  return {
    ...(id ? { deviceId: { exact: id } } : {}),
    echoCancellation: voice,
    noiseSuppression: voice,
    autoGainControl: voice,
    channelCount: { ideal: voice ? 1 : 2 },
  };
}

/** Constraints do vídeo: o que a câmera der, até a qualidade escolhida. */
export function videoConstraints(target: { id?: string; facing?: 'user' | 'environment' }, resolution: Resolution, fps: Fps): MediaTrackConstraints {
  const width = resolution === 1080 ? 1920 : 1280;
  return {
    ...(target.id ? { deviceId: { exact: target.id } } : { facingMode: { ideal: target.facing ?? 'user' } }),
    width: { ideal: width },
    height: { ideal: resolution },
    frameRate: { ideal: fps },
  };
}

/** Quais dispositivos ganham prévia ao vivo: a escolhida primeiro, depois os livres, até o limite. */
export function previewIds(devices: { id: string; busy?: boolean }[], selectedId: string | null, limit = PREVIEW_LIMIT): string[] {
  const free = devices.filter((d) => !d.busy);
  const first = free.find((d) => d.id === selectedId);
  const rest = free.filter((d) => d.id !== selectedId);
  return [...(first ? [first] : []), ...rest].slice(0, limit).map((d) => d.id);
}
