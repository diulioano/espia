// Cenas mockadas da sala — espelham a prop `scene` de design_handoff_espia/design/Room.dc.html.
import type { CameraView, CaptureSource, GoLivePrefs, ParticipantView, PickerView, RoomView, StreamView, ToastView } from '../lib/types';

export const ROOM_SCENES = [
  { id: 'connecting', code: 'E1', label: 'Conectando' },
  { id: 'empty', code: 'D3', label: 'Ninguém ao vivo' },
  { id: 'copied', code: 'D3', label: 'Convite copiado' },
  { id: 'modal', code: 'A1', label: 'Seletor · Janelas' },
  { id: 'modalscreen', code: 'A2', label: 'Seletor · Telas inteiras' },
  { id: 'modalmute', code: 'A3', label: 'Seletor · Sem áudio' },
  { id: 'pickerwindows', code: 'A1·app', label: 'App · grade de janelas' },
  { id: 'pickerscreens', code: 'A2·app', label: 'App · telas inteiras com som' },
  { id: 'pickerempty', code: 'A1·app', label: 'App · nenhuma janela' },
  { id: 'pickerfallback', code: 'A1·app', label: 'App · sem ponte (seletor do Windows)' },
  { id: 'pickercamperm', code: 'A5', label: 'Câmeras · pedir permissão' },
  { id: 'pickercams', code: 'A5', label: 'Câmeras · lista com prévias' },
  { id: 'pickercammic', code: 'A5', label: 'Câmeras · placa com som de jogo' },
  { id: 'pickercamnone', code: 'A5', label: 'Câmeras · nenhuma encontrada' },
  { id: 'pickercamdenied', code: 'A5', label: 'Câmeras · bloqueada' },
  { id: 'denied', code: 'E3', label: 'Permissão negada' },
  { id: 'single', code: 'D4', label: 'Uma transmissão' },
  { id: 'grid', code: 'D5', label: 'Várias (grade)' },
  { id: 'focus', code: 'D5b', label: 'Foco + miniaturas' },
  { id: 'streamfail', code: 'E4', label: 'Falha na transmissão' },
  { id: 'live', code: 'D7', label: 'Você ao vivo' },
  { id: 'livescreen', code: 'D7', label: 'Ao vivo · tela inteira sem som' },
  { id: 'livepc', code: 'D7·app', label: 'Ao vivo · tela inteira com som do PC (app)' },
  { id: 'livecam', code: 'D7·cam', label: 'Ao vivo · câmera (Lu também de câmera)' },
  { id: 'crowd', code: 'D5+', label: 'Sala cheia (10, 6 ao vivo)' },
  // Queda não é sair (specs/011).
  { id: 'reconnecting', code: 'Q1', label: 'Reconectando… (você ao vivo)' },
  { id: 'back', code: 'Q2', label: 'De volta!' },
  { id: 'reconnectfail', code: 'Q3', label: 'Não deu pra voltar' },
  // Sala fixa (specs/011).
  { id: 'fixed', code: 'S·', label: 'Sala fixa vazia (dono)' },
  { id: 'created', code: 'S1', label: 'Sala fixa recém-criada' },
  { id: 'ownermenu', code: 'S2', label: 'Menu do dono' },
  { id: 'rename', code: 'S3', label: 'Renomear' },
  { id: 'delete', code: 'S4', label: 'Apagar · com gente na sala' },
  { id: 'guest', code: 'S5', label: 'Sala fixa (não é o dono)' },
  // Assistir quando quiser (specs/014).
  { id: 'unwatched', code: 'W1', label: 'Parou de assistir uma tela' },
  { id: 'unwatched-all', code: 'W2', label: 'Parou de assistir todas' },
  { id: 'unwatched-focus', code: 'W3', label: 'Foco com miniatura sem assistir' },
  { id: 'self-stage', code: 'W4', label: 'Sua tela grande no palco' },
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
  lu: { id: 'lu', name: 'Lu', tone: 'accent', quality: '1080p · 60 fps · com som', caption: 'tela de Lu · Hollow Knight', label: 'Hollow Knight', status: 'ok', muted: false, volume: 0.68 },
  bia: { id: 'bia', name: 'Bia', tone: 'sage', quality: '720p · 30 fps · com som', caption: 'aba de Bia · YouTube', label: 'Aba do navegador', status: 'ok', muted: true, volume: 0.68 },
  tomas: { id: 'tomas', name: 'Tomás', tone: 'sand', quality: '1080p · 30 fps · sem som', caption: 'janela de Tomás · VS Code', label: 'Visual Studio Code', status: 'loading', muted: true, volume: 0.68 },
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
  // Um texto comprido (cortado) e um de versão antiga (sem texto), specs/012.
  const label = id === 'caio' ? 'Microsoft Flight Simulator 2024 — Edição Premium Deluxe' : id === 'duda' ? undefined : 'Tela inteira';
  return { id, name: p.name, tone: p.tone, quality: '1080p · 30 fps · sem som', caption: `tela de ${p.name}`, label, status: 'ok', muted: true, volume: 0.68 };
}

const TOASTS: Partial<Record<RoomScene, Omit<ToastView, 'id'>>> = {
  single: { kind: 'join', text: 'Bia entrou na sala' },
  grid: { kind: 'live', text: 'Tomás começou a transmitir', action: 'Assistir' },
  focus: { kind: 'leave', text: 'Caio saiu da sala' },
  copied: { kind: 'copied', text: 'Link copiado — é só mandar pra galera' },
  live: { kind: 'live', text: 'Você está ao vivo pra sala' },
  livescreen: { kind: 'live', text: 'Tela inteira vai sem som. Pra levar o som, transmita a janela do jogo ou uma aba.' },
  crowd: { kind: 'join', text: 'Vini entrou na sala' },
};

const DEFAULT_PREFS: GoLivePrefs = { resolution: 1080, fps: 30, audio: true, surface: 'window' };

// ---------- Seletor do app desktop (specs/005, tela-esperada.png) ----------

/** Miniatura falsa: listras na cor do programa, com o rótulo "janela"/"tela" (como no design). */
function fakeThumb(a: string, b: string, label: string): string {
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="320" height="180">` +
    `<defs><pattern id="p" width="36" height="36" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">` +
    `<rect width="18" height="36" fill="${a}"/><rect x="18" width="18" height="36" fill="${b}"/></pattern></defs>` +
    `<rect width="320" height="180" fill="url(#p)"/>` +
    `<text x="160" y="94" fill="#f5ead8" fill-opacity=".55" font-family="monospace" font-size="13" text-anchor="middle">${label}</text></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

/** Ícone falso: inicial numa pastilha (como no design). */
function fakeIcon(letter: string, bg: string, fg: string): string {
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" rx="8" fill="${bg}"/>` +
    `<text x="16" y="22" fill="${fg}" font-family="Georgia,serif" font-weight="700" font-size="17" text-anchor="middle">${letter}</text></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

const WARM = ['#3a2416', '#33200f'];
const GREEN = ['#25301c', '#212a18'];
const GRAY = ['#2c2925', '#282521'];
const RED = ['#3d2219', '#361e16'];

export const MOCK_SOURCES: CaptureSource[] = [
  { id: 'w:1', kind: 'window', title: 'Hollow Knight', app: 'hollow_knight.exe', icon: fakeIcon('H', '#ffc6a5', '#643312'), thumb: fakeThumb(WARM[0], WARM[1], 'janela'), playing: true, primary: false },
  { id: 'w:2', kind: 'window', title: 'YouTube — Firefox', app: 'Firefox', icon: fakeIcon('Y', '#ccdbb2', '#3d472b'), thumb: fakeThumb(GREEN[0], GREEN[1], 'janela'), playing: true, primary: false },
  { id: 'w:3', kind: 'window', title: 'espia-app — Visual Studio Code', app: 'Code', icon: fakeIcon('e', '#dcd3c4', '#474238'), thumb: fakeThumb(GRAY[0], GRAY[1], 'janela'), playing: false, primary: false },
  { id: 'w:4', kind: 'window', title: 'Spotify', app: 'Spotify', icon: fakeIcon('S', '#ccdbb2', '#3d472b'), thumb: fakeThumb(GREEN[0], GREEN[1], 'janela'), playing: true, primary: false },
  { id: 'w:5', kind: 'window', title: 'Planilha do campeonato', app: 'LibreOffice Calc', icon: fakeIcon('P', '#f6a06b', '#402310'), thumb: fakeThumb(RED[0], RED[1], 'janela'), playing: false, primary: false },
  { id: 'w:6', kind: 'window', title: 'Downloads', app: 'Explorador de arquivos', icon: fakeIcon('D', '#dcd3c4', '#474238'), thumb: null, playing: false, primary: false },
  { id: 'm:1', kind: 'monitor', title: 'Tela 1', app: 'Principal', icon: null, thumb: fakeThumb(GRAY[0], GRAY[1], 'tela'), playing: false, primary: true },
  { id: 'm:2', kind: 'monitor', title: 'Tela 2', app: '', icon: null, thumb: fakeThumb(WARM[0], WARM[1], 'tela'), playing: false, primary: false },
];

const BROWSER_PICKER: PickerView = { mode: 'browser', sources: [], loading: false, selectedId: null };

function pickerFor(scene: RoomScene): PickerView {
  if (scene === 'pickerwindows') return { mode: 'native', sources: MOCK_SOURCES, loading: false, selectedId: 'w:1' };
  if (scene === 'pickerscreens') return { mode: 'native', sources: MOCK_SOURCES, loading: false, selectedId: 'm:1' };
  if (scene === 'pickerempty') return { mode: 'native', sources: MOCK_SOURCES.filter((s) => s.kind === 'monitor'), loading: false, selectedId: null };
  return BROWSER_PICKER;
}

const PICKER_SCENES: RoomScene[] = [
  'modal',
  'modalscreen',
  'modalmute',
  'pickerwindows',
  'pickerscreens',
  'pickerempty',
  'pickerfallback',
  'pickercamperm',
  'pickercams',
  'pickercammic',
  'pickercamnone',
  'pickercamdenied',
];
const CAMERA_SCENES: RoomScene[] = ['pickercamperm', 'pickercams', 'pickercammic', 'pickercamnone', 'pickercamdenied'];

// ---------- Aba Câmeras (specs/006) ----------

/** Prévia falsa animada (canvas): nada de câmera de verdade nos mocks. */
export function fakeCameraStream(bg: string, label: string): MediaStream | null {
  if (typeof document === 'undefined') return null;
  const canvas = document.createElement('canvas');
  canvas.width = 320;
  canvas.height = 180;
  const ctx = canvas.getContext('2d');
  if (!ctx || !('captureStream' in canvas)) return null;
  let t = 0;
  const draw = () => {
    t += 1;
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, 320, 180);
    ctx.fillStyle = 'rgba(245,234,216,0.18)';
    ctx.beginPath();
    ctx.arc(160 + Math.sin(t / 20) * 40, 82, 34, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = 'rgba(245,234,216,0.6)';
    ctx.font = '13px monospace';
    ctx.textAlign = 'center';
    ctx.fillText(label, 160, 160);
  };
  draw();
  setInterval(draw, 66);
  return canvas.captureStream(15);
}

export const MOCK_CAMERA_DEVICES = [
  { id: 'cam1', label: 'HD Pro Webcam C920', kind: 'camera' as const, bg: '#2c3a2a' },
  { id: 'cam2', label: 'Elgato HD60 X', kind: 'capture' as const, bg: '#3a2416' },
  { id: 'cam3', label: 'OBS Virtual Camera', kind: 'camera' as const, bg: '#2c2925' },
];
export const MOCK_AUDIO_INPUTS = [
  { id: 'mic1', label: 'Microfone (HD Pro Webcam C920)' },
  { id: 'mic2', label: 'Digital Audio Interface (HD60 X)' },
  { id: 'mic3', label: 'Headset Microphone (HyperX Cloud II)' },
];

export function mockCameraDevices(): CameraView['devices'] {
  return MOCK_CAMERA_DEVICES.map((d) => ({ id: d.id, label: d.label, kind: d.kind, preview: fakeCameraStream(d.bg, d.kind === 'capture' ? 'console' : 'câmera'), busy: false }));
}

const NO_CAMERA: CameraView = { permission: 'unknown', devices: [], audios: [], videoId: null, audioId: null, audioRole: 'voice', mobile: false, facing: 'user' };

function cameraFor(scene: RoomScene): CameraView {
  if (scene === 'pickercamperm') return { ...NO_CAMERA, permission: 'prompt' };
  if (scene === 'pickercamdenied') return { ...NO_CAMERA, permission: 'denied' };
  if (scene === 'pickercamnone') return { ...NO_CAMERA, permission: 'granted', audios: MOCK_AUDIO_INPUTS };
  if (scene === 'pickercams') return { ...NO_CAMERA, permission: 'granted', devices: mockCameraDevices(), audios: MOCK_AUDIO_INPUTS, videoId: 'cam1', audioId: 'mic1', audioRole: 'voice' };
  if (scene === 'pickercammic') return { ...NO_CAMERA, permission: 'granted', devices: mockCameraDevices(), audios: MOCK_AUDIO_INPUTS, videoId: 'cam2', audioId: 'mic2', audioRole: 'game' };
  return NO_CAMERA;
}

export function isRoomScene(value: string | null): value is RoomScene {
  return ROOM_SCENES.some((s) => s.id === value);
}

export function buildRoomScene(scene: RoomScene): RoomView {
  // Queda (specs/011 Q1–Q3): a mesma sala de "Você ao vivo", com o aviso por cima.
  // Sala fixa: a mesma sala, com o nome no lugar do título.
  if (scene === 'fixed' || scene === 'created' || scene === 'guest' || scene === 'ownermenu' || scene === 'rename' || scene === 'delete') {
    const view = buildRoomScene(scene === 'guest' ? 'single' : scene === 'created' ? 'copied' : 'empty');
    view.room = { kind: 'fixed', name: 'CS da galera', own: scene !== 'guest' };
    if (scene === 'ownermenu') view.ownerMenu = true;
    if (scene === 'rename') view.nameDialog = { mode: 'rename', value: 'CS da galera 2', busy: false, error: null };
    if (scene === 'delete') view.deleteDialog = { name: 'CS da galera', people: 3, busy: false, error: null };
    if (scene === 'created') view.toasts = [{ id: 'scene-created', kind: 'copied', text: 'Sala criada — fixe o link no grupo' }];
    return view;
  }
  if (scene === 'unwatched' || scene === 'unwatched-all' || scene === 'unwatched-focus') {
    const view = buildRoomScene(scene === 'unwatched-focus' ? 'focus' : 'grid');
    view.toasts = [];
    for (const s of view.streams) {
      if (scene === 'unwatched-all' || s.id === (scene === 'unwatched' ? 'bia' : 'tomas')) s.watching = false;
    }
    if (scene === 'unwatched-focus') view.streams.find((s) => s.id === 'tomas')!.status = 'ok';
    return view;
  }
  if (scene === 'self-stage') {
    // Clicou na prévia: a sua tela grande, em foco; a de Lu vai pra faixa.
    const view = buildRoomScene('live');
    view.toasts = [];
    view.myBroadcast!.onStage = true;
    view.streams.push({ id: 'rafa', name: 'Você', tone: 'deep', caption: 'sua tela · Celeste', label: view.myBroadcast!.label, status: 'ok', muted: true, volume: 0, you: true });
    view.focusedId = 'rafa';
    return view;
  }
  if (scene === 'reconnecting' || scene === 'back' || scene === 'reconnectfail') {
    const view = buildRoomScene('live');
    view.link = scene === 'back' ? 'ok' : scene === 'reconnecting' ? 'reconnecting' : 'failed';
    view.toasts = scene === 'back' ? [{ id: 'scene-back', kind: 'copied', text: 'De volta!' }] : [];
    return view;
  }
  const liveIds: Record<string, string[]> = {
    single: ['lu'],
    grid: ['lu', 'bia', 'tomas'],
    focus: ['lu', 'bia', 'tomas'],
    streamfail: ['lu', 'bia'],
    live: ['lu'],
    livescreen: ['lu'],
    livepc: ['lu'],
    livecam: ['lu'],
    crowd: ['lu', 'bia', 'tomas', 'caio', 'duda', 'gabi'],
  };
  const ids = liveIds[scene] ?? [];
  const streams = ids.map((id) => ({ ...(STREAMS[id] ?? crowdStream(id)) }));
  if (scene === 'crowd') streams.forEach((s) => (s.quality = '720p · 30 fps · ajustado · sem som'));
  // Na grade o Tomás ainda está conectando; no foco ele já carregou.
  if (scene === 'focus') streams.find((s) => s.id === 'tomas')!.status = 'ok';
  if (scene === 'streamfail') streams.find((s) => s.id === 'bia')!.status = 'failed';

  const meLive = scene === 'live' || scene === 'livescreen' || scene === 'livepc' || scene === 'livecam';
  if (scene === 'livecam') streams.forEach((s) => ((s.source = 'camera'), (s.label = 'Câmera · Logitech Brio')));
  const myLabel = { live: 'Celeste', livecam: 'Câmera · HD Pro Webcam C920', livepc: 'Tela inteira', livescreen: 'Tela inteira' }[scene as string];
  const toast = TOASTS[scene];

  return {
    roomId: MOCK_ROOM_ID,
    inviteUrl: `${location.origin}/s/${MOCK_ROOM_ID}`,
    status: scene === 'connecting' ? 'connecting' : 'ready',
    participants: (scene === 'crowd' ? [...PEOPLE, ...CROWD] : PEOPLE).map((p) => ({
      ...p,
      live: ids.includes(p.id) || (p.you && meLive),
      liveLabel: p.you ? (meLive ? myLabel : undefined) : streams.find((s) => s.id === p.id)?.label,
    })),
    capacity: 10,
    streams,
    focusedId: scene === 'focus' || scene === 'streamfail' ? 'bia' : null,
    myBroadcast: meLive
      ? {
          ...DEFAULT_PREFS,
          resolution: 1080,
          fps: 60,
          viewers: 3,
          startedAt: Date.now() - (12 * 60 + 4) * 1000,
          label: myLabel,
          snapshot: fakeSnapshot(),
          ...(scene === 'live'
            ? { audioKind: 'window' as const }
            : scene === 'livecam'
              ? { surface: 'camera' as const, audioKind: 'device' as const, source: 'camera' as const, sourceTitle: 'HD Pro Webcam C920' }
            : scene === 'livepc'
              ? { surface: 'monitor' as const, audioKind: 'pc' as const, sourceTitle: 'Tela 1' }
              : { surface: 'monitor' as const, audioKind: 'none' as const, audioReason: 'monitor' as const }),
        }
      : null,
    copied: scene === 'copied',
    goLiveOpen: PICKER_SCENES.includes(scene),
    goLivePrefs: {
      ...DEFAULT_PREFS,
      ...(scene === 'pickerwindows' ? { fps: 60 } : {}),
      surface: CAMERA_SCENES.includes(scene) ? 'camera' : scene === 'modalscreen' || scene === 'pickerscreens' ? 'monitor' : 'window',
      audio: scene !== 'modalmute',
    },
    picker: pickerFor(scene),
    camera: cameraFor(scene),
    permissionDenied: scene === 'denied',
    toasts: toast ? [{ id: `scene-${scene}`, ...toast }] : [],
    canShare: true,
    canShareScreen: true,
    link: 'ok',
    room: { kind: 'quick' },
  };
}

/** Prévia parada falsa (specs/014 US3): listras com "sua tela · quadro parado". Fora do navegador: null. */
function fakeSnapshot(): ImageBitmap | null {
  if (typeof OffscreenCanvas === 'undefined') return null;
  const c = new OffscreenCanvas(320, 180);
  const g = c.getContext('2d');
  if (!g) return null;
  for (let x = -180; x < 320; x += 36) {
    g.fillStyle = (x / 36) % 2 ? '#40261a' : '#381f14';
    g.beginPath();
    g.moveTo(x, 180);
    g.lineTo(x + 18, 180);
    g.lineTo(x + 198, 0);
    g.lineTo(x + 180, 0);
    g.fill();
  }
  g.fillStyle = 'rgba(245, 234, 216, 0.6)';
  g.font = '13px monospace';
  g.textAlign = 'center';
  g.fillText('sua tela · quadro parado', 160, 95);
  return c.transferToImageBitmap();
}
