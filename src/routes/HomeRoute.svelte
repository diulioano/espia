<script lang="ts">
  // "/" real: cria sala no servidor ou abre um link colado.
  import { createFixedRoom, createRoom, fetchStatuses } from '../lib/api';
  import { appInviteVisible, readInviteEnv } from '../lib/app-invite';
  import {
    forgetOwnerKey,
    loadAppInviteDismissed,
    loadNickname,
    loadOwnerKey,
    saveAppInviteDismissed,
    saveNickname,
    saveOwnerKey,
  } from '../lib/prefs';
  import { buildSavedRooms, forgetRoom, loadSavedRooms, rememberRoom, renameSaved, saveSavedRooms } from '../lib/saved-rooms';
  import type { RoomStatus } from '../shared/protocol';
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

  // Suas salas (specs/011 US3): a lista do aparelho + quantas pessoas em cada uma agora.
  let saved = loadSavedRooms();
  let statuses: RoomStatus[] | null = null;
  let menuCode: string | null = null;

  function showRooms() {
    view.rooms = buildSavedRooms(saved, statuses, (code) => !!loadOwnerKey(code), Date.now(), menuCode);
  }

  async function refreshRooms() {
    showRooms();
    if (saved.length === 0) return;
    const res = await fetchStatuses(saved.map((r) => r.code));
    if (!res) return; // Sem rede ou no limite: fica sem os números; "Entrar" continua funcionando.
    statuses = res;
    for (const s of res) {
      // Nome mudou (o dono renomeou): guarda o novo.
      if (s.state === 'ok' && s.name && saved.some((r) => r.code === s.code && r.name !== s.name)) {
        saved = renameSaved(saved, s.code, s.name);
        saveSavedRooms(saved);
      }
      // A sala acabou: a chave de dono não serve mais.
      if (s.state !== 'ok') forgetOwnerKey(s.code);
    }
    showRooms();
  }
  refreshRooms();

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
    enterSaved(code, nickname) {
      // Com apelido, entra direto; sem, a tela de entrada pede.
      if (nickname) {
        saveNickname(nickname);
        router.go(roomPath(code), { nickname });
      } else router.go(roomPath(code));
    },
    forgetSaved(code) {
      // Só deste aparelho; a chave de dono fica (abrir o link de novo devolve "Sua").
      saved = forgetRoom(saved, code);
      saveSavedRooms(saved);
      menuCode = null;
      showRooms();
    },
    toggleSavedMenu(code) {
      menuCode = code;
      showRooms();
    },
    async createFixedRoom(nickname, name) {
      if (!view.fixedDialog) return;
      saveNickname(nickname);
      view.fixedDialog = { ...view.fixedDialog, value: name, busy: true, error: null };
      const res = await createFixedRoom(name);
      if (res.ok) {
        saveOwnerKey(res.code, res.ownerKey);
        // Já fica em Suas salas, mesmo se sair antes de conectar.
        saveSavedRooms(rememberRoom(saved, { code: res.code, name }, Date.now()));
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
