// Captura de tela (architecture.md §4.3). Precisa ser chamada direto do clique (ativação do usuário).
import { decideAudio, type Surface } from './audio-source';
import type { AudioKind, GoLivePrefs } from './types';

export const canShare = () => typeof navigator !== 'undefined' && !!navigator.mediaDevices?.getDisplayMedia;

export async function startCapture(prefs: GoLivePrefs): Promise<MediaStream> {
  const width = prefs.resolution === 1080 ? 1920 : 1280;
  // Tela inteira nunca leva som (specs/002 FR-202): nem pede.
  const wantAudio = prefs.audio && prefs.surface !== 'monitor';
  const options = {
    video: {
      width: { ideal: width },
      height: { ideal: prefs.resolution },
      frameRate: { ideal: prefs.fps },
      // Abre o seletor nativo na aba escolhida no modal (specs/004). É só uma dica.
      displaySurface: prefs.surface,
    },
    audio: wantAudio ? { echoCancellation: false, noiseSuppression: false, autoGainControl: false } : false,
    // Som do sistema = PC inteiro, inclusive a chamada do Discord (specs/002 research R1/R2).
    systemAudio: 'exclude',
    // Janela: só o som dela (Chrome/Edge 141+, research R3).
    windowAudio: 'window',
    // Esconde a própria aba do Espia (espelho infinito).
    selfBrowserSurface: 'exclude',
    surfaceSwitching: 'include',
  } as DisplayMediaStreamOptions;
  const stream = await navigator.mediaDevices.getDisplayMedia(options);
  // 60 fps: jogo/vídeo, prioriza fluidez. 30 fps: código/slides, prioriza nitidez.
  for (const track of stream.getVideoTracks()) track.contentHint = prefs.fps === 60 ? 'motion' : 'detail';
  return stream;
}

export interface AppliedAudio {
  kind: AudioKind;
  warn: boolean;
  reason?: 'monitor';
}

/**
 * Rede de segurança (specs/002 FR-203/FR-204): decide pelo que o navegador realmente entregou e
 * remove do stream o som que não pode sair, antes de ele chegar à malha.
 */
export function applyAudioRule(stream: MediaStream, wanted: boolean): AppliedAudio {
  const surface = stream.getVideoTracks()[0]?.getSettings().displaySurface as Surface | undefined;
  const track = stream.getAudioTracks()[0] ?? null;
  const d = decideAudio({ wanted, surface, audioLabel: track?.label ?? null });
  console.debug('[espia] áudio:', surface, track?.label ?? '(sem faixa)', d.kind);
  if (d.drop) {
    for (const t of stream.getAudioTracks()) {
      t.stop();
      stream.removeTrack(t);
    }
  }
  return { kind: d.kind, warn: d.warn, reason: surface === 'monitor' ? 'monitor' : undefined };
}

export function stopCapture(stream: MediaStream) {
  for (const track of stream.getTracks()) track.stop();
}

/** Fechou o seletor ou negou a permissão (E3). */
export function isPermissionDenied(err: unknown) {
  return err instanceof DOMException && err.name === 'NotAllowedError';
}
