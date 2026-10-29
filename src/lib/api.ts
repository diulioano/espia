// Chamadas HTTP do Worker (contracts/http-api.md).
import type { RoomPreview } from '../shared/protocol';

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

export type PreviewResult =
  | { status: 'ready'; preview: RoomPreview }
  | { status: 'full'; preview: RoomPreview }
  | { status: 'notfound' }
  /** Não deu pra perguntar (rede, limite): a tela de entrada esconde a prévia e deixa tentar. */
  | { status: 'unknown' };

export async function fetchPreview(code: string): Promise<PreviewResult> {
  try {
    const res = await fetch(`/api/rooms/${code}`);
    if (res.status === 404) return { status: 'notfound' };
    if (!res.ok) return { status: 'unknown' };
    const preview = (await res.json()) as RoomPreview;
    return { status: preview.people.length >= preview.capacity ? 'full' : 'ready', preview };
  } catch {
    return { status: 'unknown' };
  }
}
