// Preferências guardadas no navegador. localStorage pode não existir ou lançar (aba privada,
// dados bloqueados): tudo em try/catch, e o app funciona sem.
import type { GoLivePrefs } from './types';

const KEY = {
  nickname: 'espia:nickname',
  goLive: 'espia:golive',
  volume: (name: string) => `espia:volume:${name.toLowerCase()}`,
  session: (code: string) => `espia:session:${code}`,
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
const session = () => sessionStorage;

export function loadNickname(): string {
  return read(local, KEY.nickname) ?? '';
}

export function saveNickname(nickname: string) {
  write(local, KEY.nickname, nickname);
}

export const DEFAULT_GO_LIVE: GoLivePrefs = { resolution: 1080, fps: 30, audio: true, surface: 'window' };

export function loadGoLivePrefs(): GoLivePrefs {
  try {
    const p = JSON.parse(read(local, KEY.goLive) ?? 'null');
    if (p && (p.resolution === 720 || p.resolution === 1080) && (p.fps === 30 || p.fps === 60)) {
      return { resolution: p.resolution, fps: p.fps, audio: !!p.audio, surface: p.surface === 'monitor' ? 'monitor' : 'window' };
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
 * Identidade na sala, por aba (sessionStorage sobrevive ao F5, some ao fechar a aba).
 * Com ela, recarregar a página retoma o mesmo lugar sem "saiu/entrou".
 */
export interface RoomSession {
  peerId: string;
  resumeToken: string;
}

export function loadSession(code: string): RoomSession | null {
  try {
    const s = JSON.parse(read(session, KEY.session(code)) ?? 'null');
    return s && typeof s.peerId === 'string' && typeof s.resumeToken === 'string' ? s : null;
  } catch {
    return null;
  }
}

export function saveSession(code: string, s: RoomSession | null) {
  write(session, KEY.session(code), s && JSON.stringify(s));
}
