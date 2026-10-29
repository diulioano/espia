// Câmeras e placas de captura no navegador e no app (specs/006 research R1–R6). As decisões ficam em
// devices.ts (puro); aqui só o que fala com navigator.mediaDevices.
import { audioConstraints, videoConstraints, type AudioRole, type DeviceInfo } from './devices';
import type { Fps, Resolution } from './types';

export type CameraPermission = 'unknown' | 'prompt' | 'granted' | 'denied';

const media = () => (typeof navigator !== 'undefined' ? navigator.mediaDevices : undefined);

/** Pode abrir câmera (inclusive no celular). */
export const canUseCamera = () => !!media()?.getUserMedia && !!media()?.enumerateDevices;

/** Estado da permissão sem pedir nada (FR-602). */
export async function cameraPermission(): Promise<CameraPermission> {
  try {
    const s = await navigator.permissions.query({ name: 'camera' as PermissionName });
    return s.state === 'granted' ? 'granted' : s.state === 'denied' ? 'denied' : 'prompt';
  } catch {
    // Safari antigo / sem a API: só dá pra saber pedindo.
    return 'unknown';
  }
}

/** Por que a câmera não abriu (a aba mostra o motivo e "Tentar de novo"). */
export type CameraError = 'busy' | 'notfound' | 'other';

/** Pede a permissão (no clique em "Permitir câmeras") e fecha a câmera na hora. */
export async function requestCamera(): Promise<{ permission: CameraPermission; error?: CameraError }> {
  try {
    const s = await navigator.mediaDevices.getUserMedia({ video: true });
    for (const t of s.getTracks()) t.stop();
    return { permission: 'granted' };
  } catch (err) {
    const name = err instanceof DOMException ? err.name : '';
    console.warn('[espia] câmera não abriu:', name, err instanceof Error ? err.message : err);
    if (name === 'NotAllowedError' || name === 'SecurityError') return { permission: 'denied' };
    // Ocupada ou não encontrada: a permissão em si não foi negada, mas sem abrir a câmera o
    // navegador não mostra os nomes — a aba precisa explicar o motivo.
    if (name === 'NotReadableError' || name === 'AbortError' || name === 'TrackStartError') return { permission: 'granted', error: 'busy' };
    if (name === 'NotFoundError' || name === 'OverconstrainedError' || name === 'DevicesNotFoundError') return { permission: 'granted', error: 'notfound' };
    return { permission: 'granted', error: 'other' };
  }
}

/**
 * Dispositivos com nome. `unnamed`: há câmeras que o navegador ainda não deixou nomear (acontece mesmo com
 * a permissão dada, até a página abrir a câmera uma vez).
 */
export async function listDevices(): Promise<{ videos: DeviceInfo[]; audios: DeviceInfo[]; unnamed: boolean }> {
  const all = await navigator.mediaDevices.enumerateDevices();
  const pick = (kind: MediaDeviceKind) =>
    all.filter((d) => d.kind === kind && d.label).map((d) => ({ id: d.deviceId, groupId: d.groupId, label: d.label }));
  const unnamed = all.some((d) => d.kind === 'videoinput' && !d.label);
  return { videos: pick('videoinput'), audios: pick('audioinput'), unnamed };
}

/** Lista mudou (câmera plugada/desplugada). Devolve a função de parar. */
export function watchDevices(cb: () => void): () => void {
  const m = media();
  if (!m) return () => {};
  m.addEventListener('devicechange', cb);
  return () => m.removeEventListener('devicechange', cb);
}

/** Prévia pequena e leve. `busy` = câmera em uso por outro programa. */
export async function openPreview(id: string): Promise<{ stream: MediaStream | null; busy: boolean }> {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({
      video: { deviceId: { exact: id }, width: { ideal: 320 }, height: { ideal: 180 }, frameRate: { ideal: 15, max: 15 } },
    });
    return { stream, busy: false };
  } catch (err) {
    return { stream: null, busy: err instanceof DOMException && err.name === 'NotReadableError' };
  }
}

export function stopStream(stream: MediaStream | null | undefined) {
  for (const t of stream?.getTracks() ?? []) t.stop();
}

export interface CameraRequest {
  /** deviceId (PC) ou, sem ele, `facing` (celular). */
  videoId?: string;
  facing?: 'user' | 'environment';
  /** null = sem som; '' = microfone padrão (celular). */
  audioId: string | null;
  audioRole: AudioRole;
  resolution: Resolution;
  fps: Fps;
}

/** Abre a câmera/placa para transmitir. O som, se houver, vem do mesmo pedido (FR-606b). */
export async function startCamera(req: CameraRequest): Promise<MediaStream> {
  const stream = await navigator.mediaDevices.getUserMedia({
    video: videoConstraints({ id: req.videoId, facing: req.facing }, req.resolution, req.fps),
    audio: req.audioId !== null ? audioConstraints(req.audioId, req.audioRole) : false,
  });
  for (const t of stream.getVideoTracks()) t.contentHint = 'motion';
  // Som de jogo: dica pro codificador não tratar como fala.
  for (const t of stream.getAudioTracks()) t.contentHint = req.audioRole === 'game' ? 'music' : 'speech';
  return stream;
}

/** Câmera ocupada por outro programa. */
export function isBusy(err: unknown) {
  return err instanceof DOMException && err.name === 'NotReadableError';
}
