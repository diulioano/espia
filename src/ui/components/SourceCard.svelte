<script lang="ts">
  import { Camera, Check, Monitor, PanelTop, Volume2 } from '@lucide/svelte';
  import type { CaptureSource, ShareSurface } from '../../lib/types';

  // Cartão da grade do seletor: janelas/telas do app (specs/005, tela-esperada.png) e câmeras com
  // prévia ao vivo (specs/006).
  let {
    source,
    stream = null,
    busy = false,
    selected = false,
    onselect,
  }: {
    source: Omit<CaptureSource, 'kind'> & { kind: ShareSurface };
    /** Prévia ao vivo (câmera). */
    stream?: MediaStream | null;
    /** Câmera em uso por outro programa. */
    busy?: boolean;
    selected?: boolean;
    onselect?: (id: string) => void;
  } = $props();

  const initial = $derived((source.app || source.title).trim().charAt(0).toUpperCase() || '?');

  let video = $state<HTMLVideoElement>();
  $effect(() => {
    if (video) video.srcObject = stream;
  });
</script>

<button
  class="card"
  class:selected
  aria-pressed={selected}
  disabled={busy}
  title={busy ? `${source.title} — em uso por outro programa` : source.title}
  onclick={() => onselect?.(source.id)}
>
  <div class="preview">
    {#if stream}
      <video bind:this={video} autoplay muted playsinline aria-label={`Prévia de ${source.title}`}></video>
    {:else if source.thumb}
      <img src={source.thumb} alt="" draggable="false" />
    {:else if source.icon}
      <img class="big-icon" src={source.icon} alt="" draggable="false" />
    {:else}
      <span class="big-icon fallback">
        {#if source.kind === 'monitor'}<Monitor size={34} strokeWidth={2.25} />{:else if source.kind === 'camera'}<Camera size={34} strokeWidth={2.25} />{:else}<PanelTop size={34} strokeWidth={2.25} />{/if}
      </span>
    {/if}
    {#if busy}
      <span class="chip">Em uso por outro programa</span>
    {:else if source.playing}
      <span class="chip"><Volume2 size={12} strokeWidth={2.75} />tocando som</span>
    {/if}
    {#if selected}
      <span class="check" aria-hidden="true"><Check size={16} strokeWidth={3} /></span>
    {/if}
  </div>
  <div class="meta">
    {#if source.icon}
      <img class="icon" src={source.icon} alt="" draggable="false" />
    {:else}
      <span class="icon letter">{source.kind === 'monitor' ? source.title.replace(/\D/g, '') || initial : initial}</span>
    {/if}
    <span class="names">
      <span class="title">{source.title}</span>
      {#if source.app}<span class="app">{source.app}</span>{/if}
    </span>
  </div>
</button>

<style>
  .card {
    display: flex; flex-direction: column; gap: 10px; min-width: 0; padding: 8px 8px 12px; border: 0;
    border-radius: var(--radius-tile); background: rgba(245, 234, 216, 0.04); color: inherit; font: inherit;
    text-align: left; cursor: pointer; box-shadow: inset 0 0 0 1px var(--color-divider);
    transition: background-color 120ms ease-out, box-shadow 120ms ease-out;
  }
  .card:hover { background: rgba(245, 234, 216, 0.07); }
  .card:focus-visible { outline: none; box-shadow: inset 0 0 0 1px var(--color-divider), var(--ring-focus); }
  .card:disabled { cursor: not-allowed; opacity: 0.55; }
  .card.selected {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    box-shadow: inset 0 0 0 3px var(--color-accent);
  }

  .preview {
    position: relative; aspect-ratio: 16 / 9; border-radius: var(--radius-tile-sm); overflow: hidden;
    display: grid; place-items: center;
    background: repeating-linear-gradient(135deg, #2c2925 0 18px, #282521 18px 36px);
  }
  .preview img:not(.big-icon), .preview video { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain; }
  .preview video { object-fit: cover; }
  .big-icon { width: 44px; height: 44px; }
  .fallback { display: grid; place-items: center; color: color-mix(in srgb, var(--color-text) 55%, transparent); }

  .chip {
    position: absolute; left: 8px; bottom: 8px; display: inline-flex; align-items: center; gap: 5px;
    padding: 4px 9px; border-radius: var(--radius-pill); background: var(--color-overlay); font-size: 11px; font-weight: 600;
  }
  .check {
    position: absolute; top: 8px; right: 8px; width: 28px; height: 28px; border-radius: 50%; display: grid; place-items: center;
    background: var(--color-accent); color: var(--color-accent-900);
  }

  .meta { display: flex; align-items: center; gap: 10px; min-width: 0; padding: 0 4px; }
  .icon { flex: none; width: 22px; height: 22px; border-radius: 6px; }
  .letter {
    display: grid; place-items: center; background: var(--color-accent-200); color: var(--color-accent-800);
    font-family: var(--font-heading); font-size: 12px;
  }
  .names { min-width: 0; display: flex; flex-direction: column; }
  .title, .app { overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  .title { font-size: 14px; font-weight: 600; }
  .app { font-size: 12px; color: color-mix(in srgb, var(--color-text) 58%, transparent); }
</style>
