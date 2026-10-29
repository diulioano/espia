# Contrato: repositório, CI e releases

| Workflow | Gatilho | O que faz | Segredos |
|---|---|---|---|
| `ci.yml` | push e pull request | `npm ci` → `npm run check` → `npm test` → `npm run build` (Ubuntu); `cargo test` em `src-tauri` (Windows) | — |
| `release.yml` | tag `v*` (ou manual) | `tauri-apps/tauri-action` gera o instalador NSIS (`Espia_<versão>_x64-setup.exe`) e publica a Release com ele | `GITHUB_TOKEN` (automático) |
| `deploy.yml` | push na `main` (ou manual) | `npm ci` → `npm run deploy` (vite build + wrangler deploy) | `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`; sem eles o job é pulado |

## Versão

A versão do app vem de `src-tauri/tauri.conf.json` (`version`). Para lançar: subir `version` (e
`Cargo.toml`), commitar, `git tag vX.Y.Z && git push --tags`.

## Segredos

- Nunca no repositório: `.dev.vars` (token do TURN) está no `.gitignore`.
- O token do TURN de produção é segredo do Worker (`wrangler secret put TURN_KEY_API_TOKEN`), não do GitHub.
- `CLOUDFLARE_API_TOKEN`: token com permissão "Edit Cloudflare Workers", criado no painel da Cloudflare e
  salvo em *Settings → Secrets and variables → Actions* do repositório.
