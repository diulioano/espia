<script lang="ts">
  import { ArrowRight } from '@lucide/svelte';
  import { cleanNickname, liveLabel, NICKNAME_MAX, whoIsThere, whoIsThereLong } from '../../lib/join-view';
  import type { JoinActions, JoinView } from '../../lib/types';
  import AvatarStack from '../components/AvatarStack.svelte';
  import LiveBadge from '../components/LiveBadge.svelte';
  import Logo from '../components/Logo.svelte';
  import Mascot from '../components/Mascot.svelte';
  import TextField from '../components/TextField.svelte';

  let { view, actions }: { view: JoinView; actions: JoinActions } = $props();

  // svelte-ignore state_referenced_locally
  let nickname = $state(view.nickname);

  const canJoin = $derived(cleanNickname(nickname).length > 0);
  const badge = $derived(view.people ? liveLabel(view.people) : null);
  const subtitle = $derived.by(() => {
    if (!view.people || view.people.length === 0) return `Sala ${view.roomId} · até ${view.capacity} pessoas`;
    return view.canShare ? `${whoIsThere(view.people)} · sala ${view.roomId}` : whoIsThereLong(view.people);
  });
</script>

<div class="join-page" class:watch-only={!view.canShare} class:problem={view.status !== 'ready'}>
  <div class="blob" aria-hidden="true"></div>
  <header class="top">
    <span class="logo-desktop"><Logo size={36} /></span>
    <span class="logo-mobile"><Logo size={29} /></span>
  </header>
  {#if view.status === 'ready'}
    <!-- Espia por trás do card (vem antes dele no DOM). -->
    <div class="castor" aria-hidden="true"><Mascot mood="happy" size={110} /></div>
  {/if}

  <main class="card">
    {#if view.status === 'ready'}
      {#if view.people && view.people.length > 0}
        <div class="who">
          <span class="stack-desktop"><AvatarStack people={view.people} size={44} /></span>
          <span class="stack-mobile"><AvatarStack people={view.people} size={40} ring="var(--color-bg)" /></span>
          {#if badge}<span class="badge-wrap"><LiveBadge label={badge} /></span>{/if}
        </div>
      {/if}
      <div class="titles">
        <h1>Te chamaram pra uma sala.</h1>
        <p class="sub">{subtitle}</p>
      </div>
      <form
        class="form"
        onsubmit={(e) => {
          e.preventDefault();
          if (canJoin) actions.join(cleanNickname(nickname));
        }}
      >
        <div class="nick">
          <TextField
            id="join-nickname"
            label="Seu apelido"
            hint={view.canShare && view.nickname ? 'Lembramos da última vez. Fica só neste navegador.' : undefined}
            bind:value={nickname}
            maxlength={NICKNAME_MAX}
            autocomplete="nickname"
            placeholder="Como a galera te chama?"
          />
        </div>
        <button class="btn btn-primary enter" disabled={!canJoin}>
          {#if view.canShare}
            Entrar <ArrowRight size={20} strokeWidth={2.75} />
          {:else}
            Entrar pra assistir
          {/if}
        </button>
        {#if !view.canShare}
          <span class="mobile-note">No celular dá pra assistir. Transmitir só no computador.</span>
        {/if}
      </form>
    {:else}
      <Mascot mood="sad" size={96} />
      <div class="titles">
        {#if view.status === 'notfound'}
          <h1>Essa sala não existe mais.</h1>
          <p class="sub">O link pode estar incompleto, ou todo mundo saiu — salas somem quando ficam vazias.</p>
        {:else}
          <h1>Essa sala está cheia ({view.capacity} de {view.capacity}).</h1>
          <p class="sub">
            Cabem até {view.capacity} pessoas pra transmissão não pesar pra ninguém. Quando alguém sair, dá pra entrar.
          </p>
        {/if}
      </div>
      <div class="actions">
        {#if view.status === 'notfound'}
          <button class="btn btn-primary act-primary" onclick={actions.createRoom}>Criar uma sala nova</button>
          <button class="btn btn-secondary act-secondary" onclick={actions.pasteOtherLink}>Colar outro link</button>
        {:else}
          <button class="btn btn-primary act-primary" disabled>Entrar</button>
          <button class="btn btn-secondary act-secondary" onclick={actions.createRoom}>Criar uma sala nova</button>
        {/if}
      </div>
    {/if}
  </main>
</div>

<style>
  .join-page {
    position: relative; min-height: 100%; overflow: hidden; display: flex; flex-direction: column;
    background: var(--color-bg); color: var(--color-text);
  }
  .blob {
    position: absolute; right: -200px; bottom: -260px; width: 760px; height: 760px; border-radius: 50%;
    background: rgba(122, 138, 94, 0.16); pointer-events: none;
  }
  .problem .blob { right: -180px; top: -180px; bottom: auto; width: 640px; height: 640px; background: rgba(198, 113, 57, 0.1); }
  .top { position: relative; padding: 36px 72px; }
  .logo-mobile { display: none; }
  .castor { position: absolute; left: 470px; top: 84px; transform: rotate(5deg); }

  .card {
    position: relative; margin: 60px 0 72px 72px; width: 560px; max-width: calc(100% - 144px);
    display: flex; flex-direction: column; gap: 26px; padding: 44px;
    border-radius: 36px; background: var(--color-surface);
  }
  .problem .card { gap: 24px; }

  .who { display: flex; align-items: center; gap: 14px; }
  .stack-mobile { display: none; }
  .badge-wrap :global(.badge) { padding: 3px 9px; }

  .titles { display: flex; flex-direction: column; gap: 8px; }
  .problem .titles { gap: 10px; }
  h1 { margin: 0; font-family: var(--font-heading); font-weight: 400; font-size: 40px; line-height: 1.05; letter-spacing: -0.015em; }
  .sub { margin: 0; font-size: 17px; text-wrap: pretty; color: color-mix(in srgb, var(--color-text) 68%, transparent); }

  .form { display: flex; flex-direction: column; gap: 26px; }
  .nick :global(.input) { min-height: 56px; font-size: 17px; padding-inline: 22px; background: var(--color-bg); }
  .enter { min-height: 58px; font-size: 19px; }
  .mobile-note { font-size: 13px; text-align: center; color: color-mix(in srgb, var(--color-text) 55%, transparent); }

  .actions { display: flex; flex-wrap: wrap; gap: 10px; }
  .act-primary { min-height: 52px; padding: 0 24px; font-size: 17px; }
  .act-secondary { min-height: 52px; padding: 0 22px; font-size: 16px; }

  /* Celular (M1): sem card, conteúdo na metade de baixo. */
  @media (max-width: 639px) {
    .join-page { padding: 60px 22px 34px; }
    .blob, .problem .blob {
      right: -140px; top: -80px; bottom: auto; width: 360px; height: 360px; background: rgba(198, 113, 57, 0.14);
    }
    .top { padding: 0; }
    .logo-desktop, .castor { display: none; }
    .logo-mobile { display: block; }
    .card {
      margin: auto 0 0; width: auto; max-width: none; padding: 0; gap: 20px;
      background: none; border-radius: 0;
    }
    .stack-desktop { display: none; }
    .stack-mobile { display: block; }
    h1 { font-size: 36px; }
    .titles { gap: 0; }
    .sub { font-size: 16px; margin-top: 4px; }
    .form { gap: 20px; margin-top: 8px; }
    .nick :global(.input) { min-height: 54px; padding-inline: 20px; background: var(--color-surface); }
    .enter { min-height: 56px; font-size: 18px; }
    .actions { flex-direction: column; }
  }
</style>
