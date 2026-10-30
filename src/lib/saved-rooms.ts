// Suas salas (specs/011 US3, contracts/local-storage.md): as salas fixas em que se entrou neste
// aparelho. Cada navegador e o app têm a sua lista. Só números do servidor, nunca apelidos.
import { ROOM_ID_RE } from './room-id';
import { SAVED_ROOMS_MAX, TONES, type RoomStatus, type Tone } from '../shared/protocol';
import type { SavedRoomView, SavedRoomsView } from './types';

export interface SavedRoom {
  code: string;
  /** Último nome conhecido (atualizado pelo status e pelo room-renamed). */
  name: string;
  /** Última entrada neste aparelho. */
  lastUsed: number;
}

const KEY = 'espia:rooms';

export function loadSavedRooms(): SavedRoom[] {
  try {
    const list = JSON.parse(localStorage.getItem(KEY) ?? '[]');
    if (!Array.isArray(list)) return [];
    return list
      .filter(
        (r): r is SavedRoom =>
          !!r && typeof r.code === 'string' && ROOM_ID_RE.test(r.code) && typeof r.name === 'string' && typeof r.lastUsed === 'number',
      )
      .slice(0, SAVED_ROOMS_MAX);
  } catch {
    return [];
  }
}

export function saveSavedRooms(list: SavedRoom[]) {
  try {
    localStorage.setItem(KEY, JSON.stringify(list));
  } catch {
    // Sem armazenamento: só não lembra.
  }
}

/** Entrou numa sala fixa: vai pro topo. Passou de 5, sai a usada há mais tempo. */
export function rememberRoom(list: SavedRoom[], room: { code: string; name: string }, now: number): SavedRoom[] {
  const rest = list.filter((r) => r.code !== room.code);
  return [{ ...room, lastUsed: now }, ...rest].sort((a, b) => b.lastUsed - a.lastUsed).slice(0, SAVED_ROOMS_MAX);
}

export function forgetRoom(list: SavedRoom[], code: string): SavedRoom[] {
  return list.filter((r) => r.code !== code);
}

export function renameSaved(list: SavedRoom[], code: string, name: string): SavedRoom[] {
  return list.map((r) => (r.code === code ? { ...r, name } : r));
}

/** Cor da inicial: sempre a mesma pra mesma sala. */
export function roomTone(code: string): Tone {
  let h = 0;
  for (const ch of code) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  return TONES[h % TONES.length];
}

const DAY = 24 * 60 * 60_000;

/** "usada hoje", "usada ontem", "há 3 dias", "há 1 semana", "há 2 semanas", "há mais de um mês". */
export function recentLabel(lastUsed: number, now: number): string {
  const startOf = (t: number) => new Date(t).setHours(0, 0, 0, 0);
  const days = Math.round((startOf(now) - startOf(lastUsed)) / DAY);
  if (days <= 0) return 'usada hoje';
  if (days === 1) return 'usada ontem';
  if (days < 7) return `há ${days} dias`;
  if (days < 14) return 'há 1 semana';
  if (days < 30) return `há ${Math.floor(days / 7)} semanas`;
  return 'há mais de um mês';
}

/** Linha de baixo do item: "3 pessoas agora · 1 transmitindo" ou "Vazia · usada ontem". */
export function savedRoomSub(room: Pick<SavedRoomView, 'people' | 'streaming' | 'recent'>): string {
  if (room.people > 0) {
    const who = `${room.people} ${room.people === 1 ? 'pessoa' : 'pessoas'} agora`;
    return room.streaming > 0 ? `${who} · ${room.streaming} transmitindo` : who;
  }
  return `Vazia · ${room.recent}`;
}

/**
 * Monta o painel. `statuses` null = ainda perguntando (todas "carregando"). Ordem: com gente agora
 * primeiro (mais gente antes), depois as usadas mais recentemente.
 */
export function buildSavedRooms(
  list: SavedRoom[],
  statuses: RoomStatus[] | null,
  owned: (code: string) => boolean,
  now: number,
  menuCode: string | null = null,
): SavedRoomsView | null {
  if (list.length === 0) return null;
  const byCode = new Map((statuses ?? []).map((s) => [s.code, s]));
  const rooms: (SavedRoomView & { lastUsed: number })[] = list.map((r) => {
    const s = byCode.get(r.code);
    const state = !s ? 'loading' : s.state === 'ok' ? 'ok' : 'gone';
    return {
      code: r.code,
      name: s?.state === 'ok' && s.name ? s.name : r.name,
      own: state !== 'gone' && owned(r.code),
      tone: roomTone(r.code),
      state,
      people: s?.state === 'ok' ? s.people : 0,
      streaming: s?.state === 'ok' ? s.streaming : 0,
      recent: recentLabel(r.lastUsed, now),
      lastUsed: r.lastUsed,
      ...(menuCode === r.code ? { menuOpen: true } : {}),
    };
  });
  rooms.sort((a, b) => b.people - a.people || b.lastUsed - a.lastUsed);
  const strip = ({ lastUsed: _, ...r }: SavedRoomView & { lastUsed: number }): SavedRoomView => r;

  const busy = rooms.filter((r) => r.people > 0);
  const idle = rooms.filter((r) => r.people === 0);
  if (rooms.length >= 4 && busy.length > 0 && idle.length > 0) {
    return {
      groups: [
        { label: 'Com gente agora', rooms: busy.map(strip) },
        { label: 'Usadas recentemente', rooms: idle.map(strip) },
      ],
      note: `${rooms.length} salas neste aparelho`,
    };
  }
  return { groups: [{ label: null, rooms: rooms.map(strip) }], note: 'neste aparelho' };
}
