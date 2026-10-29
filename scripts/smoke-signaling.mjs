// Teste de fumaça da sinalização contra um servidor rodando (wrangler dev ou o deploy).
// Uso: node scripts/smoke-signaling.mjs [http://127.0.0.1:8787]
// Leva ~80 s: espera a sala hibernar e usa a tolerância real de 20 s.

const base = new URL(process.argv[2] ?? 'http://127.0.0.1:8787');
const origin = base.origin;
const wsBase = origin.replace(/^http/, 'ws');
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const t0 = Date.now();
const log = (msg) => console.log(`[${((Date.now() - t0) / 1000).toFixed(1).padStart(5)}s] ${msg}`);

function client(code, name) {
  const ws = new WebSocket(`${wsBase}/ws/${code}`, { headers: { Origin: origin } });
  const inbox = [];
  const c = {
    ws,
    inbox,
    closed: null,
    send: (m) => ws.send(typeof m === 'string' ? m : JSON.stringify(m)),
    async next(t, timeout = 5000) {
      const end = Date.now() + timeout;
      while (Date.now() < end) {
        const i = inbox.findIndex((m) => m.t === t);
        if (i >= 0) return inbox.splice(i, 1)[0];
        await sleep(25);
      }
      throw new Error(`${name}: esperava "${t}", caixa: ${JSON.stringify(inbox)}`);
    },
    has: (t) => inbox.some((m) => m.t === t),
  };
  ws.addEventListener('message', (e) => inbox.push(JSON.parse(e.data)));
  ws.addEventListener('close', (e) => (c.closed = e.code));
  return new Promise((resolve, reject) => {
    ws.addEventListener('open', () => resolve(c), { once: true });
    ws.addEventListener('error', () => reject(new Error(`${name}: não conectou`)), { once: true });
  });
}

function check(cond, msg) {
  if (!cond) throw new Error(`FALHOU: ${msg}`);
  log(`ok  ${msg}`);
}

const res = await fetch(`${origin}/api/rooms`, { method: 'POST', headers: { Origin: origin } });
check(res.status === 201, `criar sala (HTTP ${res.status})`);
const { code } = await res.json();
log(`sala ${code}`);

const a = await client(code, 'Rafa');
a.send({ t: 'join', nickname: 'Rafa' });
const wa = await a.next('welcome');
const b = await client(code, 'Lu');
b.send({ t: 'join', nickname: 'Lu' });
const wb = await b.next('welcome');
check(wb.peers[0]?.id === wa.you.id, 'Lu vê Rafa na sala');
check((await a.next('peer-joined')).peer.id === wb.you.id, 'Rafa recebe "Lu entrou"');

log('parado 30 s (a sala pode hibernar)…');
await sleep(30_000);
check(a.closed === null && b.closed === null, 'as duas conexões continuam abertas');
a.send('{"t":"ping"}');
await a.next('pong');
check(true, 'ping → pong');
a.send({ t: 'signal', to: wb.you.id, data: { candidate: null } });
check((await b.next('signal')).from === wa.you.id, 'repasse Rafa → Lu funciona depois da hibernação');
a.send({ t: 'live', resolution: 1080, fps: 60, audio: true });
check((await b.next('peer-live')).live?.fps === 60, 'Lu vê Rafa ao vivo');

log('Rafa cai (internet piscou)…');
a.ws.close();
await sleep(5_000);
const a2 = await client(code, 'Rafa (de volta)');
a2.send({ t: 'resume', peerId: wa.you.id, resumeToken: wa.resumeToken });
const again = await a2.next('welcome');
check(again.you.id === wa.you.id && again.you.live?.fps === 60, 'Rafa volta em 5 s com a mesma identidade e ainda ao vivo');
log('esperando 25 s para ver se Lu recebe "saiu"…');
await sleep(25_000);
check(!b.has('peer-left'), 'Lu não viu Rafa sair');

log('Rafa cai e não volta…');
const dropped = Date.now();
a2.ws.close();
const left = await b.next('peer-left', 30_000);
const secs = (Date.now() - dropped) / 1000;
check(left.id === wa.you.id && secs >= 19 && secs <= 25, `Lu recebe "Rafa saiu" depois de ${secs.toFixed(1)} s`);

b.ws.close(4001);
log('tudo certo ✔');
process.exit(0);
