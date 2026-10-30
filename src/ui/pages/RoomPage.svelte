<script lang="ts">
  import { Check, Link, LogOut, MonitorUp, ScreenShareOff } from '@lucide/svelte';
  import { audioLabelText } from '../../lib/audio-source';
  import { formatElapsed, header, watchingId } from '../../lib/room-view';
  import type { CameraActions, RoomActions, RoomView } from '../../lib/types';
  import GoLiveModal from '../components/GoLiveModal.svelte';
  import Logo from '../components/Logo.svelte';
  import Participant from '../components/Participant.svelte';
  import Stage from '../components/Stage.svelte';
  import StreamTile from '../components/StreamTile.svelte';

  let { view, actions }: { view: RoomView; actions: RoomActions } = $props();

  // Aba Câmeras do seletor (specs/006).
  const cameraActions: CameraActions = {
    allow: () => actions.allowCamera(),
    select: (id) => actions.selectCamera(id),
    selectAudio: (id) => actions.selectAudio(id),
    setRole: (role) => actions.setAudioRole(role),
    setFacing: (facing) => actions.setFacing(facing),
  };

  const head = $derived(header(view));
  const watching = $derived(watchingId(view));
  const me = $derived(view.participants.find((p) => p.you));
  const connecting = $derived(view.status === 'connecting');
  /** Caiu (specs/011 US1): a sala fica, a lista esmaece e transmitir espera a volta. */
  const down = $derived(view.link !== 'ok');
  const inviteLabel = $derived(view.inviteUrl.replace(/^https?:\/\//, ''));

  let now = $state(Date.now());
  $effect(() => {
    if (!view.myBroadcast) return;
    now = Date.now();
    const timer = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(timer);
  });
</script>

<div class="room">
  <aside class="sidebar">
    <Logo />

    <div class="people">
      <div class="people-head">
        <span class="people-title">Na sala</span>
        {#if !connecting}<span class="people-count">{view.participants.length} de {view.capacity}</span>{/if}
      </div>
      {#if connecting}
        <div class="skeletons" aria-hidden="true">
          {#each [110, 80, 96] as w, i}
            <div class="skeleton" style:opacity={i === 2 ? 0.75 : 1}>
              <span class="sk-avatar"></span><span class="sk-line" style:width="{w}px"></span>
            </div>
          {/each}
        </div>
      {:else}
        <ul class="list" class:faded={down}>
          {#each view.participants as p (p.id)}
            <Participant person={p} watching={p.id === watching} />
          {/each}
        </ul>
      {/if}
    </div>

    <div class="grow"></div>

    {#if view.myBroadcast && me}
      {@const b = view.myBroadcast}
      <div class="me-live">
        <div class="me-live-head">
          <span class="pulse"></span>
          <span class="me-live-title">Você está ao vivo</span>
        </div>
        <div class="me-live-preview">
          <StreamTile
            you
            size="thumb"
            view={{ id: me.id, name: me.name, tone: me.tone, caption: b.source === 'camera' ? 'sua câmera' : 'sua tela', status: 'ok', muted: true, volume: 0, stream: b.stream, source: b.source }}
          />
        </div>
        <span class="me-live-stats">
          {#if down}
            Sua transmissão continua · volta sozinha
          {:else}
          {b.viewers} assistindo · {b.resolution}p · {b.fps} fps{b.adjusted ? ' · ajustado' : ''} · {audioLabelText(b.audioKind, b.audioReason)}{b.source === 'camera' && b.sourceTitle ? ` · câmera ${b.sourceTitle}` : ''}
          {/if}
        </span>
      </div>
    {/if}
  </aside>

  <main class="main">
    <header class="header">
      <div class="titles">
        <h1>{head.title}</h1>
        <span class="subtitle">{head.subtitle}</span>
      </div>
      <div class="invite">
        <span class="invite-url">{inviteLabel}</span>
        <button class="btn btn-invite" class:is-copied={view.copied} onclick={actions.copyInvite}>
          {#if view.copied}
            <Check size={16} strokeWidth={2.75} /> Copiado!
          {:else}
            <Link size={16} strokeWidth={2.75} /> Copiar convite
          {/if}
        </button>
      </div>
    </header>

    <Stage {view} {actions} />

    <footer class="footer">
      {#if view.myBroadcast}
        <button class="btn btn-stop footer-main" onclick={actions.stopGoLive}>
          <ScreenShareOff size={20} strokeWidth={2.75} />
          Parar transmissão
        </button>
        <span class="live-pill">
          <span class="dot"></span>{#if down}Ao vivo · reconectando…{:else}Você está ao vivo há {formatElapsed(now - view.myBroadcast.startedAt)}{/if}
        </span>
      {:else}
        <button class="btn btn-primary footer-main" disabled={connecting || down || !view.canShare} onclick={actions.openGoLive}>
          <MonitorUp size={20} strokeWidth={2.75} />
          Transmitir
        </button>
        <span class="hint">Tela, janela ou aba — você escolhe.</span>
      {/if}
      <div class="grow"></div>
      <button class="btn btn-secondary" onclick={actions.leave}>
        <LogOut size={18} strokeWidth={2.75} />
        Sair da sala
      </button>
    </footer>
  </main>

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
  .room {
    position: relative; height: 100%; min-height: 560px; overflow: hidden;
    display: grid; grid-template-columns: 300px minmax(0, 1fr);
    background: var(--color-bg); color: var(--color-text);
  }

  .sidebar { display: flex; flex-direction: column; gap: 28px; padding: 28px 20px 24px 28px; min-height: 0; }
  .people { display: flex; flex-direction: column; gap: 10px; min-height: 0; flex: 0 1 auto; }
  .people-head { display: flex; align-items: baseline; justify-content: space-between; padding-right: 8px; }
  .people-title { font-family: var(--font-heading); font-size: 18px; }
  .people-count { font-size: 13px; color: var(--color-text-muted); }
  .list {
    list-style: none; margin: 0 0 0 -8px; padding: 0; display: flex; flex-direction: column; gap: 4px;
    min-height: 0; overflow-y: auto; transition: opacity 0.3s;
  }
  .list.faded { opacity: 0.55; }
  .skeletons { display: flex; flex-direction: column; gap: 14px; padding-top: 6px; }
  .skeleton { display: flex; align-items: center; gap: 12px; }
  .sk-avatar { width: 38px; height: 38px; border-radius: 50%; background: rgba(245, 234, 216, 0.08); }
  .sk-line { height: 12px; border-radius: 999px; background: rgba(245, 234, 216, 0.08); }
  .grow { flex: 1; }

  .me-live {
    display: flex; flex-direction: column; gap: 12px; padding: 14px;
    border-radius: var(--radius-card); background: var(--color-surface); box-shadow: var(--ring-live);
  }
  .me-live-head { display: flex; align-items: center; gap: 8px; padding: 2px 4px 0; }
  .pulse {
    width: 10px; height: 10px; border-radius: 50%; background: var(--color-accent);
    box-shadow: 0 0 0 4px rgba(198, 113, 57, 0.3);
  }
  .me-live-title { font-family: var(--font-heading); font-size: 17px; }
  .me-live-preview { height: 142px; }
  .me-live-stats { font-size: 12px; padding: 0 4px; color: color-mix(in srgb, var(--color-text) 65%, transparent); }

  .main { display: flex; flex-direction: column; gap: 18px; padding: 24px 28px 24px 0; min-width: 0; min-height: 0; }
  .header { display: flex; align-items: center; gap: 16px; min-height: 52px; }
  .titles { display: flex; flex-direction: column; gap: 2px; margin-right: auto; min-width: 0; }
  h1 { margin: 0; font-family: var(--font-heading); font-weight: 400; font-size: 28px; line-height: 1.1; }
  .subtitle { font-size: 14px; color: var(--color-text-muted); }
  .invite {
    display: flex; align-items: center; gap: 6px; padding: 6px 6px 6px 18px;
    border-radius: var(--radius-pill); background: var(--color-surface); min-width: 0;
  }
  .invite-url {
    font-family: var(--font-mono); font-size: 14px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    color: color-mix(in srgb, var(--color-text) 75%, transparent);
  }
  .invite .btn { padding: 10px 18px; }

  .footer { display: flex; align-items: center; gap: 16px; }
  .footer-main { padding: 14px 26px; font-size: 17px; }
  .btn-stop.footer-main { padding: 12px 24px; }
  .hint { font-size: 14px; color: color-mix(in srgb, var(--color-text) 55%, transparent); }
  .live-pill {
    display: inline-flex; align-items: center; gap: 8px; padding: 8px 14px; border-radius: var(--radius-pill);
    background: rgba(198, 113, 57, 0.16); font-size: 14px; color: var(--color-accent-300);
    font-variant-numeric: tabular-nums;
  }
  .live-pill .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--color-accent); }

  /* < 1024px: sidebar vira faixa horizontal acima do palco. */
  @media (max-width: 1023px) {
    .room { grid-template-columns: minmax(0, 1fr); grid-template-rows: auto minmax(0, 1fr); }
    .sidebar { flex-direction: row; align-items: center; gap: 20px; padding: 16px 20px 0; overflow-x: auto; }
    .people { flex-direction: row; align-items: center; gap: 12px; }
    .people-head { flex-direction: column; align-items: flex-start; gap: 0; padding: 0; }
    .list, .skeletons { flex-direction: row; margin: 0; padding: 0; }
    .grow { display: none; }
    .me-live { flex-direction: row; align-items: center; padding: 8px 14px 8px 8px; }
    .me-live-preview { width: 96px; height: 54px; order: -1; }
    .me-live-head { display: none; }
    .main { padding: 16px 20px 20px; }
    .footer .hint { display: none; }
  }
</style>
