<script lang="ts">
  // "/s/:código" real: tela de entrada (prévia) → sala. Quem acabou de criar a sala, ou recarregou a
  // página já dentro dela, vai direto pra sala.
  import { onDestroy } from 'svelte';
  import { fetchPreview } from '../lib/api';
  import { canUseCamera } from '../lib/camera';
  import { canShareScreen } from '../lib/capture';
  import { installStopHook, reportLive } from '../lib/desktop';
  import { loadNickname, loadSession, saveNickname } from '../lib/prefs';
  import { router } from '../lib/router.svelte';
  import { RoomStore } from '../lib/stores/room.svelte';
  import type { JoinActions, JoinView } from '../lib/types';
  import JoinPage from '../ui/pages/JoinPage.svelte';
  import MobileRoomPage from '../ui/pages/MobileRoomPage.svelte';
  import RoomPage from '../ui/pages/RoomPage.svelte';

  let { code }: { code: string } = $props();

  // svelte-ignore state_referenced_locally
  const directNickname = router.state.nickname ?? (loadSession(code) ? loadNickname() : '');

  let store = $state<RoomStore | null>(null);
  // svelte-ignore state_referenced_locally
  let join = $state<JoinView>({
    roomId: code,
    status: 'ready',
    people: null,
    capacity: 10,
    nickname: loadNickname(),
    canShare: canShareScreen() || canUseCamera(),
  });

  // Celular: layout próprio abaixo de 640px.
  const narrow = matchMedia('(max-width: 639px)');
  let mobile = $state(narrow.matches);
  narrow.addEventListener('change', (e) => (mobile = e.matches));

  // Celular vê só uma tela por vez: as outras pausam o envio pra cá.
  $effect(() => {
    if (store) store.mobile = mobile;
  });

  function enter(nickname: string) {
    store?.destroy();
    store = new RoomStore(code, nickname, () => router.go('/'));
  }

  async function loadPreview() {
    const res = await fetchPreview(code);
    if (store) return;
    if (res.status === 'notfound') join = { ...join, status: 'notfound', people: null };
    else if (res.status === 'unknown') join = { ...join, status: 'ready', people: null };
    else
      join = {
        ...join,
        status: res.status,
        capacity: res.preview.capacity,
        people: res.preview.people.map((p) => ({ name: p.nickname, tone: p.tone, live: p.live })),
      };
  }

  if (directNickname) enter(directNickname);
  else loadPreview();

  // A sala acabou de vez: volta pra tela de entrada com o motivo.
  $effect(() => {
    const ended = store?.ended;
    if (!ended) return;
    store = null;
    if (ended === 'not-found') join = { ...join, status: 'notfound', people: null };
    else if (ended === 'full') join = { ...join, status: 'full' };
    else loadPreview();
  });

  const joinActions: JoinActions = {
    join(nickname) {
      saveNickname(nickname);
      join.nickname = nickname;
      enter(nickname);
    },
    createRoom: () => router.go('/'),
    pasteOtherLink: () => router.go('/'),
  };

  // App desktop: link de outra sala enquanto você transmite pergunta antes (src/lib/desktop.ts).
  const leaveGuard = () => (store?.myBroadcast ? 'Você está transmitindo. Parar e ir pra outra sala?' : null);
  router.leaveGuard = leaveGuard;

  // App desktop: ícone "ao vivo" na bandeja e "Parar transmissão" pelo menu (specs/007).
  const live = $derived(!!store?.myBroadcast);
  $effect(() => reportLive(live));
  const removeStopHook = installStopHook(() => store?.actions.stopGoLive());

  onDestroy(() => {
    if (router.leaveGuard === leaveGuard) router.leaveGuard = null;
    removeStopHook();
    reportLive(false);
    store?.destroy();
  });
</script>

{#if store}
  {#if mobile}
    <MobileRoomPage view={store.view} actions={store.actions} />
  {:else}
    <RoomPage view={store.view} actions={store.actions} />
  {/if}
{:else}
  <JoinPage view={join} actions={joinActions} />
{/if}
