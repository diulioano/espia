// Derivações puras do RoomView, usadas igualmente pelos mocks e pela sala real.
import type { ViewMode } from './quality';
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
  const s = layout === 'single' ? view.streams[0] : layout === 'focus' ? focusedStream(view) : undefined;
  // Parou de assistir (specs/014): não está vendo, mesmo em destaque.
  // A sua própria tela no palco (specs/014 US3) também não conta.
  return s && s.watching !== false && !s.you ? s.id : null;
}

export function header(view: RoomView): { title: string; subtitle: string } {
  const n = view.streams.length;
  const layout = stageLayout(view);
  if (layout === 'connecting') return { title: `Sala ${view.roomId}`, subtitle: 'Conectando…' };
  // Sala fixa (specs/011): o nome é o título; o subtítulo resume o que está acontecendo.
  // A sua tela no palco (specs/014 US3).
  const shown = layout === 'single' ? view.streams[0] : layout === 'focus' ? focusedStream(view) : undefined;
  if (shown?.you) {
    const others = view.streams.length - 1;
    const sub = others ? `${others} ${others === 1 ? 'tela' : 'telas'} da galera · Esc volta` : 'do jeito que a sala vê · Esc volta';
    if (view.room?.kind === 'fixed') return { title: view.room.name, subtitle: `Sua tela, ao vivo · ${sub}` };
    return { title: 'Sua tela, ao vivo', subtitle: sub[0].toUpperCase() + sub.slice(1) };
  }
  if (view.room?.kind === 'fixed') return { title: view.room.name, subtitle: fixedSubtitle(view, layout, n) };
  if (layout === 'empty') {
    return view.myBroadcast
      ? { title: 'Você está ao vivo', subtitle: 'Ninguém mais transmitindo agora' }
      : { title: 'Sala da galera', subtitle: 'Ninguém transmitindo agora' };
  }
  if (layout === 'single') {
    const s = view.streams[0];
    return {
      title: `${s.name} está ao vivo`,
      subtitle: view.myBroadcast ? 'Você também está transmitindo' : streamLine(s),
    };
  }
  if (layout === 'grid') return { title: `${n} telas ao vivo`, subtitle: 'Escolha qual assistir' };
  return { title: `Vendo ${focusedStream(view)!.name}`, subtitle: `${n} telas ao vivo` };
}

/** "Hollow Knight · 1080p · 60 fps · com som" (o que está sendo transmitido vem primeiro, specs/012). */
function streamLine(s: StreamView): string {
  return [s.label, s.quality].filter(Boolean).join(' · ');
}

function fixedSubtitle(view: RoomView, layout: StageLayout, n: number): string {
  if (layout === 'empty') return view.myBroadcast ? 'Você está ao vivo · ninguém mais transmitindo' : 'Ninguém transmitindo agora';
  if (layout === 'single') {
    const s = view.streams[0];
    return `${s.name} está ao vivo · ${view.myBroadcast ? 'você também está transmitindo' : streamLine(s)}`;
  }
  if (layout === 'grid') return `${n} telas ao vivo · escolha qual assistir`;
  return `Vendo ${focusedStream(view)!.name} · ${n} telas ao vivo`;
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

export interface ViewModesInput {
  streams: { id: string }[];
  layout: StageLayout;
  focusedId: string | null;
  mobile: boolean;
  /** Quem eu parei de assistir (specs/014 US1). */
  unwatched: ReadonlySet<string>;
  /** A página está à vista (specs/014 US2): minimizada, na bandeja ou aba de trás = false. */
  pageVisible: boolean;
  /** Tela em picture-in-picture: continua chegando com a página escondida. */
  pipId: string | null;
  /** A minha própria tela no palco (specs/014 US3): nunca vai pra ninguém. */
  selfId: string | null;
}

/**
 * Como estou vendo a tela de cada pessoa (specs/014 research R5). Quem não está no mapa: escondida.
 * `hidden` (segundo plano) pausa só o vídeo; `off` (parou de assistir) pausa vídeo e som.
 */
export function viewModes(i: ViewModesInput): Record<string, ViewMode> {
  const out: Record<string, ViewMode> = {};
  const streams = i.streams.filter((s) => s.id !== i.selfId);
  const selected = i.focusedId ?? streams[0]?.id;
  for (const { id } of streams) {
    // Parou de assistir: nem vídeo nem som.
    if (i.unwatched.has(id)) out[id] = 'off';
    // Celular: só a selecionada aparece e toca; as outras ficam nos chips (como na 010, sem som).
    else if (i.mobile && id !== selected) out[id] = 'off';
    // Segundo plano: sem vídeo, o som continua.
    else if (!i.pageVisible && id !== i.pipId) out[id] = 'hidden';
    else if (i.mobile) out[id] = 'full';
    else if (i.layout === 'focus') out[id] = id === i.focusedId ? 'full' : 'thumb';
    else out[id] = 'full';
  }
  return out;
}

/**
 * Escolhas de "parar de assistir" que ainda valem (specs/014 research R4): a escolha é por transmissão,
 * identificada pelo início dela. Parou, recomeçou ou saiu da sala → volta a assistir sozinho.
 */
export function pruneUnwatched(
  unwatched: ReadonlyMap<string, number>,
  peers: { id: string; live: { startedAt: number } | null }[],
): Map<string, number> {
  const out = new Map<string, number>();
  for (const [id, startedAt] of unwatched) {
    const p = peers.find((x) => x.id === id);
    if (p?.live && p.live.startedAt === startedAt) out.set(id, startedAt);
  }
  return out;
}
