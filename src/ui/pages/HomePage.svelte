<script lang="ts">
  import { cleanNickname, NICKNAME_MAX } from '../../lib/join-view';
  import { Download, Pin } from '@lucide/svelte';
  import { DOWNLOAD_URL } from '../../lib/app-invite';
  import type { HomeActions, HomeView } from '../../lib/types';
  import AppInvite from '../components/AppInvite.svelte';
  import Logo from '../components/Logo.svelte';
  import Mascot from '../components/Mascot.svelte';
  import DeleteRoomDialog from '../components/DeleteRoomDialog.svelte';
  import NameDialog from '../components/NameDialog.svelte';
  import SavedRoomsPanel from '../components/SavedRoomsPanel.svelte';
  import StreamTile from '../components/StreamTile.svelte';
  import ToastStack from '../components/ToastStack.svelte';

  let { view, actions }: { view: HomeView; actions: HomeActions } = $props();

  // svelte-ignore state_referenced_locally
  let nickname = $state(view.nickname);
  // svelte-ignore state_referenced_locally
  let link = $state(view.link);

  const canCreate = $derived(cleanNickname(nickname).length > 0);

  // Celular (M4): Suas salas vai no meio do formulário, compacta. Um painel só no DOM (o menu "⋯" fecha
  // com clique fora; dois painéis se atrapalhariam).
  const narrowQuery = matchMedia('(max-width: 639px)');
  let narrow = $state(narrowQuery.matches);
  $effect(() => {
    const onchange = (e: MediaQueryListEvent) => (narrow = e.matches);
    narrowQuery.addEventListener('change', onchange);
    return () => narrowQuery.removeEventListener('change', onchange);
  });
</script>

{#snippet savedRooms(compact: boolean)}
  {#if view.rooms}
    <SavedRoomsPanel
      view={view.rooms}
      {compact}
      onenter={(code) => actions.enterSaved(code, cleanNickname(nickname))}
      onmenu={actions.toggleSavedMenu}
      onforget={actions.forgetSaved}
      onrename={actions.renameSaved}
      ondelete={actions.deleteSaved}
    />
  {/if}
{/snippet}

<div class="home" class:has-rooms={!!view.rooms}>
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

  {#if view.notice}
    <!-- Aviso de chegada (specs/014 US4): "O dono te tirou da sala". -->
    <ToastStack toasts={[{ id: 'notice', kind: 'leave', text: view.notice }]} ondismiss={() => actions.dismissNotice?.()} />
  {/if}
  <header class="top">
    <Logo size={36} onhome={() => scrollTo({ top: 0, behavior: 'smooth' })} />
    <!-- Sempre à mão no site, no Windows (specs/013), mesmo depois do "Agora não" no convite. -->
    {#if view.download}
      <a class="btn btn-secondary download" href={DOWNLOAD_URL} download><Download size={16} strokeWidth={2.75} />Baixar o app</a>
    {/if}
  </header>

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
        <div class="buttons">
          <button class="btn btn-primary big-btn" disabled={!canCreate || view.creating}>Criar sala</button>
          <!-- Celular (M4): "Sala fixa" ao lado de "Criar sala". -->
          <button type="button" class="btn fixed-chip" disabled={!canCreate || view.creating} onclick={actions.openFixedDialog}>
            <Pin size={14} strokeWidth={2.75} />Sala fixa
          </button>
        </div>
      </div>
      <!-- Sala fixa (specs/011 F1): com nome e um link que não muda. -->
      <div class="fixed-row">
        <button type="button" class="btn fixed-btn" disabled={!canCreate || view.creating} onclick={actions.openFixedDialog}>
          <Pin size={17} strokeWidth={2.75} />
          Criar sala fixa
        </button>
        <span class="fixed-note">Com nome e um link que não muda.</span>
      </div>
      {#if view.createError}<span class="error" role="alert">{view.createError}</span>{/if}
    </form>

    {#if narrow}{@render savedRooms(true)}{/if}

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

    {#if view.appInvite}
      <AppInvite state={view.appInvite} ondownload={actions.downloadApp} ondismiss={actions.dismissAppInvite} />
    {/if}
  </main>

  <!-- Suas salas (specs/011 F2): toma o lugar da ilustração. -->
  {#if !narrow && view.rooms}<div class="saved">{@render savedRooms(false)}</div>{/if}

  {#if view.fixedDialog}
    <NameDialog
      view={view.fixedDialog}
      onsubmit={(name) =>
        view.fixedDialog?.mode === 'rename' ? actions.submitRename(name) : actions.createFixedRoom(cleanNickname(nickname), name)}
      onclose={actions.closeFixedDialog}
    />
  {/if}
  {#if view.deleteDialog}
    <DeleteRoomDialog view={view.deleteDialog} onconfirm={actions.confirmDelete} onclose={actions.closeDeleteDialog} />
  {/if}
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

  .top { position: relative; z-index: 2; display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 36px 72px; }
  .download { min-height: 40px; padding: 0 16px; font-size: 14px; gap: 8px; }
  /* Com Suas salas, a ilustração sai e o painel entra no lugar. */
  .has-rooms .art { display: none; }
  .saved { position: absolute; right: 72px; top: 104px; z-index: 1; }
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
  .buttons { display: flex; gap: 8px; }
  .fixed-chip { display: none; }
  .fixed-row { display: flex; align-items: center; gap: 12px; margin-left: -6px; }
  .fixed-btn {
    min-height: 44px; padding: 0 16px 0 12px; font-size: 16px; color: var(--color-accent-400); background: transparent;
  }
  .fixed-btn:hover:not(:disabled) { background: rgba(198, 113, 57, 0.12); }
  .fixed-note { font-size: 14px; color: rgba(245, 234, 216, 0.55); }

  .join { display: flex; align-items: flex-start; gap: 10px; width: min(480px, 100%); margin-top: 22px; }
  .join-label { font-size: 14px; white-space: nowrap; color: var(--color-text-muted); line-height: 44px; }
  .join-field { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
  .link-input { min-height: 44px; font-size: 15px; background: transparent; }
  .link-input::placeholder { color: rgba(245, 234, 216, 0.4); }
  .join-btn { min-height: 44px; padding: 0 18px; }
  .error { font-size: 12px; color: var(--color-accent-300); padding-left: 20px; }

  /* Notebook (768 px de altura) com o convite do app: compacta para caber sem rolar (specs/008 SC-804). */
  @media (min-width: 640px) and (max-height: 820px) {
    .home:has(:global(.invite)) .top { padding-block: 24px 0; }
    .content:has(:global(.invite)) { padding-block: 28px 20px; gap: 16px; }
    .content:has(:global(.invite)) h1 { font-size: 64px; }
    .content:has(:global(.invite)) .join { margin-top: 12px; }
  }

  /* Sem espaço ao lado do texto: o painel desce pra baixo dele. */
  @media (max-width: 1399px) {
    .saved { position: relative; right: auto; top: auto; padding: 0 72px 72px; }
  }
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
    .top { padding: max(28px, calc(env(safe-area-inset-top) + 12px)) max(22px, env(safe-area-inset-right)) 0 max(22px, env(safe-area-inset-left)); }
    .content { padding: 40px max(22px, env(safe-area-inset-right)) max(40px, env(safe-area-inset-bottom)) max(22px, env(safe-area-inset-left)); gap: 18px; }
    h1 { font-size: 44px; }
    .lead { font-size: 17px; }
    .row { flex-direction: column; }
    .buttons .big-btn { flex: 1; min-height: 52px; font-size: 17px; }
    .fixed-chip {
      display: inline-flex; min-height: 52px; padding: 0 16px; font-size: 15px;
      color: var(--color-accent-400); background: rgba(198, 113, 57, 0.12);
    }
    .fixed-row { display: none; }
    .join { flex-wrap: wrap; }
    .join-label { width: 100%; line-height: 1.4; }
  }

  /* Celular deitado (specs/018 R7): o entalhe fica de lado. */
  @media (pointer: coarse) and (max-height: 540px) and (min-width: 640px) {
    .home { padding-left: env(safe-area-inset-left); padding-right: env(safe-area-inset-right); padding-bottom: env(safe-area-inset-bottom); }
  }
</style>
