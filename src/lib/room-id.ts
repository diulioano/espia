// Código de sala curto no formato "xxx-xxx" (ex.: k7f-q2x). Ver specs/001-espia-screen-share/research.md (R2).

/** Sem caracteres ambíguos: 0/o, 1/l/i. */
export const ROOM_ID_ALPHABET = 'abcdefghjkmnpqrstuvwxyz23456789';
export const ROOM_ID_RE = /^[a-hjkmnp-z2-9]{3}-[a-hjkmnp-z2-9]{3}$/;

export function generateRoomId(): string {
  const n = ROOM_ID_ALPHABET.length;
  // Rejeita bytes acima do maior múltiplo de n para não enviesar a distribuição.
  const limit = 256 - (256 % n);
  let out = '';
  while (out.length < 6) {
    for (const byte of crypto.getRandomValues(new Uint8Array(12))) {
      if (byte < limit && out.length < 6) out += ROOM_ID_ALPHABET[byte % n];
    }
  }
  return `${out.slice(0, 3)}-${out.slice(3)}`;
}

/**
 * Extrai o código de um link inteiro ("https://…/s/k7f-q2x"), de um código com ou sem hífen
 * ("K7F Q2X", "k7fq2x") ou devolve null se não parecer uma sala.
 */
export function parseRoomId(input: string): string | null {
  let text = input.trim().toLowerCase();
  const fromPath = text.match(/\/s\/([^/?#\s]+)/);
  if (fromPath) text = fromPath[1];
  const chars = text.replace(/[\s-]/g, '');
  if (chars.length !== 6 || [...chars].some((c) => !ROOM_ID_ALPHABET.includes(c))) return null;
  return `${chars.slice(0, 3)}-${chars.slice(3)}`;
}

export function roomPath(id: string): string {
  return `/s/${id}`;
}
