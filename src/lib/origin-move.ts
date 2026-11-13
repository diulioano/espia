// Ida do endereço antigo para o oficial (specs/019, contracts/convite-e-endereco.md, R12).
// O localStorage é separado por endereço, então os dados do aparelho vão junto no fragmento
// `#mv=`, que o navegador nunca manda ao servidor. Aqui ficam só as regras: quem lê `location` e
// navega é o src/main.ts.
import { NICKNAME_MAX, ROOM_NAME_MAX, SAVED_ROOMS_MAX } from '../shared/protocol';
import { ROOM_ID_RE } from './room-id';
import type { SavedRoom } from './saved-rooms';
import { OFFICIAL_ORIGIN } from './site-origin';

/** Chave do localStorage → valor cru. */
export type MoveItems = Record<string, string>;

/** O pedaço do localStorage que a ida usa. */
export type MoveSource = Pick<Storage, 'length' | 'key' | 'getItem'>;
export type MoveTarget = MoveSource & Pick<Storage, 'setItem'>;

/** Teto do texto codificado: 16 × 1024 caracteres. */
export const MOVE_MAX = 16_384;

const PREFIX = 'espia:';
const HASH = '#mv=';
const ROOMS = 'espia:rooms';
const OWNER = 'espia:owner:';
const SESSION = 'espia:session:';
const VOLUME = 'espia:volume:';
const OWNER_KEY_RE = /^[0-9a-f]{64}$/;

/** O que sai primeiro quando o pacote passa do teto. Salas, chaves de dono e apelido sempre vão. */
const DROP_ORDER: ((key: string) => boolean)[] = [
  (k) => k.startsWith(SESSION),
  (k) => k.startsWith(VOLUME),
  (k) => k === 'espia:golive',
  (k) => k === 'espia:appinvite',
];

/** As chaves `espia:*` guardadas. Sem armazenamento (janela anônima, dados bloqueados): vazio. */
export function readItems(storage: MoveSource | null): MoveItems {
  const items: MoveItems = {};
  try {
    if (!storage) return items;
    for (let i = 0; i < storage.length; i++) {
      const key = storage.key(i);
      if (!key?.startsWith(PREFIX)) continue;
      const value = storage.getItem(key);
      if (value !== null) items[key] = value;
    }
  } catch {
    return {};
  }
  return items;
}

function encode(text: string): string {
  let bin = '';
  for (const byte of new TextEncoder().encode(text)) bin += String.fromCharCode(byte);
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function decode(packed: string): string {
  const b64 = packed.replace(/-/g, '+').replace(/_/g, '/');
  const bin = atob(b64 + '='.repeat((4 - (b64.length % 4)) % 4));
  return new TextDecoder('utf-8', { fatal: true }).decode(Uint8Array.from(bin, (c) => c.charCodeAt(0)));
}

/**
 * Pacote em base64url, sem `=` no fim. null = nada para levar, ou nem o que sempre vai coube no
 * teto (a ida acontece sem pacote).
 */
export function packMove(items: MoveItems): string | null {
  let rest = items;
  for (let step = 0; ; step++) {
    if (Object.keys(rest).length === 0) return null;
    const packed = encode(JSON.stringify({ v: 1, items: rest }));
    if (packed.length <= MOVE_MAX) return packed;
    if (step === DROP_ORDER.length) return null;
    const drop = DROP_ORDER[step];
    rest = Object.fromEntries(Object.entries(rest).filter(([key]) => !drop(key)));
  }
}

/** Endereço oficial com o mesmo caminho e a mesma busca, mais o pacote quando há o que levar. */
export function leaveUrl(loc: Pick<Location, 'pathname' | 'search'>, storage: MoveSource | null): string {
  const packed = packMove(readItems(storage));
  return OFFICIAL_ORIGIN + loc.pathname + loc.search + (packed ? HASH + packed : '');
}

function validRooms(raw: string | undefined): SavedRoom[] {
  if (raw === undefined) return [];
  try {
    const list: unknown = JSON.parse(raw);
    if (!Array.isArray(list)) return [];
    return list
      .filter(
        (r): r is SavedRoom =>
          !!r &&
          typeof r.code === 'string' &&
          ROOM_ID_RE.test(r.code) &&
          typeof r.name === 'string' &&
          r.name.length <= ROOM_NAME_MAX &&
          typeof r.lastUsed === 'number' &&
          Number.isFinite(r.lastUsed),
      )
      .map(({ code, name, lastUsed }) => ({ code, name, lastUsed }));
  } catch {
    return [];
  }
}

/** Valor aceito para a chave, ou null se o item deve ser ignorado. */
function validItem(key: string, value: unknown): string | null {
  if (typeof value !== 'string') return null;
  if (key === 'espia:nickname') return value.length <= NICKNAME_MAX ? value : null;
  if (key === 'espia:golive' || key === 'espia:appinvite') return value;
  if (key === ROOMS) {
    const rooms = validRooms(value);
    return rooms.length > 0 ? JSON.stringify(rooms) : null;
  }
  if (key.startsWith(OWNER)) return ROOM_ID_RE.test(key.slice(OWNER.length)) && OWNER_KEY_RE.test(value) ? value : null;
  if (key.startsWith(SESSION)) return ROOM_ID_RE.test(key.slice(SESSION.length)) ? value : null;
  if (key.startsWith(VOLUME)) {
    const volume = Number(value);
    return value.trim() !== '' && volume >= 0 && volume <= 1 ? value : null;
  }
  return null;
}

/** Lê `#mv=…` e devolve só os itens válidos. Sem pacote, ou pacote inválido: null, sem erro. */
export function unpackMove(hash: string): MoveItems | null {
  if (!hash.startsWith(HASH)) return null;
  const packed = hash.slice(HASH.length);
  if (packed.length === 0 || packed.length > MOVE_MAX) return null;
  try {
    const data: unknown = JSON.parse(decode(packed));
    if (!data || typeof data !== 'object' || (data as { v?: unknown }).v !== 1) return null;
    const raw = (data as { items?: unknown }).items;
    if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return null;
    const items: MoveItems = {};
    for (const [key, value] of Object.entries(raw)) {
      const valid = validItem(key, value);
      if (valid !== null) items[key] = valid;
    }
    return items;
  } catch {
    return null;
  }
}

/**
 * O que gravar no oficial. Só acrescenta: valor que o oficial já tem fica como está. A lista de
 * salas é unida por código (no mesmo código, vale a entrada usada por último), da mais recente
 * para a mais antiga, até 5. Chave de dono entra mesmo de sala que ficou fora da lista.
 */
export function mergeMove(current: MoveItems, incoming: MoveItems): MoveItems {
  const writes: MoveItems = {};
  for (const [key, value] of Object.entries(incoming)) {
    if (key !== ROOMS && !(key in current)) writes[key] = value;
  }

  const mine = validRooms(current[ROOMS]).slice(0, SAVED_ROOMS_MAX);
  const byCode = new Map(mine.map((r) => [r.code, r]));
  for (const room of validRooms(incoming[ROOMS])) {
    const have = byCode.get(room.code);
    if (!have || room.lastUsed > have.lastUsed) byCode.set(room.code, room);
  }
  const merged = [...byCode.values()].sort((a, b) => b.lastUsed - a.lastUsed).slice(0, SAVED_ROOMS_MAX);
  if (JSON.stringify(merged) !== JSON.stringify(mine)) writes[ROOMS] = JSON.stringify(merged);
  return writes;
}

/** Chegada no oficial: junta o pacote de `#mv=` ao que já está guardado. Nunca lança. */
export function importMove(hash: string, storage: MoveTarget | null): void {
  const incoming = unpackMove(hash);
  if (!incoming || !storage) return;
  for (const [key, value] of Object.entries(mergeMove(readItems(storage), incoming))) {
    try {
      storage.setItem(key, value);
    } catch {
      // Sem armazenamento ou cheio: só não lembra.
    }
  }
}
