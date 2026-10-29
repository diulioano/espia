<script lang="ts">
  import Mascot from './Mascot.svelte';

  // size = largura do castor; a palavra acompanha: 36 (home) → 28px, 32 (sala) → 26px, 29 (celular) → 24px.
  let { size = 32 }: { size?: number } = $props();

  const word = $derived(size >= 36 ? 28 : size <= 30 ? 24 : 26);
</script>

<div class="logo" style:--w="{word}px">
  <Mascot mood="happy" {size} animate="blink" />
  <span class="word">Espia</span>
</div>

<style>
  .logo { display: flex; align-items: center; gap: 10px; }
  .word { font-family: var(--font-heading); font-size: var(--w); line-height: 1; }
</style>
