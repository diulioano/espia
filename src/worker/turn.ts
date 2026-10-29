// Credenciais do Cloudflare Realtime TURN (security.md A7). Só o Worker conhece o token da API;
// o navegador recebe credenciais temporárias dentro do `welcome` e do `ice-servers`.
import type { IceServer } from '../shared/protocol';

const API = 'https://rtc.live.cloudflare.com/v1/turn/keys';
export const TURN_TTL_S = 6 * 60 * 60;
const STUN_ONLY: IceServer[] = [{ urls: 'stun:stun.cloudflare.com:3478' }];

export interface TurnGrant {
  iceServers: IceServer[];
  iceExpiresAt: number;
  /** Para revogar quando a pessoa sair. Ausente quando caiu no STUN puro. */
  username?: string;
}

type TurnEnv = Pick<Env, 'TURN_KEY_ID' | 'TURN_KEY_API_TOKEN'>;

/** Gera credenciais novas. Se o TURN falhar ou não estiver configurado, devolve só STUN (o direto ainda funciona). */
export async function mintIceServers(env: TurnEnv): Promise<TurnGrant> {
  const iceExpiresAt = Date.now() + TURN_TTL_S * 1000;
  if (!env.TURN_KEY_ID || !env.TURN_KEY_API_TOKEN) return { iceServers: STUN_ONLY, iceExpiresAt };
  try {
    const res = await fetch(`${API}/${env.TURN_KEY_ID}/credentials/generate-ice-servers`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${env.TURN_KEY_API_TOKEN}`, 'Content-Type': 'application/json' },
      body: JSON.stringify({ ttl: TURN_TTL_S }),
    });
    if (!res.ok) throw new Error(`TURN ${res.status}`);
    const body = (await res.json()) as { iceServers: IceServer | IceServer[] };
    const servers = (Array.isArray(body.iceServers) ? body.iceServers : [body.iceServers]).map(withoutPort53);
    const username = servers.find((s) => s.username)?.username;
    return { iceServers: servers, iceExpiresAt, username };
  } catch (err) {
    console.error('[turn] falhou ao gerar credenciais, usando só STUN:', (err as Error).message);
    return { iceServers: STUN_ONLY, iceExpiresAt };
  }
}

/** Revoga credenciais de quem saiu, para ninguém usar nosso TURN depois. Erros só são logados. */
export async function revokeTurn(env: TurnEnv, usernames: string[]) {
  if (!env.TURN_KEY_ID || !env.TURN_KEY_API_TOKEN) return;
  await Promise.allSettled(
    usernames.map(async (u) => {
      const res = await fetch(`${API}/${env.TURN_KEY_ID}/credentials/${encodeURIComponent(u)}/revoke`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${env.TURN_KEY_API_TOKEN}` },
      });
      if (!res.ok) console.error('[turn] revogação falhou:', res.status);
    }),
  );
}

/** Navegadores bloqueiam a porta 53; a Cloudflare recomenda tirar essas URLs. */
function withoutPort53(server: IceServer): IceServer {
  const urls = (Array.isArray(server.urls) ? server.urls : [server.urls]).filter((u) => !/:53(\?|$)/.test(u));
  return { ...server, urls };
}
