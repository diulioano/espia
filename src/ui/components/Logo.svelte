<script lang="ts">
  import Mascot from './Mascot.svelte';

  // size = largura do castor; a palavra acompanha: 36 (home) → 28px, 32 (sala) → 26px, 29 (celular) → 24px.
  // Clicar leva pro início (specs/013): a tela decide como (sair da sala, voltar ao topo…).
  let { size = 32, onhome }: { size?: number; onhome?: () => void } = $props();

  const word = $derived(size >= 36 ? 28 : size <= 30 ? 24 : 26);

  function onclick(e: MouseEvent) {
    // Ctrl/⌘/Shift ou botão do meio: deixa o navegador abrir o início em outra aba.
    if (!onhome || e.ctrlKey || e.metaKey || e.shiftKey || e.button !== 0) return;
    e.preventDefault();
    onhome();
  }
</script>

<a class="logo" href="/" aria-label="Espia — início" style:--w="{word}px" {onclick}>
  <Mascot mood="happy" {size} animate="blink" />
  <span class="word" aria-hidden="true">Espia</span>
</a>

<style>
  .logo {
    display: inline-flex; align-items: center; gap: 10px; color: inherit; text-decoration: none;
    border-radius: var(--radius-pill);
  }
  .logo:focus-visible { outline: none; box-shadow: var(--ring-focus); }
  .word { font-family: var(--font-heading); font-size: var(--w); line-height: 1; }
</style>
