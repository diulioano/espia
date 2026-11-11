// Mensagens do WebSocket de sinalização (contracts/signaling.md). Usado pelo navegador e pelo Worker:
// sem DOM e sem APIs do Workers aqui.

import type { BroadcastOrder, MediaPolicy } from './media';
export type Tone = 'accent' | 'sage' | 'sand' | 'deep';
export const TONES: readonly Tone[] = ['deep', 'accent', 'sage', 'sand'];

export const CAPACITY = 10;
export const NICKNAME_MAX = 24;
export const MAX_MESSAGE_BYTES = 64 * 1024;
/** Texto do que está sendo transmitido (specs/012). */
export const LIVE_LABEL_MAX = 60;
/** Nome de sala fixa (specs/011): 1–40 caracteres, nunca no link. */
export const ROOM_NAME_MAX = 40;
/** Suas salas: quantas salas fixas cada aparelho lembra (pedido de 2026-10-30). */
export const SAVED_ROOMS_MAX = 5;

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
  /**
   * O que está sendo transmitido (specs/012): "Hollow Knight", "Tela inteira", "Aba do navegador",
   * "Câmera · C920". Nunca o título de uma janela. Ausente = cliente antigo.
   */
  label?: string;
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

/** Tipo da sala (specs/011): rápida (some vazia) ou fixa (com nome, link que não muda). */
export type RoomInfo = { kind: 'quick' } | { kind: 'fixed'; name: string };

/** SDP ou candidato ICE; o servidor só repassa, sem olhar dentro. */
export type SignalData =
  | { description: { type: string; sdp?: string } }
  | { candidate: Record<string, unknown> | null };

export type ClientMessage =
  | { t: 'join'; nickname: string; mediaProtocol: 2 }
  | { t: 'resume'; peerId: string; resumeToken: string; mediaProtocol: 2 }
  | { t: 'signal'; to: string; data: SignalData }
  | { t: 'live'; requestId: string; broadcastSeq: number; resolution: 720 | 1080; fps: 30 | 60; audio: AudioKind; source?: LiveSource; label?: string }
  | { t: 'unlive'; requestId: string; broadcastSeq: number }
  | { t: 'turn-refresh' };

export type ErrorCode = 'bad-message' | 'not-joined' | 'unknown-peer' | 'rate-limited';

export type ServerMessage =
  | {
      t: 'welcome';
      mediaProtocol: 2;
      mediaPolicy: MediaPolicy;
      broadcastOrder: BroadcastOrder | null;
      you: Peer;
      peers: Peer[];
      iceServers: IceServer[];
      iceExpiresAt: number;
      resumeToken: string;
      capacity: number;
      /** specs/011. Servidor antigo não manda: trate como sala rápida. */
      room?: RoomInfo;
    }
  | { t: 'live-ack'; requestId: string; broadcastSeq: number; live: LiveInfo }
  | { t: 'room-not-found' }
  | { t: 'room-full'; capacity: number }
  | { t: 'resume-failed' }
  | { t: 'peer-joined'; peer: Peer }
  | { t: 'peer-left'; id: string }
  | { t: 'peer-live'; id: string; live: LiveInfo | null }
  | { t: 'signal'; from: string; data: SignalData }
  | { t: 'ice-servers'; iceServers: IceServer[]; iceExpiresAt: number }
  | { t: 'error'; code: ErrorCode; message: string }
  /** O dono renomeou a sala fixa (specs/011). */
  | { t: 'room-renamed'; name: string }
  /** O dono apagou a sala: vem antes do fechamento 4410 (specs/011). */
  | { t: 'room-deleted' }
  /** O dono tirou você da sala: vem antes do fechamento 4403 (specs/014 US4). */
  | { t: 'removed' }
  | { t: 'pong' };

/** Códigos de fechamento do WebSocket. */
export const CLOSE = {
  update: 4426,
  policy: 1008,
  notFound: 4404,
  full: 4409,
  /** Outra conexão retomou a mesma identidade. */
  replaced: 4000,
  /** Saída voluntária: não espera a tolerância de 60 s. */
  leave: 4001,
  /** O dono apagou a sala fixa (specs/011): não reconecta. */
  deleted: 4410,
  /** O dono tirou você da sala (specs/014 US4): não reconecta. Pode entrar de novo pelo link. */
  removed: 4403,
} as const;

/** Resposta de GET /api/rooms/:code. */
export interface RoomPreview {
  code: string;
  people: { nickname: string; tone: Tone; live: boolean }[];
  capacity: number;
  /** Só em sala fixa (specs/011). */
  fixed?: { name: string };
}

/** Por que uma sala fixa deixou de existir (specs/011): o link responde 410. */
export type GoneReason = 'deleted' | 'expired';

/** Item de POST /api/rooms/status (specs/011): só números, nunca apelidos. */
export type RoomSummary =
  | { state: 'ok'; name: string | null; people: number; streaming: number }
  | { state: 'gone'; reason: GoneReason }
  | { state: 'notfound' };
export type RoomStatus = RoomSummary & { code: string };
