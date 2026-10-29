// Worker do Espia: API das salas e WebSocket (contracts/http-api.md). O resto é o site em dist/.
import { generateRoomId, ROOM_ID_RE } from '../lib/room-id';
import type { RoomPreview } from '../shared/protocol';
import { Room } from './room';

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
    // O código nasce no servidor. Colisão é raríssima, mas tenta de novo se acontecer.
    for (let i = 0; i < 5; i++) {
      const code = generateRoomId();
      if (await room(env, code).reserve()) return json({ code }, 201);
    }
    return json({ error: 'busy' }, 503);
  }

  const api = pathname.match(/^\/api\/rooms\/([^/]+)$/);
  if (api) {
    if (request.method !== 'GET') return json({ error: 'method' }, 405);
    const code = api[1];
    if (!ROOM_ID_RE.test(code)) return json({ error: 'code' }, 400);
    if (!(await allowed(env.RL_LOOKUP, request))) return tooMany();
    const preview: RoomPreview | null = await room(env, code).preview(code);
    return preview ? json(preview) : json({ error: 'not-found' }, 404);
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
