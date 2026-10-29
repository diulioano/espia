// Cenas mockadas da sala — espelham a prop `scene` de design_handoff_espia/design/Room.dc.html.
import type { GoLivePrefs, ParticipantView, RoomView, StreamView, ToastView } from '../lib/types';

export const ROOM_SCENES = [
  { id: 'connecting', code: 'E1', label: 'Conectando' },
  { id: 'empty', code: 'D3', label: 'Ninguém ao vivo' },
  { id: 'copied', code: 'D3', label: 'Convite copiado' },
  { id: 'modal', code: 'D6', label: 'Modal Transmitir' },
  { id: 'denied', code: 'E3', label: 'Permissão negada' },
  { id: 'single', code: 'D4', label: 'Uma transmissão' },
  { id: 'grid', code: 'D5', label: 'Várias (grade)' },
  { id: 'focus', code: 'D5b', label: 'Foco + miniaturas' },
  { id: 'streamfail', code: 'E4', label: 'Falha na transmissão' },
  { id: 'live', code: 'D7', label: 'Você ao vivo' },
  { id: 'crowd', code: 'D5+', label: 'Sala cheia (10, 6 ao vivo)' },
] as const;

export type RoomScene = (typeof ROOM_SCENES)[number]['id'];
export const DEFAULT_ROOM_SCENE: RoomScene = 'single';

export const MOCK_ROOM_ID = 'k7f-q2x';

const PEOPLE: ParticipantView[] = [
  { id: 'rafa', name: 'Rafa', tone: 'deep', you: true, live: false },
  { id: 'lu', name: 'Lu', tone: 'accent', you: false, live: false },
  { id: 'bia', name: 'Bia', tone: 'sage', you: false, live: false },
  { id: 'tomas', name: 'Tomás', tone: 'sand', you: false, live: false },
];

const STREAMS: Record<string, StreamView> = {
  lu: { id: 'lu', name: 'Lu', tone: 'accent', quality: '1080p · 60 fps', caption: 'tela de Lu · Hollow Knight', status: 'ok', muted: false, volume: 0.68 },
  bia: { id: 'bia', name: 'Bia', tone: 'sage', quality: '720p · 30 fps', caption: 'aba de Bia · YouTube', status: 'ok', muted: true, volume: 0.68 },
  tomas: { id: 'tomas', name: 'Tomás', tone: 'sand', quality: '1080p · 30 fps', caption: 'janela de Tomás · VS Code', status: 'loading', muted: true, volume: 0.68 },
};

/** Cena `crowd`: o grupo inteiro (10) na mesma sala. Tons em rodízio, como no servidor. */
const CROWD: ParticipantView[] = (['Caio', 'Duda', 'Gabi', 'Nina', 'Theo', 'Vini'] as const).map((name, i) => ({
  id: name.toLowerCase(),
  name,
  tone: (['deep', 'accent', 'sage', 'sand'] as const)[i % 4],
  you: false,
  live: false,
}));

function crowdStream(id: string): StreamView {
  const p = CROWD.find((c) => c.id === id)!;
  return { id, name: p.name, tone: p.tone, caption: `tela de ${p.name}`, status: 'ok', muted: true, volume: 0.68 };
}

const TOASTS: Partial<Record<RoomScene, Omit<ToastView, 'id'>>> = {
  single: { kind: 'join', text: 'Bia entrou na sala' },
  grid: { kind: 'live', text: 'Tomás começou a transmitir', action: 'Assistir' },
  focus: { kind: 'leave', text: 'Caio saiu da sala' },
  copied: { kind: 'copied', text: 'Link copiado — é só mandar pra galera' },
  live: { kind: 'live', text: 'Você está ao vivo pra sala' },
  crowd: { kind: 'join', text: 'Vini entrou na sala' },
};

const DEFAULT_PREFS: GoLivePrefs = { resolution: 1080, fps: 30, audio: true };

export function isRoomScene(value: string | null): value is RoomScene {
  return ROOM_SCENES.some((s) => s.id === value);
}

export function buildRoomScene(scene: RoomScene): RoomView {
  const liveIds: Record<string, string[]> = {
    single: ['lu'],
    grid: ['lu', 'bia', 'tomas'],
    focus: ['lu', 'bia', 'tomas'],
    streamfail: ['lu', 'bia'],
    live: ['lu'],
    crowd: ['lu', 'bia', 'tomas', 'caio', 'duda', 'gabi'],
  };
  const ids = liveIds[scene] ?? [];
  const streams = ids.map((id) => ({ ...(STREAMS[id] ?? crowdStream(id)) }));
  if (scene === 'crowd') streams.forEach((s) => (s.quality = '720p · 30 fps · ajustado'));
  // Na grade o Tomás ainda está conectando; no foco ele já carregou.
  if (scene === 'focus') streams.find((s) => s.id === 'tomas')!.status = 'ok';
  if (scene === 'streamfail') streams.find((s) => s.id === 'bia')!.status = 'failed';

  const meLive = scene === 'live';
  const toast = TOASTS[scene];

  return {
    roomId: MOCK_ROOM_ID,
    inviteUrl: `${location.origin}/s/${MOCK_ROOM_ID}`,
    status: scene === 'connecting' ? 'connecting' : 'ready',
    participants: (scene === 'crowd' ? [...PEOPLE, ...CROWD] : PEOPLE).map((p) => ({
      ...p,
      live: ids.includes(p.id) || (p.you && meLive),
    })),
    capacity: 10,
    streams,
    focusedId: scene === 'focus' || scene === 'streamfail' ? 'bia' : null,
    myBroadcast: meLive
      ? { resolution: 1080, fps: 60, audio: true, viewers: 3, startedAt: Date.now() - (12 * 60 + 4) * 1000 }
      : null,
    copied: scene === 'copied',
    goLiveOpen: scene === 'modal',
    goLivePrefs: { ...DEFAULT_PREFS },
    permissionDenied: scene === 'denied',
    toasts: toast ? [{ id: `scene-${scene}`, ...toast }] : [],
    canShare: true,
  };
}
