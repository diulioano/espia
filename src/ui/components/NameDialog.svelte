<script lang="ts">
  // Nome da sala fixa (specs/011): criar (Home.dc.html F7) ou renomear (Room.dc.html S3).
  import { Pin, X } from '@lucide/svelte';
  import { fade, scale } from 'svelte/transition';
  import { ROOM_NAME_MAX } from '../../shared/protocol';
  import { normalizeRoomName } from '../../shared/room-name';
  import type { NameDialogView } from '../../lib/types';

  let {
    view,
    onsubmit,
    onclose,
  }: { view: NameDialogView; onsubmit: (name: string) => void; onclose: () => void } = $props();

  const create = $derived(view.mode === 'create');
  // svelte-ignore state_referenced_locally
  let value = $state(view.value);
  const count = $derived([...value].length);
  const name = $derived(normalizeRoomName(value));

  let input: HTMLInputElement;
  $effect(() => {
    input.focus();
    input.select();
  });

  /** Limite em caracteres (emoji conta 1), não em unidades de UTF-16 como o maxlength. */
  function oninput() {
    const chars = [...value];
    if (chars.length > ROOM_NAME_MAX) value = chars.slice(0, ROOM_NAME_MAX).join('');
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      onclose();
    }
  }
</script>

<svelte:window {onkeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" transition:fade={{ duration: 150 }} onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="name-dialog-title" transition:scale={{ start: 0.97, duration: 150 }}>
    <form
      class="body"
      onsubmit={(e) => {
        e.preventDefault();
        if (name && !view.busy) onsubmit(name);
      }}
    >
      <div class="head">
        {#if create}<div class="badge"><Pin size={22} strokeWidth={2.75} /></div>{/if}
        <div class="titles">
          <span id="name-dialog-title" class="title">{create ? 'Criar sala fixa' : 'Renomear sala'}</span>
          <span class="subtitle">
            {create ? 'O link não muda. Fixe no grupo e use sempre o mesmo. Some se ficar 30 dias sem uso.' : 'O link continua o mesmo.'}
          </span>
        </div>
        <button type="button" class="btn btn-icon close" aria-label="Fechar" onclick={onclose}>
          <X size={18} strokeWidth={2.75} />
        </button>
      </div>

      <div class="field">
        <label for="room-name">Nome da sala</label>
        <input
          id="room-name"
          class="input"
          bind:this={input}
          bind:value
          {oninput}
          autocomplete="off"
          aria-invalid={view.error ? 'true' : undefined}
          aria-describedby="room-name-hint"
        />
        <div id="room-name-hint" class="hint">
          {#if view.error}
            <span class="error" role="alert">{view.error}</span>
          {:else if create}
            <span>Aparece só na tela, nunca no link.</span>
          {:else}
            <span></span>
          {/if}
          <span class="count">{count}/{ROOM_NAME_MAX}</span>
        </div>
      </div>

      <div class="actions">
        <button type="button" class="btn btn-secondary" onclick={onclose}>Cancelar</button>
        <button class="btn btn-primary" disabled={!name || view.busy}>{create ? 'Criar e entrar' : 'Salvar'}</button>
      </div>
    </form>
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0; z-index: 20; display: grid; place-items: safe center; padding: 16px;
    overflow-y: auto; background: var(--color-backdrop);
  }
  .dialog {
    width: min(480px, 100%); padding: 30px;
    border-radius: var(--radius-stage); background: var(--color-surface); color: var(--color-text);
    box-shadow: 0 0 0 1px rgba(245, 234, 216, 0.08), 0 24px 60px rgba(0, 0, 0, 0.6);
  }
  .body { display: flex; flex-direction: column; gap: 22px; }
  .head { display: flex; align-items: flex-start; gap: 14px; }
  .badge {
    flex: none; width: 48px; height: 48px; border-radius: 50%; display: grid; place-items: center;
    background: var(--color-accent-2-300); color: var(--color-accent-2-900);
  }
  .titles { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .title { font-family: var(--font-heading); font-size: 26px; line-height: 1.1; }
  .subtitle { font-size: 14px; text-wrap: pretty; color: color-mix(in srgb, var(--color-text) 62%, transparent); }
  .close { color: var(--color-text); }

  .field { display: flex; flex-direction: column; gap: 8px; }
  label { font-size: 13px; font-weight: 600; color: color-mix(in srgb, var(--color-text) 75%, transparent); }
  .input { min-height: 54px; font-size: 17px; padding-inline: 20px; background: var(--color-bg); }
  .hint { display: flex; justify-content: space-between; gap: 12px; font-size: 12px; color: color-mix(in srgb, var(--color-text) 50%, transparent); }
  .count { font-variant-numeric: tabular-nums; }
  .error { color: var(--color-accent-300); }

  .actions { display: flex; justify-content: flex-end; gap: 10px; }
  .actions .btn { padding: 12px 20px; font-size: 15px; }
  .actions .btn-primary { padding: 12px 22px; }
</style>
