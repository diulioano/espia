<script lang="ts">
  import MockHome from './MockHome.svelte';
  import MockIndex from './MockIndex.svelte';
  import MockJoin from './MockJoin.svelte';
  import MockNotFound from './MockNotFound.svelte';
  import MockRoom from './MockRoom.svelte';
  import MockSwitcher from './MockSwitcher.svelte';
  import type { HomeScene } from './home';
  import type { JoinScene } from './join';
  import { MOCK_SCREENS, mockHref } from './registry';
  import { buildMobileScene, type MobileScene } from './mobile';
  import type { NotFoundScene } from './notfound';
  import { buildRoomScene, type RoomScene } from './room';

  let params = $state(new URLSearchParams(location.search));

  /** Dentro do quadro de celular (iframe): só a tela, sem seletor. */
  const framed = $derived(params.has('frame'));
  const screen = $derived(MOCK_SCREENS.find((s) => s.id === params.get('mock')));
  const scene = $derived(
    screen && (screen.scenes.find((s) => s.id === params.get('scene')) ?? screen.scenes.find((s) => s.id === screen.defaultScene)!),
  );
  const inPhoneFrame = $derived(!!scene?.device && !framed);
  /** Celular deitado (specs/018): quadro 844×390. */
  const landscape = $derived(scene?.device === 'mobile-landscape');

  function navigate(screenId: string, sceneId?: string) {
    history.replaceState(null, '', mockHref(screenId, sceneId, framed));
    params = new URLSearchParams(location.search);
  }
</script>

<svelte:window onpopstate={() => (params = new URLSearchParams(location.search))} />

{#if screen && scene}
  {#if inPhoneFrame}
    <div class="phone-stage">
      <iframe
        class="phone"
        class:landscape
        title={landscape ? 'Celular deitado 844×390' : 'Celular 390×844'}
        src={mockHref(screen.id, scene.id, true)}
      ></iframe>
    </div>
  {:else}
    {#key `${screen.id}/${scene.id}`}
      {#if screen.id === 'home'}
        <MockHome scene={scene.id as HomeScene} onnavigate={navigate} />
      {:else if screen.id === 'join'}
        <MockJoin scene={scene.id as JoinScene} onnavigate={navigate} />
      {:else if screen.id === 'room'}
        <MockRoom initial={buildRoomScene(scene.id as RoomScene)} onnavigate={navigate} />
      {:else if screen.id === 'notfound'}
        <MockNotFound scene={scene.id as NotFoundScene} saved={params.get('saved') === '1'} onnavigate={navigate} />
      {:else if screen.id === 'mobile'}
        <MockRoom initial={buildMobileScene(scene.id as MobileScene)} mobile onnavigate={navigate} />
      {/if}
    {/key}
  {/if}
  {#if !framed}
    <MockSwitcher {screen} scene={scene.id} onnavigate={navigate} />
  {/if}
{:else}
  <MockIndex />
{/if}

<style>
  .phone-stage { min-height: 100%; display: grid; place-items: center; padding: 32px 16px; background: var(--color-stage); }
  .phone {
    width: 390px; height: 844px; max-width: 100%; border: 0; border-radius: 44px;
    box-shadow: var(--shadow-lg), 0 0 0 10px #0c0b0a;
  }
  .phone.landscape { width: 844px; height: 390px; }
</style>
