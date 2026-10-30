<script lang="ts">
  // Não voltou em 1 minuto (specs/011 Q3). Continua tentando por trás: se voltar, isto some sozinho.
  import { RefreshCw } from '@lucide/svelte';
  import Mascot from './Mascot.svelte';

  let { onretry, onleave }: { onretry: () => void; onleave: () => void } = $props();
</script>

<div class="veil">
  <div class="card" role="alertdialog" aria-labelledby="reconnect-fail-title">
    <Mascot mood="sad" size={84} />
    <div class="titles">
      <span id="reconnect-fail-title" class="title">Não deu pra voltar pra sala.</span>
      <span class="body">A conexão caiu faz 1 minuto e não voltou. Confira sua internet e tente de novo — a sala continua lá.</span>
    </div>
    <div class="actions">
      <button class="btn btn-primary retry" onclick={onretry}>
        <RefreshCw size={18} strokeWidth={2.75} />
        Tentar de novo
      </button>
      <button class="btn btn-secondary leave" onclick={onleave}>Sair da sala</button>
    </div>
  </div>
</div>

<style>
  .veil {
    position: absolute; inset: 0; z-index: 5; display: grid; place-items: center; padding: 20px;
    background: rgba(18, 17, 15, 0.78);
  }
  .card {
    width: 520px; max-width: 100%; display: flex; flex-direction: column; align-items: flex-start; gap: 18px;
    padding: 36px; border-radius: 32px; background: var(--color-surface);
    box-shadow: 0 0 0 1px rgba(245, 234, 216, 0.08), 0 24px 60px rgba(0, 0, 0, 0.6);
  }
  .titles { display: flex; flex-direction: column; gap: 8px; }
  .title { font-family: var(--font-heading); font-size: 30px; line-height: 1.1; }
  .body { font-size: 16px; text-wrap: pretty; color: color-mix(in srgb, var(--color-text) 68%, transparent); }
  .actions { display: flex; flex-wrap: wrap; gap: 10px; }
  .retry { min-height: 50px; padding: 0 22px; font-size: 16px; }
  .leave { min-height: 50px; padding: 0 20px; font-size: 15px; }

  @media (max-width: 639px) {
    .card { padding: 28px 24px; }
    .title { font-size: 26px; }
  }
</style>
