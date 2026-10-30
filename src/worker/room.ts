// Durable Object de uma sala (contracts/signaling.md, architecture.md §2).
//
// Onde fica cada coisa, para sobreviver à hibernação do WebSocket e ao reinício do DO:
// - quem está conectado: no attachment de cada socket (serializeAttachment) e, espelhado, no
//   `roster` do storage. Uma publicação reinicia o DO e corta os sockets sem rodar webSocketClose
//   (specs/011 research R1); o roster é o que deixa essas pessoas voltarem com `resume`;
// - quem caiu e ainda pode voltar (tolerância de 60 s), a reserva de 60 s, a sala rápida segurada
//   5 min depois que todos caíram (`holdUntil`) e o contador de tons: no storage do DO, apagado
//   inteiro quando a sala esvazia. Nada sobre a pessoa fica depois disso.
import { DurableObject } from 'cloudflare:workers';
import {
  CAPACITY,
  CLOSE,
  MAX_MESSAGE_BYTES,
  PING,
  PONG,
  TONES,
  type ClientMessage,
  type IceServer,
  type LiveInfo,
  type Peer,
  type RoomPreview,
  type ServerMessage,
  type Tone,
} from '../shared/protocol';
import { mintIceServers, revokeTurn } from './turn';
import { parseClientMessage } from './validate';

interface Member extends Peer {
  token: string;
  /** Ordem de entrada: a lista sai sempre nessa ordem (o runtime não garante a dos sockets). */
  seq: number;
  /** Usernames TURN entregues a essa pessoa; todos são revogados quando ela sai. */
  turn: string[];
}

type Attachment = { joined: false } | { joined: true; member: Member };

/** Alguém cujo socket fechou, esperando `resume` até `until`. */
interface Ghost {
  member: Member;
  until: number;
}

const FLOOD_MAX = 200;
const FLOOD_WINDOW_MS = 10_000;
const INVALID_MAX = 5;
/** Credenciais TURN guardadas por pessoa (uma por welcome/refresh). Passou disso, revoga as mais velhas. */
const TURN_KEEP = 8;

export class Room extends DurableObject<Env> {
  /** Contadores de flood por socket. Só em memória: zeram se o DO hibernar, o que só acontece parado. */
  private counters = new Map<WebSocket, { start: number; count: number; invalid: number }>();

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    // Keep-alive respondido pelo runtime, sem acordar o DO.
    ctx.setWebSocketAutoResponse(new WebSocketRequestResponsePair(PING, PONG));
    void ctx.blockConcurrencyWhile(() => this.recover());
  }

  private get graceMs() {
    return Number(this.env.GRACE_MS) || 60_000;
  }

  private get holdMs() {
    return Number(this.env.HOLD_MS) || 300_000;
  }

  private get reserveMs() {
    return Number(this.env.RESERVE_MS) || 60_000;
  }

  // ---------- RPC chamado pelo Worker ----------

  /** Reserva o código por 60 s para o criador conectar. false se a sala já existe. */
  async reserve(): Promise<boolean> {
    if (await this.exists()) return false;
    await this.ctx.storage.put('reservedUntil', Date.now() + this.reserveMs);
    await this.scheduleAlarm();
    return true;
  }

  /** Prévia para a tela de entrada; null se a sala não existe. */
  async preview(code: string): Promise<RoomPreview | null> {
    if (!(await this.exists())) return null;
    const people = await this.everyone();
    return {
      code,
      people: people.map((p) => ({ nickname: p.nickname, tone: p.tone, live: p.live !== null })),
      capacity: CAPACITY,
    };
  }

  // ---------- WebSocket ----------

  async fetch(request: Request): Promise<Response> {
    if (request.headers.get('Upgrade') !== 'websocket') return new Response('Esperava WebSocket', { status: 426 });
    const { 0: client, 1: server } = new WebSocketPair();
    this.ctx.acceptWebSocket(server);
    server.serializeAttachment({ joined: false } satisfies Attachment);
    return new Response(null, { status: 101, webSocket: client });
  }

  async webSocketMessage(ws: WebSocket, raw: string | ArrayBuffer) {
    if (typeof raw !== 'string' || tooBig(raw)) return this.kick(ws);

    const c = this.counter(ws);
    if (++c.count > FLOOD_MAX) return this.kick(ws);

    const msg = parseClientMessage(raw);
    if (!msg) {
      if (++c.invalid > INVALID_MAX) return this.kick(ws);
      return send(ws, { t: 'error', code: 'bad-message', message: 'Mensagem inválida.' });
    }

    const att = ws.deserializeAttachment() as Attachment;
    if (!att.joined) {
      if (msg.t === 'join') return this.join(ws, msg.nickname);
      if (msg.t === 'resume') return this.resume(ws, msg.peerId, msg.resumeToken);
      return send(ws, { t: 'error', code: 'not-joined', message: 'Mande join primeiro.' });
    }
    return this.handle(ws, att.member, msg);
  }

  async webSocketClose(ws: WebSocket, code: number) {
    await this.gone(ws, code);
    try {
      ws.close(1000, 'tchau');
    } catch {
      // Já fechado.
    }
  }

  async webSocketError(ws: WebSocket) {
    await this.gone(ws, 1006);
  }

  // ---------- Entrada ----------

  private async join(ws: WebSocket, nickname: string) {
    if (await this.refuse(ws)) return;
    // As credenciais TURN vêm ANTES de registrar a pessoa. Esperar a API deixa outras mensagens
    // passarem na frente; depois dela, registrar, montar a lista e avisar os outros acontece sem
    // pausa (só storage, que segura as outras mensagens). Sem isso, quem entra junto aparecia
    // duplicado ou faltando na lista.
    const { username, ...ice } = await mintIceServers(this.env);
    if (ws.readyState !== WebSocket.OPEN || (await this.refuse(ws))) {
      if (username) await revokeTurn(this.env, [username]);
      return;
    }

    const joins = (await this.ctx.storage.get<number>('joins')) ?? 0;
    const tone: Tone = TONES[joins % TONES.length];
    const member: Member = {
      id: randomHex(8),
      nickname,
      tone,
      live: null,
      token: randomHex(32),
      seq: joins,
      turn: username ? [username] : [],
    };
    await this.ctx.storage.put('joins', joins + 1);
    await this.ctx.storage.delete(['reservedUntil', 'holdUntil']);

    ws.serializeAttachment({ joined: true, member } satisfies Attachment);
    await this.saveMember(member);
    await this.welcome(ws, member, ice);
    this.broadcast({ t: 'peer-joined', peer: publicPeer(member) }, member.id);
  }

  /** Sala inexistente ou cheia: avisa, fecha e devolve true. */
  private async refuse(ws: WebSocket): Promise<boolean> {
    if (!(await this.exists())) {
      send(ws, { t: 'room-not-found' });
      ws.close(CLOSE.notFound, 'sala não existe');
      return true;
    }
    if (this.members().length + Object.keys(await this.ghosts()).length >= CAPACITY) {
      send(ws, { t: 'room-full', capacity: CAPACITY });
      ws.close(CLOSE.full, 'sala cheia');
      return true;
    }
    return false;
  }

  private async resume(ws: WebSocket, peerId: string, token: string) {
    // Credenciais antes, pelo mesmo motivo do join.
    const { username, ...ice } = await mintIceServers(this.env);
    const withTurn = (m: Member): Member => (username ? { ...m, turn: keepTurn([...m.turn, username], this.env) } : m);

    const ghosts = await this.ghosts();
    const ghost = ghosts[peerId];
    if (ws.readyState === WebSocket.OPEN && ghost && tokensEqual(ghost.member.token, token)) {
      delete ghosts[peerId];
      await this.ctx.storage.put('ghosts', ghosts);
      await this.ctx.storage.delete('holdUntil');
      const member = withTurn(ghost.member);
      ws.serializeAttachment({ joined: true, member } satisfies Attachment);
      await this.saveMember(member);
      // Para os outros a pessoa nunca saiu: nada de peer-joined.
      return this.welcome(ws, member, ice);
    }

    // Socket antigo ainda aberto (conexão meio morta): a nova conexão assume a identidade.
    const old = this.members().find((m) => m.member.id === peerId);
    if (ws.readyState === WebSocket.OPEN && old && tokensEqual(old.member.token, token)) {
      old.ws.serializeAttachment({ joined: false } satisfies Attachment);
      const member = withTurn(old.member);
      ws.serializeAttachment({ joined: true, member } satisfies Attachment);
      await this.saveMember(member);
      try {
        old.ws.close(CLOSE.replaced, 'reconectou em outra conexão');
      } catch {
        // Já fechado.
      }
      return this.welcome(ws, member, ice);
    }

    if (username) await revokeTurn(this.env, [username]);
    send(ws, { t: 'resume-failed' });
  }

  private async welcome(ws: WebSocket, me: Member, ice: { iceServers: IceServer[]; iceExpiresAt: number }) {
    const others = (await this.everyone()).filter((p) => p.id !== me.id);
    send(ws, {
      t: 'welcome',
      you: publicPeer(me),
      peers: others.map(publicPeer),
      ...ice,
      resumeToken: me.token,
      capacity: CAPACITY,
    });
  }

  /** turn-refresh: gera credenciais e anota o username na pessoa (no attachment do socket). */
  private async grantTurn(ws: WebSocket, me: Member) {
    const { username, ...ice } = await mintIceServers(this.env);
    if (username) {
      // Relê o attachment: pode ter mudado (live/unlive) enquanto esperava a API.
      const att = ws.deserializeAttachment() as Attachment | null;
      if (att?.joined && att.member.id === me.id) {
        const turn = keepTurn([...att.member.turn, username], this.env);
        const member = { ...att.member, turn };
        ws.serializeAttachment({ joined: true, member } satisfies Attachment);
        await this.saveMember(member);
      } else {
        // Saiu enquanto gerava: não deixa credencial solta.
        await revokeTurn(this.env, [username]);
      }
    }
    return ice;
  }

  // ---------- Depois de entrar ----------

  private async handle(ws: WebSocket, me: Member, msg: ClientMessage) {
    switch (msg.t) {
      case 'join':
      case 'resume':
        return send(ws, { t: 'error', code: 'bad-message', message: 'Você já está na sala.' });
      case 'signal': {
        const target = this.members().find((m) => m.member.id === msg.to);
        if (!target || target.ws === ws) {
          return send(ws, { t: 'error', code: 'unknown-peer', message: 'Essa pessoa não está conectada.' });
        }
        // `from` vem sempre do servidor (security.md A3).
        return send(target.ws, { t: 'signal', from: me.id, data: msg.data });
      }
      case 'live':
      case 'unlive': {
        const live: LiveInfo | null =
          msg.t === 'live'
            ? { resolution: msg.resolution, fps: msg.fps, audio: msg.audio, source: msg.source ?? 'screen', startedAt: me.live?.startedAt ?? Date.now() }
            : null;
        const member = { ...me, live };
        ws.serializeAttachment({ joined: true, member } satisfies Attachment);
        await this.saveMember(member);
        return this.broadcast({ t: 'peer-live', id: me.id, live }, me.id);
      }
      case 'turn-refresh':
        return send(ws, { t: 'ice-servers', ...(await this.grantTurn(ws, me)) });
    }
  }

  // ---------- Saída ----------

  private async gone(ws: WebSocket, code: number) {
    this.counters.delete(ws);
    const att = ws.deserializeAttachment() as Attachment | null;
    if (!att?.joined) return;
    ws.serializeAttachment({ joined: false } satisfies Attachment);

    const { member } = att;
    // Outra conexão já retomou essa identidade.
    if (this.members().some((m) => m.member.id === member.id)) return;

    if (code === CLOSE.leave) {
      this.broadcast({ t: 'peer-left', id: member.id });
      await this.dropMembers([member.id]);
      await revokeTurn(this.env, member.turn);
      // Saiu o último conectado, mas tem gente que caiu: a sala espera por eles.
      if (this.members().length === 0 && Object.keys(await this.ghosts()).length > 0) {
        await this.holdIfQuick();
        await this.scheduleAlarm();
      }
      return this.disposeIfEmpty();
    }

    const ghosts = await this.ghosts();
    ghosts[member.id] = { member, until: Date.now() + this.graceMs };
    await this.ctx.storage.put('ghosts', ghosts);
    // Caiu o último conectado (todos caíram): sala rápida espera 5 min (specs/011 R2).
    if (this.members().length === 0) await this.holdIfQuick();
    await this.scheduleAlarm();
  }

  /**
   * Depois de um reinício (publicação), quem estava no roster e não tem mais socket vira fantasma,
   * como numa queda comum. Acordar da hibernação não cria fantasma: os sockets continuam lá.
   */
  private async recover() {
    const roster = await this.roster();
    const ids = Object.keys(roster);
    if (ids.length === 0) return;
    const connected = new Set(this.members().map((m) => m.member.id));
    const ghosts = await this.ghosts();
    const until = Date.now() + this.graceMs;
    let created = false;
    for (const id of ids) {
      if (connected.has(id) || ghosts[id]) continue;
      ghosts[id] = { member: roster[id], until };
      created = true;
    }
    if (!created) return;
    await this.ctx.storage.put('ghosts', ghosts);
    if (connected.size === 0) await this.holdIfQuick();
    await this.scheduleAlarm();
  }

  /** Sala rápida em que todos caíram fica de pé por HOLD_MS. */
  private async holdIfQuick() {
    if (await this.ctx.storage.get('fixed')) return;
    await this.ctx.storage.put('holdUntil', Date.now() + this.holdMs);
  }

  async alarm() {
    const now = Date.now();
    const ghosts = await this.ghosts();
    const expired = Object.values(ghosts).filter((g) => g.until <= now);
    for (const g of expired) delete ghosts[g.member.id];
    if (expired.length) {
      await this.ctx.storage.put('ghosts', ghosts);
      await this.dropMembers(expired.map((g) => g.member.id));
    }
    for (const g of expired) this.broadcast({ t: 'peer-left', id: g.member.id });
    await revokeTurn(this.env, expired.flatMap((g) => g.member.turn));

    const reservedUntil = await this.ctx.storage.get<number>('reservedUntil');
    if (reservedUntil !== undefined && reservedUntil <= now) await this.ctx.storage.delete('reservedUntil');
    const holdUntil = await this.ctx.storage.get<number>('holdUntil');
    if (holdUntil !== undefined && holdUntil <= now) await this.ctx.storage.delete('holdUntil');

    if (!(await this.disposeIfEmpty(true))) await this.scheduleAlarm();
  }

  /**
   * Sala sem ninguém, sem fantasmas e sem reserva: apaga tudo. O código volta a "não existe".
   * Apaga chave por chave: `deleteAll()` dentro do próprio alarme dá erro interno no runtime.
   */
  private async disposeIfEmpty(inAlarm = false): Promise<boolean> {
    if (await this.exists()) return false;
    if (!inAlarm) await this.ctx.storage.deleteAlarm();
    await this.ctx.storage.delete(['ghosts', 'reservedUntil', 'joins', 'roster', 'holdUntil']);
    return true;
  }

  // ---------- Utilitários ----------

  /** Sockets que já mandaram join/resume. */
  private members(): { ws: WebSocket; member: Member }[] {
    const out: { ws: WebSocket; member: Member }[] = [];
    for (const ws of this.ctx.getWebSockets()) {
      const att = ws.deserializeAttachment() as Attachment | null;
      if (att?.joined) out.push({ ws, member: att.member });
    }
    return out;
  }

  /** Conectados e quem está na tolerância, em ordem de entrada. */
  private async everyone(): Promise<Member[]> {
    const ghosts = Object.values(await this.ghosts()).map((g) => g.member);
    return [...this.members().map((m) => m.member), ...ghosts].sort((a, b) => a.seq - b.seq);
  }

  private async roster(): Promise<Record<string, Member>> {
    return (await this.ctx.storage.get<Record<string, Member>>('roster')) ?? {};
  }

  /** Espelha a pessoa no roster (entrada, retomada, live, TURN). */
  private async saveMember(member: Member) {
    const roster = await this.roster();
    roster[member.id] = member;
    await this.ctx.storage.put('roster', roster);
  }

  private async dropMembers(ids: string[]) {
    const roster = await this.roster();
    for (const id of ids) delete roster[id];
    await this.ctx.storage.put('roster', roster);
  }

  private async ghosts(): Promise<Record<string, Ghost>> {
    return (await this.ctx.storage.get<Record<string, Ghost>>('ghosts')) ?? {};
  }

  private async exists(): Promise<boolean> {
    if (this.members().length > 0) return true;
    if (Object.keys(await this.ghosts()).length > 0) return true;
    const now = Date.now();
    const reservedUntil = await this.ctx.storage.get<number>('reservedUntil');
    if (reservedUntil !== undefined && reservedUntil > now) return true;
    const holdUntil = await this.ctx.storage.get<number>('holdUntil');
    return holdUntil !== undefined && holdUntil > now;
  }

  private async scheduleAlarm() {
    const times = Object.values(await this.ghosts()).map((g) => g.until);
    const reservedUntil = await this.ctx.storage.get<number>('reservedUntil');
    if (reservedUntil !== undefined) times.push(reservedUntil);
    const holdUntil = await this.ctx.storage.get<number>('holdUntil');
    if (holdUntil !== undefined) times.push(holdUntil);
    if (times.length) await this.ctx.storage.setAlarm(Math.min(...times));
  }

  private broadcast(msg: ServerMessage, exceptId?: string) {
    for (const { ws, member } of this.members()) if (member.id !== exceptId) send(ws, msg);
  }

  private counter(ws: WebSocket) {
    const now = Date.now();
    let c = this.counters.get(ws);
    if (!c || now - c.start > FLOOD_WINDOW_MS) {
      c = { start: now, count: 0, invalid: c?.invalid ?? 0 };
      this.counters.set(ws, c);
    }
    return c;
  }

  private kick(ws: WebSocket) {
    try {
      ws.close(CLOSE.policy, 'mensagem grande demais, inválida ou rápida demais');
    } catch {
      // Já fechado.
    }
  }

}

/** Guarda só as últimas TURN_KEEP credenciais; revoga as mais velhas em segundo plano. */
function keepTurn(turn: string[], env: Env): string[] {
  const old = turn.slice(0, Math.max(0, turn.length - TURN_KEEP));
  if (old.length) void revokeTurn(env, old);
  return turn.slice(-TURN_KEEP);
}

function send(ws: WebSocket, msg: ServerMessage) {
  try {
    ws.send(JSON.stringify(msg));
  } catch {
    // Socket fechando; a saída é tratada no close.
  }
}

function publicPeer({ id, nickname, tone, live }: Member): Peer {
  return { id, nickname, tone, live };
}

function tooBig(raw: string) {
  // Cada caractere ocupa de 1 a 3 bytes em UTF-8; só codifica quando pode passar do limite.
  if (raw.length > MAX_MESSAGE_BYTES) return true;
  return raw.length * 3 > MAX_MESSAGE_BYTES && new TextEncoder().encode(raw).byteLength > MAX_MESSAGE_BYTES;
}

function randomHex(bytes: number) {
  return [...crypto.getRandomValues(new Uint8Array(bytes))].map((b) => b.toString(16).padStart(2, '0')).join('');
}

function tokensEqual(a: string, b: string) {
  const enc = new TextEncoder();
  const x = enc.encode(a);
  const y = enc.encode(b);
  return x.byteLength === y.byteLength && crypto.subtle.timingSafeEqual(x, y);
}
