<script lang="ts">
  // Confirmar "Apagar sala" (specs/011 S4, Room.dc.html showDelete).
  import { Trash, Users } from '@lucide/svelte';
  import { fade, scale } from 'svelte/transition';
  import type { DeleteDialogView } from '../../lib/types';

  let { view, onconfirm, onclose }: { view: DeleteDialogView; onconfirm: () => void; onclose: () => void } = $props();

  let cancel: HTMLButtonElement;
  // Foco no "Cancelar": apagar não tem volta.
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
  <div class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="delete-title" aria-describedby="delete-body" transition:scale={{ start: 0.97, duration: 150 }}>
    <div class="badge"><Trash size={22} strokeWidth={2.75} /></div>
    <div class="titles">
      <span id="delete-title" class="title">Apagar "{view.name}"?</span>
      <span id="delete-body" class="body">O link para de funcionar para todo mundo. Não tem como desfazer.</span>
    </div>
    {#if view.people > 0}
      <div class="warn">
        <Users size={16} strokeWidth={2.75} />
        <span>
          Tem {view.people} {view.people === 1 ? 'pessoa' : 'pessoas'} aqui agora. {view.people === 1 ? 'Ela sai' : 'Elas saem'} da sala e
          {view.people === 1 ? 'vê' : 'veem'} um aviso.
        </span>
      </div>
    {/if}
    {#if view.error}<span class="error" role="alert">{view.error}</span>{/if}
    <div class="actions">
      <button class="btn btn-secondary" bind:this={cancel} onclick={onclose}>Cancelar</button>
      <button class="btn btn-delete" disabled={view.busy} onclick={onconfirm}>Apagar sala</button>
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
    width: min(480px, 100%); display: flex; flex-direction: column; gap: 20px; padding: 30px;
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
  .warn {
    display: flex; gap: 10px; align-items: flex-start; padding: 12px 14px; border-radius: 18px;
    background: rgba(198, 113, 57, 0.12); font-size: 14px; color: var(--color-accent-200);
  }
  .warn :global(svg) { flex: none; margin-top: 2px; }
  .error { font-size: 13px; color: var(--color-accent-300); }
  .actions { display: flex; justify-content: flex-end; gap: 10px; }
  .actions .btn { padding: 12px 20px; font-size: 15px; }
  .btn-delete { padding: 12px 22px; background: var(--color-accent-300); color: var(--color-accent-900); }
  .btn-delete:hover:not(:disabled) { background: var(--color-accent-200); }

  /* Celular (specs/018 R7): no topo, pro teclado não cobrir o campo nem o botão. */
  @media (max-width: 639px), (pointer: coarse) and (hover: none) and (max-height: 540px) {
    .backdrop { place-items: start center; }
  }
</style>
