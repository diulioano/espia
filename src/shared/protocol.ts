// Mensagens do WebSocket de sinalização (contracts/signaling.md). Usado pelo navegador e pelo Worker:
// sem DOM e sem APIs do Workers aqui.

export type Tone = 'accent' | 'sage' | 'sand' | 'deep';
export const TONES: readonly Tone[] = ['deep', 'accent', 'sage', 'sand'];

export const CAPACITY = 10;
export const NICKNAME_MAX = 24;
export const MAX_MESSAGE_BYTES = 64 * 1024;

/** Mensagem de keep-alive. Precisa ser exatamente esta string: o DO responde sem acordar. */
export const PING = '{"t":"ping"}';
export const PONG = '{"t":"pong"}';

/**
 * Som que sai na transmissão (specs/002): da aba, da janela ou nenhum. 'pc' (specs/005, só no app
 * desktop): o PC inteiro **menos o Discord**. 'device' (specs/006): a entrada de som escolhida junto da
 * câmera/placa. Nunca o som do sistema com o Discord junto.
 */
export type AudioKind = 'tab' | 'window' | 'pc' | 'device' | 'none';
export const AUDIO_KINDS: readonly AudioKind[] = ['tab', 'window', 'pc', 'device', 'none'];

/** O que está sendo transmitido (specs/006): tela/janela/aba ou câmera/placa de captura. */
export type LiveSource = 'screen' | 'camera';
export const LIVE_SOURCES: readonly LiveSource[] = ['screen', 'camera'];

export interface LiveInfo {
  resolution: 720 | 1080;
  fps: 30 | 60;
  audio: AudioKind;
  source: LiveSource;
  startedAt: number;
}

export interface Peer {
  id: string;
  nickname: string;
  tone: Tone;
  live: LiveInfo | null;
}

/** Ice servers no formato do RTCIceServer, sem depender dos tipos do DOM. */
export interface IceServer {
  urls: string | string[];
  username?: string;
  credential?: string;
}

/** SDP ou candidato ICE; o servidor só repassa, sem olhar dentro. */
export type SignalData =
  | { description: { type: string; sdp?: string } }
  | { candidate: Record<string, unknown> | null };

export type ClientMessage =
  | { t: 'join'; nickname: string }
  | { t: 'resume'; peerId: string; resumeToken: string }
  | { t: 'signal'; to: string; data: SignalData }
  | { t: 'live'; resolution: 720 | 1080; fps: 30 | 60; audio: AudioKind; source?: LiveSource }
  | { t: 'unlive' }
  | { t: 'turn-refresh' };

export type ErrorCode = 'bad-message' | 'not-joined' | 'unknown-peer' | 'rate-limited';

export type ServerMessage =
  | {
      t: 'welcome';
      you: Peer;
      peers: Peer[];
      iceServers: IceServer[];
      iceExpiresAt: number;
      resumeToken: string;
      capacity: number;
    }
  | { t: 'room-not-found' }
  | { t: 'room-full'; capacity: number }
  | { t: 'resume-failed' }
  | { t: 'peer-joined'; peer: Peer }
  | { t: 'peer-left'; id: string }
  | { t: 'peer-live'; id: string; live: LiveInfo | null }
  | { t: 'signal'; from: string; data: SignalData }
  | { t: 'ice-servers'; iceServers: IceServer[]; iceExpiresAt: number }
  | { t: 'error'; code: ErrorCode; message: string }
  | { t: 'pong' };

/** Códigos de fechamento do WebSocket. */
export const CLOSE = {
  policy: 1008,
  notFound: 4404,
  full: 4409,
  /** Outra conexão retomou a mesma identidade. */
  replaced: 4000,
  /** Saída voluntária: não espera a tolerância de 20 s. */
  leave: 4001,
} as const;

/** Resposta de GET /api/rooms/:code. */
export interface RoomPreview {
  code: string;
  people: { nickname: string; tone: Tone; live: boolean }[];
  capacity: number;
}
