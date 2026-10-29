<script lang="ts">
  let { size = 40, width = 3 }: { size?: number; width?: number } = $props();
</script>

<div class="spinner" style:--s="{size}px" style:--w="{width}px" role="status" aria-label="Carregando"></div>

<style>
  .spinner {
    width: var(--s); height: var(--s); border-radius: 50%;
    border: var(--w) solid rgba(245, 234, 216, 0.14); border-top-color: var(--color-accent);
    animation: spin 0.9s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .spinner { animation-duration: 3s; } }
</style>
