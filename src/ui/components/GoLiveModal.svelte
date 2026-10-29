<script lang="ts">
  import { Info, Monitor, MonitorUp, PanelTop, Radio, Volume2, VolumeX, X } from '@lucide/svelte';
  import { fade, scale } from 'svelte/transition';
  import type { Fps, GoLivePrefs, Resolution, ShareSurface } from '../../lib/types';

  // Seletor de tela (design SourcePicker, specs/004). As miniaturas do design não existem aqui: o
  // navegador/WebView2 sempre mostra o seletor nativo, que abre na aba escolhida (displaySurface).
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

  // Cópia local: só vale quando clicar em "Transmitir".
  // svelte-ignore state_referenced_locally
  let surface = $state<ShareSurface>(prefs.surface);
  // svelte-ignore state_referenced_locally
  let resolution = $state<Resolution>(prefs.resolution);
  // svelte-ignore state_referenced_locally
  let fps = $state<Fps>(prefs.fps);
  // svelte-ignore state_referenced_locally
  let audio = $state(prefs.audio);

  // Tela inteira vai sempre sem som (specs/002 FR-202); a escolha de som da janela fica guardada.
  const screen = $derived(surface === 'monitor');
  const audioOn = $derived(audio && !screen);
  const audioHint = $derived(
    screen
      ? 'Tela inteira vai sem som, pra não vazar o Discord. Pra levar o som, escolha uma janela.'
      : audio
        ? 'Só o som da janela que você escolher — Discord e notificações ficam de fora.'
        : 'Só a imagem vai pra sala.',
  );
  const footHint = $derived(
    people > 5
      ? `Com ${people} pessoas na sala, a qualidade baixa sozinha se muita gente estiver assistindo.`
      : resolution === 1080 && fps === 60
        ? '1080p a 60 fps usa mais internet. Se travar pra galera, tente 720p.'
        : `${resolution}p · ${fps} fps — leve pra todo mundo.`,
  );

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
        <span id="golive-title" class="title">Escolha o que compartilhar</span>
        <span class="subtitle">Uma janela de jogo ou app, ou a tela inteira.</span>
      </div>
      <button class="btn btn-icon" aria-label="Fechar" onclick={() => onclose?.()}>
        <X size={18} strokeWidth={2.75} />
      </button>
    </div>

    <div class="tabs" role="radiogroup" aria-label="O que compartilhar">
      <label class="tab">
        <input type="radio" name="surface" value="window" bind:group={surface} />
        <PanelTop size={16} strokeWidth={2.75} />Janelas
      </label>
      <label class="tab">
        <input type="radio" name="surface" value="monitor" bind:group={surface} />
        <Monitor size={16} strokeWidth={2.75} />Telas inteiras
      </label>
    </div>

    <div class="pick">
      <div class="pick-icon">
        {#if screen}<Monitor size={30} strokeWidth={2.5} />{:else}<PanelTop size={30} strokeWidth={2.5} />{/if}
      </div>
      <div class="pick-text">
        <span class="pick-title">
          {screen ? 'Você escolhe a tela no próximo passo' : 'Você escolhe a janela no próximo passo'}
        </span>
        <span class="pick-sub">
          {screen
            ? 'O Windows mostra suas telas. Tudo o que aparecer nela vai pra sala.'
            : 'O Windows mostra suas janelas abertas: escolha o jogo, o vídeo ou o app.'}
        </span>
        <span class="chip">
          {#if audioOn}<Volume2 size={12} strokeWidth={2.75} />com som{:else}<VolumeX size={12} strokeWidth={2.75} />sem som{/if}
        </span>
      </div>
    </div>

    <div class="bar">
      <label class="toggle-row" class:locked={screen}>
        <input
          class="switch"
          type="checkbox"
          role="switch"
          checked={audioOn}
          disabled={screen}
          onchange={(e) => (audio = e.currentTarget.checked)}
        />
        <span class="toggle-text">
          <span class="toggle-title">Compartilhar áudio</span>
          <span class="toggle-sub">{audioHint}</span>
        </span>
      </label>
      <fieldset class="opt">
        <legend>Resolução</legend>
        <div class="seg">
          {#each [720, 1080] as const as r}
            <label class="seg-opt"><input type="radio" name="res" value={r} bind:group={resolution} />{r}p</label>
          {/each}
        </div>
      </fieldset>
      <fieldset class="opt">
        <legend>FPS</legend>
        <div class="seg">
          {#each [30, 60] as const as f}
            <label class="seg-opt"><input type="radio" name="fps" value={f} bind:group={fps} />{f}</label>
          {/each}
        </div>
      </fieldset>
    </div>

    <div class="foot">
      <p class="note"><Info size={15} strokeWidth={2.75} />{footHint}</p>
      <div class="actions">
        <button class="btn btn-secondary" onclick={() => onclose?.()}>Cancelar</button>
        <button class="btn btn-primary" onclick={() => onstart?.({ surface, resolution, fps, audio })}>
          <Radio size={18} strokeWidth={2.75} />Transmitir
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0; z-index: 20; display: grid; place-items: safe center; padding: 16px;
    overflow-y: auto; background: var(--color-backdrop);
  }
  .dialog {
    width: min(680px, 100%); display: flex; flex-direction: column; gap: 20px; padding: 28px;
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

  .tabs {
    align-self: flex-start; display: flex; padding: 4px; gap: 4px; border-radius: 999px;
    background: rgba(245, 234, 216, 0.06);
  }
  .tab {
    display: inline-flex; align-items: center; gap: 8px; padding: 9px 18px; border-radius: 999px;
    font-size: 15px; font-weight: 600; cursor: pointer;
  }
  .tab input { position: absolute; opacity: 0; pointer-events: none; }
  .tab:has(input:checked) { background: var(--color-accent); color: var(--color-bg); }
  .tab:has(input:focus-visible) { box-shadow: var(--ring-focus); }

  .pick {
    display: flex; align-items: center; gap: 16px; padding: 20px; min-height: 120px;
    border-radius: 22px; box-shadow: inset 0 0 0 3px var(--color-accent);
    background:
      linear-gradient(rgba(198, 113, 57, 0.1), rgba(198, 113, 57, 0.1)),
      repeating-linear-gradient(135deg, #3a2416 0 22px, #33200f 22px 44px);
  }
  .pick-icon {
    flex: none; width: 56px; height: 56px; border-radius: 16px; display: grid; place-items: center;
    background: var(--color-accent-300); color: var(--color-accent-800);
  }
  .pick-text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
  .pick-title { font-size: 16px; font-weight: 600; }
  .pick-sub { font-size: 13px; color: color-mix(in srgb, var(--color-text) 70%, transparent); text-wrap: pretty; }
  .chip {
    align-self: flex-start; margin-top: 4px; display: inline-flex; align-items: center; gap: 5px;
    padding: 4px 9px; border-radius: 999px; background: var(--color-overlay); font-size: 11px; font-weight: 600;
  }

  .bar {
    display: flex; flex-wrap: wrap; align-items: center; gap: 14px; padding: 14px; border-radius: 24px;
    background: rgba(245, 234, 216, 0.05);
  }
  .toggle-row { flex: 1 1 100%; display: flex; align-items: center; gap: 14px; padding: 4px 6px; cursor: pointer; }
  .toggle-row.locked { cursor: default; }
  .toggle-text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .toggle-title { font-size: 15px; font-weight: 600; }
  .toggle-sub { font-size: 13px; color: color-mix(in srgb, var(--color-text) 60%, transparent); text-wrap: pretty; }
  .switch {
    appearance: none; flex: none; position: relative; width: 46px; height: 28px; margin: 0;
    border-radius: 999px; background: rgba(245, 234, 216, 0.18); cursor: pointer;
    transition: background-color 150ms ease-out;
  }
  .switch::after {
    content: ''; position: absolute; left: 4px; top: 4px; width: 20px; height: 20px; border-radius: 50%;
    background: var(--color-text); transition: transform 150ms ease-out;
  }
  .switch:checked { background: var(--color-accent); }
  .switch:checked::after { transform: translateX(18px); background: var(--color-bg); }
  .switch:disabled { cursor: default; opacity: 0.55; }
  .switch:focus-visible { outline: none; box-shadow: var(--ring-focus); }

  .opt { border: 0; margin: 0; padding: 0; display: flex; align-items: center; gap: 8px; }
  .opt legend {
    float: left; padding: 0; margin-right: 8px; font-size: 13px; font-weight: 600;
    color: color-mix(in srgb, var(--color-text) 70%, transparent);
  }
  .opt .seg { padding: 3px; gap: 3px; }
  .opt .seg-opt { padding: 7px 13px; font-size: 14px; }

  .foot { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; }
  .note {
    flex: 1 1 220px; margin: 0; display: flex; align-items: center; gap: 8px; font-size: 13px;
    color: color-mix(in srgb, var(--color-text) 55%, transparent);
  }
  .note :global(svg) { flex: none; }
  .actions { margin-left: auto; display: flex; gap: 8.8px; }
  .actions .btn { padding: 12px 20px; font-size: 15px; }
  .actions .btn-primary { padding: 12px 22px; }
</style>
