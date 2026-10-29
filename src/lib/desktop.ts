// Gancho do app desktop (src-tauri/src/lib.rs → open_room): um link de sala aberto com o app já
// aberto chama window.espiaOpen("/s/xxx-xxx"). No navegador nada chama isso.
import { ROOM_ID_RE } from './room-id';

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
  (window as Window & { espiaOpen?: (path: string) => void }).espiaOpen = (path) => {
    const leaving = router.leaveGuard?.() ?? null;
    const action = openAction(path, location.pathname, leaving);
    if (action === 'ignore' || (action === 'confirm' && !confirm(leaving!))) return;
    router.go(path);
  };
}
