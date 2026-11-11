// Rotas da SPA como dados puros (sem History API), para o router e os testes.
import { ROOM_ID_RE } from './room-id';

export type Route = { name: 'home' } | { name: 'room'; code: string };

export function parseRoute(pathname: string): Route {
  const m = pathname.match(/^\/s\/([^/]+)\/?$/);
  if (m && ROOM_ID_RE.test(m[1])) return { name: 'room', code: m[1] };
  return { name: 'home' };
}

/** Mesma tela: um "voltar" que só fecha a tela cheia (specs/018 R3) não pode remontar nada. */
export function sameRoute(a: Route, b: Route): boolean {
  return a.name === b.name && (a.name !== 'room' || (b.name === 'room' && a.code === b.code));
}
