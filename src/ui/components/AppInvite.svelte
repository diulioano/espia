<script lang="ts">
  // Convite para baixar o app desktop (specs/008). Secundário: fica abaixo de "Tem um link?".
  import { Download, X } from '@lucide/svelte';
  import { DOWNLOAD_URL, RELEASES_URL } from '../../lib/app-invite';
  import Mascot from './Mascot.svelte';

  let { state, ondownload, ondismiss }: { state: 'offer' | 'hint'; ondownload: () => void; ondismiss: () => void } = $props();
</script>

<aside class="invite" aria-label="App do Espia para Windows">
  <div class="castor" aria-hidden="true"><Mascot mood="peek" size={44} /></div>
  {#if state === 'offer'}
    <div class="text">
      <strong>Tem Windows? Baixe o app do Espia</strong>
      <span>Escolha a janela numa grade com prévias, som sem vazar o Discord e feche sem sair da sala.</span>
      <div class="actions">
        <a class="btn btn-secondary dl" href={DOWNLOAD_URL} onclick={() => ondownload()}>
          <Download size={16} strokeWidth={2.75} />Baixar o app
        </a>
        <button class="btn btn-ghost later" onclick={() => ondismiss()}>Agora não</button>
      </div>
    </div>
  {:else}
    <div class="text" role="status">
      <strong>Baixando…</strong>
      <span>
        Se o Windows avisar que protegeu o computador, clique em <b>Mais informações</b> → <b>Executar assim mesmo</b>.
      </span>
      <a class="fallback" href={RELEASES_URL} target="_blank" rel="noopener">Não baixou? Abra a página de versões</a>
    </div>
    <button class="btn btn-icon close" aria-label="Fechar" onclick={() => ondismiss()}><X size={16} strokeWidth={2.75} /></button>
  {/if}
</aside>

<style>
  .invite {
    position: relative; display: flex; gap: 14px; align-items: flex-start;
    width: min(480px, 100%); box-sizing: border-box; margin-top: 18px; padding: 14px 16px;
    border: 1px solid var(--color-divider); border-radius: var(--radius-tile);
    background: color-mix(in srgb, var(--color-surface) 70%, transparent);
  }
  .castor { flex: none; margin-top: 2px; }
  .text { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
  strong { font-size: 15px; font-weight: 700; }
  span { font-size: 13px; line-height: 1.4; color: var(--color-text-muted); text-wrap: pretty; }
  b { color: var(--color-text); font-weight: 600; }
  .actions { display: flex; gap: 6px; margin-top: 8px; flex-wrap: wrap; }
  .dl { min-height: 36px; padding: 0 16px; font-size: 14px; gap: 8px; text-decoration: none; color: inherit; }
  .later { min-height: 36px; padding: 0 14px; font-size: 14px; }
  .fallback { margin-top: 4px; font-size: 13px; color: var(--color-accent-400); }
  .close { position: absolute; top: 10px; right: 10px; width: 28px; height: 28px; }
  .invite:has(.close) .text { padding-right: 30px; }
</style>
