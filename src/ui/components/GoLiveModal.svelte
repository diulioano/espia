<script lang="ts">
  import { Camera, Info, Monitor, MonitorUp, PanelTop, Radio, Volume2, VolumeX, X } from '@lucide/svelte';
  import { untrack } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { nativeAudioText } from '../../lib/audio-source';
  import type { CameraActions, CameraView, Fps, GoLivePrefs, PickerView, Resolution, ShareSurface } from '../../lib/types';
  import SourceCard from './SourceCard.svelte';

  // Seletor de tela (design SourcePicker). No app desktop (specs/005, picker.mode 'native') mostra a
  // grade de janelas/telas com miniaturas e o app captura direto. No navegador (specs/004) não dá pra
  // listar janelas: um cartão explica que a escolha é no próximo passo (seletor nativo).
  // Aba Câmeras (specs/006): câmeras/placas com prévia ao vivo, no site e no app; no celular, só ela.
  let {
    prefs,
    picker = { mode: 'browser', sources: [], loading: false, selectedId: null },
    camera = { permission: 'unknown', devices: [], audios: [], videoId: null, audioId: null, audioRole: 'voice', mobile: false, facing: 'user' },
    people = 0,
    onstart,
    onselect,
    onsurface,
    oncamera,
    onclose,
  }: {
    prefs: GoLivePrefs;
    picker?: PickerView;
    camera?: CameraView;
    onsurface?: (surface: ShareSurface) => void;
    oncamera?: CameraActions;
    /** Pessoas na sala, você incluído. Com mais de 5, avisa que a qualidade pode baixar. */
    people?: number;
    onstart?: (prefs: GoLivePrefs) => void;
    onselect?: (id: string) => void;
    onclose?: () => void;
  } = $props();

  const native = $derived(picker.mode === 'native');

  // Cópia local: só vale quando clicar em "Transmitir".
  // Celular não transmite tela: abre direto (e só) na aba Câmeras.
  // svelte-ignore state_referenced_locally
  let surface = $state<ShareSurface>(camera.mobile ? 'camera' : prefs.surface);
  // A store liga/desliga as prévias de câmera conforme a aba. untrack: o efeito só depende da aba,
  // não do estado que a store lê/escreve ao trocar (senão vira laço).
  $effect(() => {
    const s = surface;
    untrack(() => onsurface?.(s));
  });
  // svelte-ignore state_referenced_locally
  let resolution = $state<Resolution>(prefs.resolution);
  // svelte-ignore state_referenced_locally
  let fps = $state<Fps>(prefs.fps);
  // svelte-ignore state_referenced_locally
  let audio = $state(prefs.audio);

  // Site: tela inteira vai sempre sem som (specs/002 FR-202); a escolha de som da janela fica guardada.
  // App: tela inteira pode levar o som do PC, menos o Discord (specs/005 FR-509).
  const screen = $derived(surface === 'monitor');
  const isCamera = $derived(surface === 'camera');
  const audioLocked = $derived(screen && !native);
  const audioOn = $derived(audio && !audioLocked);

  const counts = $derived({
    window: picker.sources.filter((s) => s.kind === 'window').length,
    monitor: picker.sources.filter((s) => s.kind === 'monitor').length,
  });
  const shown = $derived(picker.sources.filter((s) => s.kind === surface));
  // A escolha vale só na aba em que foi feita.
  const selected = $derived(shown.find((s) => s.id === picker.selectedId));
  const camGranted = $derived(camera.permission === 'granted');
  const camAudio = $derived(camera.audios.find((a) => a.id === camera.audioId));
  const canStart = $derived(isCamera ? camera.mobile || !!camera.videoId : !native || !!selected);
  // Grade larga: janelas do app ou câmeras no PC.
  const wide = $derived(native || (isCamera && !camera.mobile));

  const audioHint = $derived(
    isCamera
      ? !audio
        ? 'Só a imagem vai pra sala.'
        : camera.mobile
          ? 'Vai o som do microfone do celular junto com a câmera.'
          : !camGranted
            ? 'A entrada de som você escolhe depois de permitir as câmeras.'
            : camAudio
              ? `Vai o som de ${camAudio.label} junto com a câmera.`
              : 'Escolha abaixo a entrada de som que vai junto.'
      : native
        ? nativeAudioText({ surface: screen ? 'monitor' : 'window', audio, selectedTitle: selected?.title })
        : screen
          ? 'Tela inteira vai sem som, pra não vazar o Discord. Pra levar o som, escolha uma janela.'
          : audio
            ? 'Só o som da janela que você escolher — Discord e notificações ficam de fora.'
            : 'Só a imagem vai pra sala.',
  );
  const footHint = $derived(
    camera.mobile
      ? 'Transmitir pelo celular gasta internet e bateria. 720p · 30 fps.'
      : people > 5
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
    class:wide
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
        <span class="subtitle">
          {camera.mobile ? 'A câmera do celular, frontal ou traseira.' : 'Uma janela de jogo ou app, a tela inteira ou uma câmera.'}
        </span>
      </div>
      <button class="btn btn-icon" aria-label="Fechar" onclick={() => onclose?.()}>
        <X size={18} strokeWidth={2.75} />
      </button>
    </div>

    <div class="tabs-row">
      <div class="tabs" role="radiogroup" aria-label="O que compartilhar">
        {#if !camera.mobile}
          <label class="tab">
            <input type="radio" name="surface" value="window" bind:group={surface} />
            <PanelTop size={16} strokeWidth={2.75} />Janelas
            {#if native && !picker.loading}<span class="count">{counts.window}</span>{/if}
          </label>
          <label class="tab">
            <input type="radio" name="surface" value="monitor" bind:group={surface} />
            <Monitor size={16} strokeWidth={2.75} />Telas inteiras
            {#if native && !picker.loading}<span class="count">{counts.monitor}</span>{/if}
          </label>
        {/if}
        <label class="tab">
          <input type="radio" name="surface" value="camera" bind:group={surface} />
          <Camera size={16} strokeWidth={2.75} />Câmeras
          {#if camGranted && !camera.mobile}<span class="count">{camera.devices.length}</span>{/if}
        </label>
      </div>
      {#if native && !isCamera}<span class="refresh">Prévias atualizam a cada 2s</span>{/if}
    </div>

    {#if isCamera}
      {#if camera.mobile}
        <fieldset class="opt facing">
          <legend>Câmera</legend>
          <div class="seg">
            <label class="seg-opt">
              <input type="radio" name="facing" checked={camera.facing === 'user'} onchange={() => oncamera?.setFacing('user')} />Frontal
            </label>
            <label class="seg-opt">
              <input type="radio" name="facing" checked={camera.facing === 'environment'} onchange={() => oncamera?.setFacing('environment')} />Traseira
            </label>
          </div>
        </fieldset>
      {:else if camera.permission === 'denied'}
        <div class="empty column">
          <strong>Câmera bloqueada</strong>
          <span>Libere a câmera nas permissões do site (o cadeado na barra de endereço) e abra de novo.</span>
        </div>
      {:else if !camGranted}
        <div class="empty column">
          <Camera size={28} strokeWidth={2.25} />
          <span>Pra mostrar suas câmeras e placas de captura, o navegador vai pedir permissão.</span>
          <button class="btn btn-primary" onclick={() => oncamera?.allow()}>Permitir câmeras</button>
        </div>
      {:else if camera.devices.length === 0}
        <div class="empty column">
          {#if camera.error === 'busy'}
            <strong>A câmera está sendo usada por outro programa</strong>
            <span>Feche a câmera no Discord, OBS, Teams ou Zoom (ou o app que estiver usando) e tente de novo.</span>
          {:else if camera.error === 'notfound'}
            <strong>Nenhuma câmera ou placa encontrada</strong>
            <span>Confira se ela está conectada e liberada em Configurações do Windows › Privacidade › Câmera.</span>
          {:else if camera.error}
            <strong>A câmera não abriu</strong>
            <span>Confira as permissões de câmera do Windows e tente de novo.</span>
          {:else}
            <span>Nenhuma câmera ou placa encontrada.</span>
          {/if}
          <button class="btn btn-secondary" onclick={() => oncamera?.allow()}>Tentar de novo</button>
        </div>
      {:else}
        <div class="grid" role="listbox" aria-label="Câmeras">
          {#each camera.devices as d (d.id)}
            <SourceCard
              source={{
                id: d.id,
                kind: 'camera',
                title: d.label,
                app: d.kind === 'capture' ? 'Placa de captura' : 'Câmera',
                icon: null,
                thumb: null,
                playing: false,
                primary: false,
              }}
              stream={d.preview}
              busy={d.busy}
              selected={d.id === camera.videoId}
              onselect={(id) => oncamera?.select(id)}
            />
          {/each}
        </div>
      {/if}
    {:else if native}
      {#if picker.loading}
        <div class="empty">Procurando suas janelas…</div>
      {:else if shown.length === 0}
        <div class="empty">{screen ? 'Nenhuma tela encontrada.' : 'Nenhuma janela aberta pra mostrar.'}</div>
      {:else}
        <div class="grid" role="listbox" aria-label={screen ? 'Telas' : 'Janelas'}>
          {#each shown as source (source.id)}
            <SourceCard {source} selected={source.id === selected?.id} {onselect} />
          {/each}
        </div>
      {/if}
    {:else}
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
    {/if}

    <div class="bar">
      <label class="toggle-row" class:locked={audioLocked}>
        <input
          class="switch"
          type="checkbox"
          role="switch"
          checked={audioOn}
          disabled={audioLocked}
          onchange={(e) => (audio = e.currentTarget.checked)}
        />
        <span class="toggle-text">
          <span class="toggle-title">Compartilhar áudio</span>
          <span class="toggle-sub">{audioHint}</span>
        </span>
      </label>
      {#if isCamera && audio && camGranted && !camera.mobile}
        <div class="audio-pick">
          <label class="audio-select">
            <span class="visually-hidden">Entrada de som</span>
            <select value={camera.audioId ?? ''} onchange={(e) => oncamera?.selectAudio(e.currentTarget.value || null)}>
              <option value="">Sem som</option>
              {#each camera.audios as a (a.id)}<option value={a.id}>{a.label}</option>{/each}
            </select>
          </label>
          <div class="seg" role="radiogroup" aria-label="Tipo de som">
            <label class="seg-opt">
              <input type="radio" name="role" checked={camera.audioRole === 'voice'} onchange={() => oncamera?.setRole('voice')} />Som de voz
            </label>
            <label class="seg-opt">
              <input type="radio" name="role" checked={camera.audioRole === 'game'} onchange={() => oncamera?.setRole('game')} />Som de jogo
            </label>
          </div>
          {#if camera.audioId && camera.audioRole === 'voice'}
            <p class="tip">Se você está no Discord, a galera pode te ouvir duas vezes. Fone de ouvido evita eco.</p>
          {/if}
        </div>
      {/if}
      {#if !camera.mobile || !isCamera}
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
      {/if}
    </div>

    <div class="foot">
      <p class="note"><Info size={15} strokeWidth={2.75} />{footHint}</p>
      <div class="actions">
        <button class="btn btn-secondary" onclick={() => onclose?.()}>Cancelar</button>
        <button class="btn btn-primary" disabled={!canStart} onclick={() => onstart?.({ ...prefs, surface, resolution, fps, audio })}>
          <Radio size={18} strokeWidth={2.75} />Transmitir
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0; z-index: 20; display: grid; place-items: safe center;
    padding: max(16px, env(safe-area-inset-top)) max(16px, env(safe-area-inset-right)) max(16px, env(safe-area-inset-bottom)) max(16px, env(safe-area-inset-left));
    overflow-y: auto; background: var(--color-backdrop);
  }
  .dialog {
    width: min(680px, 100%); display: flex; flex-direction: column; gap: 20px; padding: 28px;
    border-radius: var(--radius-stage); background: var(--color-surface);
    box-shadow: 0 0 0 1px rgba(245, 234, 216, 0.08), 0 24px 60px rgba(0, 0, 0, 0.6);
  }
  .dialog:focus { outline: none; }
  .dialog.wide { width: min(920px, 100%); }
  .head { display: flex; align-items: flex-start; gap: 14px; }
  .badge {
    flex: none; width: 48px; height: 48px; border-radius: 50%; display: grid; place-items: center;
    background: var(--color-accent); color: var(--color-accent-900);
  }
  .titles { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .title { font-family: var(--font-heading); font-size: 26px; line-height: 1.1; }
  .subtitle { font-size: 14px; color: color-mix(in srgb, var(--color-text) 62%, transparent); }

  .tabs-row { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 8px 16px; }
  .tabs {
    align-self: flex-start; display: flex; padding: 4px; gap: 4px; border-radius: 999px;
    background: rgba(245, 234, 216, 0.06);
  }
  .count { font-weight: 500; opacity: 0.7; }
  .refresh { font-size: 13px; color: color-mix(in srgb, var(--color-text) 55%, transparent); }

  .grid {
    display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px;
    max-height: min(52vh, 470px); overflow-y: auto; overscroll-behavior: contain; padding: 2px 6px 2px 2px;
  }
  @media (max-width: 760px) { .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
  @media (max-width: 440px) { .grid { grid-template-columns: minmax(0, 1fr); } }
  .empty {
    display: grid; place-items: center; min-height: 160px; border-radius: 22px; font-size: 14px;
    color: color-mix(in srgb, var(--color-text) 60%, transparent); background: rgba(245, 234, 216, 0.04);
  }
  .wide .toggle-row { flex: 1 1 320px; }
  .empty.column { display: flex; flex-direction: column; justify-content: center; gap: 12px; padding: 24px; text-align: center; text-wrap: pretty; }
  .empty.column strong { color: var(--color-text); font-size: 15px; }

  /* Aba Câmeras: entrada de som + voz/jogo (specs/006). */
  .audio-pick { flex: 1 1 100%; display: flex; flex-wrap: wrap; align-items: center; gap: 10px; padding: 0 6px; }
  .audio-select { flex: 1 1 240px; min-width: 0; }
  .audio-select select {
    width: 100%; min-height: 40px; padding: 0 14px; border-radius: var(--radius-pill); font: inherit; font-size: 14px;
    color: var(--color-text); background: rgba(245, 234, 216, 0.07); border: 1px solid var(--color-divider);
  }
  .audio-select select:focus-visible { outline: none; box-shadow: var(--ring-focus); border-color: var(--color-accent); }
  .audio-pick .seg { padding: 4px; gap: 2px; border: 0; background: rgba(245, 234, 216, 0.07); }
  .audio-pick .seg-opt { padding: 7px 14px; font-size: 14px; font-weight: 600; border-radius: var(--radius-pill); white-space: nowrap; }
  .audio-pick .seg-opt + .seg-opt { border-left: 0; }
  .tip { flex: 1 1 100%; margin: 0; font-size: 13px; color: color-mix(in srgb, var(--color-accent-400) 85%, transparent); }
  .facing { align-self: flex-start; }
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
  /* Trilho com pílulas dentro (design): sem divisória e com a escolhida arredondada. */
  .opt .seg { padding: 4px; gap: 2px; border: 0; background: rgba(245, 234, 216, 0.07); }
  .opt .seg-opt { padding: 7px 14px; font-size: 14px; font-weight: 600; border-radius: var(--radius-pill); }
  .opt .seg-opt + .seg-opt { border-left: 0; }

  .foot { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; }
  .note {
    flex: 1 1 220px; margin: 0; display: flex; align-items: center; gap: 8px; font-size: 13px;
    color: color-mix(in srgb, var(--color-text) 55%, transparent);
  }
  .note :global(svg) { flex: none; }
  .actions { margin-left: auto; display: flex; gap: 8.8px; }
  .actions .btn { padding: 12px 20px; font-size: 15px; }
  .actions .btn-primary { padding: 12px 22px; }

  /* Celular (specs/018 R7): no topo, pro teclado não cobrir o campo nem o botão. */
  @media (max-width: 639px), (pointer: coarse) and (hover: none) and (max-height: 540px) {
    .backdrop { place-items: start center; }
  }
</style>
