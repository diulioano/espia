<script lang="ts">
  // Uma linha de Suas salas (specs/011, SavedRoom.dc.html): carregando, com números, ou que não existe mais.
  import { Ellipsis } from '@lucide/svelte';
  import { savedRoomSub } from '../../lib/saved-rooms';
  import type { SavedRoomView } from '../../lib/types';
  import RoomMenu from './RoomMenu.svelte';

  let {
    room,
    compact = false,
    onenter,
    onmenu,
    onforget,
    onrename,
    ondelete,
  }: {
    room: SavedRoomView;
    /** Celular (M4): sem o botão "Entrar", a linha toda entra. */
    compact?: boolean;
    onenter: () => void;
    onmenu: (open: boolean) => void;
    onforget: () => void;
    onrename?: () => void;
    ondelete?: () => void;
  } = $props();

  const gone = $derived(room.state === 'gone');
  const live = $derived(room.state === 'ok' && room.streaming > 0);
</script>

<div class="row" class:menu-open={room.menuOpen} class:compact>
  {#if compact && !gone}
    <!-- Celular: a linha inteira entra. -->
    <button class="hit" aria-label="Entrar em {room.name}" onclick={onenter}></button>
  {/if}
  <div class="avatar tone-{room.tone}" class:live class:gone aria-hidden="true">{room.name.trim().charAt(0).toUpperCase()}</div>
  <div class="text">
    <div class="name-line">
      <span class="name" class:dim={gone}>{room.name}</span>
      {#if room.own}<span class="own">Sua</span>{/if}
    </div>
    {#if room.state === 'loading'}
      <span class="skeleton" aria-label="Carregando"></span>
    {:else if room.state === 'ok'}
      <span class="sub">{savedRoomSub(room)}</span>
    {:else}
      <span class="sub gone-text">Esta sala não existe mais</span>
    {/if}
  </div>
  {#if live}
    <span class="live-badge"><span class="dot"></span>AO VIVO</span>
  {/if}
  {#if gone}
    <button class="btn forget" onclick={onforget}>Tirar da lista</button>
  {:else}
    <div class="actions">
      {#if !compact}<button class="btn btn-secondary enter" onclick={onenter}>Entrar</button>{/if}
      <button
        class="btn btn-icon more"
        aria-label="Mais opções"
        aria-haspopup="menu"
        aria-expanded={!!room.menuOpen}
        data-room-menu-toggle
        onclick={() => onmenu(!room.menuOpen)}
      >
        <Ellipsis size={18} strokeWidth={2.75} />
      </button>
    </div>
  {/if}
  {#if room.menuOpen}
    <RoomMenu variant="saved" own={room.own} {onrename} {ondelete} {onforget} onclose={() => onmenu(false)} />
  {/if}
</div>

<style>
  .row {
    position: relative; display: flex; align-items: center; gap: 14px; padding: 10px; border-radius: 26px;
    color: var(--color-text);
  }
  .row:hover, .row.menu-open { background: rgba(245, 234, 216, 0.07); }
  .hit { position: absolute; inset: 0; border: 0; border-radius: inherit; background: transparent; cursor: pointer; }

  .avatar {
    flex: none; width: 46px; height: 46px; border-radius: 50%; display: grid; place-items: center;
    font-family: var(--font-heading); font-size: 19px; line-height: 1;
  }
  .compact .avatar { width: 42px; height: 42px; }
  .avatar.live { box-shadow: 0 0 0 2px var(--color-surface), 0 0 0 4px var(--color-accent); }
  /* Celular: sem cartão, o anel separa do fundo da página. */
  .compact .avatar.live { box-shadow: 0 0 0 2px var(--color-bg), 0 0 0 4px var(--color-accent); }
  .tone-accent { background: var(--color-accent-300); color: var(--color-accent-800); }
  .tone-sage { background: var(--color-accent-2-300); color: var(--color-accent-2-800); }
  .tone-sand { background: var(--color-neutral-300); color: var(--color-neutral-800); }
  .tone-deep { background: var(--color-accent-400); color: var(--color-accent-900); }
  .avatar.gone { background: rgba(245, 234, 216, 0.08); color: rgba(245, 234, 216, 0.5); }

  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
  .name-line { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .name { font-size: 16px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .name.dim { opacity: 0.6; }
  .own {
    flex: none; padding: 2px 8px; border-radius: var(--radius-pill); background: rgba(245, 234, 216, 0.1);
    font-size: 11px; font-weight: 600; color: rgba(245, 234, 216, 0.75);
  }
  .sub { font-size: 13px; color: rgba(245, 234, 216, 0.62); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .gone-text { color: var(--color-accent-300); }
  .skeleton { width: 128px; height: 10px; margin: 4px 0 3px; border-radius: var(--radius-pill); background: rgba(245, 234, 216, 0.1); }

  .live-badge {
    flex: none; display: inline-flex; align-items: center; gap: 5px; padding: 3px 8px; border-radius: var(--radius-pill);
    background: var(--color-accent); color: var(--color-accent-900); font-size: 10px; font-weight: 700; letter-spacing: 0.08em;
  }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--color-accent-900); }

  .actions { position: relative; z-index: 1; flex: none; display: flex; align-items: center; gap: 4px; }
  .enter { min-height: 40px; padding: 0 16px; font-size: 14px; }
  .more { width: 44px; height: 44px; color: var(--color-text); background: transparent; }
  .more:hover, .menu-open .more { background: rgba(245, 234, 216, 0.12); }
  .forget {
    position: relative; z-index: 1; flex: none; min-height: 40px; padding: 0 16px; font-size: 14px; color: var(--color-text); background: rgba(245, 234, 216, 0.1);
  }
</style>
