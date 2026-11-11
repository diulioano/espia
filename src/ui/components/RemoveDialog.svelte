<script lang="ts">
  // Confirmar "Tirar da sala" (specs/014 US4): só o dono de uma sala fixa vê.
  import { UserMinus } from '@lucide/svelte';
  import { fade, scale } from 'svelte/transition';
  import type { RemoveDialogView } from '../../lib/types';

  let { view, onconfirm, onclose }: { view: RemoveDialogView; onconfirm: () => void; onclose: () => void } = $props();

  let cancel: HTMLButtonElement;
  $effect(() => cancel.focus());

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
  <div class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="remove-title" aria-describedby="remove-body" transition:scale={{ start: 0.97, duration: 150 }}>
    <div class="badge"><UserMinus size={22} strokeWidth={2.75} /></div>
    <div class="titles">
      <span id="remove-title" class="title">Tirar {view.name} da sala?</span>
      <span id="remove-body" class="body">Sai na hora e vê um aviso. Se quiser, pode voltar pelo link.</span>
    </div>
    {#if view.error}<span class="error" role="alert">{view.error}</span>{/if}
    <div class="actions">
      <button class="btn btn-secondary" bind:this={cancel} onclick={onclose}>Cancelar</button>
      <button class="btn btn-remove" disabled={view.busy} onclick={onconfirm}>Tirar da sala</button>
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
    width: min(440px, 100%); display: flex; flex-direction: column; gap: 20px; padding: 30px;
    border-radius: var(--radius-stage); background: var(--color-surface); color: var(--color-text);
    box-shadow: 0 0 0 1px rgba(245, 234, 216, 0.08), 0 24px 60px rgba(0, 0, 0, 0.6);
  }
  .badge {
    flex: none; width: 52px; height: 52px; border-radius: 50%; display: grid; place-items: center;
    background: var(--color-accent-300); color: var(--color-accent-900);
  }
  .titles { display: flex; flex-direction: column; gap: 8px; }
  .title { font-family: var(--font-heading); font-size: 26px; line-height: 1.15; overflow-wrap: anywhere; }
  .body { font-size: 15px; text-wrap: pretty; color: color-mix(in srgb, var(--color-text) 72%, transparent); }
  .error { font-size: 13px; color: var(--color-accent-300); }
  .actions { display: flex; justify-content: flex-end; gap: 10px; }
  .actions .btn { padding: 12px 20px; font-size: 15px; }
  .btn-remove { padding: 12px 22px; background: var(--color-accent-300); color: var(--color-accent-900); }
  .btn-remove:hover:not(:disabled) { background: var(--color-accent-200); }

  /* Celular (specs/018 R7): no topo, pro teclado não cobrir o campo nem o botão. */
  @media (max-width: 639px), (pointer: coarse) and (hover: none) and (max-height: 540px) {
    .backdrop { place-items: start center; }
  }
</style>
