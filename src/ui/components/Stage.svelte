<script lang="ts">
  import { MonitorUp } from '@lucide/svelte';
  import { focusedStream, gridSize, stageLayout } from '../../lib/room-view';
  import type { RoomActions, RoomView } from '../../lib/types';
  import PermissionAlert from './PermissionAlert.svelte';
  import Mascot from './Mascot.svelte';
  import ReconnectFail from './ReconnectFail.svelte';
  import ReconnectPill from './ReconnectPill.svelte';
  import StreamTile from './StreamTile.svelte';
  import ToastStack from './ToastStack.svelte';

  let { view, actions }: { view: RoomView; actions: RoomActions } = $props();

  const layout = $derived(stageLayout(view));
  const focused = $derived(focusedStream(view));
  const others = $derived(view.streams.filter((s) => s.id !== view.focusedId));
  const gridShape = $derived(gridSize(view.streams.length));

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && layout === 'focus' && !view.goLiveOpen) actions.focus(null);
  }
</script>

<svelte:window {onkeydown} />

<section class="stage" aria-label="Palco">
  {#if layout === 'connecting'}
    <div class="center">
      <Mascot mood="loading" size={120} />
      <span class="center-title">Entrando na sala…</span>
      <span class="center-sub">Conectando com quem já está lá.</span>
    </div>
  {:else if layout === 'empty'}
    <div class="empty">
      <div class="blob blob-a"></div>
      <div class="blob blob-b"></div>
      <div class="castor" aria-hidden="true">
        <Mascot mood={view.myBroadcast ? 'live' : 'sleepy'} size={220} />
      </div>
      {#if view.myBroadcast}
        <h2>Sua tela está no ar.</h2>
        <p>Ela não aparece aqui pra não virar um espelho infinito — a prévia fica na barra do lado.</p>
      {:else}
        <h2>Ninguém ao vivo ainda.</h2>
        <p>Clique em Transmitir pra mostrar sua tela — ou mande o convite e espere alguém começar.</p>
        {#if view.canShare}
          <div class="empty-actions">
            <button class="btn btn-primary big" onclick={actions.openGoLive}>
              <MonitorUp size={20} strokeWidth={2.75} />
              Transmitir
            </button>
          </div>
        {/if}
      {/if}
    </div>
  {:else if layout === 'single'}
    {@const s = view.streams[0]}
    <div class="fill">
      <StreamTile
        view={s}
        radius="var(--radius-stage)"
        ontogglemute={() => actions.toggleMute(s.id)}
        onvolume={(v) => actions.setVolume(s.id, v)}
        onretry={() => actions.retry(s.id)}
        onwatch={(on) => actions.watch(s.id, on)}
      />
    </div>
  {:else if layout === 'grid'}
    <div class="grid" style:--cols={gridShape.cols} style:--rows={gridShape.rows}>
      {#each view.streams as s (s.id)}
        <StreamTile
          view={s}
          size="grid"
          onselect={() => actions.focus(s.id)}
          ontogglemute={() => actions.toggleMute(s.id)}
          onvolume={(v) => actions.setVolume(s.id, v)}
          onretry={() => actions.retry(s.id)}
          onwatch={(on) => actions.watch(s.id, on)}
        />
      {/each}
      {#if view.streams.length < gridShape.cols * gridShape.rows}
        <div class="hint-cell">
          <span class="hint-title">Clique numa tela pra focar</span>
          <span class="hint-sub">As outras vão pra faixa de baixo.</span>
        </div>
      {/if}
    </div>
  {:else if focused}
    <div class="focus">
      <div class="focus-main">
        <StreamTile
          view={focused}
          focused
          radius="24px"
          ontogglemute={() => actions.toggleMute(focused.id)}
          onvolume={(v) => actions.setVolume(focused.id, v)}
          onretry={() => actions.retry(focused.id)}
          onwatch={(on) => actions.watch(focused.id, on)}
        />
      </div>
      <div class="strip">
        {#each others as s (s.id)}
          <div class="thumb">
            <StreamTile
              view={s}
              size="thumb"
              onselect={() => actions.focus(s.id)}
              onretry={() => actions.retry(s.id)}
              onwatch={(on) => actions.watch(s.id, on)}
            />
          </div>
        {/each}
        <span class="strip-hint">Clique pra trocar · Esc volta pra grade</span>
      </div>
    </div>
  {/if}

  <!-- Queda (specs/011 Q1/Q3): o palco continua por baixo. -->
  {#if view.link === 'reconnecting'}
    <ReconnectPill />
  {:else if view.link === 'failed'}
    <ReconnectFail onretry={actions.retryConnection} onleave={actions.leave} />
  {/if}

  <!-- Alerta e toasts dividem o canto superior direito: o alerta em cima, os toasts embaixo. -->
  <div class="corner">
    {#if view.permissionDenied}
      <PermissionAlert onretry={actions.openGoLive} ondismiss={actions.dismissPermission} />
    {/if}
    <ToastStack
      toasts={view.toasts}
      placement="inline"
      onaction={actions.toastAction}
      ondismiss={actions.dismissToast}
    />
  </div>
</section>

<style>
  .stage {
    position: relative; flex: 1; min-height: 0; overflow: hidden;
    border-radius: var(--radius-stage); background: var(--color-stage);
  }
  .center {
    position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 18px;
  }
  .center-title { font-family: var(--font-heading); font-size: 26px; }
  .center-sub { font-size: 15px; color: var(--color-text-muted); }

  .empty {
    position: absolute; inset: 0; display: flex; flex-direction: column; justify-content: center; gap: 20px;
    padding: 0 9%;
  }
  .blob { position: absolute; border-radius: 50%; }
  .blob-a { right: -90px; top: -120px; width: 440px; height: 440px; background: rgba(198, 113, 57, 0.1); }
  .blob-b { right: 180px; bottom: -140px; width: 280px; height: 280px; background: rgba(122, 138, 94, 0.12); }
  .castor { position: absolute; right: 12%; bottom: -70px; transform: rotate(-6deg); }
  .empty h2 {
    position: relative; margin: 0; max-width: 560px;
    font-family: var(--font-heading); font-weight: 400; font-size: 52px; line-height: 1.05;
  }
  .empty p {
    position: relative; margin: 0; max-width: 460px; font-size: 18px; text-wrap: pretty;
    color: color-mix(in srgb, var(--color-text) 68%, transparent);
  }
  .empty-actions { position: relative; display: flex; gap: 12px; margin-top: 6px; }
  .big { padding: 15px 26px; font-size: 18px; }

  .fill { position: absolute; inset: 0; }

  .corner {
    position: absolute; top: 24px; right: 24px; z-index: 5; max-width: calc(100% - 48px);
    display: flex; flex-direction: column; align-items: flex-end; gap: 10px;
  }

  .grid {
    position: absolute; inset: 0; padding: 20px; gap: 16px;
    display: grid; grid-template-columns: repeat(var(--cols), 1fr); grid-template-rows: repeat(var(--rows), 1fr);
  }
  .hint-cell {
    border-radius: var(--radius-tile); border: 2px dashed rgba(245, 234, 216, 0.12);
    display: flex; flex-direction: column; justify-content: flex-end; gap: 4px; padding: 24px;
  }
  .hint-title { font-family: var(--font-heading); font-size: 20px; }
  .hint-sub { font-size: 14px; color: var(--color-text-muted); }

  .focus { position: absolute; inset: 0; display: flex; flex-direction: column; gap: 14px; padding: 14px; }
  .focus-main { flex: 1; min-height: 0; }
  .strip { display: flex; gap: 12px; align-items: center; overflow-x: auto; }
  .thumb { flex: none; width: 200px; height: 113px; }
  .strip-hint { margin-left: 8px; font-size: 13px; color: color-mix(in srgb, var(--color-text) 55%, transparent); }

  @media (max-width: 1023px) {
    .empty h2 { font-size: 40px; }
    .castor { right: 4%; opacity: 0.5; }
    .thumb { width: 160px; height: 90px; }
  }
</style>
