<script lang="ts">
  // "/" real: cria sala no servidor ou abre um link colado.
  import { createFixedRoom, createRoom } from '../lib/api';
  import { appInviteVisible, readInviteEnv } from '../lib/app-invite';
  import { loadAppInviteDismissed, loadNickname, saveAppInviteDismissed, saveNickname, saveOwnerKey } from '../lib/prefs';
  import { parseRoomId, roomPath } from '../lib/room-id';
  import { router } from '../lib/router.svelte';
  import type { HomeActions, HomeView } from '../lib/types';
  import HomePage from '../ui/pages/HomePage.svelte';

  let view = $state<HomeView>({
    nickname: loadNickname(),
    link: '',
    linkError: null,
    createError: null,
    creating: false,
    appInvite: appInviteVisible({ ...readInviteEnv(), dismissed: loadAppInviteDismissed() }) ? 'offer' : undefined,
  });

  const actions: HomeActions = {
    async createRoom(nickname) {
      saveNickname(nickname);
      view.creating = true;
      view.createError = null;
      const res = await createRoom();
      view.creating = false;
      if (res.ok) router.go(roomPath(res.code), { nickname });
      else
        view.createError =
          res.reason === 'rate-limited'
            ? 'Muitas salas criadas daqui agora. Tenta de novo em um minuto.'
            : 'Não deu pra falar com o servidor. Confere a internet e tenta de novo.';
    },
    openLink(text) {
      const code = parseRoomId(text);
      if (code) router.go(roomPath(code));
      else view.linkError = 'Esse link não parece de uma sala.';
    },
    clearLinkError() {
      view.linkError = null;
    },
    downloadApp() {
      view.appInvite = 'hint';
    },
    dismissAppInvite() {
      saveAppInviteDismissed();
      view.appInvite = undefined;
    },
    openFixedDialog() {
      view.fixedDialog = { mode: 'create', value: '', busy: false, error: null };
    },
    closeFixedDialog() {
      view.fixedDialog = undefined;
    },
    async createFixedRoom(nickname, name) {
      if (!view.fixedDialog) return;
      saveNickname(nickname);
      view.fixedDialog = { ...view.fixedDialog, value: name, busy: true, error: null };
      const res = await createFixedRoom(name);
      if (res.ok) {
        saveOwnerKey(res.code, res.ownerKey);
        router.go(roomPath(res.code), { nickname, created: true });
        return;
      }
      const error = {
        'rate-limited': 'Muitas salas criadas daqui agora. Tenta de novo em um minuto.',
        name: 'Esse nome não dá. Use letras, números e espaços.',
        network: 'Não deu pra falar com o servidor. Confere a internet e tenta de novo.',
      }[res.reason];
      if (view.fixedDialog) view.fixedDialog = { ...view.fixedDialog, busy: false, error };
    },
  };
</script>

<HomePage {view} {actions} />
