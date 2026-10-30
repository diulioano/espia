<script lang="ts">
  // Queda não é sair (specs/011 Q1/M5): aviso discreto sobre o palco enquanto reconecta.
  import Mascot from './Mascot.svelte';

  let { compact = false }: { compact?: boolean } = $props();
</script>

<div class="wrap" class:compact role="status" aria-live="polite">
  <div class="pill">
    <span class="castor"><Mascot mood="loading" size={compact ? 26 : 32} /></span>
    <span class="title">Reconectando…</span>
    {#if !compact}<span class="sub">Você continua na sala</span>{/if}
  </div>
</div>

<style>
  .wrap {
    position: absolute; left: 0; right: 0; top: 20px; z-index: 4;
    display: flex; justify-content: center; pointer-events: none;
  }
  .wrap.compact { top: auto; bottom: 12px; }
  .pill {
    display: flex; align-items: center; gap: 12px; padding: 6px 20px 6px 8px; border-radius: var(--radius-pill);
    background: #322e28; box-shadow: 0 0 0 1px rgba(245, 234, 216, 0.08), 0 14px 36px rgba(0, 0, 0, 0.5);
  }
  .compact .pill { gap: 8px; padding: 4px 14px 4px 6px; box-shadow: 0 0 0 1px rgba(245, 234, 216, 0.08), 0 10px 28px rgba(0, 0, 0, 0.5); }
  .castor { width: 36px; height: 40px; display: grid; place-items: center; }
  .compact .castor { width: 28px; height: 32px; }
  .title { font-family: var(--font-heading); font-size: 17px; }
  .compact .title { font-size: 15px; }
  .sub { font-size: 13px; color: color-mix(in srgb, var(--color-text) 60%, transparent); }
</style>
