<script lang="ts">
  // Painel "Suas salas" da tela inicial (specs/011, Home.dc.html F2–F6; celular M4).
  import type { SavedRoomsView } from '../../lib/types';
  import Mascot from './Mascot.svelte';
  import SavedRoom from './SavedRoom.svelte';

  let {
    view,
    compact = false,
    onenter,
    onmenu,
    onforget,
    onrename,
    ondelete,
  }: {
    view: SavedRoomsView;
    compact?: boolean;
    onenter: (code: string) => void;
    onmenu: (code: string | null) => void;
    onforget: (code: string) => void;
    /** Só pro dono (US4). */
    onrename?: (code: string) => void;
    ondelete?: (code: string) => void;
  } = $props();
</script>

<section class="panel" class:compact aria-labelledby="saved-rooms-title">
  {#if !compact}<div class="castor" aria-hidden="true"><Mascot mood="peek" size={92} /></div>{/if}
  <div class="card">
    <div class="head">
      <h2 id="saved-rooms-title">Suas salas</h2>
      <span class="note">{view.note}</span>
    </div>
    <div class="groups">
      {#each view.groups as group, i (group.label ?? i)}
        <div class="group">
          {#if group.label}<span class="label">{group.label}</span>{/if}
          {#each group.rooms as room (room.code)}
            <SavedRoom
              {room}
              {compact}
              onenter={() => onenter(room.code)}
              onmenu={(open) => onmenu(open ? room.code : null)}
              onforget={() => onforget(room.code)}
              onrename={onrename && (() => onrename(room.code))}
              ondelete={ondelete && (() => ondelete(room.code))}
            />
          {/each}
        </div>
      {/each}
    </div>
  </div>
</section>

<style>
  .panel { position: relative; width: 600px; max-width: 100%; }
  /* À esquerda: a direita do topo é do botão "Baixar o app" (specs/013). */
  .castor { position: absolute; left: 48px; top: -78px; transform: rotate(-4deg); }
  .card {
    position: relative; display: flex; flex-direction: column; gap: 14px; padding: 26px 18px 18px;
    border-radius: 36px; background: var(--color-surface); box-shadow: 0 30px 70px rgba(0, 0, 0, 0.35);
  }
  .head { display: flex; align-items: baseline; gap: 12px; padding: 0 12px; }
  h2 { margin: 0; font-family: var(--font-heading); font-weight: 400; font-size: 26px; }
  .note { font-size: 13px; color: rgba(245, 234, 216, 0.55); }
  .groups { display: flex; flex-direction: column; gap: 14px; }
  .group { display: flex; flex-direction: column; gap: 2px; }
  .label {
    padding: 8px 12px 4px; font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase;
    color: rgba(245, 234, 216, 0.5);
  }

  /* Celular (M4): sem cartão. */
  .compact { width: auto; margin: 0 -12px; }
  .compact .card { gap: 4px; padding: 0; border-radius: 0; background: none; box-shadow: none; }
  .compact .head { gap: 10px; padding: 0 12px 6px; }
  .compact h2 { font-size: 22px; }
  .compact .note { font-size: 12px; }
  .compact .groups { gap: 6px; }
</style>
