// Cenas mockadas da sala no celular (M2, M3). Reaproveitam as cenas da sala de computador.
import type { RoomView } from '../lib/types';
import { clampZoom, ZOOM_NONE } from '../lib/zoom';
import { buildRoomScene, isRoomScene } from './room';

export const MOBILE_SCENES = [
  ...(['available','stopped','opening','playing','unavailable','reconnecting'] as const).map(state=>({id:`watch-${state}` as const,code:'017',label:`Assistir: ${state}`,device:'mobile' as const})),
  { id: 'watching', code: 'M2', label: 'Assistindo (2 ao vivo)', device: 'mobile' },
  { id: 'empty', code: 'M3', label: 'Ninguém ao vivo', device: 'mobile' },
  { id: 'connecting', code: 'M·E1', label: 'Conectando', device: 'mobile' },
  { id: 'streamfail', code: 'M·E4', label: 'Falha na transmissão', device: 'mobile' },
  { id: 'camera', code: 'M·A5', label: 'Transmitir a câmera do celular', device: 'mobile' },
  { id: 'reconnecting', code: 'M5', label: 'Assistindo · reconectando', device: 'mobile' },
  { id: 'unwatched', code: 'M·W1', label: 'Parou de assistir (specs/014)', device: 'mobile' },
  // Tela cheia e celular deitado (specs/018).
  { id: 'viewer', code: 'M·F1', label: 'Tela cheia em pé', device: 'mobile' },
  { id: 'viewer-controls', code: 'M·F2', label: 'Tela cheia com controles', device: 'mobile' },
  { id: 'viewer-zoom', code: 'M·F3', label: 'Tela cheia aproximada 2,5×', device: 'mobile' },
  { id: 'viewer-reconnecting', code: 'M·F4', label: 'Tela cheia · reconectando', device: 'mobile' },
  { id: 'viewer-nosound', code: 'M·F5', label: 'Tela cheia sem volume (iPhone)', device: 'mobile' },
  { id: 'landscape', code: 'M·L1', label: 'Sala deitada (2 ao vivo)', device: 'mobile-landscape' },
  { id: 'viewer-landscape', code: 'M·L2', label: 'Tela cheia deitada', device: 'mobile-landscape' },
  { id: 'landscape-camera', code: 'M·L3', label: 'Sala deitada transmitindo a câmera', device: 'mobile-landscape' },
] as const;

export type MobileScene = (typeof MOBILE_SCENES)[number]['id'];

const VIEWER_SCENES: readonly MobileScene[] = ['viewer', 'viewer-controls', 'viewer-zoom', 'viewer-reconnecting', 'viewer-nosound', 'viewer-landscape'];

export function buildMobileScene(scene: MobileScene): RoomView {
  // specs/018: tela cheia parte de "Assistindo (2 ao vivo)" com a Lu aberta; deitada também.
  if (VIEWER_SCENES.includes(scene) || scene === 'landscape') {
    const view = buildMobileScene('watching');
    view.streams.find((s) => s.id === 'lu')!.watching = true;
    if (scene === 'landscape') return view;
    view.fullscreenId = 'lu';
    view.mockViewer = {
      controls: scene === 'viewer-controls' || scene === 'viewer-nosound',
      zoom: scene === 'viewer-zoom' ? clampZoom({ scale: 2.5, x: -300, y: -600 }, { width: 390, height: 844 }) : ZOOM_NONE,
      ...(scene === 'viewer-nosound' ? { volume: false } : {}),
    };
    if (scene === 'viewer-reconnecting') view.link = 'reconnecting';
    return view;
  }
  if (scene === 'landscape-camera') {
    const view = buildMobileScene('camera');
    view.goLiveOpen = false;
    view.myBroadcast = {
      ...view.goLivePrefs,
      viewers: view.participants.length - 1,
      startedAt: Date.now() - 4 * 60_000,
      audioKind: 'device',
      source: 'camera',
      sourceTitle: 'câmera traseira',
    };
    view.participants.forEach((p) => p.you && (p.live = true));
    return view;
  }
  if(scene.startsWith('watch-') && isRoomScene(scene)) { const view=buildRoomScene(scene);view.canShareScreen=false;view.focusedId=view.streams[0]?.id??null;for(const stream of view.streams.slice(1)) {stream.watching=false;stream.watchState='available';}return view; }
  const base: Partial<Record<MobileScene, Parameters<typeof buildRoomScene>[0]>> = {
    watching: 'grid',
    empty: 'empty',
    connecting: 'connecting',
    streamfail: 'streamfail',
    camera: 'empty',
    reconnecting: 'grid',
    unwatched: 'grid',
  };
  const view = buildRoomScene(base[scene] ?? 'empty');
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
