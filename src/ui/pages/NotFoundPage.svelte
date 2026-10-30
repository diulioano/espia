<script lang="ts">
  // Sala não encontrada (specs/011, NotFound.dc.html N1–N4): rápida que acabou, fixa apagada,
  // apagada com você dentro ou expirada depois de 30 dias.
  import type { NotFoundActions, NotFoundView } from '../../lib/types';
  import Logo from '../components/Logo.svelte';
  import Mascot from '../components/Mascot.svelte';

  let { view, actions }: { view: NotFoundView; actions: NotFoundActions } = $props();

  const COPY: Record<NotFoundView['reason'], [string, string]> = {
    quick: [
      'Essa sala não existe mais.',
      'O link pode estar incompleto, ou todo mundo saiu — salas rápidas somem quando ficam vazias.',
    ],
    deleted: ['Esta sala foi apagada.', 'Quem criou a sala apagou. O link não funciona mais.'],
    kicked: ['Esta sala foi apagada.', 'Quem criou a sala apagou agora há pouco, e todo mundo saiu junto.'],
    expired: [
      'Esta sala ficou 30 dias sem uso e deixou de existir.',
      'Salas fixas somem sozinhas depois de um mês vazias. Dá pra criar outra com o mesmo nome.',
    ],
  };

  const copy = $derived(COPY[view.reason]);
  const name = $derived(view.reason === 'quick' ? null : view.name);
  const saved = $derived(view.saved && view.reason !== 'quick');
</script>

<div class="not-found">
  <div class="blob" aria-hidden="true"></div>
  <header class="top">
    <span class="logo-desktop"><Logo size={36} /></span>
    <span class="logo-mobile"><Logo size={29} /></span>
  </header>

  <main class="card">
    <Mascot mood={view.reason === 'expired' ? 'sleepy' : 'sad'} size={96} />
    <div class="titles">
      {#if name}<span class="name">{name}</span>{/if}
      <h1>{copy[0]}</h1>
      <p class="sub">{copy[1]}</p>
    </div>
    <div class="actions">
      <button class="btn btn-primary act-primary" onclick={actions.createRoom}>Criar uma sala nova</button>
      {#if saved}
        <button class="btn btn-secondary act-secondary" onclick={actions.forget}>Tirar de Suas salas</button>
      {:else}
        <button class="btn btn-secondary act-secondary" onclick={actions.pasteOtherLink}>Colar outro link</button>
      {/if}
    </div>
  </main>
</div>

<style>
  .not-found {
    position: relative; min-height: 100%; overflow: hidden; display: flex; flex-direction: column;
    background: var(--color-bg); color: var(--color-text);
  }
  .blob {
    position: absolute; right: -180px; top: -180px; width: 640px; height: 640px; border-radius: 50%;
    background: rgba(198, 113, 57, 0.1); pointer-events: none;
  }
  .top { position: relative; padding: 36px 72px; }
  .logo-mobile { display: none; }

  .card {
    position: relative; margin: 60px 0 72px 72px; width: 560px; max-width: calc(100% - 144px);
    display: flex; flex-direction: column; gap: 24px; padding: 44px;
    border-radius: 36px; background: var(--color-surface);
  }
  .titles { display: flex; flex-direction: column; gap: 10px; }
  .name { font-size: 15px; font-weight: 600; color: color-mix(in srgb, var(--color-text) 60%, transparent); overflow-wrap: anywhere; }
  h1 { margin: 0; font-family: var(--font-heading); font-weight: 400; font-size: 40px; line-height: 1.05; letter-spacing: -0.015em; text-wrap: pretty; }
  .sub { margin: 0; font-size: 17px; text-wrap: pretty; color: color-mix(in srgb, var(--color-text) 68%, transparent); }

  .actions { display: flex; flex-wrap: wrap; gap: 10px; }
  .act-primary { min-height: 52px; padding: 0 24px; font-size: 17px; }
  .act-secondary { min-height: 52px; padding: 0 22px; font-size: 16px; }

  /* Celular: sem card, conteúdo na metade de baixo (como a tela de entrada). */
  @media (max-width: 639px) {
    .not-found { padding: 60px 22px 34px; }
    .blob { right: -140px; top: -80px; width: 360px; height: 360px; background: rgba(198, 113, 57, 0.14); }
    .top { padding: 0; }
    .logo-desktop { display: none; }
    .logo-mobile { display: block; }
    .card { margin: auto 0 0; width: auto; max-width: none; padding: 0; gap: 20px; background: none; border-radius: 0; }
    h1 { font-size: 34px; }
    .actions { flex-direction: column; }
  }
</style>
