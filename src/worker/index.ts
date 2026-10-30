// Worker do Espia: API das salas e WebSocket (contracts/http-api.md). O resto é o site em dist/.
import { generateRoomId, ROOM_ID_RE } from '../lib/room-id';
import type { RoomPreview, RoomStatus } from '../shared/protocol';
import { randomHex, sha256Hex } from './crypto';
import { Room, type OwnerResult } from './room';
import { parseCreateBody, parseRenameBody, parseStatusBody } from './validate';

export { Room };

// security.md §5. Os assets estáticos recebem os mesmos via public/_headers.
const SECURITY_HEADERS: Record<string, string> = {
  'Content-Security-Policy':
    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data:; " +
    "media-src 'self' blob:; connect-src 'self' http://ipc.localhost; frame-ancestors 'self'; base-uri 'none'; form-action 'self'; object-src 'none'",
  'Referrer-Policy': 'no-referrer',
  'X-Content-Type-Options': 'nosniff',
  'Permissions-Policy':
    'display-capture=(self), camera=(self), microphone=(self), geolocation=(), picture-in-picture=(self), fullscreen=(self)',
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Strict-Transport-Security': 'max-age=31536000; includeSubDomains',
};

export default {
  async fetch(request, env): Promise<Response> {
    const url = new URL(request.url);
    const res = await route(request, env, url);
    if (res.status === 101) return res;
    const out = new Response(res.body, res);
    for (const [k, v] of Object.entries(SECURITY_HEADERS)) out.headers.set(k, v);
    return out;
  },
} satisfies ExportedHandler<Env>;

async function route(request: Request, env: Env, url: URL): Promise<Response> {
  const { pathname } = url;

  if (pathname === '/api/rooms') {
    if (request.method !== 'POST') return json({ error: 'method' }, 405);
    if (!sameOrigin(request, url)) return json({ error: 'origin' }, 403);
    if (!(await allowed(env.RL_CREATE, request))) return tooMany();
    const body = parseCreateBody(await request.text());
    if (!body) return json({ error: 'name' }, 400);
    // Sala fixa (specs/011 R4): a chave do dono vai uma vez só pro aparelho; o DO guarda o hash.
    const ownerKey = body.kind === 'fixed' ? randomHex(32) : null;
    const fixed = body.kind === 'fixed' ? { name: body.name, ownerHash: await sha256Hex(ownerKey!) } : undefined;
    // O código nasce no servidor. Colisão é raríssima, mas tenta de novo se acontecer.
    for (let i = 0; i < 5; i++) {
      const code = generateRoomId();
      if (await room(env, code).reserve(fixed)) return json(ownerKey ? { code, ownerKey } : { code }, 201);
    }
    return json({ error: 'busy' }, 503);
  }

  // Suas salas (specs/011 R7): uma consulta só, conta uma vez no limite; só números, nunca apelidos.
  if (pathname === '/api/rooms/status') {
    if (request.method !== 'POST') return json({ error: 'method' }, 405);
    if (!sameOrigin(request, url)) return json({ error: 'origin' }, 403);
    if (!(await allowed(env.RL_LOOKUP, request))) return tooMany();
    const codes = parseStatusBody(await request.text());
    if (!codes) return json({ error: 'codes' }, 400);
    const rooms: RoomStatus[] = await Promise.all(
      codes.map(async (code) => ({ code, ...(await room(env, code).summary()) })),
    );
    return json({ rooms });
  }

  const api = pathname.match(/^\/api\/rooms\/([^/]+)$/);
  if (api) {
    const code = api[1];
    if (request.method === 'PATCH' || request.method === 'DELETE') return ownerAction(request, env, url, code);
    if (request.method !== 'GET') return json({ error: 'method' }, 405);
    if (!ROOM_ID_RE.test(code)) return json({ error: 'code' }, 400);
    if (!(await allowed(env.RL_LOOKUP, request))) return tooMany();
    const preview = await room(env, code).preview(code);
    if (!preview) return json({ error: 'not-found' }, 404);
    // Sala fixa apagada ou expirada (specs/011 R5): só o motivo.
    if ('gone' in preview) return json({ reason: preview.gone }, 410);
    return json(preview satisfies RoomPreview);
  }

  const ws = pathname.match(/^\/ws\/([^/]+)$/);
  if (ws) {
    if (request.headers.get('Upgrade') !== 'websocket') return json({ error: 'upgrade' }, 426);
    if (!sameOrigin(request, url)) return json({ error: 'origin' }, 403);
    const code = ws[1];
    if (!ROOM_ID_RE.test(code)) return json({ error: 'code' }, 400);
    if (!(await allowed(env.RL_WS, request))) return tooMany();
    return room(env, code).fetch(request);
  }

  if (pathname.startsWith('/api/') || pathname.startsWith('/ws/')) return json({ error: 'not-found' }, 404);
  return env.ASSETS.fetch(request);
}

/** Renomear (PATCH) ou apagar (DELETE) sala fixa, só com a chave do dono (specs/011 US4). */
async function ownerAction(request: Request, env: Env, url: URL, code: string): Promise<Response> {
  if (!sameOrigin(request, url)) return json({ error: 'origin' }, 403);
  if (!ROOM_ID_RE.test(code)) return json({ error: 'code' }, 400);
  if (!(await allowed(env.RL_CREATE, request))) return tooMany();
  const key = request.headers.get('Authorization')?.match(/^Bearer ([0-9a-f]{64})$/)?.[1];
  if (!key) return json({ error: 'owner' }, 403);
  const ownerHash = await sha256Hex(key);

  let result: OwnerResult;
  let name = '';
  if (request.method === 'PATCH') {
    const parsed = parseRenameBody(await request.text());
    if (!parsed) return json({ error: 'name' }, 400);
    name = parsed;
    result = await room(env, code).rename(ownerHash, name);
  } else {
    result = await room(env, code).remove(ownerHash);
  }

  if (result === 'ok') return request.method === 'PATCH' ? json({ name }) : new Response(null, { status: 204 });
  if (result === 'forbidden') return json({ error: 'owner' }, 403);
  if (result === 'notfound') return json({ error: 'not-found' }, 404);
  return json({ reason: result }, 410);
}

function room(env: Env, code: string) {
  return env.ROOM.get(env.ROOM.idFromName(code));
}

/** POST e WebSocket só do próprio site (security.md A6). Em dev, localhost com qualquer porta. */
function sameOrigin(request: Request, url: URL) {
  const origin = request.headers.get('Origin');
  if (!origin) return false;
  let o: URL;
  try {
    o = new URL(origin);
  } catch {
    return false;
  }
  if (o.host === url.host) return true;
  return isLocal(o.hostname) && isLocal(url.hostname);
}

function isLocal(hostname: string) {
  return hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '[::1]';
}

async function allowed(limiter: RateLimit, request: Request) {
  const key = request.headers.get('CF-Connecting-IP') ?? 'local';
  return (await limiter.limit({ key })).success;
}

function tooMany() {
  return json({ error: 'rate-limited' }, 429, { 'Retry-After': '60' });
}

function json(body: unknown, status = 200, headers: Record<string, string> = {}) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json; charset=utf-8', 'Cache-Control': 'no-store', ...headers },
  });
}
