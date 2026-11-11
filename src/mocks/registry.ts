// Telas disponíveis no modo mock (?mock=<tela>&scene=<cena>). Ver specs/001-espia-screen-share/screens.md.
import { HOME_SCENES } from './home';
import { JOIN_SCENES } from './join';
import { MOBILE_SCENES } from './mobile';
import { NOTFOUND_SCENES } from './notfound';
import { DEFAULT_ROOM_SCENE, ROOM_SCENES } from './room';

export interface MockScene {
  id: string;
  code: string;
  label: string;
  /**
   * 'mobile' = renderiza num quadro de 390×844 (iframe), para os media queries valerem.
   * 'mobile-landscape' = celular deitado, quadro de 844×390 (specs/018).
   */
  device?: 'mobile' | 'mobile-landscape';
}

export interface MockScreen {
  id: string;
  label: string;
  scenes: readonly MockScene[];
  defaultScene: string;
}

export type Navigate = (screen: string, scene?: string) => void;

export const MOCK_SCREENS: MockScreen[] = [
  { id: 'home', label: 'Página inicial', scenes: HOME_SCENES, defaultScene: 'default' },
  { id: 'join', label: 'Entrar por link', scenes: JOIN_SCENES, defaultScene: 'default' },
  { id: 'room', label: 'Sala', scenes: ROOM_SCENES, defaultScene: DEFAULT_ROOM_SCENE },
  { id: 'mobile', label: 'Sala no celular', scenes: MOBILE_SCENES, defaultScene: 'watching' },
  { id: 'notfound', label: 'Sala não encontrada', scenes: NOTFOUND_SCENES, defaultScene: 'quick' },
];

/** Telas do handoff que ainda não têm mock (aparecem no índice como "em breve"). */
export const PLANNED_SCREENS: { id: string; label: string }[] = [];

/** `framed` mantém o parâmetro usado dentro do quadro de celular. */
export function mockHref(screen: string, scene?: string, framed = false): string {
  const params = new URLSearchParams({ mock: screen });
  if (scene) params.set('scene', scene);
  if (framed) params.set('frame', '1');
  return `${location.pathname}?${params}`;
}
