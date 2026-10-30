<script lang="ts">
  import type { JoinActions } from '../lib/types';
  import JoinPage from '../ui/pages/JoinPage.svelte';
  import { buildJoinScene, type JoinScene } from './join';
  import type { Navigate } from './registry';

  let { scene, onnavigate }: { scene: JoinScene; onnavigate: Navigate } = $props();

  // svelte-ignore state_referenced_locally
  const view = buildJoinScene(scene);

  const actions: JoinActions = {
    goHome: () => onnavigate('home', 'default'),
    join() {
      // Simula o fluxo real: conectando (E1) → sala.
      const screen = view.canShare ? 'room' : 'mobile';
      onnavigate(screen, 'connecting');
      setTimeout(() => {
        if (new URLSearchParams(location.search).get('scene') === 'connecting') {
          onnavigate(screen, view.canShare ? 'single' : 'watching');
        }
      }, 1500);
    },
    createRoom() {
      onnavigate('home', 'default');
    },
    pasteOtherLink() {
      onnavigate('home', 'default');
    },
  };
</script>

<JoinPage {view} {actions} />
