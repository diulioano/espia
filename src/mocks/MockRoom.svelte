<script lang="ts">
  // Sala com dados falsos: as ações mexem só no estado local, para dar pra clicar e ver as transições.
  import type { RoomActions, RoomView, ToastView } from '../lib/types';
  import MobileRoomPage from '../ui/pages/MobileRoomPage.svelte';
  import RoomPage from '../ui/pages/RoomPage.svelte';
  import type { Navigate } from './registry';

  let { initial, mobile = false, onnavigate }: { initial: RoomView; mobile?: boolean; onnavigate: Navigate } = $props();

  // svelte-ignore state_referenced_locally
  let view = $state<RoomView>(initial);
  let toastSeq = 0;

  function toast(t: Omit<ToastView, 'id'>) {
    const id = `t${++toastSeq}`;
    view.toasts.push({ id, ...t });
    if (t.kind !== 'error') setTimeout(() => actions.dismissToast(id), 4000);
  }

  function stream(id: string) {
    return view.streams.find((s) => s.id === id);
  }

  const actions: RoomActions = {
    copyInvite() {
      navigator.clipboard?.writeText(view.inviteUrl).catch(() => {});
      view.copied = true;
      toast({ kind: 'copied', text: 'Link copiado — é só mandar pra galera' });
      setTimeout(() => (view.copied = false), 2000);
    },
    openGoLive() {
      view.permissionDenied = false;
      view.goLiveOpen = true;
    },
    closeGoLive() {
      view.goLiveOpen = false;
    },
    startGoLive(prefs) {
      view.goLivePrefs = prefs;
      view.goLiveOpen = false;
      view.myBroadcast = { ...prefs, viewers: view.participants.length - 1, startedAt: Date.now() };
      view.participants.forEach((p) => p.you && (p.live = true));
      toast({ kind: 'live', text: 'Você está ao vivo pra sala' });
    },
    stopGoLive() {
      view.myBroadcast = null;
      view.participants.forEach((p) => p.you && (p.live = false));
    },
    dismissPermission() {
      view.permissionDenied = false;
    },
    focus(id) {
      view.focusedId = id;
    },
    toggleMute(id) {
      const s = stream(id);
      if (!s) return;
      s.muted = !s.muted;
      if (!s.muted && s.volume === 0) s.volume = 0.68;
    },
    setVolume(id, volume) {
      const s = stream(id);
      if (!s) return;
      s.volume = volume;
      s.muted = volume === 0;
    },
    retry(id) {
      const s = stream(id);
      if (!s) return;
      s.status = 'loading';
      setTimeout(() => (s.status = 'ok'), 1500);
    },
    toastAction(id) {
      const t = view.toasts.find((x) => x.id === id);
      // "Assistir" no toast de live foca quem começou (no mock, o Tomás).
      if (t?.kind === 'live' && t.action) {
        const s = stream('tomas');
        if (s) {
          s.status = 'ok';
          view.focusedId = s.id;
        }
      }
      actions.dismissToast(id);
    },
    dismissToast(id) {
      view.toasts = view.toasts.filter((t) => t.id !== id);
    },
    leave() {
      onnavigate('home', 'default');
    },
  };
</script>

{#if mobile}
  <MobileRoomPage {view} {actions} />
{:else}
  <RoomPage {view} {actions} />
{/if}
