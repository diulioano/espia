// Captura de tela (architecture.md §4.3). Precisa ser chamada direto do clique (ativação do usuário).
import type { GoLivePrefs } from './types';

export const canShare = () => typeof navigator !== 'undefined' && !!navigator.mediaDevices?.getDisplayMedia;

export async function startCapture(prefs: GoLivePrefs): Promise<MediaStream> {
  const width = prefs.resolution === 1080 ? 1920 : 1280;
  const options = {
    video: { width: { ideal: width }, height: { ideal: prefs.resolution }, frameRate: { ideal: prefs.fps } },
    audio: prefs.audio ? { echoCancellation: false, noiseSuppression: false, autoGainControl: false } : false,
    // Chrome/Edge: oferece o áudio do sistema; esconde a própria aba do Espia (espelho infinito).
    systemAudio: 'include',
    selfBrowserSurface: 'exclude',
    surfaceSwitching: 'include',
  } as DisplayMediaStreamOptions;
  const stream = await navigator.mediaDevices.getDisplayMedia(options);
  // 60 fps: jogo/vídeo, prioriza fluidez. 30 fps: código/slides, prioriza nitidez.
  for (const track of stream.getVideoTracks()) track.contentHint = prefs.fps === 60 ? 'motion' : 'detail';
  return stream;
}

export function stopCapture(stream: MediaStream) {
  for (const track of stream.getTracks()) track.stop();
}

/** Fechou o seletor ou negou a permissão (E3). */
export function isPermissionDenied(err: unknown) {
  return err instanceof DOMException && err.name === 'NotAllowedError';
}
