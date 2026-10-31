<script lang="ts">
  import { UserMinus } from '@lucide/svelte';
  import type { ParticipantView } from '../../lib/types';
  import Avatar from './Avatar.svelte';
  import LiveBadge from './LiveBadge.svelte';

  let {
    person,
    watching = false,
    onremove,
  }: {
    person: ParticipantView;
    watching?: boolean;
    /** Dono (specs/014 US4): "Remover da sala". */
    onremove?: () => void;
  } = $props();

  // Com a descrição (specs/012), ela toma o lugar do "transmitindo": o selo AO VIVO já diz isso.
  const sub = $derived(
    watching
      ? 'você está assistindo'
      : person.live
        ? (person.liveLabel ?? (person.you ? 'transmitindo pra sala' : 'transmitindo'))
        : '',
  );
</script>

<li class="row" class:watching>
  <Avatar name={person.name} tone={person.tone} live={person.live} />
  <div class="text">
    <div class="name-line">
      <span class="name">{person.name}</span>
      {#if person.you}<span class="you">você</span>{/if}
    </div>
    {#if sub}<span class="sub">{sub}</span>{/if}
  </div>
  {#if person.live}<LiveBadge />{/if}
  {#if person.removable && !person.you && onremove}
    <button class="remove" aria-label={`Remover ${person.name} da sala`} title="Remover da sala" onclick={onremove}>
      <UserMinus size={16} strokeWidth={2.75} />
    </button>
  {/if}
</li>

<style>
  .row {
    position: relative;
    display: flex; align-items: center; gap: 12px;
    padding: 8px 12px 8px 8px; border-radius: var(--radius-pill);
  }
  .watching { background: rgba(245, 234, 216, 0.07); }
  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .name-line { display: flex; align-items: center; gap: 6px; }
  .name { font-size: 15px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .you {
    font-size: 11px; padding: 1px 8px; border-radius: var(--radius-pill);
    background: rgba(245, 234, 216, 0.1); color: color-mix(in srgb, var(--color-text) 75%, transparent);
  }
  /* Por cima do fim da linha, sem ocupar espaço (não corta o texto). */
  .remove {
    position: absolute; right: 8px; top: 50%; translate: 0 -50%;
    width: 32px; height: 32px; border: 0; border-radius: 50%;
    display: grid; place-items: center; cursor: pointer;
    color: var(--color-text); background: var(--color-surface);
    box-shadow: 0 0 0 4px var(--color-surface);
    opacity: 0; transition: opacity 150ms ease-out;
  }
  .row:hover .remove, .remove:focus-visible { opacity: 1; }
  .remove:hover { background: rgba(245, 234, 216, 0.24); }
  @media (hover: none) { .remove { opacity: 1; } }
  .sub { font-size: 12px; color: var(--color-text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
