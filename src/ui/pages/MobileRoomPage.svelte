<script lang="ts">
  // Sala no celular (M2/M3): só assistir. Mesmo RoomView/RoomActions da RoomPage.
  import { Check, Ellipsis, Link, Smartphone } from '@lucide/svelte';
  import type { CameraActions, RoomActions, RoomView } from '../../lib/types';
  import Avatar from '../components/Avatar.svelte';
  import DeleteRoomDialog from '../components/DeleteRoomDialog.svelte';
  import GoLiveModal from '../components/GoLiveModal.svelte';
  import NameDialog from '../components/NameDialog.svelte';
  import RoomMenu from '../components/RoomMenu.svelte';
  import Participant from '../components/Participant.svelte';
  import Mascot from '../components/Mascot.svelte';
  import StreamTile from '../components/StreamTile.svelte';
  import ReconnectFail from '../components/ReconnectFail.svelte';
  import ReconnectPill from '../components/ReconnectPill.svelte';
  import ToastStack from '../components/ToastStack.svelte';

  let { view, actions }: { view: RoomView; actions: RoomActions } = $props();

  // Aba Câmeras do seletor (specs/006).
  const cameraActions: CameraActions = {
    allow: () => actions.allowCamera(),
    select: (id) => actions.selectCamera(id),
    selectAudio: (id) => actions.selectAudio(id),
    setRole: (role) => actions.setAudioRole(role),
    setFacing: (facing) => actions.setFacing(facing),
  };

  const connecting = $derived(view.status === 'connecting');
  /** Caiu (specs/011 M5): a sala fica, a lista esmaece e transmitir espera a volta. */
  const down = $derived(view.link !== 'ok');
  // No celular sempre há uma transmissão em destaque: a focada ou a primeira.
  const selected = $derived(view.streams.find((s) => s.id === view.focusedId) ?? view.streams[0]);
  const fixed = $derived(view.room.kind === 'fixed' ? view.room : null);
  const watchingText = $derived(selected ? `Vendo ${selected.name}` : 'Ninguém transmitindo agora');
  // Sala fixa (M5): o nome em cima, quem você vê embaixo.
  const title = $derived(connecting ? 'Entrando na sala…' : fixed ? fixed.name : selected ? `Vendo ${selected.name}` : 'Sala da galera');
  const subtitle = $derived(fixed && !connecting ? watchingText : `sala ${view.roomId}`);
</script>

<div class="mobile-room">
  <div class="scroll">
    <header class="header">
      <div class="titles">
        <div class="title-line">
          <h1>{title}</h1>
          {#if fixed?.own}
            <button
              class="btn btn-icon owner-btn"
              aria-label="Opções da sala"
              aria-haspopup="menu"
              aria-expanded={!!view.ownerMenu}
              data-room-menu-toggle
              disabled={down}
              onclick={() => actions.toggleOwnerMenu(!view.ownerMenu)}
            >
              <Ellipsis size={18} strokeWidth={2.75} />
            </button>
          {/if}
        </div>
        {#if view.ownerMenu}
          <RoomMenu variant="room" own onrename={actions.openRename} ondelete={actions.openDelete} onclose={() => actions.toggleOwnerMenu(false)} />
        {/if}
        <span class="subtitle">{subtitle}</span>
      </div>
      <button class="btn btn-invite invite" class:is-copied={view.copied} onclick={actions.copyInvite}>
        {#if view.copied}<Check size={16} strokeWidth={2.75} /> Copiado!{:else}<Link size={16} strokeWidth={2.75} /> Convidar{/if}
      </button>
    </header>

    <div class="stage">
      {#if connecting}
        <div class="stage-msg center">
          <Mascot mood="loading" size={64} />
          <span class="stage-sub">Conectando com quem já está lá.</span>
        </div>
      {:else if selected}
        {#key selected.id}
          <StreamTile
            view={selected}
            size="grid"
            ontogglemute={() => actions.toggleMute(selected.id)}
            onvolume={(v) => actions.setVolume(selected.id, v)}
            onretry={() => actions.retry(selected.id)}
            onwatch={(on) => actions.watch(selected.id, on)}
          />
        {/key}
      {:else}
        <div class="stage-msg">
          <div class="blob" aria-hidden="true"></div>
          <div class="castor" aria-hidden="true"><Mascot mood="sleepy" size={96} /></div>
          <span class="stage-title">Ninguém ao vivo ainda.</span>
          <span class="stage-sub">Quando alguém transmitir, aparece aqui.</span>
        </div>
      {/if}
      {#if view.link === 'reconnecting'}<ReconnectPill compact />{/if}
    </div>

    {#if view.streams.length > 1}
      <section class="section">
        <span class="label">Ao vivo agora · toque pra trocar</span>
        <div class="chips" role="radiogroup" aria-label="Transmissão">
          {#each view.streams as s (s.id)}
            <button
              class="chip"
              class:current={s.id === selected?.id}
              role="radio"
              aria-checked={s.id === selected?.id}
              onclick={() => actions.focus(s.id)}
            >
              <Avatar name={s.name} tone={s.tone} size={32} />
              <span class="chip-text">
                <span>{s.name}</span>
                {#if s.label}<span class="chip-what">{s.label}</span>{/if}
              </span>
            </button>
          {/each}
        </div>
      </section>
    {/if}

    <section class="section people" class:faded={down}>
      <span class="label">Na sala · {view.participants.length} de {view.capacity}</span>
      <ul class="list">
        {#each view.participants as p (p.id)}
          <Participant person={p} watching={p.id === selected?.id && selected.watching !== false} />
        {/each}
      </ul>
    </section>
  </div>

  <footer class="bar">
    {#if !view.canShare}
      <p class="note">
        <Smartphone size={16} strokeWidth={2.75} />
        <span>
          {#if view.streams.length > 0}
            Celulares não deixam compartilhar a tela pelo navegador. Pra transmitir, abra o link no computador.
          {:else}
            Pra transmitir, abra o link no computador.
          {/if}
        </span>
      </p>
    {/if}
    <div class="bar-actions">
      {#if view.myBroadcast}
        <!-- Transmitindo a câmera do celular (specs/006 US4): parar fica à mão. -->
        <button class="btn btn-stop golive" onclick={actions.stopGoLive}>Parar câmera</button>
      {:else}
        <button class="btn btn-primary golive" disabled={!view.canShare || connecting || down} onclick={actions.openGoLive}>Transmitir</button>
      {/if}
      <button class="btn btn-secondary leave" onclick={actions.leave}>Sair</button>
    </div>
  </footer>

  <div class="toasts">
    <ToastStack toasts={view.toasts} placement="bottom-center" onaction={actions.toastAction} ondismiss={actions.dismissToast} />
  </div>

  {#if view.nameDialog}
    <NameDialog view={view.nameDialog} onsubmit={actions.submitRename} onclose={actions.closeDialogs} />
  {/if}
  {#if view.deleteDialog}
    <DeleteRoomDialog view={view.deleteDialog} onconfirm={actions.confirmDelete} onclose={actions.closeDialogs} />
  {/if}

  {#if view.link === 'failed'}
    <ReconnectFail onretry={actions.retryConnection} onleave={actions.leave} />
  {/if}

  {#if view.goLiveOpen}
    <GoLiveModal
      prefs={view.goLivePrefs}
      picker={view.picker}
      camera={view.camera}
      people={view.participants.length}
      onstart={actions.startGoLive}
      onselect={actions.selectSource}
      onsurface={actions.setSurface}
      oncamera={cameraActions}
      onclose={actions.closeGoLive}
    />
  {/if}
</div>

<style>
  .mobile-room {
    position: relative; height: 100%; display: flex; flex-direction: column; overflow: hidden;
    background: var(--color-bg); color: var(--color-text);
  }
  .scroll {
    flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 18px;
    padding: max(54px, env(safe-area-inset-top)) 0 18px;
  }

  .header { display: flex; align-items: center; gap: 10px; padding: 0 16px 0 20px; }
  .titles { position: relative; display: flex; flex-direction: column; margin-right: auto; min-width: 0; }
  .title-line { display: flex; align-items: center; gap: 6px; min-width: 0; }
  .title-line h1 { min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .owner-btn { flex: none; width: 36px; height: 36px; color: var(--color-text); background: transparent; }
  .titles :global(.menu.room) { z-index: 8; }
  h1 { margin: 0; font-family: var(--font-heading); font-weight: 400; font-size: 22px; line-height: 1.1; }
  .subtitle { font-size: 13px; color: var(--color-text-muted); }
  .invite { min-height: 44px; padding: 0 16px; font-size: 14px; }

  .stage {
    position: relative; flex: none; margin: 0 12px; aspect-ratio: 16 / 9;
    border-radius: var(--radius-tile); background: var(--color-stage); overflow: hidden;
  }
  .stage-msg {
    position: absolute; inset: 0; display: flex; flex-direction: column; justify-content: center; gap: 6px;
    padding: 0 24px; overflow: hidden;
  }
  .stage-msg.center { align-items: center; gap: 14px; }
  .blob {
    position: absolute; right: -50px; top: -60px; width: 200px; height: 200px; border-radius: 50%;
    background: rgba(198, 113, 57, 0.12);
  }
  .castor { position: absolute; right: 14px; bottom: -14px; transform: rotate(-6deg); }
  .stage-title { position: relative; font-family: var(--font-heading); font-size: 26px; }
  .stage-sub { position: relative; font-size: 14px; color: color-mix(in srgb, var(--color-text) 62%, transparent); }

  .section { display: flex; flex-direction: column; gap: 10px; padding: 0 16px; }
  .label { font-size: 13px; font-weight: 600; opacity: 0.7; }
  .chips { display: flex; gap: 8px; overflow-x: auto; scrollbar-width: none; }
  .chip {
    flex: none; display: inline-flex; align-items: center; gap: 8px; min-height: 44px; padding: 0 16px 0 6px;
    border: 0; border-radius: var(--radius-pill); cursor: pointer;
    background: var(--color-surface); color: var(--color-text); font-weight: 600; font-size: 15px;
  }
  .chip.current { background: var(--color-accent); color: var(--color-accent-900); font-weight: 700; }
  .chip-text { display: flex; flex-direction: column; align-items: flex-start; line-height: 1.15; min-width: 0; }
  /* O que está sendo transmitido (specs/012), embaixo do nome. */
  .chip-what { max-width: 140px; font-size: 12px; font-weight: 500; opacity: 0.72; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .people { gap: 4px; padding: 0 8px; }
  .people .label { padding: 0 8px 4px; }
  .people.faded { opacity: 0.55; transition: opacity 0.3s; }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }

  .bar {
    flex: none; display: flex; flex-direction: column; gap: 12px;
    padding: 16px 16px max(30px, env(safe-area-inset-bottom));
    border-radius: 32px 32px 0 0; background: var(--color-surface);
  }
  .note {
    margin: 0; display: flex; gap: 10px; align-items: flex-start; font-size: 13px;
    color: color-mix(in srgb, var(--color-text) 70%, transparent);
  }
  .note :global(svg) { flex: none; margin-top: 2px; }
  .bar-actions { display: flex; gap: 10px; }
  .golive { flex: 1; min-height: 50px; font-size: 16px; }
  .leave { min-height: 50px; padding: 0 18px; }

  /* Toasts centralizados acima da barra (M3). */
  .toasts { position: absolute; left: 0; right: 0; bottom: 150px; pointer-events: none; }
  .toasts :global(.stack) { pointer-events: auto; }
</style>
