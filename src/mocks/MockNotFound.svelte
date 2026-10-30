<script lang="ts">
  import type { NotFoundActions } from '../lib/types';
  import NotFoundPage from '../ui/pages/NotFoundPage.svelte';
  import { buildNotFoundScene, type NotFoundScene } from './notfound';
  import type { Navigate } from './registry';

  let { scene, saved, onnavigate }: { scene: NotFoundScene; saved: boolean; onnavigate: Navigate } = $props();

  // svelte-ignore state_referenced_locally
  const view = buildNotFoundScene(scene, saved);

  const actions: NotFoundActions = {
    createRoom: () => onnavigate('home', 'default'),
    pasteOtherLink: () => onnavigate('home', 'default'),
    forget: () => onnavigate('home', 'default'),
  };
</script>

<NotFoundPage {view} {actions} />
