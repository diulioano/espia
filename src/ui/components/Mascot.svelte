<script lang="ts" module>
  export type MascotMood = 'happy' | 'peek' | 'live' | 'sleepy' | 'loading' | 'sad';

  type Mood = {
    dx: number; dy: number; eh: number; teeth: number; body: string;
    open?: boolean; noBlink?: boolean; look?: boolean; noEars?: boolean;
  };

  const MOODS: Record<MascotMood, Mood> = {
    happy: { dx: 0, dy: 0, eh: 10, teeth: 10, body: 'castor-bob 3.2s ease-in-out infinite' },
    peek: { dx: 5, dy: -2, eh: 10, teeth: 10, body: 'castor-bob 2.6s ease-in-out infinite' },
    live: { dx: 0, dy: -1, eh: 12, teeth: 6, open: true, body: 'castor-hop 0.9s ease-in-out infinite' },
    sleepy: { dx: 0, dy: 4, eh: 2.5, teeth: 8, body: 'castor-breathe 4.5s ease-in-out infinite', noBlink: true },
    loading: { dx: 5, dy: -4, eh: 10, teeth: 8, body: 'castor-bob 1.6s ease-in-out infinite', look: true },
    sad: { dx: 0, dy: 4, eh: 7, teeth: 7, body: 'castor-breathe 5s ease-in-out infinite', noEars: true },
  };
</script>

<script lang="ts">
  // Castor (Mascot.dc.html): só formas CSS, tudo em % de uma caixa size × size·1.12.
  // animate = 'blink' liga só o piscar (usado no logo, pra não distrair).
  let {
    mood = 'happy',
    size = 140,
    tail = false,
    animate = true,
  }: { mood?: MascotMood; size?: number; tail?: boolean; animate?: boolean | 'blink' } = $props();

  const m = $derived(MOODS[mood] ?? MOODS.happy);
  const all = $derived(animate === true);
  const a = (on: boolean, v: string) => (on ? v : 'none');
</script>

<div
  class="castor"
  aria-hidden="true"
  style:width="{size}px"
  style:height="{Math.round(size * 1.12)}px"
  style:animation={a(all, m.body)}
>
  {#if tail}<div class="tail"></div>{/if}
  <div class="ear ear-l" style:animation={a(all && !m.noEars, 'castor-ear-l 5s ease-in-out infinite')}></div>
  <div class="ear ear-r" style:animation={a(all && !m.noEars, 'castor-ear-r 7s ease-in-out infinite')}></div>
  <div class="head"></div>
  {#each [26, 63] as left}
    <div
      class="eye"
      style:left="{left + (m.look ? 0 : m.dx)}%"
      style:top="{41 + m.dy}%"
      style:height="{m.eh}%"
      style:animation={a(all && !!m.look, 'castor-look 1.6s ease-in-out infinite')}
    >
      <div style:animation={a(animate !== false && !m.noBlink, 'castor-blink 4.2s ease-in-out infinite')}></div>
    </div>
  {/each}
  <div class="nose"></div>
  {#if m.open}<div class="mouth"></div>{/if}
  <div class="teeth" style:height="{m.teeth}%" style:gap="{Math.max(1, Math.round(size * 0.012))}px">
    <div></div>
    <div></div>
  </div>
</div>

<style>
  .castor { position: relative; flex: none; transform-origin: 50% 100%; }
  .castor > div { position: absolute; }
  .tail {
    right: -14%; top: 58%; width: 30%; height: 38%; border-radius: 50%;
    background: var(--color-accent-900); transform: rotate(32deg);
  }
  .ear { top: 6%; width: 24%; height: 22%; border-radius: 50%; background: var(--color-accent-700); }
  .ear-l { left: 8%; transform-origin: 70% 90%; }
  .ear-r { right: 8%; transform-origin: 30% 90%; }
  .head {
    left: 0; top: 13%; width: 100%; height: 87%;
    border-radius: 50% 50% 46% 46% / 58% 58% 42% 42%; background: var(--color-accent-700);
  }
  .eye { width: 11%; }
  .eye div { width: 100%; height: 100%; border-radius: 999px; background: #201e1d; }
  .nose { left: 42%; top: 57%; width: 16%; height: 8%; border-radius: 50%; background: #201e1d; }
  .mouth { left: 41%; top: 66%; width: 18%; height: 13%; border-radius: 20% 20% 999px 999px; background: #201e1d; }
  .teeth { left: 43.5%; top: 66%; width: 13%; display: flex; }
  .teeth div { flex: 1; background: #fdf6ec; border-radius: 0 0 999px 999px; }

  @keyframes -global-castor-bob { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-4%); } }
  @keyframes -global-castor-hop {
    0%, 100% { transform: translateY(0) scaleY(1); }
    35% { transform: translateY(-9%) scaleY(1.02); }
    70% { transform: translateY(0) scaleY(0.96); }
  }
  @keyframes -global-castor-breathe { 0%, 100% { transform: scale(1); } 50% { transform: scale(1.035, 0.985); } }
  @keyframes -global-castor-blink { 0%, 90%, 100% { transform: scaleY(1); } 94% { transform: scaleY(0.1); } }
  @keyframes -global-castor-look { 0%, 100% { transform: translateX(-35%); } 50% { transform: translateX(35%); } }
  @keyframes -global-castor-ear-l { 0%, 78%, 100% { transform: rotate(0); } 84% { transform: rotate(-14deg); } 90% { transform: rotate(6deg); } }
  @keyframes -global-castor-ear-r { 0%, 60%, 100% { transform: rotate(0); } 66% { transform: rotate(12deg); } 72% { transform: rotate(-5deg); } }

  @media (prefers-reduced-motion: reduce) {
    .castor, .castor :global(*) { animation: none !important; }
  }
</style>
