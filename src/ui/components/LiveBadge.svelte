<script lang="ts">
  let { size = 'sm', label = 'AO VIVO' }: { size?: 'sm' | 'md'; label?: string } = $props();
</script>

<span class="badge {size}"><span class="dot"></span>{label}</span>

<style>
  .badge {
    flex: none; display: inline-flex; align-items: center; gap: 5px;
    border-radius: var(--radius-pill); background: var(--color-accent); color: var(--color-accent-900);
    font-weight: 700; letter-spacing: 0.08em; line-height: 1.3; text-transform: uppercase;
  }
  .sm { font-size: 10px; padding: 3px 8px; }
  .md { font-size: 12px; padding: 5px 12px; gap: 7px; }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--color-accent-900); }
</style>
