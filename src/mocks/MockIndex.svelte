<script lang="ts">
  import Logo from '../ui/components/Logo.svelte';
  import { MOCK_SCREENS, PLANNED_SCREENS, mockHref } from './registry';
</script>

<div class="index">
  <Logo size={36} />
  <h1>Telas mockadas</h1>
  <p class="muted">Dados falsos, sem servidor. Use <code>?mock=&lt;tela&gt;&amp;scene=&lt;cena&gt;</code>.</p>

  {#each MOCK_SCREENS as screen}
    <section>
      <h2>{screen.label} <code>?mock={screen.id}</code></h2>
      <ul>
        {#each screen.scenes as s}
          <li>
            <a href={mockHref(screen.id, s.id)}>
              <span class="code">{s.code}</span>
              {s.label}
              <code>{s.id}</code>
            </a>
          </li>
        {/each}
      </ul>
    </section>
  {/each}

  {#if PLANNED_SCREENS.length > 0}
    <section>
      <h2>Em breve</h2>
      <ul>
        {#each PLANNED_SCREENS as s}
          <li class="planned">{s.label} <code>?mock={s.id}</code></li>
        {/each}
      </ul>
    </section>
  {/if}
</div>

<style>
  .index { max-width: 720px; margin: 0 auto; padding: 48px 16px; display: flex; flex-direction: column; gap: 16px; }
  h1 { margin: 16px 0 0; font-family: var(--font-heading); font-weight: 400; font-size: 40px; line-height: 1.1; }
  h2 { margin: 0 0 10px; font-family: var(--font-heading); font-weight: 400; font-size: 22px; }
  p { margin: 0; }
  section { padding: 20px; border-radius: var(--radius-card); background: var(--color-surface); }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  a {
    display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-radius: var(--radius-pill);
    color: var(--color-text); text-decoration: none;
  }
  a:hover { background: rgba(245, 234, 216, 0.07); }
  a code { margin-left: auto; }
  .code {
    min-width: 44px; text-align: center; font-size: 11px; font-weight: 700; letter-spacing: 0.06em;
    padding: 3px 8px; border-radius: var(--radius-pill); background: var(--color-accent); color: var(--color-accent-900);
  }
  .planned { padding: 10px 14px; color: var(--color-text-muted); display: flex; justify-content: space-between; }
  code { font-family: var(--font-mono); font-size: 13px; color: var(--color-accent-400); }
</style>
