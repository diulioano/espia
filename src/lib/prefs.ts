// Preferências guardadas no navegador. localStorage pode não existir ou lançar (aba privada,
// dados bloqueados): tudo em try/catch, e o app funciona sem.
import type { GoLivePrefs } from './types';

const KEY = {
  nickname: 'espia:nickname',
  goLive: 'espia:golive',
  volume: (name: string) => `espia:volume:${name.toLowerCase()}`,
  session: (code: string) => `espia:session:${code}`,
  appInvite: 'espia:appinvite',
  owner: (code: string) => `espia:owner:${code}`,
};

function read(storage: () => Storage, key: string): string | null {
  try {
    return storage().getItem(key);
  } catch {
    return null;
  }
}

function write(storage: () => Storage, key: string, value: string | null) {
  try {
    if (value === null) storage().removeItem(key);
    else storage().setItem(key, value);
  } catch {
    // Sem armazenamento: só não lembra.
  }
}

const local = () => localStorage;

export function loadNickname(): string {
  return read(local, KEY.nickname) ?? '';
}

export function saveNickname(nickname: string) {
  write(local, KEY.nickname, nickname);
}

/** Chave de dono de uma sala fixa criada neste aparelho (specs/011, contracts/local-storage.md). */
export function loadOwnerKey(code: string): string | null {
  const key = read(local, KEY.owner(code));
  return key && /^[0-9a-f]{64}$/.test(key) ? key : null;
}

export function saveOwnerKey(code: string, key: string) {
  write(local, KEY.owner(code), key);
}

/** Só quando a sala deixou de existir: tirar da lista não tira a posse. */
export function forgetOwnerKey(code: string) {
  write(local, KEY.owner(code), null);
}

/** "Agora não" no convite do app (specs/008): vale para sempre nesse navegador. */
export function loadAppInviteDismissed(): boolean {
  return read(local, KEY.appInvite) === 'dismissed';
}

export function saveAppInviteDismissed() {
  write(local, KEY.appInvite, 'dismissed');
}

export const DEFAULT_GO_LIVE: GoLivePrefs = { resolution: 1080, fps: 30, audio: true, surface: 'window' };

export function loadGoLivePrefs(): GoLivePrefs {
  try {
    const p = JSON.parse(read(local, KEY.goLive) ?? 'null');
    if (p && (p.resolution === 720 || p.resolution === 1080) && (p.fps === 30 || p.fps === 60)) {
      return {
        resolution: p.resolution,
        fps: p.fps,
        audio: !!p.audio,
        surface: p.surface === 'monitor' || p.surface === 'camera' ? p.surface : 'window',
        ...(typeof p.lastSourceKey === 'string' ? { lastSourceKey: p.lastSourceKey } : {}),
        ...(typeof p.lastVideoLabel === 'string' ? { lastVideoLabel: p.lastVideoLabel } : {}),
        ...(typeof p.lastAudioLabel === 'string' ? { lastAudioLabel: p.lastAudioLabel } : {}),
        ...(p.lastAudioRole === 'voice' || p.lastAudioRole === 'game' ? { lastAudioRole: p.lastAudioRole } : {}),
      };
    }
  } catch {
    // Valor corrompido: usa o padrão.
  }
  return { ...DEFAULT_GO_LIVE };
}

export function saveGoLivePrefs(prefs: GoLivePrefs) {
  write(local, KEY.goLive, JSON.stringify(prefs));
}

/** Volume por pessoa (pelo apelido: o id muda a cada sala). null = nunca ajustou. */
export function loadVolume(name: string): number | null {
  const v = Number(read(local, KEY.volume(name)));
  return read(local, KEY.volume(name)) !== null && v >= 0 && v <= 1 ? v : null;
}

export function saveVolume(name: string, volume: number) {
  write(local, KEY.volume(name), String(volume));
}

/**
 * Identidade na sala, no navegador inteiro (localStorage): recarregar a página, ou fechar a aba e abrir
 * a sala em outra, retoma o mesmo lugar sem "saiu/entrou" nem a mesma pessoa duas vezes. Se a outra
 * aba ainda estiver aberta, ela cede o lugar (o servidor fecha com `replaced`).
 */
export interface RoomSession {
  peerId: string;
  resumeToken: string;
}

/** Identidade velha não serve mais (o servidor já esqueceu); some pra não acumular. */
const SESSION_MAX_AGE_MS = 24 * 60 * 60_000;

export function loadSession(code: string): RoomSession | null {
  try {
    const s = JSON.parse(read(local, KEY.session(code)) ?? 'null');
    if (!s || typeof s.peerId !== 'string' || typeof s.resumeToken !== 'string') return null;
    if (typeof s.at !== 'number' || Date.now() - s.at > SESSION_MAX_AGE_MS) return null;
    return { peerId: s.peerId, resumeToken: s.resumeToken };
  } catch {
    return null;
  }
}

export function saveSession(code: string, s: RoomSession | null) {
  write(local, KEY.session(code), s && JSON.stringify({ ...s, at: Date.now() }));
  if (s) pruneSessions();
}

function pruneSessions() {
  try {
    const store = localStorage;
    for (let i = store.length - 1; i >= 0; i--) {
      const key = store.key(i);
      if (!key?.startsWith('espia:session:')) continue;
      const at = (JSON.parse(store.getItem(key) ?? 'null') as { at?: number } | null)?.at;
      if (typeof at !== 'number' || Date.now() - at > SESSION_MAX_AGE_MS) store.removeItem(key);
    }
  } catch {
    // Sem armazenamento: nada a limpar.
  }
}
