// O que cada pessoa está transmitindo (specs/012, research R1). Nunca o título de uma janela: no app
// vai só o nome do programa; no site, só o tipo que o navegador entregou; na câmera, o nome dela.
import { LIVE_LABEL_MAX } from '../shared/protocol';

export type LabelInput =
  | { kind: 'native-window'; app: string }
  | { kind: 'native-monitor'; title: string; monitors: number }
  | { kind: 'browser'; surface: string | undefined }
  | { kind: 'camera'; title: string | undefined; facing?: 'user' | 'environment' };

const SURFACE: Record<string, string> = { monitor: 'Tela inteira', window: 'Janela', browser: 'Aba do navegador' };

export function liveLabel(input: LabelInput): string | undefined {
  switch (input.kind) {
    case 'native-window':
      return cut(input.app.trim() || 'Janela');
    case 'native-monitor':
      return cut(input.monitors > 1 && input.title.trim() ? input.title.trim() : 'Tela inteira');
    case 'browser':
      return input.surface ? SURFACE[input.surface] : undefined;
    case 'camera': {
      const title = input.title?.trim();
      if (title) return cut(`Câmera · ${title}`);
      if (input.facing) return input.facing === 'user' ? 'Câmera frontal' : 'Câmera traseira';
      return 'Câmera';
    }
  }
}

function cut(text: string): string {
  const chars = [...text];
  return chars.length > LIVE_LABEL_MAX ? `${chars.slice(0, LIVE_LABEL_MAX - 1).join('').trimEnd()}…` : text;
}
