// Cenas mockadas da tela de entrada (D2, E2, sala cheia, M1).
import type { JoinView } from '../lib/types';
import { MOCK_ROOM_ID } from './room';

export const JOIN_SCENES = [
  { id: 'default', code: 'D2', label: 'Entrar por link' },
  { id: 'nopreview', code: 'D2', label: 'Sem prévia da sala' },
  { id: 'notfound', code: 'E2', label: 'Sala não encontrada' },
  { id: 'full', code: 'E2*', label: 'Sala cheia (10 de 10)' },
  { id: 'mobile', code: 'M1', label: 'Entrar no celular', device: 'mobile' },
] as const;

export type JoinScene = (typeof JOIN_SCENES)[number]['id'];

export function buildJoinScene(scene: JoinScene): JoinView {
  return {
    roomId: MOCK_ROOM_ID,
    status: scene === 'notfound' ? 'notfound' : scene === 'full' ? 'full' : 'ready',
    people:
      scene === 'nopreview'
        ? null
        : [
            { name: 'Lu', tone: 'accent', live: true },
            { name: 'Bia', tone: 'sage', live: false },
            { name: 'Tomás', tone: 'sand', live: false },
          ],
    capacity: 10,
    nickname: 'Rafa',
    canShare: scene !== 'mobile',
  };
}
