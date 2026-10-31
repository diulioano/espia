// Cenas mockadas da sala no celular (M2, M3). Reaproveitam as cenas da sala de computador.
import type { RoomView } from '../lib/types';
import { buildRoomScene } from './room';

export const MOBILE_SCENES = [
  { id: 'watching', code: 'M2', label: 'Assistindo (2 ao vivo)', device: 'mobile' },
  { id: 'empty', code: 'M3', label: 'Ninguém ao vivo', device: 'mobile' },
  { id: 'connecting', code: 'M·E1', label: 'Conectando', device: 'mobile' },
  { id: 'streamfail', code: 'M·E4', label: 'Falha na transmissão', device: 'mobile' },
  { id: 'camera', code: 'M·A5', label: 'Transmitir a câmera do celular', device: 'mobile' },
  { id: 'reconnecting', code: 'M5', label: 'Assistindo · reconectando', device: 'mobile' },
  { id: 'unwatched', code: 'M·W1', label: 'Parou de assistir (specs/014)', device: 'mobile' },
] as const;

export type MobileScene = (typeof MOBILE_SCENES)[number]['id'];

export function buildMobileScene(scene: MobileScene): RoomView {
  const base: Record<MobileScene, Parameters<typeof buildRoomScene>[0]> = {
    watching: 'grid',
    empty: 'empty',
    connecting: 'connecting',
    streamfail: 'streamfail',
    camera: 'empty',
    reconnecting: 'grid',
    unwatched: 'grid',
  };
  const view = buildRoomScene(base[scene]);
  // Celular: não transmite tela; só a câmera (specs/006 US4) na cena 'camera'.
  view.canShare = scene === 'camera';
  view.canShareScreen = false;
  if (scene === 'camera') {
    view.goLiveOpen = true;
    view.goLivePrefs = { ...view.goLivePrefs, surface: 'camera', resolution: 720, fps: 30 };
    view.camera = { ...view.camera, mobile: true, facing: 'environment' };
  }
  view.toasts = [];

  if (scene === 'watching' || scene === 'reconnecting' || scene === 'unwatched') {
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
  // M5 (specs/011): caiu enquanto assistia a Lu.
  if (scene === 'reconnecting') view.link = 'reconnecting';
  if (scene === 'unwatched') view.streams.find((s) => s.id === 'lu')!.watching = false;
  return view;
}
