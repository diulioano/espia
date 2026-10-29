<script lang="ts">
  // "/" real: cria sala no servidor ou abre um link colado.
  import { createRoom } from '../lib/api';
  import { appInviteVisible, readInviteEnv } from '../lib/app-invite';
  import { loadAppInviteDismissed, loadNickname, saveAppInviteDismissed, saveNickname } from '../lib/prefs';
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
  };
</script>

<HomePage {view} {actions} />
