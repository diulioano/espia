// Cenas mockadas da tela de entrada (D2, sala cheia, M1, sala fixa F8). "Não existe" é ?mock=notfound.
import type { JoinView } from '../lib/types';
import { MOCK_ROOM_ID } from './room';

export const JOIN_SCENES = [
  { id: 'default', code: 'D2', label: 'Entrar por link' },
  { id: 'nopreview', code: 'D2', label: 'Sem prévia da sala' },
  { id: 'full', code: 'E2*', label: 'Sala cheia (10 de 10)' },
  { id: 'mobile', code: 'M1', label: 'Entrar no celular', device: 'mobile' },
  { id: 'fixed', code: 'F8', label: 'Sala fixa (specs/011)' },
  { id: 'replaced', code: 'D2·aba', label: 'Aberta em outra aba' },
  { id: 'fixedmobile', code: 'F8·M', label: 'Sala fixa no celular', device: 'mobile' },
] as const;

export type JoinScene = (typeof JOIN_SCENES)[number]['id'];

export function buildJoinScene(scene: JoinScene): JoinView {
  return {
    roomId: MOCK_ROOM_ID,
    status: scene === 'full' ? 'full' : 'ready',
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
    canShare: scene !== 'mobile' && scene !== 'fixedmobile',
    ...(scene === 'fixed' || scene === 'fixedmobile' ? { fixedName: 'CS da galera' } : {}),
    ...(scene === 'replaced' ? { replaced: true } : {}),
  };
}
