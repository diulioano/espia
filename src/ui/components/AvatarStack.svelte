<script lang="ts">
  import type { Tone } from '../../lib/types';
  import Avatar from './Avatar.svelte';

  let {
    people,
    size = 44,
    ring = 'var(--color-surface)',
  }: { people: { name: string; tone: Tone }[]; size?: number; ring?: string } = $props();
</script>

<div class="stack" style:--ring={ring}>
  {#each people as p, i (i)}
    <span class="item"><Avatar name={p.name} tone={p.tone} {size} /></span>
  {/each}
</div>

<style>
  .stack { display: flex; }
  .item { display: block; border-radius: 50%; box-shadow: 0 0 0 3px var(--ring); }
  .item + .item { margin-left: -10px; }
</style>
