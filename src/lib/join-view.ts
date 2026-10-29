// Textos derivados da prévia da sala na tela de entrada (D2 / M1).
import type { JoinPreviewPerson } from './types';

export const NICKNAME_MAX = 24;

/** "Lu", "Lu e Bia", "Lu, Bia e Tomás". */
export function listNames(names: string[]): string {
  if (names.length <= 1) return names[0] ?? '';
  return `${names.slice(0, -1).join(', ')} e ${names[names.length - 1]}`;
}

/** "Lu ao vivo", "Lu e Bia ao vivo", "3 ao vivo". */
export function liveLabel(people: JoinPreviewPerson[]): string | null {
  const live = people.filter((p) => p.live).map((p) => p.name);
  if (live.length === 0) return null;
  return live.length <= 2 ? `${listNames(live)} ao vivo` : `${live.length} ao vivo`;
}

/** "Lu, Bia e Tomás já estão lá" / "Lu já está lá". */
export function whoIsThere(people: JoinPreviewPerson[]): string {
  const names = people.map((p) => p.name);
  return `${listNames(names)} ${names.length === 1 ? 'já está lá' : 'já estão lá'}`;
}

/** Versão do celular: "Lu, Bia e Tomás já estão lá. Lu está ao vivo." */
export function whoIsThereLong(people: JoinPreviewPerson[]): string {
  const live = people.filter((p) => p.live).map((p) => p.name);
  const liveText = live.length === 0 ? '' : ` ${listNames(live)} ${live.length === 1 ? 'está' : 'estão'} ao vivo.`;
  return `${whoIsThere(people)}.${liveText}`;
}

export function cleanNickname(value: string): string {
  return value.trim().slice(0, NICKNAME_MAX);
}
