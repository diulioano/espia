// Nome de sala fixa (specs/011, research R10): mesma regra no navegador e no Worker.
import { ROOM_NAME_MAX } from './protocol';

// O ZWJ (U+200D) passa: sem ele, emojis compostos (família, profissões) seriam recusados.
const CONTROL_OR_INVISIBLE = /(?!‍)[\p{Cc}\p{Cf}]/u;

/** Nome pronto pra guardar, ou null se não dá (vazio, longo demais, caractere invisível ou de controle). */
export function normalizeRoomName(raw: string): string | null {
  const name = raw.normalize('NFC').replace(/\s+/g, ' ').trim();
  if (CONTROL_OR_INVISIBLE.test(name)) return null;
  const length = [...name].length;
  return length === 0 || length > ROOM_NAME_MAX ? null : name;
}
