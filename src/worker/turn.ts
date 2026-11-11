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

type TurnEnv = { [K in 'TURN_KEY_ID' | 'TURN_KEY_API_TOKEN']?: string };

/** Sem limite financeiro comprovado: nunca emite credenciais de retransmissão. */
export async function mintIceServers(_env: TurnEnv): Promise<TurnGrant> {
  return { iceServers: STUN_ONLY, iceExpiresAt: Date.now() + TURN_TTL_S * 1000 };
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
