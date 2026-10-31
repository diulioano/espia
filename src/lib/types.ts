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
  /** O que essa pessoa está transmitindo (specs/012). */
  liveLabel?: string;
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
  /**
   * Só o som da pessoa (specs/014): toca num elemento próprio, então continua quando o vídeo some
   * (parou de assistir, página escondida, vídeo pausado).
   */
  audioStream?: MediaStream;
  /** O que está sendo transmitido (specs/012): "Hollow Knight", "Aba do navegador"… */
  label?: string;
  /** Câmera/placa (specs/006): o tile diz "câmera de …". */
  source?: LiveSource;
  /** false = parou de assistir (specs/014 US1): cartão "Assistir", sem vídeo. Ausente = assistindo. */
  watching?: boolean;
  /** A sua própria transmissão no palco (specs/014 US3): sempre muda, sem controles de som. */
  you?: boolean;
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
  /** Título da janela/tela escolhida no app desktop, ou o nome da câmera. Só pra você; não vai pra sala. */
  sourceTitle?: string;
  /** O que a sala vê como descrição da sua transmissão (specs/012). */
  label?: string;
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
  /**
   * Ligação com o servidor (specs/011 US1). Queda não é sair: 'reconnecting' mantém a sala na tela;
   * 'failed' depois de 60 s sem voltar (continua tentando em segundo plano).
   */
  link: 'ok' | 'reconnecting' | 'failed';
  /** Sala rápida, ou fixa com nome (specs/011). `own` = este aparelho criou (tem a chave do dono). */
  room: { kind: 'quick' } | { kind: 'fixed'; name: string; own: boolean };
  /** Dono (specs/011 S2–S4): menu "⋯" ao lado do nome, renomear e apagar. */
  ownerMenu?: boolean;
  nameDialog?: NameDialogView;
  deleteDialog?: DeleteDialogView;
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
  /** Convite do app desktop (specs/008): oferta, ou a dica depois de baixar. Ausente = não mostra. */
  appInvite?: 'offer' | 'hint';
  /** Botão "Baixar o app" no topo (specs/013): só no site, no Windows, mesmo depois de "Agora não". */
  download?: boolean;
  /** Suas salas (specs/011 US3). null = nenhuma neste aparelho → a ilustração de sempre. */
  rooms?: SavedRoomsView | null;
  /** Modal "Criar sala fixa" aberto (specs/011 F7) ou "Renomear sala" de uma linha de Suas salas. */
  fixedDialog?: NameDialogView;
  /** "Apagar sala" pelo "⋯" de Suas salas (dono). */
  deleteDialog?: DeleteDialogView;
}

/** Uma linha de Suas salas (specs/011, SavedRoom.dc.html). Só números, nunca apelidos. */
export interface SavedRoomView {
  code: string;
  name: string;
  /** Este aparelho criou a sala (tem a chave do dono). */
  own: boolean;
  tone: Tone;
  /** loading = números ainda não chegaram; gone = apagada, expirada ou não existe mais. */
  state: 'loading' | 'ok' | 'gone';
  people: number;
  streaming: number;
  /** "usada ontem", "há 3 dias"… */
  recent: string;
  menuOpen?: boolean;
}

export interface SavedRoomsView {
  /** Com 4+ salas e gente em umas e vazias em outras: "Com gente agora" / "Usadas recentemente". */
  groups: { label: string | null; rooms: SavedRoomView[] }[];
  /** "neste aparelho" ou "5 salas neste aparelho". */
  note: string;
}

/** Confirmar "Apagar sala" (specs/011 S4). */
export interface DeleteDialogView {
  name: string;
  /** Pessoas na sala agora (fora você): o aviso "Elas saem da sala". */
  people: number;
  busy: boolean;
  error: string | null;
}

/** Modal de nome da sala fixa: criar (F7) ou renomear (S3). */
export interface NameDialogView {
  mode: 'create' | 'rename';
  value: string;
  busy: boolean;
  error: string | null;
}

export interface HomeActions {
  createRoom(nickname: string): void;
  openLink(text: string): void;
  clearLinkError(): void;
  /** Clicou em "Baixar o app" (o link baixa sozinho; aqui só troca para a dica). */
  downloadApp(): void;
  dismissAppInvite(): void;
  /** Sala fixa (specs/011). */
  openFixedDialog(): void;
  closeFixedDialog(): void;
  createFixedRoom(nickname: string, name: string): void;
  /** Suas salas (specs/011 US3). */
  enterSaved(code: string, nickname: string): void;
  forgetSaved(code: string): void;
  /** Abre o "⋯" de uma linha (null fecha). */
  toggleSavedMenu(code: string | null): void;
  /** Dono (US4): renomear/apagar pelo "⋯" de Suas salas. */
  renameSaved(code: string): void;
  submitRename(name: string): void;
  deleteSaved(code: string): void;
  confirmDelete(): void;
  closeDeleteDialog(): void;
}

export interface JoinPreviewPerson {
  name: string;
  tone: Tone;
  live: boolean;
}

export interface JoinView {
  roomId: string;
  /** "Não existe" virou a NotFoundPage (specs/011). */
  status: 'ready' | 'full';
  /** null = não deu pra buscar a sala antes de entrar (esconde a pilha de avatares). */
  people: JoinPreviewPerson[] | null;
  capacity: number;
  nickname: string;
  canShare: boolean;
  /** Sala fixa (specs/011 F8): selo "Sala fixa" e o nome no título. */
  fixedName?: string;
  /** Você abriu esta sala em outra aba (a mesma pessoa não aparece duas vezes): esta cedeu o lugar. */
  replaced?: boolean;
}

/** "Sala não encontrada" (specs/011 N1–N4). */
export interface NotFoundView {
  /** quick = sala rápida que acabou ou link errado; kicked = apagada com você dentro. */
  reason: 'quick' | 'deleted' | 'kicked' | 'expired';
  /** Nome conhecido neste aparelho (Suas salas). */
  name: string | null;
  /** Está em Suas salas: a ação secundária vira "Tirar de Suas salas". */
  saved: boolean;
}

export interface NotFoundActions {
  /** Logo (specs/013). */
  goHome(): void;
  createRoom(): void;
  pasteOtherLink(): void;
  forget(): void;
}

export interface JoinActions {
  /** Logo (specs/013). */
  goHome(): void;
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
  /** Parar de assistir (false) ou voltar a assistir (true) a tela de alguém (specs/014 US1). */
  watch(id: string, on: boolean): void;
  /** Uma tela entrou (id) ou saiu (null) do picture-in-picture: continua chegando fora de vista (specs/014 US2). */
  pip(id: string | null): void;
  toggleMute(id: string): void;
  setVolume(id: string, volume: number): void;
  retry(id: string): void;
  toastAction(id: string): void;
  dismissToast(id: string): void;
  /** "Tentar de novo" na tela "Não deu pra voltar pra sala" (specs/011). */
  retryConnection(): void;
  /** Logo (specs/013): sai da sala e vai pro início; transmitindo, pergunta antes. */
  goHome(): void;
  /** Dono da sala fixa (specs/011 US4). */
  toggleOwnerMenu(open: boolean): void;
  openRename(): void;
  submitRename(name: string): void;
  openDelete(): void;
  confirmDelete(): void;
  closeDialogs(): void;
  leave(): void;
}
