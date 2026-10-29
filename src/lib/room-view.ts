// Derivações puras do RoomView, usadas igualmente pelos mocks e pela sala real.
import type { RoomView, StreamView } from './types';

export type StageLayout = 'connecting' | 'empty' | 'single' | 'grid' | 'focus';

export function stageLayout(view: RoomView): StageLayout {
  if (view.status === 'connecting') return 'connecting';
  if (view.streams.length === 0) return 'empty';
  if (view.streams.length === 1) return 'single';
  return focusedStream(view) ? 'focus' : 'grid';
}

/** Grade para n transmissões (máx. 9 = 10 pessoas menos você): 2×1, 2×2, 3×2, 3×3. */
export function gridSize(n: number): { cols: number; rows: number } {
  if (n <= 2) return { cols: 2, rows: 1 };
  if (n <= 4) return { cols: 2, rows: 2 };
  if (n <= 6) return { cols: 3, rows: 2 };
  return { cols: 3, rows: 3 };
}

export function focusedStream(view: RoomView): StreamView | undefined {
  return view.streams.find((s) => s.id === view.focusedId);
}

/** Quem você está vendo em destaque (palco cheio ou foco). */
export function watchingId(view: RoomView): string | null {
  const layout = stageLayout(view);
  if (layout === 'single') return view.streams[0].id;
  if (layout === 'focus') return view.focusedId;
  return null;
}

export function header(view: RoomView): { title: string; subtitle: string } {
  const n = view.streams.length;
  const layout = stageLayout(view);
  if (layout === 'connecting') return { title: `Sala ${view.roomId}`, subtitle: 'Conectando…' };
  if (layout === 'empty') {
    return view.myBroadcast
      ? { title: 'Você está ao vivo', subtitle: 'Ninguém mais transmitindo agora' }
      : { title: 'Sala da galera', subtitle: 'Ninguém transmitindo agora' };
  }
  if (layout === 'single') {
    const s = view.streams[0];
    return {
      title: `${s.name} está ao vivo`,
      subtitle: view.myBroadcast ? 'Você também está transmitindo' : (s.quality ?? ''),
    };
  }
  if (layout === 'grid') return { title: `${n} telas ao vivo`, subtitle: 'Escolha qual assistir' };
  return { title: `Vendo ${focusedStream(view)!.name}`, subtitle: `${n} telas ao vivo` };
}

/** "12:04" ou "1:02:03". */
export function formatElapsed(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const pad = (x: number) => String(x).padStart(2, '0');
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`;
}
