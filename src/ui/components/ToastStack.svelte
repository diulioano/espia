<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import type { ToastView } from '../../lib/types';
  import Toast from './Toast.svelte';

  let {
    toasts,
    placement = 'top-right',
    onaction,
    ondismiss,
  }: {
    toasts: ToastView[];
    placement?: 'top-right' | 'bottom-center' | 'inline';
    onaction?: (id: string) => void;
    ondismiss?: (id: string) => void;
  } = $props();

  // No máximo 3, os mais recentes.
  const visible = $derived(toasts.slice(-3));
</script>

<div class="stack {placement}" aria-live="polite">
  {#each visible as toast (toast.id)}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div
      in:fly={{ y: -8, duration: 200 }}
      out:fade={{ duration: 150 }}
      onclick={() => toast.kind === 'error' && ondismiss?.(toast.id)}
    >
      <Toast {toast} onaction={() => onaction?.(toast.id)} />
    </div>
  {/each}
</div>

<style>
  .stack {
    position: absolute; z-index: 5;
    display: flex; flex-direction: column; gap: 10px;
  }
  .top-right { top: 24px; right: 24px; align-items: flex-end; }
  .inline { position: static; align-items: flex-end; }
  .bottom-center { position: static; align-items: center; padding: 0 16px; }
</style>
