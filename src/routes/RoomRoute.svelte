<script lang="ts">
  // "/s/:código" real: tela de entrada (prévia) → sala. Quem acabou de criar a sala, ou recarregou a
  // página já dentro dela, vai direto pra sala.
  import { onDestroy } from 'svelte';
  import { fetchPreview } from '../lib/api';
  import { canUseCamera } from '../lib/camera';
  import { canShareScreen } from '../lib/capture';
  import { installStopHook, reportLive, reportRoom } from '../lib/desktop';
  import { forgetOwnerKey, loadNickname, loadSession, saveNickname } from '../lib/prefs';
  import { forgetRoom, loadSavedRooms, saveSavedRooms } from '../lib/saved-rooms';
  import { router } from '../lib/router.svelte';
  import { RoomStore } from '../lib/stores/room.svelte';
  import type { JoinActions, JoinView, NotFoundActions, NotFoundView } from '../lib/types';
  import JoinPage from '../ui/pages/JoinPage.svelte';
  import MobileRoomPage from '../ui/pages/MobileRoomPage.svelte';
  import NotFoundPage from '../ui/pages/NotFoundPage.svelte';
  import RoomPage from '../ui/pages/RoomPage.svelte';

  let { code }: { code: string } = $props();

  // svelte-ignore state_referenced_locally
  const directNickname = router.state.nickname ?? (loadSession(code) ? loadNickname() : '');
  // svelte-ignore state_referenced_locally
  const justCreated = !!router.state.created;

  let store = $state<RoomStore | null>(null);
  /** A sala não existe (mais): rápida que acabou, fixa apagada/expirada (specs/011). */
  let gone = $state<NotFoundView | null>(null);
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
    store = new RoomStore(code, nickname, () => router.go('/'), justCreated);
  }

  function notFound(reason: NotFoundView['reason']) {
    // O nome vem de Suas salas, se este aparelho conhecia a sala; o servidor não guarda (research R5).
    const saved = loadSavedRooms().find((r) => r.code === code);
    gone = { reason, name: saved?.name ?? null, saved: !!saved };
    // A sala acabou: a chave de dono não serve mais pra nada.
    forgetOwnerKey(code);
  }

  async function loadPreview() {
    const res = await fetchPreview(code);
    if (store) return;
    if (res.status === 'notfound') notFound('quick');
    else if (res.status === 'gone') notFound(res.reason);
    else if (res.status === 'unknown') join = { ...join, status: 'ready', people: null };
    else
      join = {
        ...join,
        status: res.status,
        capacity: res.preview.capacity,
        people: res.preview.people.map((p) => ({ name: p.nickname, tone: p.tone, live: p.live })),
        fixedName: res.preview.fixed?.name,
      };
  }

  if (directNickname) enter(directNickname);
  else loadPreview();

  // A sala acabou de vez: volta pra tela de entrada com o motivo.
  $effect(() => {
    const ended = store?.ended;
    if (!ended) return;
    store = null;
    // O dono apagou com você dentro (N2).
    if (ended === 'deleted') notFound('kicked');
    // Não existe: a prévia diz se era rápida (404) ou fixa apagada/expirada (410).
    else if (ended === 'full') join = { ...join, status: 'full' };
    else {
      // A mesma pessoa abriu a sala em outra aba: esta cede o lugar e explica.
      if (ended === 'replaced') join = { ...join, replaced: true };
      loadPreview();
    }
  });

  const joinActions: JoinActions = {
    goHome: () => router.go('/'),
    join(nickname) {
      saveNickname(nickname);
      join.nickname = nickname;
      join.replaced = false;
      enter(nickname);
    },
    createRoom: () => router.go('/'),
    pasteOtherLink: () => router.go('/'),
  };

  const notFoundActions: NotFoundActions = {
    goHome: () => router.go('/'),
    createRoom: () => router.go('/'),
    pasteOtherLink: () => router.go('/'),
    forget: () => {
      saveSavedRooms(forgetRoom(loadSavedRooms(), code));
      router.go('/');
    },
  };

  // App desktop: link de outra sala enquanto você transmite pergunta antes (src/lib/desktop.ts).
  const leaveGuard = () => (store?.myBroadcast ? 'Você está transmitindo. Parar e ir pra outra sala?' : null);
  router.leaveGuard = leaveGuard;

  // App desktop: ícone "ao vivo" na bandeja e "Parar transmissão" pelo menu (specs/007).
  const live = $derived(!!store?.myBroadcast);
  $effect(() => reportLive(live));
  const removeStopHook = installStopHook(() => store?.actions.stopGoLive());
  // App desktop: escondido fora de sala, o app solta o motor de página (specs/009).
  const inRoom = $derived(!!store?.connected && !store?.ended);
  $effect(() => reportRoom(inRoom));

  onDestroy(() => {
    if (router.leaveGuard === leaveGuard) router.leaveGuard = null;
    removeStopHook();
    reportLive(false);
    reportRoom(false);
    store?.destroy();
  });
</script>

{#if gone}
  <NotFoundPage view={gone} actions={notFoundActions} />
{:else if store}
  {#if mobile}
    <MobileRoomPage view={store.view} actions={store.actions} />
  {:else}
    <RoomPage view={store.view} actions={store.actions} />
  {/if}
{:else}
  <JoinPage view={join} actions={joinActions} />
{/if}
