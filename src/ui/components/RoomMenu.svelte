<script lang="ts">
  // Menu "⋯" de uma sala fixa (specs/011): em Suas salas (SavedRoom.dc.html) e no cabeçalho da sala
  // do dono (Room.dc.html S2). Renomear/apagar só aparecem pro dono.
  import { ListMinus, Pencil, Trash } from '@lucide/svelte';

  let {
    variant,
    own,
    onrename,
    ondelete,
    onforget,
    onclose,
  }: {
    variant: 'saved' | 'room';
    own: boolean;
    onrename?: () => void;
    ondelete?: () => void;
    onforget?: () => void;
    onclose: () => void;
  } = $props();

  let menu: HTMLDivElement;

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      onclose();
    }
  }

  // Clique fora fecha (o botão "⋯" que abriu cuida do próprio clique).
  function onpointerdown(e: PointerEvent) {
    const target = e.target as Element;
    if (!menu.contains(target) && !target.closest('[data-room-menu-toggle]')) onclose();
  }
</script>

<svelte:window {onkeydown} {onpointerdown} />

<div class="menu {variant}" role="menu" bind:this={menu}>
  {#if own && (onrename || ondelete)}
    {#if onrename}
      <button class="item" role="menuitem" onclick={onrename}><Pencil size={16} strokeWidth={2.75} />Renomear sala</button>
    {/if}
    {#if ondelete}
      <button class="item danger" role="menuitem" onclick={ondelete}><Trash size={16} strokeWidth={2.75} />Apagar sala</button>
    {/if}
    {#if variant === 'saved'}<div class="sep"></div>{/if}
  {/if}
  {#if variant === 'saved' && onforget}
    <button class="item" role="menuitem" onclick={onforget}><ListMinus size={16} strokeWidth={2.75} />Tirar da lista</button>
    <span class="note">Some só deste aparelho. A sala continua pros outros.</span>
  {:else if variant === 'room'}
    <span class="note">Só aparece pra você — a sala foi criada neste aparelho.</span>
  {/if}
</div>

<style>
  .menu {
    position: absolute; z-index: 6; width: 220px; display: flex; flex-direction: column; gap: 2px; padding: 8px;
    border-radius: 22px; background: #322e28; color: var(--color-text);
    box-shadow: 0 0 0 1px rgba(245, 234, 216, 0.08), 0 18px 48px rgba(0, 0, 0, 0.55);
  }
  .menu.saved { right: 8px; top: calc(100% - 4px); }
  .menu.room { left: 0; top: calc(100% + 8px); width: 230px; }
  .item {
    display: flex; align-items: center; gap: 10px; min-height: 44px; padding: 0 14px; border: 0;
    border-radius: var(--radius-pill); cursor: pointer; text-align: left;
    font-family: var(--font-body); font-size: 14px; font-weight: 600; color: var(--color-text); background: transparent;
  }
  .item:hover, .item:focus-visible { background: rgba(245, 234, 216, 0.08); outline: none; }
  .item.danger { color: var(--color-accent-300); }
  .item.danger:hover, .item.danger:focus-visible { background: rgba(198, 113, 57, 0.14); }
  .sep { height: 1px; margin: 4px 10px; background: rgba(245, 234, 216, 0.1); }
  .note { padding: 2px 14px 6px; font-size: 12px; line-height: 1.35; color: rgba(245, 234, 216, 0.55); }
  .room .note { padding-top: 4px; }
</style>
