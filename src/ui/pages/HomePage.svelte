<script lang="ts">
  import { cleanNickname, NICKNAME_MAX } from '../../lib/join-view';
  import type { HomeActions, HomeView } from '../../lib/types';
  import Logo from '../components/Logo.svelte';
  import Mascot from '../components/Mascot.svelte';
  import StreamTile from '../components/StreamTile.svelte';

  let { view, actions }: { view: HomeView; actions: HomeActions } = $props();

  // svelte-ignore state_referenced_locally
  let nickname = $state(view.nickname);
  // svelte-ignore state_referenced_locally
  let link = $state(view.link);

  const canCreate = $derived(cleanNickname(nickname).length > 0);
</script>

<div class="home">
  <div class="deco" aria-hidden="true">
    <div class="blob blob-a"></div>
    <div class="blob blob-b"></div>
    <div class="art art-castor"><Mascot mood="peek" size={150} /></div>
    <div class="art art-main">
      <StreamTile
        view={{ id: 'lu', name: 'Lu', tone: 'accent', caption: 'tela de Lu', status: 'ok', muted: false, volume: 1 }}
        alwaysName
        controls={false}
      />
    </div>
    <div class="art art-thumb">
      <StreamTile
        view={{ id: 'bia', name: 'Bia', tone: 'sage', caption: 'aba de Bia', status: 'ok', muted: false, volume: 1 }}
        size="thumb"
        controls={false}
      />
    </div>
  </div>

  <header class="top"><Logo size={36} /></header>

  <main class="content">
    <h1>Mostre sua tela pros amigos.</h1>
    <p class="lead">Crie uma sala, mande o link, pronto. Sem conta, sem instalar nada — até 10 pessoas.</p>

    <form
      class="create"
      onsubmit={(e) => {
        e.preventDefault();
        if (canCreate) actions.createRoom(cleanNickname(nickname));
      }}
    >
      <label for="home-nickname">Seu apelido</label>
      <div class="row">
        <input
          id="home-nickname"
          class="input big"
          bind:value={nickname}
          maxlength={NICKNAME_MAX}
          autocomplete="nickname"
          placeholder="Como a galera te chama?"
        />
        <button class="btn btn-primary big-btn" disabled={!canCreate || view.creating}>Criar sala</button>
      </div>
      {#if view.createError}<span class="error" role="alert">{view.createError}</span>{/if}
    </form>

    <form
      class="join"
      onsubmit={(e) => {
        e.preventDefault();
        actions.openLink(link);
      }}
    >
      <label for="home-link" class="join-label">Tem um link?</label>
      <div class="join-field">
        <input
          id="home-link"
          class="input link-input"
          bind:value={link}
          oninput={() => view.linkError && actions.clearLinkError()}
          placeholder="Cole o link ou código da sala"
          aria-invalid={view.linkError ? 'true' : undefined}
          aria-describedby={view.linkError ? 'home-link-error' : undefined}
          autocomplete="off"
          spellcheck="false"
        />
        {#if view.linkError}<span id="home-link-error" class="error">{view.linkError}</span>{/if}
      </div>
      <button class="btn btn-secondary join-btn" disabled={!link.trim()}>Entrar</button>
    </form>
  </main>
</div>

<style>
  .home {
    position: relative; min-height: 100%; overflow: hidden;
    background: var(--color-bg); color: var(--color-text);
  }
  .deco { position: absolute; inset: 0; pointer-events: none; }
  .blob { position: absolute; border-radius: 50%; }
  .blob-a { right: -160px; top: 90px; width: 720px; height: 720px; background: rgba(198, 113, 57, 0.16); }
  .blob-b { right: 300px; top: 520px; width: 300px; height: 300px; background: rgba(122, 138, 94, 0.22); }
  .art { position: absolute; }
  /* O castor espia por trás do tile principal (vem antes dele no DOM). */
  .art-castor { right: 300px; top: 150px; transform: rotate(-4deg); }
  .art-main {
    right: 130px; top: 260px; width: 520px; height: 292px; transform: rotate(-4deg);
    border-radius: var(--radius-card); box-shadow: 0 30px 70px rgba(0, 0, 0, 0.5);
  }
  .art-thumb {
    right: 420px; top: 500px; width: 230px; height: 130px; transform: rotate(5deg);
    border-radius: var(--radius-tile-sm); box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  }

  .top { position: relative; padding: 36px 72px; }
  .content {
    position: relative; max-width: 640px; padding: 56px 72px 72px;
    display: flex; flex-direction: column; gap: 22px; box-sizing: content-box;
  }
  h1 { margin: 0; font-family: var(--font-heading); font-weight: 400; font-size: 76px; line-height: 1; letter-spacing: -0.015em; }
  .lead { margin: 0; font-size: 20px; text-wrap: pretty; color: color-mix(in srgb, var(--color-text) 68%, transparent); }

  .create { display: flex; flex-direction: column; gap: 12px; margin-top: 14px; width: min(480px, 100%); }
  .create label { font-size: 14px; font-weight: 600; opacity: 0.8; }
  .row { display: flex; gap: 10px; }
  .big { flex: 1; min-width: 0; min-height: 56px; font-size: 17px; padding-inline: 22px; }
  .big-btn { padding: 0 28px; font-size: 18px; min-height: 56px; }

  .join { display: flex; align-items: flex-start; gap: 10px; width: min(480px, 100%); margin-top: 22px; }
  .join-label { font-size: 14px; white-space: nowrap; color: var(--color-text-muted); line-height: 44px; }
  .join-field { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
  .link-input { min-height: 44px; font-size: 15px; background: transparent; }
  .link-input::placeholder { color: rgba(245, 234, 216, 0.4); }
  .join-btn { min-height: 44px; padding: 0 18px; }
  .error { font-size: 12px; color: var(--color-accent-300); padding-left: 20px; }

  @media (max-width: 1279px) {
    .art-main { right: -120px; opacity: 0.6; }
    .art-thumb { right: 140px; opacity: 0.6; }
    .art-castor { right: 50px; opacity: 0.6; }
  }
  @media (max-width: 1023px) {
    .art { display: none; }
    .blob-a { right: -300px; }
  }
  @media (max-width: 639px) {
    .top { padding: 28px 22px 0; }
    .content { padding: 40px 22px 40px; gap: 18px; }
    h1 { font-size: 44px; }
    .lead { font-size: 17px; }
    .row { flex-direction: column; }
    .join { flex-wrap: wrap; }
    .join-label { width: 100%; line-height: 1.4; }
  }
</style>
