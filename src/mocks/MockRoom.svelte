<script lang="ts">
  // Sala com dados falsos: as ações mexem só no estado local, para dar pra clicar e ver as transições.
  import type { RoomActions, RoomView, ToastView } from '../lib/types';
  import MobileRoomPage from '../ui/pages/MobileRoomPage.svelte';
  import RoomPage from '../ui/pages/RoomPage.svelte';
  import type { Navigate } from './registry';
  import { MOCK_AUDIO_INPUTS, mockCameraDevices } from './room';

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
      if (prefs.surface === 'camera') {
        const cam = view.camera.devices.find((d) => d.id === view.camera.videoId);
        view.myBroadcast = {
          ...prefs,
          viewers: view.participants.length - 1,
          startedAt: Date.now(),
          audioKind: prefs.audio && (view.camera.audioId || view.camera.mobile) ? 'device' : 'none',
          source: 'camera',
          sourceTitle: cam?.label ?? 'câmera traseira',
        };
        view.participants.forEach((p) => p.you && (p.live = true));
        toast({ kind: 'live', text: 'Sua câmera está ao vivo pra sala' });
        return;
      }
      const screen = prefs.surface === 'monitor';
      const source = view.picker.mode === 'native' ? view.picker.sources.find((s) => s.id === view.picker.selectedId) : undefined;
      // App (specs/005): janela com o som dela; tela com o som do PC menos o Discord.
      // Site (specs/004): o seletor nativo entregaria janela com som; tela inteira sem som.
      view.myBroadcast = {
        ...prefs,
        viewers: view.participants.length - 1,
        startedAt: Date.now(),
        audioKind: !prefs.audio ? 'none' : source ? (screen ? 'pc' : 'window') : screen ? 'none' : 'window',
        audioReason: screen && !source ? 'monitor' : undefined,
        sourceTitle: source?.title,
      };
      view.participants.forEach((p) => p.you && (p.live = true));
      toast({ kind: 'live', text: 'Você está ao vivo pra sala' });
    },
    selectSource(id) {
      view.picker.selectedId = id;
    },
    // Aba Câmeras (specs/006): só estado local, nada de câmera de verdade.
    setSurface() {},
    allowCamera() {
      view.camera = { ...view.camera, permission: 'granted', devices: mockCameraDevices(), audios: MOCK_AUDIO_INPUTS, videoId: 'cam1', audioId: 'mic1' };
    },
    selectCamera(id) {
      view.camera.videoId = id;
      // Mesma sugestão da lógica real: placa → a entrada dela, como jogo.
      if (id === 'cam2') Object.assign(view.camera, { audioId: 'mic2', audioRole: 'game' });
      else if (id === 'cam1') Object.assign(view.camera, { audioId: 'mic1', audioRole: 'voice' });
      else Object.assign(view.camera, { audioId: null, audioRole: 'voice' });
    },
    selectAudio(id) {
      view.camera.audioId = id;
    },
    setAudioRole(role) {
      view.camera.audioRole = role;
    },
    setFacing(facing) {
      view.camera.facing = facing;
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
    watch(id, on) {
      const s = stream(id);
      if (s) s.watching = on;
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
    goHome() {
      if (view.myBroadcast && !confirm('Você está transmitindo. Parar e sair da sala?')) return;
      onnavigate('home', 'default');
    },
    // Dono da sala fixa (specs/011 S2–S4).
    toggleOwnerMenu(open) {
      view.ownerMenu = open;
    },
    openRename() {
      view.ownerMenu = false;
      if (view.room.kind === 'fixed') view.nameDialog = { mode: 'rename', value: view.room.name, busy: false, error: null };
    },
    submitRename(name) {
      if (view.room.kind === 'fixed') view.room = { ...view.room, name };
      view.nameDialog = undefined;
    },
    openDelete() {
      view.ownerMenu = false;
      if (view.room.kind === 'fixed') view.deleteDialog = { name: view.room.name, people: view.participants.length - 1, busy: false, error: null };
    },
    confirmDelete() {
      onnavigate('home', 'default');
    },
    closeDialogs() {
      view.ownerMenu = false;
      view.nameDialog = undefined;
      view.deleteDialog = undefined;
    },
    // Mock da queda (specs/011): "Tentar de novo" volta a reconectar e, logo depois, "De volta!".
    retryConnection() {
      view.link = 'reconnecting';
      setTimeout(() => {
        view.link = 'ok';
        toast({ kind: 'copied', text: 'De volta!' });
      }, 1500);
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
