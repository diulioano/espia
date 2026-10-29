import { cloudflareTest } from '@cloudflare/vitest-plugin';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: { configPath: './wrangler.jsonc' },
      // Tolerância e reserva curtas para os testes não esperarem 20 s / 60 s.
      // Token falso: a API do TURN é simulada nos testes (nunca usa o token real do .dev.vars).
      miniflare: { bindings: { GRACE_MS: '400', RESERVE_MS: '600', TURN_KEY_API_TOKEN: 'test-token' } },
    }),
  ],
  test: { include: ['test/**/*.test.ts'] },
});
