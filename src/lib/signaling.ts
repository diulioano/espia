// Cliente do WebSocket da sala (contracts/signaling.md): entra, mantém viva, reconecta e retoma a
// mesma identidade com `resume`. Não sabe nada de tela; quem usa recebe as mensagens por callback.
import { CLOSE, PING, type ClientMessage, type ServerMessage } from '../shared/protocol';
import type { RoomSession } from './prefs';

export type SignalingStatus = 'connecting' | 'open' | 'reconnecting';
/** Motivos que encerram a sala de vez (sem reconectar). */
export type SignalingEnd = 'not-found' | 'full' | 'replaced' | 'kicked';

export interface SignalingOptions {
  code: string;
  nickname: string;
  session: RoomSession | null;
  onMessage(msg: ServerMessage): void;
  onStatus(status: SignalingStatus): void;
  onSession(session: RoomSession | null): void;
  onEnd(reason: SignalingEnd): void;
  /** Voltou depois de uma queda (welcome depois de 'reconnecting'). */
  onBack?(): void;
}

const PING_EVERY_MS = 20_000;
const PONG_TIMEOUT_MS = 8_000;
const BACKOFF_MS = [1_000, 2_000, 4_000, 8_000, 10_000];

export class Signaling {
  private ws: WebSocket | null = null;
  private session: RoomSession | null;
  private attempt = 0;
  private done = false;
  /** Caiu desde o último welcome: o próximo avisa onBack. */
  private wasDown = false;
  private pingTimer: ReturnType<typeof setInterval> | undefined;
  private pongTimer: ReturnType<typeof setTimeout> | undefined;
  private retryTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(private opts: SignalingOptions) {
    this.session = opts.session;
    addEventListener('online', this.onOnline);
    this.connect();
  }

  /** Envia se estiver conectado. Mensagens perdidas durante a queda não fazem falta: o estado volta no welcome. */
  send(msg: ClientMessage): boolean {
    if (this.ws?.readyState !== WebSocket.OPEN) return false;
    this.ws.send(JSON.stringify(msg));
    return true;
  }

  /** "Sair da sala": os outros recebem peer-left na hora. */
  leave() {
    this.finish();
    this.ws?.close(CLOSE.leave, 'saiu');
    this.session = null;
    this.opts.onSession(null);
  }

  private connect() {
    this.opts.onStatus(this.attempt === 0 && !this.session ? 'connecting' : 'reconnecting');
    const proto = location.protocol === 'https:' ? 'wss' : 'ws';
    const ws = new WebSocket(`${proto}://${location.host}/ws/${this.opts.code}`);
    this.ws = ws;
    ws.onopen = () => {
      if (this.session) this.send({ t: 'resume', ...this.session });
      else this.send({ t: 'join', nickname: this.opts.nickname });
      this.startPing();
    };
    ws.onmessage = (e) => this.receive(JSON.parse(e.data as string) as ServerMessage);
    ws.onclose = (e) => {
      if (this.ws === ws) this.dropped(e.code);
    };
  }

  private receive(msg: ServerMessage) {
    switch (msg.t) {
      case 'pong':
        clearTimeout(this.pongTimer);
        return;
      case 'welcome':
        this.attempt = 0;
        this.session = { peerId: msg.you.id, resumeToken: msg.resumeToken };
        this.opts.onSession(this.session);
        this.opts.onStatus('open');
        if (this.wasDown) this.opts.onBack?.();
        this.wasDown = false;
        break;
      case 'resume-failed':
        // Passou da tolerância: entra de novo como pessoa nova.
        this.session = null;
        this.opts.onSession(null);
        this.send({ t: 'join', nickname: this.opts.nickname });
        return;
      case 'room-not-found':
        return this.end('not-found');
      case 'room-full':
        return this.end('full');
    }
    this.opts.onMessage(msg);
  }

  private dropped(code: number) {
    this.stopPing();
    if (this.done) return;
    if (code === CLOSE.replaced) return this.end('replaced');
    if (code === CLOSE.policy) return this.end('kicked');
    if (code === CLOSE.notFound) return this.end('not-found');
    if (code === CLOSE.full) return this.end('full');
    this.wasDown = true;
    this.opts.onStatus('reconnecting');
    const delay = BACKOFF_MS[Math.min(this.attempt, BACKOFF_MS.length - 1)];
    this.attempt++;
    this.retryTimer = setTimeout(() => this.connect(), delay);
  }

  /** Abandona o socket atual (pode estar morto sem saber) e reconecta já. */
  private abandon() {
    const ws = this.ws;
    this.ws = null;
    if (ws) {
      ws.onclose = null;
      ws.onmessage = null;
      ws.close();
    }
    this.dropped(1006);
  }

  private startPing() {
    this.stopPing();
    this.pingTimer = setInterval(() => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
      this.ws.send(PING);
      clearTimeout(this.pongTimer);
      this.pongTimer = setTimeout(() => this.abandon(), PONG_TIMEOUT_MS);
    }, PING_EVERY_MS);
  }

  private stopPing() {
    clearInterval(this.pingTimer);
    clearTimeout(this.pongTimer);
  }

  /** "Tentar de novo" (specs/011): tenta já, sem esperar o próximo intervalo do backoff. */
  retryNow() {
    if (this.done || this.ws?.readyState === WebSocket.OPEN) return;
    clearTimeout(this.retryTimer);
    this.attempt = 0;
    this.abandon();
  }

  /** A internet voltou: não espera o próximo intervalo do backoff. */
  private onOnline = () => this.retryNow();

  private end(reason: SignalingEnd) {
    this.finish();
    this.ws?.close();
    if (reason !== 'replaced') {
      this.session = null;
      this.opts.onSession(null);
    }
    this.opts.onEnd(reason);
  }

  private finish() {
    this.done = true;
    this.stopPing();
    clearTimeout(this.retryTimer);
    removeEventListener('online', this.onOnline);
  }
}
