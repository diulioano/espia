// View-models: contrato entre a camada visual (src/ui) e quem fornece os dados (mocks ou lógica real).
import type { AudioKind, LiveSource } from '../shared/protocol';

export type { AudioKind, LiveSource };

export type Tone = 'accent' | 'sage' | 'sand' | 'deep';
export type TileStatus = 'ok' | 'loading' | 'failed';
export type TileSize = 'stage' | 'grid' | 'thumb';
export type ToastKind = 'join' | 'leave' | 'live' | 'copied' | 'error';
export type Resolution = 720 | 1080;
export type Fps = 30 | 60;
/** Aba do seletor: janela/tela (specs/004/005) ou câmera/placa (specs/006). */
export type ShareSurface = 'window' | 'monitor' | 'camera';
/** Aba de janela/tela: o que uma CaptureSource do app pode ser. */
export type ScreenSurface = 'window' | 'monitor';

export interface ParticipantView {
  id: string;
  name: string;
  tone: Tone;
  you: boolean;
  live: boolean;
}

export interface StreamView {
  id: string;
  name: string;
  tone: Tone;
  /** Ex.: "1080p · 60 fps". */
  quality?: string;
  /** Texto do placeholder quando não há vídeo (mock). */
  caption?: string;
  status: TileStatus;
  muted: boolean;
  /** 0..1 */
  volume: number;
  stream?: MediaStream;
  /** Câmera/placa (specs/006): o tile diz "câmera de …". */
  source?: LiveSource;
}

export interface ToastView {
  id: string;
  kind: ToastKind;
  text: string;
  action?: string;
}

export interface GoLivePrefs {
  resolution: Resolution;
  fps: Fps;
  /** "Compartilhar áudio" ligado. No site, tela inteira vai sempre sem som; no app, vai o PC menos o Discord. */
  audio: boolean;
  surface: ShareSurface;
  /** Última fonte transmitida no app (specs/005 US5): "<app>|<title>" ou "m:<title>". */
  lastSourceKey?: string;
  /** Última câmera/entrada de som/perfil (specs/006 FR-609), pelo nome. */
  lastVideoLabel?: string;
  lastAudioLabel?: string;
  lastAudioRole?: 'voice' | 'game';
}

/** O que a aba Câmeras do seletor pode pedir (specs/006). */
export interface CameraActions {
  allow(): void;
  select(id: string): void;
  selectAudio(id: string | null): void;
  setRole(role: 'voice' | 'game'): void;
  setFacing(facing: 'user' | 'environment'): void;
}

/** Aba Câmeras do seletor (specs/006 data-model.md). */
export interface CameraView {
  permission: 'unknown' | 'prompt' | 'granted' | 'denied';
  devices: { id: string; label: string; kind: 'camera' | 'capture'; preview: MediaStream | null; busy: boolean }[];
  audios: { id: string; label: string }[];
  videoId: string | null;
  /** null = sem som. */
  audioId: string | null;
  audioRole: 'voice' | 'game';
  /** Celular: só câmera, frontal/traseira, 720p30. */
  mobile: boolean;
  facing: 'user' | 'environment';
  /** A câmera não abriu: 'busy' = em uso por outro programa; 'notfound' = o Windows não achou. */
  error?: 'busy' | 'notfound' | 'other';
}

/** Janela ou monitor que o app desktop pode transmitir (specs/005 data-model.md). */
export interface CaptureSource {
  /** "w:<hwnd>" ou "m:<hmonitor>". */
  id: string;
  kind: ScreenSurface;
  title: string;
  app: string;
  /** data:image/png 32×32. */
  icon: string | null;
  /** data:image/jpeg ≤ 320×180; null = minimizada ou sem imagem (mostra o ícone). */
  thumb: string | null;
  /** Selo "tocando som". Sempre false em monitor. */
  playing: boolean;
  primary: boolean;
}

/** Estado do seletor: 'native' = grade do app desktop; 'browser' = seletor do navegador (specs/004). */
export interface PickerView {
  mode: 'native' | 'browser';
  /** Todas as fontes (as duas abas); a grade filtra pela aba. */
  sources: CaptureSource[];
  /** Ainda buscando a primeira lista. */
  loading: boolean;
  selectedId: string | null;
}

export interface MyBroadcastView extends GoLivePrefs {
  startedAt: number;
  /** Qualidade baixada sozinha porque tem muita gente assistindo. */
  adjusted?: boolean;
  viewers: number;
  stream?: MediaStream;
  /** O som que está saindo de verdade (specs/002). */
  audioKind: AudioKind;
  /** Sem som porque é tela inteira. */
  audioReason?: 'monitor';
  /** Título da janela/tela escolhida no app desktop, ou o nome da câmera. */
  sourceTitle?: string;
  source?: LiveSource;
}

export interface RoomView {
  roomId: string;
  inviteUrl: string;
  status: 'connecting' | 'ready';
  /** Você primeiro. */
  participants: ParticipantView[];
  capacity: number;
  /** Transmissões dos outros (a sua fica só na sidebar). */
  streams: StreamView[];
  focusedId: string | null;
  myBroadcast: MyBroadcastView | null;
  copied: boolean;
  goLiveOpen: boolean;
  goLivePrefs: GoLivePrefs;
  picker: PickerView;
  camera: CameraView;
  permissionDenied: boolean;
  toasts: ToastView[];
  /** Pode transmitir algo (tela ou câmera). */
  canShare: boolean;
  /** Pode transmitir tela (só desktop). Sem isso, o seletor mostra só câmeras. */
  canShareScreen: boolean;
}

export interface HomeView {
  nickname: string;
  link: string;
  /** Ex.: "Esse link não parece de uma sala." */
  linkError: string | null;
  /** Falha ao criar a sala (rede, limite). */
  createError?: string | null;
  /** Esperando o servidor criar a sala. */
  creating?: boolean;
}

export interface HomeActions {
  createRoom(nickname: string): void;
  openLink(text: string): void;
  clearLinkError(): void;
}

export interface JoinPreviewPerson {
  name: string;
  tone: Tone;
  live: boolean;
}

export interface JoinView {
  roomId: string;
  status: 'ready' | 'notfound' | 'full';
  /** null = não deu pra buscar a sala antes de entrar (esconde a pilha de avatares). */
  people: JoinPreviewPerson[] | null;
  capacity: number;
  nickname: string;
  canShare: boolean;
}

export interface JoinActions {
  join(nickname: string): void;
  createRoom(): void;
  pasteOtherLink(): void;
}

/** Ações que a sala dispara; quem fornece o RoomView decide o que fazer. */
export interface RoomActions {
  copyInvite(): void;
  openGoLive(): void;
  closeGoLive(): void;
  startGoLive(prefs: GoLivePrefs): void;
  /** Escolheu um cartão na grade do app desktop (specs/005). */
  selectSource(id: string): void;
  /** Aba Câmeras (specs/006). */
  allowCamera(): void;
  selectCamera(id: string): void;
  selectAudio(id: string | null): void;
  setAudioRole(role: 'voice' | 'game'): void;
  setFacing(facing: 'user' | 'environment'): void;
  /** Trocou de aba no seletor (liga/desliga as prévias de câmera). */
  setSurface(surface: ShareSurface): void;
  stopGoLive(): void;
  dismissPermission(): void;
  focus(id: string | null): void;
  toggleMute(id: string): void;
  setVolume(id: string, volume: number): void;
  retry(id: string): void;
  toastAction(id: string): void;
  dismissToast(id: string): void;
  leave(): void;
}
