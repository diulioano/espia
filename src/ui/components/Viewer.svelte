<script lang="ts">
  // Tela cheia no celular (specs/018, contracts/viewer.md): uma camada sobre a sala, imagem inteira,
  // controles de toque e zoom. Onde o aparelho deixa, também vira tela cheia do sistema.
  // Só apresentação: o som continua no <audio> do tile da sala e nada muda no que chega.
  import { Minimize, Volume2, VolumeX } from '@lucide/svelte';
  import { onMount } from 'svelte';
  import type { RoomView, StreamView } from '../../lib/types';
  import { isDoubleTap, pan, pinch, ZOOM_NONE, type Point, type Tap, type ZoomState } from '../../lib/zoom';
  import LiveBadge from './LiveBadge.svelte';
  import ReconnectPill from './ReconnectPill.svelte';

  let {
    view,
    link,
    volumeAdjustable,
    initial,
    onclose,
    ontogglemute,
    onvolume,
  }: {
    view: StreamView;
    link: RoomView['link'];
    /** false = iPhone: o volume fica nos botões do aparelho, só o mudo aparece (research R8). */
    volumeAdjustable: boolean;
    /** Só mocks: controles e zoom iniciais. */
    initial?: { controls: boolean; zoom: ZoomState };
    onclose: () => void;
    ontogglemute: () => void;
    onvolume: (volume: number) => void;
  } = $props();

  const HIDE_MS = 3000;

  let layer: HTMLDivElement;
  let video: HTMLVideoElement | undefined = $state();
  // svelte-ignore state_referenced_locally
  let zoom = $state<ZoomState>(initial?.zoom ?? ZOOM_NONE);
  // svelte-ignore state_referenced_locally
  let controls = $state(initial?.controls ?? false);
  // svelte-ignore state_referenced_locally
  const pinned = !!initial?.controls;
  let gesturing = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let closed = false;
  let wasSystem = false;

  const label = $derived(view.source === 'camera' && !view.label ? `câmera de ${view.name}` : view.name);
  const level = $derived(view.muted ? 0 : view.volume);

  $effect(() => {
    if (video && video.srcObject !== (view.stream ?? null)) video.srcObject = view.stream ?? null;
  });

  // Outra transmissão aberta: imagem inteira de novo.
  let lastId: string | undefined;
  $effect(() => {
    if (lastId !== undefined && lastId !== view.id) zoom = ZOOM_NONE;
    lastId = view.id;
  });

  function close() {
    if (closed) return;
    closed = true;
    onclose();
  }

  /** Pede a tela cheia do sistema. Chamar no mesmo toque que abriu (os navegadores exigem gesto). */
  export function requestSystemFullscreen() {
    const el = layer as HTMLDivElement & { webkitRequestFullscreen?: () => Promise<void> | void };
    try {
      const p = el.requestFullscreen ? el.requestFullscreen() : el.webkitRequestFullscreen?.();
      if (p && typeof (p as Promise<void>).catch === 'function') (p as Promise<void>).catch(() => {});
    } catch {
      // iPhone: sem tela cheia de página. A camada já cobre a tela toda (modo "page").
    }
  }

  function fullscreenElement(): Element | null {
    const d = document as Document & { webkitFullscreenElement?: Element | null };
    return d.fullscreenElement ?? d.webkitFullscreenElement ?? null;
  }

  onMount(() => {
    // Voltar fecha a tela cheia sem sair da sala (research R3).
    history.pushState({ ...(history.state ?? {}), viewer: true }, '');
    const onPop = () => {
      if (!(history.state as { viewer?: boolean } | null)?.viewer) close();
    };
    const onFs = () => {
      const on = fullscreenElement() === layer;
      if (on) wasSystem = true;
      else if (wasSystem) close();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    // Giro ou janela mudou: imagem inteira de novo (FR-009).
    const onResize = () => (zoom = ZOOM_NONE);
    // Safari do iOS: sem isso, dois dedos aproximam a página em vez da imagem.
    const block = (e: Event) => e.preventDefault();
    addEventListener('popstate', onPop);
    addEventListener('keydown', onKey);
    addEventListener('resize', onResize);
    document.addEventListener('fullscreenchange', onFs);
    document.addEventListener('webkitfullscreenchange', onFs);
    layer.addEventListener('gesturestart', block);
    layer.addEventListener('gesturechange', block);
    return () => {
      closed = true;
      clearTimeout(hideTimer);
      removeEventListener('popstate', onPop);
      removeEventListener('keydown', onKey);
      removeEventListener('resize', onResize);
      document.removeEventListener('fullscreenchange', onFs);
      document.removeEventListener('webkitfullscreenchange', onFs);
      if (fullscreenElement() === layer) void document.exitFullscreen?.().catch(() => {});
      // Fechou pelo botão ou pela sala: tira a entrada do histórico que o "voltar" usaria.
      if ((history.state as { viewer?: boolean } | null)?.viewer) history.back();
    };
  });

  function show() {
    controls = true;
    clearTimeout(hideTimer);
    if (!pinned) hideTimer = setTimeout(() => (controls = false), HIDE_MS);
  }

  // Gestos (research R4): dois dedos aproximam, um arrasta a imagem aproximada, um toque mostra os controles.
  const pointers = new Map<number, Point>();
  let moved = false;
  let lastTap: Tap | null = null;
  const box = () => ({ width: layer.clientWidth, height: layer.clientHeight });

  function down(e: PointerEvent) {
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    try {
      layer.setPointerCapture(e.pointerId);
    } catch {
      // Ponteiro já solto.
    }
    if (pointers.size === 1) moved = false;
    else moved = true;
    gesturing = true;
  }
  function move(e: PointerEvent) {
    const prev = pointers.get(e.pointerId);
    if (!prev) return;
    const next = { x: e.clientX, y: e.clientY };
    if (pointers.size >= 2) {
      const [otherId, other] = [...pointers].find(([id]) => id !== e.pointerId)!;
      zoom = pinch(zoom, prev, other, next, other, box());
      pointers.set(otherId, other);
    } else if (Math.hypot(next.x - prev.x, next.y - prev.y) > 0) {
      if (zoom.scale > 1) zoom = pan(zoom, next.x - prev.x, next.y - prev.y, box());
      if (Math.abs(next.x - prev.x) + Math.abs(next.y - prev.y) > 2) moved = true;
    }
    pointers.set(e.pointerId, next);
  }
  function up(e: PointerEvent) {
    const had = pointers.delete(e.pointerId);
    if (pointers.size === 0) gesturing = false;
    if (!had || pointers.size > 0 || moved) return;
    const tap = { t: e.timeStamp, x: e.clientX, y: e.clientY };
    if (isDoubleTap(lastTap, tap)) {
      zoom = ZOOM_NONE;
      lastTap = null;
      return;
    }
    lastTap = tap;
    if (controls && !pinned) {
      clearTimeout(hideTimer);
      controls = false;
    } else show();
  }
  function cancel(e: PointerEvent) {
    pointers.delete(e.pointerId);
    if (pointers.size === 0) gesturing = false;
  }
</script>

<div
  bind:this={layer}
  class="viewer"
  role="region"
  aria-label={`Transmissão de ${view.name} em tela cheia`}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={cancel}
>
  <div class="frame" class:gesturing style:transform={`translate(${zoom.x}px, ${zoom.y}px) scale(${zoom.scale})`}>
    {#if view.stream}
      <video bind:this={video} autoplay playsinline muted></video>
    {:else}
      <div class="placeholder"><span>{view.caption ?? `${view.source === 'camera' ? 'câmera' : 'tela'} de ${view.name}`}</span></div>
    {/if}
  </div>

  {#if link === 'reconnecting'}<ReconnectPill compact />{/if}

  <div class="top" class:visible={controls}>
    <div class="name-chip">
      <LiveBadge />
      <span class="who">{label}</span>
      {#if view.label}<span class="what">· {view.label}</span>{/if}
    </div>
  </div>

  {#if zoom.scale > 1 && !controls}
    <span class="zoom-hint" aria-hidden="true">{zoom.scale.toFixed(1).replace('.', ',')}×</span>
  {/if}

  <!-- Os controles não entram nos gestos: tocar neles não aproxima nem esconde. -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="controls" class:visible={controls} onpointerdown={(e) => e.stopPropagation()} onpointerup={(e) => e.stopPropagation()}>
    <button class="ctl" aria-label={view.muted ? 'Ativar som' : 'Silenciar'} onclick={() => { ontogglemute(); show(); }}>
      {#if view.muted}<VolumeX size={22} strokeWidth={2.75} />{:else}<Volume2 size={22} strokeWidth={2.75} />{/if}
    </button>
    {#if volumeAdjustable}
      <input
        class="slider"
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={level}
        style:--fill="{level * 100}%"
        aria-label="Volume"
        oninput={(e) => { onvolume(Number(e.currentTarget.value)); show(); }}
      />
    {/if}
    <div class="spacer"></div>
    <button class="ctl exit" aria-label="Sair da tela cheia" onclick={close}>
      <Minimize size={22} strokeWidth={2.75} />
    </button>
  </div>
</div>

<style>
  .viewer {
    position: fixed; inset: 0; z-index: 50; overflow: hidden;
    background: var(--color-stage); color: var(--color-text);
    touch-action: none; user-select: none; -webkit-user-select: none; -webkit-touch-callout: none;
  }
  .frame { position: absolute; inset: 0; transform-origin: 0 0; }
  .frame.gesturing { will-change: transform; }
  video, .placeholder { position: absolute; inset: 0; width: 100%; height: 100%; }
  video { object-fit: contain; background: var(--color-stage); }
  .placeholder {
    display: grid; place-items: center;
    background: repeating-linear-gradient(135deg, var(--color-surface) 0 26px, var(--color-bg) 26px 52px);
  }
  .placeholder span { font-family: var(--font-mono); font-size: 14px; letter-spacing: 0.04em; color: var(--color-text-muted); }

  .top, .controls { position: absolute; left: 0; right: 0; opacity: 0; pointer-events: none; transition: opacity 150ms ease-out; }
  .top.visible, .controls.visible { opacity: 1; }
  .controls.visible { pointer-events: auto; }

  .top {
    top: 0; display: flex;
    padding: max(14px, env(safe-area-inset-top)) max(16px, env(safe-area-inset-right)) 32px max(16px, env(safe-area-inset-left));
    background: linear-gradient(to bottom, var(--color-overlay), transparent);
  }
  .name-chip {
    display: flex; align-items: center; gap: 8px; min-width: 0; max-width: 100%;
    padding: 5px 12px 5px 5px; border-radius: var(--radius-pill);
    background: var(--color-overlay); font-size: 14px; font-weight: 600;
  }
  .who { flex: none; }
  .what { min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 500; color: var(--color-text-muted); }

  .controls {
    bottom: 0; display: flex; align-items: center; gap: 12px;
    padding: 40px max(16px, env(safe-area-inset-right)) max(16px, env(safe-area-inset-bottom)) max(16px, env(safe-area-inset-left));
    background: linear-gradient(to top, var(--color-overlay), transparent);
  }
  .spacer { flex: 1; }
  .ctl {
    width: 48px; height: 48px; flex: none; border: 0; border-radius: 50%; display: grid; place-items: center; cursor: pointer;
    color: var(--color-text); background: var(--color-ctl);
  }
  .ctl.exit { background: var(--color-accent); color: var(--color-accent-900); }

  .slider { appearance: none; width: min(160px, 32vw); height: 48px; margin: 0; background: transparent; cursor: pointer; }
  .slider::-webkit-slider-runnable-track {
    height: 6px; border-radius: 999px;
    background: linear-gradient(to right, var(--color-accent-400) var(--fill), var(--color-ctl) var(--fill));
  }
  .slider::-moz-range-track { height: 6px; border-radius: 999px; background: var(--color-ctl); }
  .slider::-moz-range-progress { height: 6px; border-radius: 999px; background: var(--color-accent-400); }
  .slider::-webkit-slider-thumb { appearance: none; width: 20px; height: 20px; margin-top: -7px; border-radius: 50%; background: var(--color-text); border: 0; }
  .slider::-moz-range-thumb { width: 20px; height: 20px; border-radius: 50%; background: var(--color-text); border: 0; }

  .zoom-hint {
    position: absolute; right: max(16px, env(safe-area-inset-right)); bottom: max(16px, env(safe-area-inset-bottom));
    padding: 4px 10px; border-radius: var(--radius-pill); background: var(--color-overlay);
    font-size: 13px; font-weight: 700; pointer-events: none;
  }
</style>
