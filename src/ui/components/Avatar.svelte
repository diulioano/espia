<script lang="ts">
  import type { Tone } from '../../lib/types';

  let { name, tone, live = false, size = 38 }: { name: string; tone: Tone; live?: boolean; size?: number } =
    $props();
</script>

<div class="avatar tone-{tone}" class:live style:--s="{size}px" aria-hidden="true">
  {name.charAt(0).toUpperCase()}
</div>

<style>
  .avatar {
    flex: none; width: var(--s); height: var(--s); border-radius: 50%;
    display: grid; place-items: center;
    font-family: var(--font-heading); font-size: calc(var(--s) * 0.45); line-height: 1;
  }
  .live { box-shadow: 0 0 0 2px var(--color-bg), 0 0 0 4px var(--color-accent); }
  .tone-accent { background: var(--color-accent-300); color: var(--color-accent-800); }
  .tone-sage { background: var(--color-accent-2-300); color: var(--color-accent-2-800); }
  .tone-sand { background: var(--color-neutral-300); color: var(--color-neutral-800); }
  .tone-deep { background: var(--color-accent-400); color: var(--color-accent-900); }
</style>
