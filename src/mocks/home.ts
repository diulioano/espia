// Cenas mockadas da página inicial (D1).
import type { HomeView } from '../lib/types';

export const HOME_SCENES = [
  { id: 'default', code: 'D1', label: 'Página inicial' },
  { id: 'empty', code: 'D1', label: 'Primeira visita (sem apelido)' },
  { id: 'error', code: 'D1', label: 'Link inválido' },
] as const;

export type HomeScene = (typeof HOME_SCENES)[number]['id'];

export const LINK_ERROR = 'Esse link não parece de uma sala.';

export function buildHomeScene(scene: HomeScene): HomeView {
  return {
    nickname: scene === 'empty' ? '' : 'Rafa',
    link: scene === 'error' ? 'https://youtu.be/dQw4w9WgXcQ' : '',
    linkError: scene === 'error' ? LINK_ERROR : null,
  };
}
