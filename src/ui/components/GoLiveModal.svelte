<script lang="ts">
  import { Info, MonitorUp, X } from '@lucide/svelte';
  import { fade, scale } from 'svelte/transition';
  import type { Fps, GoLivePrefs, Resolution } from '../../lib/types';

  let {
    prefs,
    people = 0,
    onstart,
    onclose,
  }: {
    prefs: GoLivePrefs;
    /** Pessoas na sala, você incluído. Com mais de 5, avisa que a qualidade pode baixar. */
    people?: number;
    onstart?: (prefs: GoLivePrefs) => void;
    onclose?: () => void;
  } = $props();

  // Cópia local: só vale quando clicar em "Escolher tela e começar".
  // svelte-ignore state_referenced_locally
  let resolution = $state<Resolution>(prefs.resolution);
  // svelte-ignore state_referenced_locally
  let fps = $state<Fps>(prefs.fps);
  // svelte-ignore state_referenced_locally
  let audio = $state(prefs.audio);

  let dialog: HTMLDivElement;
  $effect(() => dialog.focus());

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      onclose?.();
    }
  }
</script>

<svelte:window {onkeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" transition:fade={{ duration: 150 }} onclick={(e) => e.target === e.currentTarget && onclose?.()}>
  <div
    bind:this={dialog}
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-labelledby="golive-title"
    tabindex="-1"
    transition:scale={{ start: 0.97, duration: 150 }}
  >
    <div class="head">
      <div class="badge"><MonitorUp size={22} strokeWidth={2.75} /></div>
      <div class="titles">
        <span id="golive-title" class="title">Transmitir</span>
        <span class="subtitle">Ajuste a qualidade antes de escolher a tela.</span>
      </div>
      <button class="btn btn-icon" aria-label="Fechar" onclick={() => onclose?.()}>
        <X size={18} strokeWidth={2.75} />
      </button>
    </div>

    <fieldset>
      <legend>Resolução</legend>
      <div class="seg">
        {#each [720, 1080] as const as r}
          <label class="seg-opt"><input type="radio" name="res" value={r} bind:group={resolution} />{r}p</label>
        {/each}
      </div>
    </fieldset>

    <fieldset>
      <legend>Taxa de quadros</legend>
      <div class="seg">
        {#each [30, 60] as const as f}
          <label class="seg-opt"><input type="radio" name="fps" value={f} bind:group={fps} />{f} fps</label>
        {/each}
      </div>
    </fieldset>

    <label class="toggle-row">
      <span class="toggle-text">
        <span class="toggle-title">Compartilhar áudio</span>
        <span class="toggle-sub">O som do jogo ou vídeo vai junto. Funciona melhor com uma aba.</span>
      </span>
      <input class="switch" type="checkbox" role="switch" bind:checked={audio} />
    </label>

    <p class="note">
      <Info size={15} strokeWidth={2.75} />
      {#if people > 5}
        Com {people} pessoas na sala, a qualidade baixa sozinha se muita gente estiver assistindo.
      {:else}
        1080p a 60 fps usa mais internet. Se travar pra galera, tente 720p.
      {/if}
    </p>

    <div class="actions">
      <button class="btn btn-secondary" onclick={() => onclose?.()}>Cancelar</button>
      <button class="btn btn-primary" onclick={() => onstart?.({ resolution, fps, audio })}>Escolher tela e começar</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0; z-index: 20; display: grid; place-items: center; padding: 16px;
    background: var(--color-backdrop);
  }
  .dialog {
    width: min(480px, 100%); display: flex; flex-direction: column; gap: 22px; padding: 30px;
    border-radius: var(--radius-stage); background: var(--color-surface);
    box-shadow: 0 0 0 1px rgba(245, 234, 216, 0.08), 0 24px 60px rgba(0, 0, 0, 0.6);
  }
  .dialog:focus { outline: none; }
  .head { display: flex; align-items: flex-start; gap: 14px; }
  .badge {
    flex: none; width: 48px; height: 48px; border-radius: 50%; display: grid; place-items: center;
    background: var(--color-accent); color: var(--color-accent-900);
  }
  .titles { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .title { font-family: var(--font-heading); font-size: 26px; line-height: 1.1; }
  .subtitle { font-size: 14px; color: color-mix(in srgb, var(--color-text) 62%, transparent); }
  fieldset { border: 0; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 8px; }
  legend {
    padding: 0; margin-bottom: 8px; font-size: 13px; font-weight: 600;
    color: color-mix(in srgb, var(--color-text) 75%, transparent);
  }
  .toggle-row {
    display: flex; align-items: center; gap: 14px; padding: 14px 16px; border-radius: 20px;
    background: rgba(245, 234, 216, 0.05); cursor: pointer;
  }
  .toggle-text { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .toggle-title { font-size: 15px; font-weight: 600; }
  .toggle-sub { font-size: 13px; color: var(--color-text-muted); }
  .switch {
    appearance: none; flex: none; position: relative; width: 46px; height: 28px; margin: 0;
    border-radius: 999px; background: rgba(245, 234, 216, 0.2); cursor: pointer;
    transition: background-color 150ms ease-out;
  }
  .switch::after {
    content: ''; position: absolute; left: 4px; top: 4px; width: 20px; height: 20px; border-radius: 50%;
    background: var(--color-bg); transition: transform 150ms ease-out;
  }
  .switch:checked { background: var(--color-accent); }
  .switch:checked::after { transform: translateX(18px); }
  .note {
    margin: 0; display: flex; align-items: center; gap: 8px; font-size: 13px;
    color: color-mix(in srgb, var(--color-text) 55%, transparent);
  }
  .note :global(svg) { flex: none; }
  .actions { display: flex; justify-content: flex-end; gap: 8.8px; }
  .actions .btn-primary { padding: 12px 22px; }
</style>
