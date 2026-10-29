// View-models: contrato entre a camada visual (src/ui) e quem fornece os dados (mocks ou lógica real).

export type Tone = 'accent' | 'sage' | 'sand' | 'deep';
export type TileStatus = 'ok' | 'loading' | 'failed';
export type TileSize = 'stage' | 'grid' | 'thumb';
export type ToastKind = 'join' | 'leave' | 'live' | 'copied' | 'error';
export type Resolution = 720 | 1080;
export type Fps = 30 | 60;

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
  audio: boolean;
}

export interface MyBroadcastView extends GoLivePrefs {
  startedAt: number;
  /** Qualidade baixada sozinha porque tem muita gente assistindo. */
  adjusted?: boolean;
  viewers: number;
  stream?: MediaStream;
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
  permissionDenied: boolean;
  toasts: ToastView[];
  canShare: boolean;
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
