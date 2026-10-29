<script lang="ts">
  import { ChevronDown, ChevronUp } from '@lucide/svelte';
  import type { MockScreen } from './registry';
  import { MOCK_SCREENS } from './registry';

  let {
    screen,
    scene,
    onnavigate,
  }: { screen: MockScreen; scene: string; onnavigate: (screen: string, scene?: string) => void } = $props();

  function readOpen(): boolean {
    try {
      return localStorage.getItem('espia:mock-switcher') !== 'closed';
    } catch {
      return true;
    }
  }

  let open = $state(readOpen());

  function toggle() {
    open = !open;
    try {
      localStorage.setItem('espia:mock-switcher', open ? 'open' : 'closed');
    } catch {
      // armazenamento bloqueado: só não lembra
    }
  }
</script>

<div class="switcher" class:open>
  <button class="bar" onclick={toggle} aria-expanded={open}>
    <span class="tag">MOCK</span>
    <span class="current-label">{screen.label} · {screen.scenes.find((s) => s.id === scene)?.label}</span>
    {#if open}<ChevronDown size={16} />{:else}<ChevronUp size={16} />{/if}
  </button>
  {#if open}
    <div class="body">
      {#if MOCK_SCREENS.length > 1}
        <select value={screen.id} onchange={(e) => onnavigate(e.currentTarget.value)} aria-label="Tela">
          {#each MOCK_SCREENS as s}<option value={s.id}>{s.label}</option>{/each}
        </select>
      {/if}
      <div class="scenes" role="radiogroup" aria-label="Cena">
        {#each screen.scenes as s}
          <button
            role="radio"
            aria-checked={s.id === scene}
            class:current={s.id === scene}
            onclick={() => onnavigate(screen.id, s.id)}
            title={s.label}
          >
            <b>{s.code}</b> {s.id}
          </button>
        {/each}
      </div>
      <a href="?mock">todas as telas</a>
    </div>
  {/if}
</div>

<style>
  .switcher {
    position: fixed; right: 16px; bottom: 16px; z-index: 50; width: 300px; max-width: calc(100vw - 32px);
    border-radius: 18px; background: rgba(12, 11, 10, 0.92); backdrop-filter: blur(8px);
    box-shadow: var(--shadow-lg); font-size: 12px; color: var(--color-text);
  }
  .switcher:not(.open) { width: auto; }
  .bar {
    width: 100%; display: flex; align-items: center; gap: 8px; padding: 8px 12px;
    border: 0; background: none; color: inherit; cursor: pointer; text-align: left;
  }
  .current-label { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tag {
    font-size: 10px; font-weight: 700; letter-spacing: 0.08em; padding: 2px 7px; border-radius: 999px;
    background: var(--color-accent-2-400); color: var(--color-accent-2-900);
  }
  .body { display: flex; flex-direction: column; gap: 8px; padding: 0 12px 12px; }
  select {
    font: inherit; padding: 6px 10px; border-radius: 999px; border: 1px solid var(--color-divider);
    background: var(--color-surface); color: inherit;
  }
  .scenes { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; }
  .scenes button {
    font: inherit; text-align: left; padding: 6px 10px; border-radius: 999px; cursor: pointer;
    border: 1px solid var(--color-divider); background: none; color: inherit;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .scenes button:hover { background: rgba(245, 234, 216, 0.07); }
  .scenes button.current { background: var(--color-accent); border-color: var(--color-accent); color: var(--color-bg); }
  .scenes b { font-weight: 700; opacity: 0.7; }
  a { color: var(--color-accent-400); text-align: right; }
</style>
