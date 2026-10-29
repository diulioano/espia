// Gancho do app desktop (src-tauri/src/lib.rs → open_room): um link de sala aberto com o app já
// aberto chama window.espiaOpen("/s/xxx-xxx"). No navegador nada chama isso.
// Bandeja (specs/007, contracts/tray.md): a página avisa quando está ao vivo, e o menu do ícone
// chama window.espiaStopLive().
// Memória (specs/009, contracts/engine.md): a página avisa quando está numa sala, para o app saber se pode
// soltar o motor de página com a janela escondida.
import { invoke } from '@tauri-apps/api/core';
import { ROOM_ID_RE } from './room-id';

type DesktopWindow = Window & { espiaOpen?: (path: string) => void; espiaStopLive?: () => void };

const inApp = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

/** Avisa o app que a transmissão própria começou ou parou (ícone "ao vivo", confirmação ao sair). */
export function reportLive(live: boolean): void {
  if (inApp()) invoke('app_set_live', { live }).catch(() => {});
}

/** Avisa o app que entrou ou saiu de uma sala (escondido fora de sala, o app solta o motor). */
export function reportRoom(inRoom: boolean): void {
  if (inApp()) invoke('app_set_room', { inRoom }).catch(() => {});
}

/** "Parar transmissão" do menu da bandeja. Devolve a função que remove o gancho. */
export function installStopHook(stop: () => void): () => void {
  const w = window as DesktopWindow;
  w.espiaStopLive = stop;
  return () => {
    if (w.espiaStopLive === stop) delete w.espiaStopLive;
  };
}

/** O pedaço do router que o gancho usa (router.svelte.ts). */
interface Nav {
  leaveGuard: (() => string | null) | null;
  go(path: string): void;
}

export type OpenAction = 'ignore' | 'go' | 'confirm';

/** O que fazer com um link: caminho inválido ou a mesma sala → nada; transmitindo → perguntar. */
export function openAction(path: string, currentPath: string, leaving: string | null): OpenAction {
  const m = path.match(/^\/s\/([^/]+)$/);
  if (!m || !ROOM_ID_RE.test(m[1]) || path === currentPath) return 'ignore';
  return leaving ? 'confirm' : 'go';
}

export function installDesktopHook(router: Nav): void {
  (window as DesktopWindow).espiaOpen = (path) => {
    const leaving = router.leaveGuard?.() ?? null;
    const action = openAction(path, location.pathname, leaving);
    if (action === 'ignore' || (action === 'confirm' && !confirm(leaving!))) return;
    router.go(path);
  };
}
