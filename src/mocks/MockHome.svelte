<script lang="ts">
  import { parseRoomId } from '../lib/room-id';
  import type { HomeActions, HomeView } from '../lib/types';
  import HomePage from '../ui/pages/HomePage.svelte';
  import { buildHomeScene, LINK_ERROR, type HomeScene } from './home';
  import type { Navigate } from './registry';

  let { scene, onnavigate }: { scene: HomeScene; onnavigate: Navigate } = $props();

  // svelte-ignore state_referenced_locally
  let view = $state<HomeView>(buildHomeScene(scene));

  const actions: HomeActions = {
    createRoom() {
      onnavigate('room', 'empty');
    },
    openLink(text) {
      if (parseRoomId(text)) onnavigate('join', 'default');
      else view.linkError = LINK_ERROR;
    },
    clearLinkError() {
      view.linkError = null;
    },
    downloadApp() {
      view.appInvite = 'hint';
    },
    dismissAppInvite() {
      view.appInvite = undefined;
    },
    openFixedDialog() {
      view.fixedDialog = { mode: 'create', value: '', busy: false, error: null };
    },
    closeFixedDialog() {
      view.fixedDialog = undefined;
    },
    createFixedRoom() {
      onnavigate('room', 'created');
    },
  };
</script>

<HomePage {view} {actions} />
