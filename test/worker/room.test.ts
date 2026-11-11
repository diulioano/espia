// Testes do Worker + Durable Object no runtime real (Miniflare). GRACE_MS = 400, RESERVE_MS = 600,
// HOLD_MS = 800, FIXED_TTL_MS = 1500 (vitest.config.ts).
import { evictDurableObject, runInDurableObject } from 'cloudflare:test';
import { env, exports } from 'cloudflare:workers';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { CLOSE, PING, type ServerMessage } from '../../src/shared/protocol';

const BASE = 'http://espia.test';
const ORIGIN = { Origin: BASE };
const GRACE = 400;
const HOLD = 800;

let ipSeq = 0;
/** IP novo a cada chamada, para os limites por IP não atrapalharem testes que não são sobre eles. */
const freshIp = () => `10.0.${Math.floor(++ipSeq / 250)}.${ipSeq % 250}`;

function call(path: string, init: RequestInit = {}) {
  const headers = new Headers(init.headers);
  if (!headers.has('CF-Connecting-IP')) headers.set('CF-Connecting-IP', freshIp());
  return exports.default.fetch(new Request(BASE + path, { ...init, headers }));
}

async function createRoom(): Promise<string> {
  const res = await call('/api/rooms', { method: 'POST', headers: ORIGIN });
  expect(res.status).toBe(201);
  return ((await res.json()) as { code: string }).code;
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

// API do Cloudflare TURN simulada: cada credencial gerada tem um username novo; registra revogações.
let turnSeq = 0;
let turnDown = false;
/** Atraso da API simulada (ms), para provocar mensagens se cruzando. */
let turnDelay = 0;
const revoked: string[] = [];
const realFetch = globalThis.fetch;
beforeEach(() => {
  turnDown = false;
  turnDelay = 0;
  revoked.length = 0;
  vi.spyOn(globalThis, 'fetch').mockImplementation(async (input, init) => {
    const url = input instanceof Request ? input.url : String(input);
    if (!url.startsWith('https://rtc.live.cloudflare.com/')) return realFetch(input, init);
    if (turnDelay) await sleep(turnDelay);
    if (turnDown) return new Response('fora do ar', { status: 500 });
    const revoke = url.match(/\/credentials\/([^/]+)\/revoke$/);
    if (revoke) {
      revoked.push(decodeURIComponent(revoke[1]));
      return new Response(null, { status: 204 });
    }
    return Response.json({
      iceServers: [
        {
          urls: ['stun:stun.cloudflare.com:3478', 'turn:turn.cloudflare.com:3478?transport=udp', 'turn:turn.cloudflare.com:53?transport=udp'],
          username: `u${++turnSeq}`,
          credential: 'segredo',
        },
      ],
    });
  });
});
afterEach(() => vi.restoreAllMocks());

async function until(cond: () => boolean, timeout = 2000) {
  const end = Date.now() + timeout;
  while (!cond() && Date.now() < end) await sleep(20);
  return cond();
}

/** Cliente de teste: guarda as mensagens recebidas e deixa esperar por um tipo específico. */
class Client {
  inbox: ServerMessage[] = [];
  closed: { code: number } | null = null;
  private wake: (() => void) | null = null;

  constructor(readonly ws: WebSocket) {
    ws.addEventListener('message', (e) => {
      this.inbox.push(JSON.parse(e.data as string));
      this.wake?.();
    });
    ws.addEventListener('close', (e) => {
      this.closed = { code: e.code };
      this.wake?.();
    });
    ws.accept();
  }

  static async open(code: string): Promise<Client> {
    const res = await call(`/ws/${code}`, { headers: { Upgrade: 'websocket', ...ORIGIN } });
    expect(res.status).toBe(101);
    return new Client(res.webSocket!);
  }

  static async join(code: string, nickname: string) {
    const c = await Client.open(code);
    c.send({ t: 'join', nickname });
    const welcome = await c.next('welcome');
    return { c, welcome };
  }

  private broadcastSeq=0;
  private requestId=crypto.randomUUID();
  send(msg: unknown) {
    if(typeof msg==='object'&&msg!==null&&'t' in msg) {
      if(msg.t==='join'||msg.t==='resume') msg={...msg,mediaProtocol:2};
      else if(msg.t==='live'||msg.t==='unlive') {
        const stopping=msg.t==='unlive';
        msg={...msg,requestId:this.requestId,broadcastSeq:this.broadcastSeq};
        if(stopping) {this.broadcastSeq++;this.requestId=crypto.randomUUID();}
      }
    }
    this.ws.send(typeof msg === 'string' ? msg : JSON.stringify(msg));
  }

  /** Próxima mensagem do tipo `t` (tira da caixa). */
  async next<T extends ServerMessage['t']>(t: T, timeout = 2000): Promise<Extract<ServerMessage, { t: T }>> {
    const deadline = Date.now() + timeout;
    for (;;) {
      const i = this.inbox.findIndex((m) => m.t === t);
      if (i >= 0) return this.inbox.splice(i, 1)[0] as Extract<ServerMessage, { t: T }>;
      if (Date.now() > deadline) throw new Error(`esperava "${t}", chegou: ${JSON.stringify(this.inbox)}`);
      await new Promise<void>((r) => {
        this.wake = r;
        setTimeout(r, 50);
      });
    }
  }

  async waitClose(timeout = 2000) {
    const deadline = Date.now() + timeout;
    while (!this.closed && Date.now() < deadline) await sleep(20);
    return this.closed;
  }

  has(t: ServerMessage['t']) {
    return this.inbox.some((m) => m.t === t);
  }
}

describe('HTTP', () => {
  it('cria sala só com Origin do próprio site', async () => {
    expect((await call('/api/rooms', { method: 'POST' })).status).toBe(403);
    expect((await call('/api/rooms', { method: 'POST', headers: { Origin: 'https://malvado.example' } })).status).toBe(403);
    expect(await createRoom()).toMatch(/^[a-hjkmnp-z2-9]{3}-[a-hjkmnp-z2-9]{3}$/);
  });

  it('prévia: 400 código ruim, 404 sala inexistente, 200 reservada', async () => {
    expect((await call('/api/rooms/xxx')).status).toBe(400);
    expect((await call('/api/rooms/aaa-aaa')).status).toBe(404);
    const code = await createRoom();
    const res = await call(`/api/rooms/${code}`);
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ code, people: [], capacity: 10 });
  });

  it('WebSocket de outra origem recebe 403', async () => {
    const code = await createRoom();
    const res = await call(`/ws/${code}`, { headers: { Upgrade: 'websocket', Origin: 'https://malvado.example' } });
    expect(res.status).toBe(403);
  });

  it('limita consultas por IP (20/min) com 429 e Retry-After', async () => {
    const ip = freshIp();
    const statuses: number[] = [];
    for (let i = 0; i < 22; i++) statuses.push((await call('/api/rooms/aaa-aaa', { headers: { 'CF-Connecting-IP': ip } })).status);
    expect(statuses.slice(0, 20).every((s) => s === 404)).toBe(true);
    const last = await call('/api/rooms/aaa-aaa', { headers: { 'CF-Connecting-IP': ip } });
    expect(last.status).toBe(429);
    expect(last.headers.get('Retry-After')).toBe('60');
  });

  it('manda os cabeçalhos de segurança', async () => {
    const res = await call('/api/rooms/aaa-aaa');
    expect(res.headers.get('Referrer-Policy')).toBe('no-referrer');
    expect(res.headers.get('Content-Security-Policy')).toContain("frame-ancestors 'self'");
  });

  it('reserva some se ninguém conectar', async () => {
    const code = await createRoom();
    await sleep(900);
    expect((await call(`/api/rooms/${code}`)).status).toBe(404);
  });
});

describe('Sala', () => {
  it('entrar: welcome, peer-joined, tons em rodízio, prévia', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    expect(wa.peers).toEqual([]);
    expect(wa.you).toMatchObject({ nickname: 'Rafa', tone: 'deep', live: null });
    expect(wa.resumeToken).toMatch(/^[0-9a-f]{64}$/);

    const { welcome: wb } = await Client.join(code, '  Lu‮ ');
    expect(wb.you).toMatchObject({ nickname: 'Lu', tone: 'accent' });
    expect(wb.peers.map((p) => p.id)).toEqual([wa.you.id]);
    expect((await a.next('peer-joined')).peer.id).toBe(wb.you.id);

    const preview = await (await call(`/api/rooms/${code}`)).json();
    expect(preview).toMatchObject({ people: [{ nickname: 'Rafa' }, { nickname: 'Lu' }] });
  });

  it('sala inexistente: room-not-found e fecha 4404', async () => {
    const c = await Client.open('zzz-zzz');
    c.send({ t: 'join', nickname: 'Rafa' });
    await c.next('room-not-found');
    expect((await c.waitClose())?.code).toBe(CLOSE.notFound);
  });

  it('11ª pessoa recebe room-full e fecha 4409', async () => {
    const code = await createRoom();
    for (let i = 0; i < 10; i++) await Client.join(code, `P${i}`);
    const c = await Client.open(code);
    c.send({ t: 'join', nickname: 'Onze' });
    expect((await c.next('room-full')).capacity).toBe(10);
    expect((await c.waitClose())?.code).toBe(CLOSE.full);
  });

  it('repassa signal com `from` do servidor; `from` forjado é recusado', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    const { c: b, welcome: wb } = await Client.join(code, 'Lu');

    const data = { description: { type: 'offer', sdp: 'v=0' } };
    b.send({ t: 'signal', to: wa.you.id, data });
    expect(await a.next('signal')).toEqual({ t: 'signal', from: wb.you.id, data });

    b.send({ t: 'signal', to: wa.you.id, from: wa.you.id, data });
    expect((await b.next('error')).code).toBe('bad-message');

    b.send({ t: 'signal', to: '0123456789abcdef', data });
    expect((await b.next('error')).code).toBe('unknown-peer');
  });

  it('antes do join só aceita join/resume', async () => {
    const code = await createRoom();
    const c = await Client.open(code);
    c.send({ t: 'unlive' });
    expect((await c.next('error')).code).toBe('not-joined');
  });

  it('live/unlive avisam os outros', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    a.send({ t: 'live', resolution: 1080, fps: 60, audio: 'tab' });
    const on = await b.next('peer-live');
    expect(on).toMatchObject({ id: wa.you.id, live: { resolution: 1080, fps: 60, audio: 'tab' } });
    expect(on.live!.startedAt).toBeTypeOf('number');
    a.send({ t: 'unlive' });
    expect((await b.next('peer-live')).live).toBeNull();
  });

  it('responde ping com pong', async () => {
    const code = await createRoom();
    const { c } = await Client.join(code, 'Rafa');
    c.send(PING);
    await c.next('pong');
  });
});

describe('Som no live (specs/002)', () => {
  it("repassa o tipo de som: 'window'", async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    a.send({ t: 'live', resolution: 720, fps: 60, audio: 'window' });
    expect((await b.next('peer-live')).live).toMatchObject({ audio: 'window' });
  });

  it('cliente antigo com audio boolean é recusado', async()=>{
    const {c:a}=await Client.join(await createRoom(),'Rafa');
    a.send(JSON.stringify({t:'live',resolution:720,fps:30,audio:true}));
    expect((await a.next('error')).code).toBe('bad-message');
  });

  it("repassa o som do PC sem o Discord (app desktop, specs/005): 'pc'", async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    a.send({ t: 'live', resolution: 1080, fps: 30, audio: 'pc' });
    expect((await b.next('peer-live')).live).toMatchObject({ audio: 'pc' });
  });

  it("câmera (specs/006): source 'camera' e som 'device' repassados", async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    a.send({ t: 'live', resolution: 720, fps: 30, audio: 'device', source: 'camera' });
    expect((await b.next('peer-live')).live).toMatchObject({ audio: 'device', source: 'camera' });
  });

  it("cliente antigo sem source → 'screen'", async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    a.send({ t: 'live', resolution: 720, fps: 30, audio: 'window' });
    expect((await b.next('peer-live')).live).toMatchObject({ source: 'screen' });
  });

  it("source desconhecido → bad-message", async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Rafa');
    a.send(JSON.stringify({ t: 'live', resolution: 720, fps: 30, audio: 'none', source: 'x' }));
    expect((await a.next('error')).code).toBe('bad-message');
  });

  it("som do sistema não existe: 'system' → bad-message", async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Rafa');
    a.send(JSON.stringify({ t: 'live', resolution: 720, fps: 30, audio: 'system' }));
    expect((await a.next('error')).code).toBe('bad-message');
  });
});

describe('Manter a conexão', () => {
  it('caiu e voltou dentro da tolerância: mesma identidade, ninguém vê saída', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    a.send({ t: 'live', resolution: 720, fps: 30, audio: 'none' });
    const { c: b } = await Client.join(code, 'Lu');
    await a.next('peer-joined');

    a.ws.close(1000, 'internet piscou');
    await sleep(100);

    const a2 = await Client.open(code);
    a2.send({ t: 'resume', peerId: wa.you.id, resumeToken: wa.resumeToken });
    const again = await a2.next('welcome');
    expect(again.you.id).toBe(wa.you.id);
    expect(again.you.live).toMatchObject({ resolution: 720 });
    expect(again.peers).toHaveLength(1);

    await sleep(GRACE + 400);
    expect(b.has('peer-left')).toBe(false);

    // Continua repassando depois de voltar.
    b.send({ t: 'signal', to: wa.you.id, data: { candidate: null } });
    expect((await a2.next('signal')).data).toEqual({ candidate: null });
  });

  it('caiu e não voltou: peer-left depois da tolerância', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    await a.next('peer-joined');

    a.ws.close(1000);
    await sleep(150);
    expect(b.has('peer-left')).toBe(false);
    expect((await b.next('peer-left', GRACE + 1500)).id).toBe(wa.you.id);

    // Depois da tolerância o resume não vale mais.
    const late = await Client.open(code);
    late.send({ t: 'resume', peerId: wa.you.id, resumeToken: wa.resumeToken });
    await late.next('resume-failed');
  });

  it('resume com token errado falha', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    await Client.join(code, 'Lu');
    a.ws.close(1000);
    await sleep(50);
    const c = await Client.open(code);
    c.send({ t: 'resume', peerId: wa.you.id, resumeToken: '0'.repeat(64) });
    await c.next('resume-failed');
  });

  it('nova conexão assume a identidade se a antiga ainda estiver aberta', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    const a2 = await Client.open(code);
    a2.send({ t: 'resume', peerId: wa.you.id, resumeToken: wa.resumeToken });
    expect((await a2.next('welcome')).you.id).toBe(wa.you.id);
    expect((await a.waitClose())?.code).toBe(CLOSE.replaced);
    await sleep(GRACE + 400);
    expect(b.has('peer-left')).toBe(false);
  });

  it('sair da sala (4001) avisa na hora', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    await a.next('peer-joined');
    a.ws.close(CLOSE.leave, 'saiu');
    expect((await b.next('peer-left', 300)).id).toBe(wa.you.id);
  });

  it('sala rápida em que todos caíram some depois da espera de HOLD_MS (specs/011)', async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Rafa');
    a.ws.close(1000);
    await sleep(GRACE + 150);
    expect((await call(`/api/rooms/${code}`)).status).toBe(200);
    await sleep(HOLD + 400);
    expect((await call(`/api/rooms/${code}`)).status).toBe(404);
  });
});

describe('Publicação e queda (specs/011)', () => {
  const stubOf = (code: string) => env.ROOM.get(env.ROOM.idFromName(code));

  it('publicação: todo mundo volta com a mesma identidade e ninguém vê saída nem entrada', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    a.send({ t: 'live', resolution: 1080, fps: 60, audio: 'none' });
    const { c: b, welcome: wb } = await Client.join(code, 'Lu');
    const { c: c, welcome: wc } = await Client.join(code, 'Bia');
    await a.next('peer-joined');
    await a.next('peer-joined');
    await b.next('peer-joined');

    // Reinício do DO como numa publicação: sockets cortados sem webSocketClose na instância antiga.
    await evictDurableObject(stubOf(code), { webSockets: 'close' });
    await Promise.all([a, b, c].map((x) => x.waitClose()));

    const back = await Promise.all(
      [wa, wb, wc].map(async (w) => {
        const x = await Client.open(code);
        x.send({ t: 'resume', peerId: w.you.id, resumeToken: w.resumeToken });
        return { x, welcome: await x.next('welcome') };
      }),
    );
    expect(back.map((r) => r.welcome.you.id)).toEqual([wa.you.id, wb.you.id, wc.you.id]);
    expect(back[0].welcome.you.live).toMatchObject({ resolution: 1080, fps: 60 });
    expect(back[1].welcome.peers.map((p) => p.id).sort()).toEqual([wa.you.id, wc.you.id].sort());
    await sleep(GRACE + 300);
    for (const { x } of back) {
      expect(x.has('peer-left')).toBe(false);
      expect(x.has('peer-joined')).toBe(false);
    }
  });

  it('publicação e ninguém volta: a sala rápida espera HOLD_MS e depois some', async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Rafa');
    await evictDurableObject(stubOf(code), { webSockets: 'close' });
    await a.waitClose();
    await sleep(GRACE + 150);
    expect((await call(`/api/rooms/${code}`)).status).toBe(200);
    await sleep(HOLD + 400);
    expect((await call(`/api/rooms/${code}`)).status).toBe(404);
  });

  it('todos caem e um volta depois da tolerância, antes de HOLD_MS: mesma sala, como pessoa nova', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    a.ws.close(1000);
    b.ws.close(1000);
    await sleep(GRACE + 150);
    const late = await Client.open(code);
    late.send({ t: 'resume', peerId: wa.you.id, resumeToken: wa.resumeToken });
    await late.next('resume-failed');
    late.send({ t: 'join', nickname: 'Rafa' });
    const again = await late.next('welcome');
    expect(again.you.id).not.toBe(wa.you.id);
    expect(again.you.nickname).toBe('Rafa');
  });

  it('todos saem com "Sair da sala": a sala some na hora, sem esperar', async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    a.ws.close(CLOSE.leave, 'saiu');
    b.ws.close(CLOSE.leave, 'saiu');
    await sleep(150);
    expect((await call(`/api/rooms/${code}`)).status).toBe(404);
  });

  it('um cai e o outro sai: a sala espera quem caiu', async () => {
    const code = await createRoom();
    const { c: a, welcome: wa } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    a.ws.close(1000);
    await sleep(80);
    b.ws.close(CLOSE.leave, 'saiu');
    await sleep(GRACE + 150);
    expect((await call(`/api/rooms/${code}`)).status).toBe(200);
    const late = await Client.open(code);
    late.send({ t: 'join', nickname: 'Rafa' });
    expect((await late.next('welcome')).you.id).not.toBe(wa.you.id);
  });
});

describe('Entrada simultânea', () => {
  it('duas pessoas entrando juntas se veem exatamente uma vez (sem duplicar nem faltar)', async () => {
    turnDelay = 200;
    const code = await createRoom();
    await Client.join(code, 'Rafa');
    const [b, c] = await Promise.all([Client.open(code), Client.open(code)]);
    b.send({ t: 'join', nickname: 'Lu' });
    c.send({ t: 'join', nickname: 'Bia' });
    const [wb, wc] = await Promise.all([b.next('welcome'), c.next('welcome')]);
    await sleep(300);
    const sightings = (me: Client, welcome: typeof wb, other: string) =>
      welcome.peers.filter((p) => p.id === other).length +
      me.inbox.filter((m) => m.t === 'peer-joined' && m.peer.id === other).length;
    expect(sightings(b, wb, wc.you.id)).toBe(1);
    expect(sightings(c, wc, wb.you.id)).toBe(1);
  });
});

describe('STUN-only',()=>{
 it('recusa attachment legado antes de processar refresh',async()=>{
  const code=await createRoom();const {c}=await Client.join(code,'Rafa');
  await runInDurableObject(env.ROOM.get(env.ROOM.idFromName(code)),async(_instance,state)=>{
   for(const ws of state.getWebSockets()) {
    const att:unknown=ws.deserializeAttachment();
    if(typeof att==='object'&&att!==null&&'member' in att&&typeof att.member==='object'&&att.member!==null&&'mediaProtocol' in att.member) {
     att.member.mediaProtocol=1;ws.serializeAttachment(att);
    }
   }
  });
  c.send({t:'turn-refresh'});expect((await c.waitClose())?.code).toBe(4426);
 });
 it('welcome, refresh e retomada não geram TURN',async()=>{
  const code=await createRoom();const {c,welcome}=await Client.join(code,'Rafa');
  expect(welcome.mediaPolicy).toEqual({kind:'direct-only',reason:'unverified-budget'});
  expect(welcome.iceServers).toEqual([{urls:'stun:stun.cloudflare.com:3478'}]);
  c.send({t:'turn-refresh'});expect((await c.next('ice-servers')).iceServers).toEqual(welcome.iceServers);
  expect(globalThis.fetch).not.toHaveBeenCalled();
  const back=await Client.open(code);back.send({t:'resume',peerId:welcome.you.id,resumeToken:welcome.resumeToken});
  expect((await back.next('welcome')).iceServers).toEqual(welcome.iceServers);
 });
 it('cliente sem versão é fechado antes da entrada',async()=>{
  const c=await Client.open(await createRoom());c.send(JSON.stringify({t:'join',nickname:'Antigo'}));
  expect((await c.waitClose())?.code).toBe(4426);
 });
});

describe('Abuso', () => {
  it('mensagem > 64 KB fecha com 1008', async () => {
    const code = await createRoom();
    const { c } = await Client.join(code, 'Rafa');
    c.send(JSON.stringify({ t: 'signal', to: '0123456789abcdef', data: { candidate: { x: 'a'.repeat(70_000) } } }));
    expect((await c.waitClose())?.code).toBe(CLOSE.policy);
  });

  it('flood (> 200 mensagens em 10 s) fecha com 1008', async () => {
    const code = await createRoom();
    const { c } = await Client.join(code, 'Rafa');
    for (let i = 0; i < 210; i++) c.send({ t: 'turn-refresh' });
    expect((await c.waitClose())?.code).toBe(CLOSE.policy);
  });

  it('mensagens inválidas repetidas fecham com 1008', async () => {
    const code = await createRoom();
    const { c } = await Client.join(code, 'Rafa');
    for (let i = 0; i < 7; i++) c.send('{"t":"lixo"}');
    expect((await c.waitClose())?.code).toBe(CLOSE.policy);
  });

  it('apelido vazio ou longo demais é recusado', async () => {
    const code = await createRoom();
    const c = await Client.open(code);
    c.send({ t: 'join', nickname: '‮  ' });
    expect((await c.next('error')).code).toBe('bad-message');
    c.send({ t: 'join', nickname: 'x'.repeat(25) });
    expect((await c.next('error')).code).toBe('bad-message');
  });
});

describe('Sala fixa (specs/011)', () => {
  const TTL = 1500;

  async function createFixed(name: string) {
    return call('/api/rooms', {
      method: 'POST',
      headers: { ...ORIGIN, 'Content-Type': 'application/json' },
      body: JSON.stringify({ kind: 'fixed', name }),
    });
  }

  it('cria com nome: devolve o código e a chave do dono (64 hex) uma vez', async () => {
    const res = await createFixed('  CS   da galera ');
    expect(res.status).toBe(201);
    const body = (await res.json()) as { code: string; ownerKey: string };
    expect(body.ownerKey).toMatch(/^[0-9a-f]{64}$/);
    const preview = (await (await call(`/api/rooms/${body.code}`)).json()) as { fixed?: { name: string } };
    expect(preview.fixed).toEqual({ name: 'CS da galera' });
  });

  it('nome vazio, longo demais ou com caractere invisível: 400', async () => {
    expect((await createFixed('   ')).status).toBe(400);
    expect((await createFixed('a'.repeat(41))).status).toBe(400);
    expect((await createFixed('a‮b')).status).toBe(400);
    const junk = await call('/api/rooms', { method: 'POST', headers: ORIGIN, body: '{"kind":"vip"}' });
    expect(junk.status).toBe(400);
  });

  it('corpo vazio continua criando sala rápida, sem chave', async () => {
    const res = await call('/api/rooms', { method: 'POST', headers: ORIGIN });
    expect(await res.json()).not.toHaveProperty('ownerKey');
  });

  it('continua existindo vazia depois da tolerância (sem a espera da rápida); welcome diz que é fixa', async () => {
    const { code } = (await (await createFixed('CS da galera')).json()) as { code: string };
    const { c: a, welcome } = await Client.join(code, 'Rafa');
    expect(welcome.room).toEqual({ kind: 'fixed', name: 'CS da galera' });
    a.ws.close(1000);
    // Depois da tolerância, sem esperar HOLD_MS: sala fixa não precisa da espera (o TTL de teste é curto).
    await sleep(GRACE + 300);
    const res = await call(`/api/rooms/${code}`);
    expect(res.status).toBe(200);
    expect(((await res.json()) as { people: unknown[] }).people).toEqual([]);
  });

  it('sala rápida: welcome diz que é rápida', async () => {
    const { welcome } = await Client.join(await createRoom(), 'Rafa');
    expect(welcome.room).toEqual({ kind: 'quick' });
  });

  it('sem ninguém entrar por FIXED_TTL_MS: expira (410), o link recusa e a lápide some depois', async () => {
    const { code } = (await (await createFixed('Filme de sexta')).json()) as { code: string };
    await sleep(TTL + 400);
    const res = await call(`/api/rooms/${code}`);
    expect(res.status).toBe(410);
    expect(await res.json()).toEqual({ reason: 'expired' });
    const c = await Client.open(code);
    c.send({ t: 'join', nickname: 'Rafa' });
    await c.next('room-not-found');
    // Código com lápide não é sorteado de novo.
    expect(await env.ROOM.get(env.ROOM.idFromName(code)).reserve()).toBe(false);
    await sleep(TTL + 400);
    expect((await call(`/api/rooms/${code}`)).status).toBe(404);
  });

  it('entrar renova o prazo; consultar a prévia não', async () => {
    const { code } = (await (await createFixed('Trampo do TCC')).json()) as { code: string };
    await sleep(TTL - 600);
    const { c } = await Client.join(code, 'Rafa');
    c.ws.close(CLOSE.leave, 'saiu');
    await sleep(TTL - 600);
    expect((await call(`/api/rooms/${code}`)).status).toBe(200);
    await sleep(800);
    expect((await call(`/api/rooms/${code}`)).status).toBe(410);
  });

  it('não guarda apelidos depois que a sala fixa esvazia', async () => {
    const { code } = (await (await createFixed('Família')).json()) as { code: string };
    const { c } = await Client.join(code, 'Rafa');
    c.ws.close(CLOSE.leave, 'saiu');
    await sleep(150);
    const keys = await runInDurableObject(env.ROOM.get(env.ROOM.idFromName(code)), async (_i, state) =>
      [...(await state.storage.list()).keys()].sort(),
    );
    expect(keys).toEqual(['fixed']);
  });
});

describe('Suas salas: status (specs/011)', () => {
  function status(codes: unknown, headers: Record<string, string> = ORIGIN) {
    return call('/api/rooms/status', { method: 'POST', headers, body: JSON.stringify({ codes }) });
  }
  async function fixedRoom(name: string) {
    const res = await call('/api/rooms', { method: 'POST', headers: ORIGIN, body: JSON.stringify({ kind: 'fixed', name }) });
    return ((await res.json()) as { code: string }).code;
  }

  it('só números e o nome da sala fixa, nunca apelidos', async () => {
    const fixed = await fixedRoom('CS da galera');
    const { c: a } = await Client.join(fixed, 'Rafaela');
    a.send({ t: 'live', resolution: 1080, fps: 60, audio: 'none' });
    await Client.join(fixed, 'Luana');
    const quick = await createRoom();
    await Client.join(quick, 'Tomás');
    await sleep(100);

    const res = await status([fixed, quick, 'zzz-zzz']);
    expect(res.status).toBe(200);
    const text = await res.text();
    expect(JSON.parse(text)).toEqual({
      rooms: [
        { code: fixed, state: 'ok', name: 'CS da galera', people: 2, streaming: 1 },
        { code: quick, state: 'ok', name: null, people: 1, streaming: 0 },
        { code: 'zzz-zzz', state: 'notfound' },
      ],
    });
    for (const nick of ['Rafaela', 'Luana', 'Tomás']) expect(text).not.toContain(nick);
  });

  it('lista inválida: mais de 5, repetido, código ruim ou vazia → 400', async () => {
    const six = ['aaa-aaa', 'bbb-bbb', 'ccc-ccc', 'ddd-ddd', 'eee-eee', 'fff-fff'];
    expect((await status(six)).status).toBe(400);
    expect((await status(['aaa-aaa', 'aaa-aaa'])).status).toBe(400);
    expect((await status(['nada'])).status).toBe(400);
    expect((await status([])).status).toBe(400);
  });

  it('de outro site → 403', async () => {
    expect((await status(['aaa-aaa'], { Origin: 'https://outro.site' })).status).toBe(403);
  });

  it('sala fixa expirada → gone; consultar não renova o prazo', async () => {
    const code = await fixedRoom('Minecraft do Caio');
    for (let i = 0; i < 4; i++) {
      await sleep(400);
      await status([code]);
    }
    await sleep(300);
    expect(((await (await status([code])).json()) as { rooms: unknown[] }).rooms).toEqual([
      { code, state: 'gone', reason: 'expired' },
    ]);
  });
});

describe('Dono (specs/011)', () => {
  async function fixedRoom(name: string) {
    const res = await call('/api/rooms', { method: 'POST', headers: ORIGIN, body: JSON.stringify({ kind: 'fixed', name }) });
    return (await res.json()) as { code: string; ownerKey: string };
  }
  const auth = (key: string) => ({ ...ORIGIN, Authorization: `Bearer ${key}` });
  const rename = (code: string, name: string, headers: Record<string, string>) =>
    call(`/api/rooms/${code}`, { method: 'PATCH', headers, body: JSON.stringify({ name }) });
  const remove = (code: string, headers: Record<string, string>) => call(`/api/rooms/${code}`, { method: 'DELETE', headers });

  it('sem chave, com chave errada ou em sala rápida → 403; de outro site → 403', async () => {
    const { code } = await fixedRoom('CS da galera');
    expect((await rename(code, 'Outro', ORIGIN)).status).toBe(403);
    expect((await rename(code, 'Outro', auth('0'.repeat(64)))).status).toBe(403);
    expect((await remove(code, auth('f'.repeat(64)))).status).toBe(403);
    const quick = await createRoom();
    expect((await remove(quick, auth('0'.repeat(64)))).status).toBe(403);
    const { code: c2, ownerKey } = await fixedRoom('Família');
    expect((await remove(c2, { Origin: 'https://outro.site', Authorization: `Bearer ${ownerKey}` })).status).toBe(403);
  });

  it('renomear: quem está dentro recebe room-renamed; prévia e status mostram o nome novo; link igual', async () => {
    const { code, ownerKey } = await fixedRoom('CS da galera');
    const { c } = await Client.join(code, 'Lu');
    const res = await rename(code, '  CS da galera 2 ', auth(ownerKey));
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ name: 'CS da galera 2' });
    expect((await c.next('room-renamed')).name).toBe('CS da galera 2');
    const preview = (await (await call(`/api/rooms/${code}`)).json()) as { fixed: { name: string } };
    expect(preview.fixed.name).toBe('CS da galera 2');
    expect((await rename(code, '', auth(ownerKey))).status).toBe(400);
  });

  it('apagar com gente dentro: room-deleted + 4410, TURN revogado, link vira 410 deleted', async () => {
    const { code, ownerKey } = await fixedRoom('CS da galera');
    const { c: a } = await Client.join(code, 'Rafa');
    const { c: b } = await Client.join(code, 'Lu');
    revoked.length = 0;
    expect((await remove(code, auth(ownerKey))).status).toBe(204);
    await a.next('room-deleted');
    await b.next('room-deleted');
    expect((await a.waitClose())?.code).toBe(CLOSE.deleted);
    expect((await b.waitClose())?.code).toBe(CLOSE.deleted);
    expect(revoked).toEqual([]);
    // Sobra só a lápide: nenhum fantasma (com apelido) recriado pelo fechamento das conexões.
    await sleep(150);
    const keys = await runInDurableObject(env.ROOM.get(env.ROOM.idFromName(code)), async (_i, state) =>
      [...(await state.storage.list()).keys()],
    );
    expect(keys).toEqual(['tombstone']);
    const res = await call(`/api/rooms/${code}`);
    expect(res.status).toBe(410);
    expect(await res.json()).toEqual({ reason: 'deleted' });
    const late = await Client.open(code);
    late.send({ t: 'join', nickname: 'Bia' });
    await late.next('room-not-found');
    // Apagar de novo: já acabou.
    expect((await remove(code, auth(ownerKey))).status).toBe(410);
  });
});

describe('Remover pessoa (specs/014 US4)', () => {
  async function fixedRoom(name: string) {
    const res = await call('/api/rooms', { method: 'POST', headers: ORIGIN, body: JSON.stringify({ kind: 'fixed', name }) });
    return (await res.json()) as { code: string; ownerKey: string };
  }
  const auth = (key: string) => ({ ...ORIGIN, Authorization: `Bearer ${key}` });
  const removePerson = (code: string, peerId: unknown, headers: Record<string, string>) =>
    call(`/api/rooms/${code}/remove`, { method: 'POST', headers, body: JSON.stringify({ peerId }) });

  it('sem a chave certa, em sala rápida ou de outro site: 403, e ninguém sai', async () => {
    const { code, ownerKey } = await fixedRoom('CS da galera');
    const { c: lu, welcome } = await Client.join(code, 'Lu');
    expect((await removePerson(code, welcome.you.id, ORIGIN)).status).toBe(403);
    expect((await removePerson(code, welcome.you.id, auth('0'.repeat(64)))).status).toBe(403);
    expect((await removePerson(code, welcome.you.id, { Origin: 'https://outro.site', Authorization: `Bearer ${ownerKey}` })).status).toBe(403);
    const quick = await createRoom();
    const { welcome: q } = await Client.join(quick, 'Bia');
    expect((await removePerson(quick, q.you.id, auth(ownerKey))).status).toBe(403);
    await sleep(100);
    expect(lu.closed).toBeNull();
  });

  it('corpo inválido: 400; método errado: 405', async () => {
    const { code, ownerKey } = await fixedRoom('CS da galera');
    expect((await removePerson(code, 'nao-e-id', auth(ownerKey))).status).toBe(400);
    expect((await removePerson(code, 42, auth(ownerKey))).status).toBe(400);
    expect((await call(`/api/rooms/${code}/remove`, { method: 'GET', headers: auth(ownerKey) })).status).toBe(405);
  });

  it('conectado: recebe removed + 4403, os outros veem peer-left, TURN revogado, não volta sozinho; entra de novo pelo link', async () => {
    const { code, ownerKey } = await fixedRoom('CS da galera');
    const { c: dono } = await Client.join(code, 'Rafa');
    const { c: lu, welcome } = await Client.join(code, 'Lu');
    await dono.next('peer-joined');
    revoked.length = 0;
    expect((await removePerson(code, welcome.you.id, auth(ownerKey))).status).toBe(204);
    await lu.next('removed');
    expect((await lu.waitClose())?.code).toBe(CLOSE.removed);
    expect((await dono.next('peer-left')).id).toBe(welcome.you.id);
    expect(revoked).toEqual([]);
    // Retomar o lugar (o que a queda faz) não funciona: saiu de verdade.
    const back = await Client.open(code);
    back.send({ t: 'resume', peerId: welcome.you.id, resumeToken: welcome.resumeToken });
    await back.next('resume-failed');
    // Mas abrir o link de novo entra normalmente, como pessoa nova (nada fica guardado).
    back.send({ t: 'join', nickname: 'Lu' });
    const again = await back.next('welcome');
    expect(again.you.id).not.toBe(welcome.you.id);
    expect((await (await call(`/api/rooms/${code}`)).json()) as { people: unknown[] }).toMatchObject({ people: [{}, {}] });
  });

  it('em queda (fantasma): some da sala e não consegue retomar', async () => {
    const { code, ownerKey } = await fixedRoom('CS da galera');
    const { c: dono } = await Client.join(code, 'Rafa');
    const { c: lu, welcome } = await Client.join(code, 'Lu');
    lu.ws.close(4999, 'caiu');
    await sleep(100);
    expect((await removePerson(code, welcome.you.id, auth(ownerKey))).status).toBe(204);
    expect((await dono.next('peer-left')).id).toBe(welcome.you.id);
    const back = await Client.open(code);
    back.send({ t: 'resume', peerId: welcome.you.id, resumeToken: welcome.resumeToken });
    await back.next('resume-failed');
  });

  it('quem não está na sala: 204 e nada muda', async () => {
    const { code, ownerKey } = await fixedRoom('CS da galera');
    const { c: dono } = await Client.join(code, 'Rafa');
    expect((await removePerson(code, 'abcdef0123456789', auth(ownerKey))).status).toBe(204);
    await sleep(100);
    expect(dono.has('peer-left')).toBe(false);
  });
});

describe('Descrição da transmissão (specs/012)', () => {
  const live = (label?: unknown) => ({ t: 'live', resolution: 1080, fps: 60, audio: 'window', ...(label !== undefined ? { label } : {}) });

  it('vai pros outros e pra quem entra depois', async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Lu');
    const { c: b } = await Client.join(code, 'Rafa');
    a.send(live('Hollow Knight'));
    expect((await b.next('peer-live')).live?.label).toBe('Hollow Knight');
    const { welcome } = await Client.join(code, 'Bia');
    expect(welcome.peers.find((p) => p.nickname === 'Lu')?.live?.label).toBe('Hollow Knight');
  });

  it('limpa caracteres invisíveis; texto ruim não derruba o "ao vivo"', async () => {
    const code = await createRoom();
    const { c: a } = await Client.join(code, 'Lu');
    const { c: b } = await Client.join(code, 'Rafa');
    a.send(live('Hollow‮ Knight'));
    expect((await b.next('peer-live')).live?.label).toBe('Hollow Knight');
    a.send(live('x'.repeat(61)));
    const long = (await b.next('peer-live')).live!;
    expect(long.resolution).toBe(1080);
    expect(long).not.toHaveProperty('label');
    a.send(live(42));
    expect((await b.next('peer-live')).live).not.toHaveProperty('label');
    a.send(live());
    expect((await b.next('peer-live')).live).not.toHaveProperty('label');
  });

  it('não aparece na prévia nem em Suas salas', async () => {
    const res = await call('/api/rooms', { method: 'POST', headers: ORIGIN, body: JSON.stringify({ kind: 'fixed', name: 'CS' }) });
    const { code } = (await res.json()) as { code: string };
    const { c: a } = await Client.join(code, 'Lu');
    a.send(live('Hollow Knight'));
    await sleep(100);
    expect(await (await call(`/api/rooms/${code}`)).text()).not.toContain('Hollow');
    const status = await call('/api/rooms/status', { method: 'POST', headers: ORIGIN, body: JSON.stringify({ codes: [code] }) });
    expect(await status.text()).not.toContain('Hollow');
  });
});


describe('Ordenação de capturas v2 no Durable Object real',()=>{
 it('não reanuncia captura encerrada nem aceita stop antigo, inclusive após reinício',async()=>{
  const code=await createRoom();const {c:a,welcome:wa}=await Client.join(code,'Rafa');
  const packet={t:'live',requestId:crypto.randomUUID(),broadcastSeq:0,resolution:720,fps:30,audio:'none'};
  a.send(JSON.stringify(packet));const first=await a.next('live-ack');
  a.send(JSON.stringify({t:'unlive',requestId:packet.requestId,broadcastSeq:packet.broadcastSeq}));
  a.send(JSON.stringify(packet));expect((await a.next('error')).code).toBe('bad-message');
  const second={...packet,requestId:crypto.randomUUID(),broadcastSeq:1};
  a.send(JSON.stringify(second));const next=await a.next('live-ack');expect(next.requestId).toBe(second.requestId);expect(next.live.startedAt).toBeGreaterThan(first.live.startedAt);
  a.send(JSON.stringify({t:'unlive',requestId:packet.requestId,broadcastSeq:packet.broadcastSeq}));expect((await a.next('error')).code).toBe('bad-message');
  await evictDurableObject(env.ROOM.get(env.ROOM.idFromName(code)),{webSockets:'close'});await a.waitClose();
  const back=await Client.open(code);back.send({t:'resume',peerId:wa.you.id,resumeToken:wa.resumeToken});
  const resumed=await back.next('welcome');expect(resumed.broadcastOrder?.highestBroadcastSeq).toBe(1);
  back.send(JSON.stringify(packet));expect((await back.next('error')).code).toBe('bad-message');
  back.send(JSON.stringify(second));expect((await back.next('live-ack')).live.startedAt).toBe(next.live.startedAt);
 });
});
