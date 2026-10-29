<script lang="ts">
  import Mascot from './Mascot.svelte';

  let { onretry, ondismiss }: { onretry?: () => void; ondismiss?: () => void } = $props();
</script>

<div class="alert" role="alert">
  <Mascot mood="sad" size={64} />
  <div class="body">
    <span class="title">O navegador não deixou compartilhar</span>
    <span class="text">
      Você fechou o seletor ou negou a permissão. Se o navegador não perguntar de novo, libere em
      <b>Configurações do site → Compartilhar tela</b>.
    </span>
    <div class="actions">
      <button class="btn btn-primary" onclick={() => onretry?.()}>Tentar de novo</button>
      <button class="btn btn-ghost" onclick={() => ondismiss?.()}>Agora não</button>
    </div>
  </div>
</div>

<style>
  .alert {
    width: min(480px, 100%); box-sizing: border-box;
    display: flex; gap: 16px; padding: 20px; border-radius: var(--radius-card);
    background: var(--color-raised); box-shadow: var(--shadow-lg);
  }
  .body { display: flex; flex-direction: column; gap: 6px; }
  .title { font-family: var(--font-heading); font-size: 19px; line-height: 1.2; }
  .text { font-size: 14px; color: color-mix(in srgb, var(--color-text) 72%, transparent); text-wrap: pretty; }
  .actions { display: flex; gap: 8px; margin-top: 8px; }
  .actions .btn { padding: 9px 16px; font-size: 14px; }
</style>
