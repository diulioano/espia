// Que som pode sair na transmissão (specs/002 data-model.md). Função pura: sem navegador, testável.
import type { AudioKind } from '../shared/protocol';

export type Surface = 'browser' | 'window' | 'monitor';

export interface AudioDecision {
  kind: AudioKind;
  /** A faixa de áudio recebida tem que ser parada e removida antes de enviar. */
  drop: boolean;
  /** Mostrar o aviso "Tela inteira vai sem som…". */
  warn: boolean;
}

/** Rótulo que o Chrome/Edge dá ao som do PC inteiro ("System Audio"). Ver research.md R3. */
const SYSTEM_LABEL = /system/i;

export function decideAudio(input: {
  wanted: boolean;
  surface: Surface | undefined;
  audioLabel: string | null;
}): AudioDecision {
  const { wanted, surface, audioLabel } = input;
  const has = audioLabel !== null;
  if (!wanted) return { kind: 'none', drop: has, warn: false };
  if (surface === 'monitor') return { kind: 'none', drop: has, warn: true };
  if (!has) return { kind: 'none', drop: false, warn: false };
  if (surface === 'browser') return { kind: 'tab', drop: false, warn: false };
  if (surface === 'window') {
    return SYSTEM_LABEL.test(audioLabel) ? { kind: 'none', drop: true, warn: true } : { kind: 'window', drop: false, warn: false };
  }
  // Superfície desconhecida: na dúvida, nada de som.
  return { kind: 'none', drop: true, warn: false };
}

/** Texto do card "Você está ao vivo" (FR-207). */
export function audioLabelText(kind: AudioKind, reason?: 'monitor'): string {
  if (kind === 'tab') return 'com som da aba';
  if (kind === 'window') return 'com som da janela';
  return reason === 'monitor' ? 'sem som (tela inteira)' : 'sem som';
}
