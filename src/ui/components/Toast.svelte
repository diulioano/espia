<script lang="ts">
  import { Check, Radio, TriangleAlert, UserMinus, UserPlus } from '@lucide/svelte';
  import type { ToastView } from '../../lib/types';

  let { toast, onaction }: { toast: ToastView; onaction?: () => void } = $props();

  const icons = { join: UserPlus, leave: UserMinus, live: Radio, copied: Check, error: TriangleAlert };
  const Icon = $derived(icons[toast.kind]);
</script>

<div class="toast" class:has-action={!!toast.action} role={toast.kind === 'error' ? 'alert' : 'status'}>
  <span class="icon kind-{toast.kind}"><Icon size={17} strokeWidth={2.75} /></span>
  <span class="text">{toast.text}</span>
  {#if toast.action}
    <button class="btn action" onclick={() => onaction?.()}>{toast.action}</button>
  {/if}
</div>

<style>
  .toast {
    display: inline-flex; align-items: center; gap: 12px;
    padding: 8px 20px 8px 8px; border-radius: var(--radius-pill);
    background: var(--color-raised); color: var(--color-text);
    box-shadow: var(--shadow-lg);
  }
  .has-action { padding-right: 8px; }
  .icon { flex: none; width: 34px; height: 34px; border-radius: 50%; display: grid; place-items: center; }
  .kind-join, .kind-copied { background: var(--color-accent-2-300); color: var(--color-accent-2-900); }
  .kind-leave { background: rgba(245, 234, 216, 0.14); color: var(--color-text); }
  .kind-live { background: var(--color-accent); color: var(--color-accent-900); }
  .kind-error { background: var(--color-accent-300); color: var(--color-accent-900); }
  .text { font-size: 14px; line-height: 1.3; white-space: nowrap; }
  .action { padding: 7px 14px; font-size: 13px; background: rgba(245, 234, 216, 0.12); }
  .action:hover { background: rgba(245, 234, 216, 0.2); }
</style>
