import { cloudflareTest } from '@cloudflare/vitest-plugin';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: { configPath: './wrangler.jsonc' },
      // Tempos curtos para os testes não esperarem 60 s / 5 min / 30 dias.
      // Token falso: a API do TURN é simulada nos testes (nunca usa o token real do .dev.vars).
      miniflare: { bindings: { GRACE_MS: '400', RESERVE_MS: '600', HOLD_MS: '800', FIXED_TTL_MS: '1500', TURN_KEY_API_TOKEN: 'test-token' } },
    }),
  ],
  test: { include: ['test/**/*.test.ts'] },
});
