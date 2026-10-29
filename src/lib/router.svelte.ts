// Rotas da SPA: "/" e "/s/:código". History API, sem biblioteca.
import { ROOM_ID_RE } from './room-id';

export type Route = { name: 'home' } | { name: 'room'; code: string };

/** Estado passado entre telas na navegação (não vai para a URL). */
export interface NavState {
  /** Apelido já escolhido: pula a tela de entrada e vai direto pra sala. */
  nickname?: string;
}

function parse(pathname: string): Route {
  const m = pathname.match(/^\/s\/([^/]+)\/?$/);
  if (m && ROOM_ID_RE.test(m[1])) return { name: 'room', code: m[1] };
  return { name: 'home' };
}

class Router {
  route = $state<Route>(parse(location.pathname));
  state = $state<NavState>((history.state as NavState | null) ?? {});
  /** Tela atual pode segurar a saída (ex.: transmitindo): devolve a pergunta, ou null pra sair direto. */
  leaveGuard: (() => string | null) | null = null;

  constructor() {
    // Endereço desconhecido vira "/".
    if (this.route.name === 'home' && location.pathname !== '/') history.replaceState(null, '', '/' + location.search);
    addEventListener('popstate', () => {
      this.route = parse(location.pathname);
      this.state = (history.state as NavState | null) ?? {};
    });
  }

  go(path: string, state: NavState = {}, replace = false) {
    if (replace) history.replaceState(state, '', path);
    else history.pushState(state, '', path);
    this.route = parse(path);
    this.state = state;
  }
}

export const router = new Router();
