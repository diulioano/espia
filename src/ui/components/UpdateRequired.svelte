<script lang="ts">
  let { onreload }: { onreload: () => void } = $props();
</script>

<main class="update-required">
  <h1>Atualize o Espia para continuar</h1>
  <p>Esta sala usa uma versão nova da transmissão.</p>
  <button class="btn btn-primary" onclick={onreload}>Atualizar</button>
</main>

<style>
  .update-required { min-height: 100dvh; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 16px; padding: 24px; text-align: center; }
</style>
