// Cenas mockadas da sala no celular (M2, M3). Reaproveitam as cenas da sala de computador.
import type { RoomView } from '../lib/types';
import { buildRoomScene } from './room';

export const MOBILE_SCENES = [
  { id: 'watching', code: 'M2', label: 'Assistindo (2 ao vivo)', device: 'mobile' },
  { id: 'empty', code: 'M3', label: 'Ninguém ao vivo', device: 'mobile' },
  { id: 'connecting', code: 'M·E1', label: 'Conectando', device: 'mobile' },
  { id: 'streamfail', code: 'M·E4', label: 'Falha na transmissão', device: 'mobile' },
] as const;

export type MobileScene = (typeof MOBILE_SCENES)[number]['id'];

export function buildMobileScene(scene: MobileScene): RoomView {
  const base: Record<MobileScene, Parameters<typeof buildRoomScene>[0]> = {
    watching: 'grid',
    empty: 'empty',
    connecting: 'connecting',
    streamfail: 'streamfail',
  };
  const view = buildRoomScene(base[scene]);
  view.canShare = false;
  view.toasts = [];

  if (scene === 'watching') {
    // M2: Lu e Bia ao vivo, vendo Lu.
    view.streams = view.streams.filter((s) => s.id !== 'tomas');
    view.participants.forEach((p) => (p.live = p.id === 'lu' || p.id === 'bia'));
    view.focusedId = 'lu';
  }
  if (scene === 'empty') {
    // M3: só 3 pessoas e o toast de entrada.
    view.participants = view.participants.filter((p) => p.id !== 'tomas');
    view.toasts = [{ id: 'scene-m3', kind: 'join', text: 'Bia entrou na sala' }];
  }
  return view;
}
