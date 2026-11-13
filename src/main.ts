import { mount } from 'svelte';
import '@fontsource/caprasimo/400.css';
import '@fontsource/figtree/400.css';
import '@fontsource/figtree/600.css';
import '@fontsource/figtree/700.css';
import './ui/tokens.css';
import './ui/base.css';
import App from './App.svelte';
import { installDesktopHook } from './lib/desktop';
import { importMove, leaveUrl } from './lib/origin-move';
import { router } from './lib/router.svelte';
import { LEGACY_HOSTS, OFFICIAL_ORIGIN } from './lib/site-origin';

/** localStorage pode não existir ou lançar (janela anônima, dados bloqueados). */
function deviceStorage(): Storage | null {
  try {
    return localStorage;
  } catch {
    return null;
  }
}

/**
 * Endereço antigo no navegador: vai para o oficial levando os dados do aparelho no fragmento
 * (specs/019, contracts/convite-e-endereco.md). O app instalado só abre o endereço antigo e nunca é
 * levado; as telas de dados falsos (`?mock`) abrem onde estão.
 */
function leaveLegacyAddress(): boolean {
  if (!(LEGACY_HOSTS as readonly string[]).includes(location.hostname)) return false;
  if ('__TAURI_INTERNALS__' in window) return false;
  if (new URLSearchParams(location.search).has('mock')) return false;
  location.replace(leaveUrl(location, deviceStorage()));
  return true;
}

/** No oficial: guarda o que veio do endereço antigo e tira o pacote da barra de endereço. */
function arriveAtOfficialAddress() {
  if (location.origin !== OFFICIAL_ORIGIN || !location.hash.startsWith('#mv=')) return;
  importMove(location.hash, deviceStorage());
  history.replaceState(history.state, '', location.pathname + location.search);
}

function start() {
  arriveAtOfficialAddress();
  installDesktopHook(router);
  return mount(App, { target: document.getElementById('app')! });
}

// Saindo do endereço antigo, a aplicação nem monta: a página já está indo embora.
export default leaveLegacyAddress() ? null : start();
