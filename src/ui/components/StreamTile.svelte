<script lang="ts">
  import { Eye, EyeOff, Maximize, PictureInPicture2, RefreshCw, Volume2, VolumeX, WifiOff } from '@lucide/svelte';
  import type { StreamView, TileSize } from '../../lib/types';
  import LiveBadge from './LiveBadge.svelte';
  import Spinner from './Spinner.svelte';

  let {
    view,
    size = 'stage',
    focused = false,
    you: youProp = false,
    alwaysName = false,
    controls = true,
    radius,
    onselect,
    ontogglemute,
    onvolume,
    onretry,
    onwatch,
    onpip,
    onfullscreen,
    videoHidden = false,
    volumeAdjustable = true,
    still,
  }: {
    view: StreamView;
    size?: TileSize;
    focused?: boolean;
    you?: boolean;
    /** Chip de nome sempre visível (ilustrações). */
    alwaysName?: boolean;
    /** false = tile decorativo, sem controles. */
    controls?: boolean;
    /** Raio vindo do pai (palco 28–32). */
    radius?: string;
    /** Clique no tile (focar na grade / trocar na faixa). */
    onselect?: () => void;
    ontogglemute?: () => void;
    onvolume?: (volume: number) => void;
    onretry?: () => void;
    /** Parar de assistir (false) / assistir (true) (specs/014 US1). */
    onwatch?: (on: boolean) => void;
    /** Entrou/saiu do picture-in-picture (specs/014 US2): a tela continua chegando com a janela escondida. */
    onpip?: (on: boolean) => void;
    /** Celular (specs/018): o botão de tela cheia abre o visualizador em vez de pôr o tile em tela cheia. */
    onfullscreen?: () => void;
    /** Visualizador aberto por cima (specs/018): sem <video> aqui, o som continua. */
    videoHidden?: boolean;
    /** false = iPhone, onde a página não muda o volume (specs/018 R8): só o mudo. */
    volumeAdjustable?: boolean;
    /**
     * Imagem parada no lugar do vídeo (specs/014 US3, a sua prévia). `null` = ainda sem quadro (placeholder).
     * Ausente = vídeo normal.
     */
    still?: ImageBitmap | null;
  } = $props();

  let container: HTMLDivElement;
  let video: HTMLVideoElement | undefined = $state();
  let sound: HTMLAudioElement | undefined = $state();
  let canvas: HTMLCanvasElement | undefined = $state();

  // Prévia parada: desenha o quadro quando ele muda (nada de vídeo ao vivo).
  $effect(() => {
    if (!canvas || !still) return;
    canvas.width = still.width;
    canvas.height = still.height;
    try {
      canvas.getContext('2d')?.drawImage(still, 0, 0);
    } catch {
      // Bitmap já fechado (trocado no meio): o próximo chega em seguida.
    }
  });
  let active = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;

  /** A sua própria tela (prévia da barra ou no palco, specs/014 US3): sem som e sem controles de som. */
  const you = $derived(youProp || !!view.you);
  const thumb = $derived(size === 'thumb');
  /** Parou de assistir (specs/014): cartão "Assistir" no lugar do vídeo; o som continua. */
  const watching = $derived(you || view.watching === true);
  const hasControls = $derived(controls && watching && view.status === 'ok' && !thumb);
  const showName = $derived(alwaysName || size !== 'stage' || active);
  // Com a descrição (specs/012), ela já diz se é câmera: "Lu · Câmera · C920".
  const label = $derived(you ? 'Você' : view.source === 'camera' && !view.label ? `câmera de ${view.name}` : view.name);
  const level = $derived(view.muted ? 0 : view.volume);

  $effect(() => {
    if (video && video.srcObject !== (view.stream ?? null)) video.srcObject = view.stream ?? null;
  });
  // O som toca à parte (specs/014): o vídeo é sempre mudo, assim sumir com ele não corta o som.
  $effect(() => {
    if (video) video.muted = true;
  });
  $effect(() => {
    if (sound && sound.srcObject !== (view.audioStream ?? null)) sound.srcObject = view.audioStream ?? null;
  });
  $effect(() => {
    if (sound) {
      sound.muted = view.muted;
      sound.volume = view.volume;
    }
  });
  // Picture-in-picture (specs/014 US2): avisa a sala, pra tela continuar chegando com a janela escondida.
  $effect(() => {
    const v = video;
    if (!v) return;
    const enter = () => onpip?.(true);
    const leave = () => onpip?.(false);
    v.addEventListener('enterpictureinpicture', enter);
    v.addEventListener('leavepictureinpicture', leave);
    return () => {
      v.removeEventListener('enterpictureinpicture', enter);
      v.removeEventListener('leavepictureinpicture', leave);
      // O <video> sumiu (vídeo parou, tile saiu) com o PiP aberto: o navegador fecha o PiP.
      if (document.pictureInPictureElement === v) onpip?.(false);
    };
  });
  // Parou de assistir a tela que estava no picture-in-picture: fecha o PiP.
  $effect(() => {
    if (!watching && video && document.pictureInPictureElement === video) void document.exitPictureInPicture().catch(() => {});
  });

  /** No toque os controles ficam mais tempo (specs/018 R8): não dá pra "passar o mouse" de novo. */
  let touch = false;
  function wake(e?: PointerEvent | FocusEvent) {
    if (e && 'pointerType' in e && e.type === 'pointerdown') touch = e.pointerType === 'touch';
    active = true;
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => (active = false), touch ? 3000 : 2000);
  }
  function sleep() {
    clearTimeout(hideTimer);
    active = false;
  }

  function handleClick() {
    if (!watching) return;
    if (view.status === 'failed' && thumb) onretry?.();
    else onselect?.();
  }
  function handleKey(e: KeyboardEvent) {
    if (onselect && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      handleClick();
    }
  }

  async function pip(e: MouseEvent) {
    e.stopPropagation();
    if (!video || !document.pictureInPictureEnabled) return;
    if (document.pictureInPictureElement === video) await document.exitPictureInPicture();
    else await video.requestPictureInPicture().catch(() => {});
  }
  async function fullscreen(e: MouseEvent) {
    e.stopPropagation();
    if (onfullscreen) return onfullscreen();
    if (document.fullscreenElement) await document.exitFullscreen();
    else await container.requestFullscreen().catch(() => {});
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  bind:this={container}
  class="tile size-{size} tone-{view.tone}"
  class:focused
  class:clickable={!!onselect}
  class:active
  style:border-radius={thumb ? 'var(--radius-tile-sm)' : radius}
  role={onselect ? 'button' : 'group'}
  tabindex={onselect || hasControls ? 0 : undefined}
  aria-label={`Transmissão de ${label}`}
  onclick={handleClick}
  onkeydown={handleKey}
  onpointermove={wake}
  onpointerdown={wake}
  onpointerleave={(e) => {
    // No toque o dedo "sai" ao soltar: esconder aqui fazia o clique cair no tile, não no botão (specs/018).
    if (e.pointerType !== 'touch') sleep();
  }}
  onfocusin={wake}
>
  <!-- Parou de assistir: sem som também (specs/014). Em segundo plano o tile continua e o som segue. -->
  {#if view.audioStream && !you && watching}
    <audio bind:this={sound} autoplay></audio>
  {/if}

  {#if !watching}
    <div class="unwatched">
      {#if thumb}
        <button
          class="watch-thumb"
          aria-label={`Assistir ${label}`}
          onclick={(e) => {
            e.stopPropagation();
            onwatch?.(true);
          }}
        >
          <Eye size={18} strokeWidth={2.75} />
        </button>
      {:else}
        <span class="unwatched-title">{view.watchState === 'stopped' ? `Você parou de assistir ${label}` : `Transmissão de ${label}`}</span>
        <span class="unwatched-sub">Nem a tela nem o som chegam até você clicar.</span>
        <button
          class="btn btn-primary watch"
          onclick={(e) => {
            e.stopPropagation();
            onwatch?.(true);
          }}
        >
          <Eye size={16} strokeWidth={2.75} />
          Assistir
        </button>
      {/if}
    </div>
  {:else if still !== undefined}
    {#if still}
      <canvas bind:this={canvas} class="still"></canvas>
    {:else}
      <div class="placeholder"><span>{view.caption ?? 'sua tela'}</span></div>
    {/if}
  {:else if view.status === 'ok'}
    {#if view.stream && !videoHidden}
      <video bind:this={video} autoplay playsinline></video>
    {:else if view.stream}
      <div class="placeholder"></div>
    {:else}
      <div class="placeholder"><span>{view.caption ?? `${view.source === 'camera' ? 'câmera' : 'tela'} de ${view.name}`}</span></div>
    {/if}
  {:else if view.status === 'loading'}
    <div class="state">
      <Spinner size={thumb ? 22 : 40} />
      {#if !thumb}<span class="loading-text">Conectando à transmissão de {view.name}…</span>{/if}
    </div>
  {:else}
    <div class="state failed">
      <div class="fail-icon"><WifiOff size={22} strokeWidth={2.75} /></div>
      {#if thumb}
        <span class="fail-thumb">Falhou · tocar p/ tentar</span>
      {:else}
        <div class="fail-text">
          <span class="fail-title">Não deu pra conectar com {view.name}</span>
          <span class="fail-body">Não deu para conectar diretamente. Esta versão está sem retransmissão; algumas redes podem não conectar.</span>
        </div>
        <button
          class="btn btn-primary retry"
          onclick={(e) => {
            e.stopPropagation();
            onretry?.();
          }}
        >
          <RefreshCw size={16} strokeWidth={2.75} />
          Tentar de novo
        </button>
      {/if}
    </div>
  {/if}

  <div class="name-chip" class:visible={showName}>
    <LiveBadge />
    <span class="who">{label}</span>
    {#if view.label && !you}<span class="what">· {view.label}</span>{/if}
  </div>

  {#if view.muted && view.status === 'ok' && !you}
    <div class="muted-badge" class:hidden={hasControls && active} aria-label="Sem som">
      <VolumeX size={15} strokeWidth={2.75} />
    </div>
  {/if}

  {#if hasControls}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="controls" class:visible={active} onclick={(e) => e.stopPropagation()}>
      {#if !you}
        <button class="ctl" aria-label={view.muted ? 'Ativar som' : 'Silenciar'} onclick={() => ontogglemute?.()}>
          {#if view.muted}<VolumeX size={18} strokeWidth={2.75} />{:else}<Volume2 size={18} strokeWidth={2.75} />{/if}
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
          oninput={(e) => onvolume?.(Number(e.currentTarget.value))}
        />
        {/if}
      {/if}
      <div class="spacer"></div>
      {#if onwatch && !you}
        <button class="ctl" aria-label="Parar de assistir" title="Parar de assistir (tela e som)" onclick={() => onwatch?.(false)}>
          <EyeOff size={18} strokeWidth={2.75} />
        </button>
      {/if}
      <button class="ctl" aria-label="Picture-in-picture" onclick={pip} disabled={!view.stream}>
        <PictureInPicture2 size={18} strokeWidth={2.75} />
      </button>
      <button class="ctl" aria-label="Tela cheia" onclick={fullscreen}>
        <Maximize size={18} strokeWidth={2.75} />
      </button>
    </div>
  {/if}
</div>

<style>
  .tile {
    --a: #3a2416; --b: #33200f;
    position: relative; width: 100%; height: 100%; overflow: hidden;
    border-radius: var(--radius-card); background: var(--color-stage); color: var(--color-text);
    box-shadow: inset 0 0 0 1px rgba(245, 234, 216, 0.06);
  }
  .tone-sage { --a: #262d1d; --b: #212818; }
  .tone-sand { --a: #2e2a24; --b: #28241f; }
  .tone-deep { --a: #40261a; --b: #381f14; }
  .size-grid { border-radius: var(--radius-tile); }
  .focused::after {
    content: ''; position: absolute; inset: 0; border-radius: inherit; pointer-events: none;
    box-shadow: inset 0 0 0 3px var(--color-accent);
  }
  .clickable { cursor: pointer; }
  .size-stage.active, .size-grid.active { cursor: default; }
  .size-grid.clickable.active { cursor: pointer; }

  .still { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain; background: var(--color-stage); }
  video { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain; background: var(--color-stage); }
  .placeholder {
    position: absolute; inset: 0; display: grid; place-items: center;
    background: repeating-linear-gradient(135deg, var(--a) 0 26px, var(--b) 26px 52px);
  }
  .placeholder span {
    font-family: var(--font-mono); font-size: 14px; letter-spacing: 0.04em; color: rgba(245, 234, 216, 0.5);
  }
  .size-thumb .placeholder span { font-size: 11px; }

  .unwatched {
    position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 10px; padding: 60px 20px 20px; text-align: center;
    background: repeating-linear-gradient(135deg, var(--a) 0 26px, var(--b) 26px 52px);
  }
  .unwatched-title { font-family: var(--font-heading); font-size: 22px; line-height: 1.2; text-wrap: balance; }
  .size-grid .unwatched-title { font-size: 18px; }
  .unwatched-sub { font-size: 14px; color: rgba(245, 234, 216, 0.65); max-width: 320px; text-wrap: pretty; }
  .watch { margin-top: 6px; padding: 10px 18px; color: var(--color-bg); }
  .size-thumb .unwatched { padding: 28px 8px 8px; }
  .watch-thumb {
    width: 40px; height: 40px; border: 0; border-radius: 50%; display: grid; place-items: center; cursor: pointer;
    color: var(--color-text); background: var(--color-ctl);
  }
  .watch-thumb:hover { background: rgba(245, 234, 216, 0.24); }

  .state {
    position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 14px; padding: 20px; text-align: center; background: #171512;
  }
  .size-thumb .state { gap: 8px; padding: 8px; }
  .loading-text { font-size: 14px; color: rgba(245, 234, 216, 0.7); }
  .failed { gap: 12px; }
  .fail-icon {
    width: 56px; height: 56px; border-radius: 50%; display: grid; place-items: center;
    background: rgba(198, 113, 57, 0.18); color: var(--color-accent-400);
  }
  .size-thumb .fail-icon { width: 36px; height: 36px; }
  .fail-text { display: flex; flex-direction: column; gap: 4px; align-items: center; }
  .fail-title { font-family: var(--font-heading); font-size: 22px; line-height: 1.2; }
  .fail-body { font-size: 14px; color: rgba(245, 234, 216, 0.65); max-width: 340px; text-wrap: pretty; }
  .fail-thumb { font-size: 12px; font-weight: 600; color: rgba(245, 234, 216, 0.75); }
  .retry { margin-top: 6px; padding: 10px 18px; color: var(--color-bg); }

  .name-chip {
    position: absolute; top: 16px; left: 16px; display: flex; align-items: center; gap: 8px;
    padding: 5px 12px 5px 5px; border-radius: var(--radius-pill);
    background: var(--color-overlay); backdrop-filter: blur(8px);
    font-size: 14px; font-weight: 600;
    opacity: 0; transition: opacity 150ms ease-out; pointer-events: none;
  }
  .name-chip.visible { opacity: 1; }
  /* Não passa por baixo do selo de mudo, no canto direito. */
  .name-chip { max-width: calc(100% - 80px); }
  .size-thumb .name-chip { max-width: calc(100% - 16px); }
  .who { flex: none; }
  .what {
    min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 500;
    color: color-mix(in srgb, var(--color-text) 72%, transparent);
  }
  .size-thumb .name-chip { top: 8px; left: 8px; font-size: 12px; }

  .muted-badge {
    position: absolute; top: 16px; right: 16px; width: 30px; height: 30px; border-radius: 50%;
    display: grid; place-items: center; background: var(--color-overlay);
    transition: opacity 150ms ease-out;
  }
  .muted-badge.hidden { opacity: 0; }
  .size-thumb .muted-badge { top: 8px; right: 8px; }

  .controls {
    position: absolute; left: 0; right: 0; bottom: 0; display: flex; align-items: center; gap: 10px;
    padding: 40px 20px 20px;
    background: linear-gradient(to top, rgba(12, 11, 10, 0.9), rgba(12, 11, 10, 0));
    opacity: 0; transition: opacity 150ms ease-out; pointer-events: none;
  }
  .controls.visible { opacity: 1; pointer-events: auto; }
  .size-grid .controls { padding: 28px 14px 14px; }
  .spacer { flex: 1; }
  .ctl {
    --c: 42px;
    width: var(--c); height: var(--c); border: 0; border-radius: 50%; flex: none;
    display: grid; place-items: center; cursor: pointer;
    color: var(--color-text); background: var(--color-ctl);
  }
  .size-grid .ctl { --c: 36px; }
  .ctl:hover:not(:disabled) { background: rgba(245, 234, 216, 0.24); }
  .ctl:disabled { opacity: 0.45; cursor: not-allowed; }

  .slider {
    --w: 120px;
    appearance: none; width: var(--w); height: 14px; margin: 0; background: transparent; cursor: pointer;
  }
  .size-grid .slider { --w: 72px; }
  .slider::-webkit-slider-runnable-track {
    height: 6px; border-radius: 999px;
    background: linear-gradient(to right, var(--color-accent-400) var(--fill), rgba(245, 234, 216, 0.22) var(--fill));
  }
  .slider::-moz-range-track { height: 6px; border-radius: 999px; background: rgba(245, 234, 216, 0.22); }
  .slider::-moz-range-progress { height: 6px; border-radius: 999px; background: var(--color-accent-400); }
  .slider::-webkit-slider-thumb {
    appearance: none; width: 14px; height: 14px; margin-top: -4px; border-radius: 50%; background: #f5ead8; border: 0;
  }
  .slider::-moz-range-thumb { width: 14px; height: 14px; border-radius: 50%; background: #f5ead8; border: 0; }

  /* Toque (specs/018 FR-013): botões de 44 px em todo tamanho, barra de volume com área de toque de 44 px. */
  @media (pointer: coarse) {
    .ctl, .size-grid .ctl { --c: 44px; }
    .slider { height: 44px; }
  }
</style>
