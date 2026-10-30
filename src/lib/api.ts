// Chamadas HTTP do Worker (contracts/http-api.md).
import type { GoneReason, RoomPreview, RoomStatus } from '../shared/protocol';

export type CreateResult = { ok: true; code: string } | { ok: false; reason: 'rate-limited' | 'network' };

export async function createRoom(): Promise<CreateResult> {
  try {
    const res = await fetch('/api/rooms', { method: 'POST' });
    if (res.status === 201) return { ok: true, code: ((await res.json()) as { code: string }).code };
    return { ok: false, reason: res.status === 429 ? 'rate-limited' : 'network' };
  } catch {
    return { ok: false, reason: 'network' };
  }
}

export type CreateFixedResult =
  | { ok: true; code: string; ownerKey: string }
  | { ok: false; reason: 'rate-limited' | 'name' | 'network' };

/** Sala fixa (specs/011): a chave do dono vem só nesta resposta. */
export async function createFixedRoom(name: string): Promise<CreateFixedResult> {
  try {
    const res = await fetch('/api/rooms', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ kind: 'fixed', name }),
    });
    if (res.status === 201) {
      const body = (await res.json()) as { code: string; ownerKey: string };
      return { ok: true, code: body.code, ownerKey: body.ownerKey };
    }
    return { ok: false, reason: res.status === 429 ? 'rate-limited' : res.status === 400 ? 'name' : 'network' };
  } catch {
    return { ok: false, reason: 'network' };
  }
}

/** Suas salas (specs/011): quantas pessoas em cada uma, numa consulta só. null = não deu (rede, limite). */
export async function fetchStatuses(codes: string[]): Promise<RoomStatus[] | null> {
  if (codes.length === 0) return [];
  try {
    const res = await fetch('/api/rooms/status', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ codes }),
    });
    if (!res.ok) return null;
    return ((await res.json()) as { rooms: RoomStatus[] }).rooms;
  } catch {
    return null;
  }
}

export type OwnerFailure = 'forbidden' | 'gone' | 'name' | 'rate-limited' | 'network';

async function ownerRequest(code: string, key: string, init: RequestInit): Promise<Response | OwnerFailure> {
  try {
    const res = await fetch(`/api/rooms/${code}`, {
      ...init,
      headers: { ...init.headers, Authorization: `Bearer ${key}` },
    });
    if (res.ok) return res;
    if (res.status === 403) return 'forbidden';
    if (res.status === 404 || res.status === 410) return 'gone';
    if (res.status === 400) return 'name';
    if (res.status === 429) return 'rate-limited';
    return 'network';
  } catch {
    return 'network';
  }
}

/** Renomear sala fixa (specs/011 US4). O nome novo chega pra todos pelo room-renamed. */
export async function renameRoom(code: string, key: string, name: string): Promise<{ ok: true; name: string } | { ok: false; reason: OwnerFailure }> {
  const res = await ownerRequest(code, key, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  });
  if (typeof res === 'string') return { ok: false, reason: res };
  return { ok: true, name: ((await res.json()) as { name: string }).name };
}

/** Apagar sala fixa (specs/011 US4): quem está dentro sai com aviso. */
export async function deleteRoom(code: string, key: string): Promise<{ ok: true } | { ok: false; reason: OwnerFailure }> {
  const res = await ownerRequest(code, key, { method: 'DELETE' });
  return typeof res === 'string' ? { ok: false, reason: res } : { ok: true };
}

/** Mensagem para o modal de renomear/apagar. */
export function ownerErrorText(reason: OwnerFailure): string {
  return {
    forbidden: 'Só quem criou a sala pode fazer isso.',
    gone: 'Essa sala não existe mais.',
    name: 'Esse nome não dá. Use letras, números e espaços.',
    'rate-limited': 'Muitas mudanças daqui agora. Tenta de novo em um minuto.',
    network: 'Não deu pra falar com o servidor. Confere a internet e tenta de novo.',
  }[reason];
}

export type PreviewResult =
  | { status: 'ready'; preview: RoomPreview }
  | { status: 'full'; preview: RoomPreview }
  | { status: 'notfound' }
  /** Sala fixa apagada ou expirada (specs/011, 410). */
  | { status: 'gone'; reason: GoneReason }
  /** Não deu pra perguntar (rede, limite): a tela de entrada esconde a prévia e deixa tentar. */
  | { status: 'unknown' };

export async function fetchPreview(code: string): Promise<PreviewResult> {
  try {
    const res = await fetch(`/api/rooms/${code}`);
    if (res.status === 404) return { status: 'notfound' };
    if (res.status === 410) {
      const { reason } = (await res.json()) as { reason: GoneReason };
      return { status: 'gone', reason: reason === 'expired' ? 'expired' : 'deleted' };
    }
    if (!res.ok) return { status: 'unknown' };
    const preview = (await res.json()) as RoomPreview;
    return { status: preview.people.length >= preview.capacity ? 'full' : 'ready', preview };
  } catch {
    return { status: 'unknown' };
  }
}
