<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  let {
    value = $bindable(''),
    label,
    hint,
    error,
    id,
    ...rest
  }: HTMLInputAttributes & { value?: string; label?: string; hint?: string; error?: string | null; id: string } =
    $props();
</script>

<div class="field">
  {#if label}<label for={id}>{label}</label>{/if}
  <input
    class="input"
    {id}
    bind:value
    aria-invalid={error ? 'true' : undefined}
    aria-describedby={error || hint ? `${id}-help` : undefined}
    {...rest}
  />
  {#if error}
    <span id="{id}-help" class="error">{error}</span>
  {:else if hint}
    <span id="{id}-help" class="hint">{hint}</span>
  {/if}
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 8px; min-width: 0; }
  label { font-size: 14px; font-weight: 600; opacity: 0.8; }
  .hint { font-size: 13px; color: color-mix(in srgb, var(--color-text) 55%, transparent); }
  .error { font-size: 12px; color: var(--color-accent-300); }
</style>
