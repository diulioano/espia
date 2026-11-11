// Rotas da SPA: "/" e "/s/:código". History API, sem biblioteca.
import { parseRoute as parse, sameRoute, type Route } from './route';

export type { Route };

/** Estado passado entre telas na navegação (não vai para a URL). */
export interface NavState {
  /** Apelido já escolhido: pula a tela de entrada e vai direto pra sala. */
  nickname?: string;
  /** Acabou de criar a sala fixa (specs/011 S1): a sala copia o convite e avisa. */
  created?: boolean;
  /** Aviso pra tela inicial (specs/014 US4). */
  notice?: 'removed';
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
      // Mesmo endereço (ex.: o "voltar" que fecha a tela cheia, specs/018): a rota fica a mesma.
      const next = parse(location.pathname);
      if (!sameRoute(next, this.route)) this.route = next;
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
