<script lang="ts">
  import type { RoomStore } from '../../lib/stores/room.svelte';

  let { read }: { read: RoomStore['mediaDiagnostics'] } = $props();
  let open = $state(false);
  let snapshot = $state<ReturnType<RoomStore['mediaDiagnostics']> | null>(null);
  const limits = { none: 'sem limitação declarada', cpu: 'processamento', bandwidth: 'rede', other: 'outra' };
  const value = (n: number | null, digits = 1) => n === null ? '—' : n.toFixed(digits);

  function keydown(event: KeyboardEvent) {
    if (event.ctrlKey && event.shiftKey && event.code === 'KeyD' && !event.repeat) {
      event.preventDefault();
      open = !open;
    } else if (event.key === 'Escape') open = false;
  }

  $effect(() => {
    if (!open) { snapshot = null; return; }
    snapshot = read();
    const timer = setInterval(() => { snapshot = read(); }, 1000);
    return () => clearInterval(timer);
  });
</script>

<svelte:window onkeydown={keydown} />

{#if open}
  <aside class="media-diagnostics" aria-label="Diagnóstico da transmissão">
    <header><strong>Diagnóstico da transmissão</strong><button class="btn" onclick={() => open = false}>Fechar</button></header>
    <p>Somente neste aparelho · atualização a cada segundo · Ctrl+Shift+D para alternar</p>
    {#each snapshot?.links ?? [] as link (link.link)}
      {@const sample = link.samples.at(-1)}
      {#if sample?.streams.some((stream) => stream.kind === 'video')}
        <h2>Conexão {link.link}</h2>
        <p>RTT {value(sample.rttMs)} ms · rota {sample.route ?? '—'}</p>
        {#each sample.streams.filter((stream) => stream.kind === 'video') as stream}
          <dl>
            <dt>Vídeo</dt><dd>{stream.direction === 'receive' ? 'Recebido' : 'Enviado'}</dd>
            <dt>Resolução real</dt><dd>{stream.width ?? '—'} × {stream.height ?? '—'}</dd>
            <dt>FPS {stream.direction === 'receive' ? 'decodificados' : 'codificados'}</dt><dd>{value(stream.fps)}</dd>
            <dt>Bitrate no intervalo</dt><dd>{value(stream.mbps, 2)} Mbps</dd>
            {#if stream.direction === 'send'}
              <dt>Limitação do encoder</dt><dd>{stream.limitation === null ? '—' : limits[stream.limitation]}</dd>
              <dt>Encoder</dt><dd>{link.encoding.implementation ?? '—'}</dd>
            {:else}
              <dt>Pacotes perdidos no intervalo</dt><dd>{stream.lostPackets ?? '—'}</dd>
              <dt>Congelamentos no intervalo</dt><dd>{stream.freezes ?? '—'}</dd>
              <dt>Espera média no buffer</dt><dd>{value(stream.jitterMs)} ms</dd>
              <dt>Decodificação média</dt><dd>{value(stream.decodeMs)} ms</dd>
            {/if}
          </dl>
        {/each}
      {/if}
    {:else}
      <p>Aguardando estatísticas da conexão.</p>
    {/each}
    <p>“—” significa dado indisponível. RTT e buffer não medem o atraso ponta a ponta.</p>
  </aside>
{/if}

<style>
  .media-diagnostics { position: fixed; z-index: 100; right: 16px; bottom: 16px; width: min(440px, calc(100vw - 32px)); max-height: calc(100dvh - 32px); overflow: auto; padding: 18px; border: 1px solid #777; border-radius: 12px; background: #1c1b17; color: #f5efdf; box-shadow: 0 8px 32px #0008; font: 14px/1.5 system-ui, sans-serif; }
  header { display: flex; justify-content: space-between; align-items: center; gap: 12px; }
  h2 { font: inherit; font-weight: 700; margin: 16px 0 4px; }
  p { margin: 8px 0; color: #c8c3b8; font-size: 12px; }
  dl { display: grid; grid-template-columns: 1fr 1fr; gap: 4px 12px; margin: 12px 0; }
  dd { margin: 0; overflow-wrap: anywhere; font-variant-numeric: tabular-nums; }
</style>
