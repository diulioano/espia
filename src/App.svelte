<script lang="ts">
  import { router } from './lib/router.svelte';
  import HomeRoute from './routes/HomeRoute.svelte';
  import RoomRoute from './routes/RoomRoute.svelte';

  // Modo mock: ?mock=<tela>&scene=<cena>. Carregado sob demanda para ficar fora do bundle principal.
  const isMock = new URLSearchParams(location.search).has('mock');
</script>

{#if isMock}
  {#await import('./mocks/MockApp.svelte') then { default: MockApp }}
    <MockApp />
  {/await}
{:else if router.route.name === 'room'}
  {#key router.route.code}
    <RoomRoute code={router.route.code} />
  {/key}
{:else}
  <HomeRoute />
{/if}
