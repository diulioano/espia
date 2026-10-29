# Quickstart — Espia

## Mock (sem backend)
```sh
npm install
npm run dev
```
Abra:
- `http://localhost:5173/?mock` — índice de telas e cenas
- `http://localhost:5173/?mock=room` — sala, cena padrão (`single`)
- `http://localhost:5173/?mock=room&scene=grid` — grade com 3 transmissões
- Cenas da sala: `connecting`, `empty`, `copied`, `modal`, `denied`, `single`, `grid`, `focus`, `streamfail`, `live`

O seletor no canto inferior direito troca a cena sem recarregar.

## Checagens
```sh
npm run check   # svelte-check + tipos
npm run build
```

## Real (fase seguinte)
```sh
npx wrangler login
# TURN: painel Cloudflare → Realtime → TURN → criar chave. TURN_KEY_ID vai em wrangler.jsonc (vars)
npx wrangler secret put TURN_KEY_API_TOKEN
# local: copiar para .dev.vars (já no .gitignore)
npm run dev:worker     # wrangler dev com DO local
npm run deploy         # build + wrangler deploy → *.workers.dev
```
