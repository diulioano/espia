---
description: "Tarefas da 003: Espia Desktop (Tauri, Windows) + GitHub"
---

# Tasks: Espia Desktop — app leve para Windows

**Input**: `/specs/003-espia-desktop/` (spec.md, plan.md, research.md, data-model.md, contracts/, quickstart.md)

**Tests**: incluídos para a lógica pura (links, regra da barra, `openAction`). Captura real e instalador: manual (quickstart Q3–Q5).

**Organização**: por história. `[P]` = pode rodar em paralelo. Caminhos relativos à raiz.

## Phase 1: Setup

- [x] T001 Instalar `@tauri-apps/cli` (devDependency) e gerar `src-tauri/` com `tauri init`
- [x] T002 [P] Ícone do castor: `src-tauri/icons-src/icon.svg` (formas do `Mascot.svelte`, cores de `tokens.css`) → `npx tauri icon`; manter só os ícones de Windows
- [x] T003 Scripts `desktop:dev` e `desktop:build` em `package.json`; `src-tauri/target` e `src-tauri/gen` no `.gitignore`
- [x] T004 Emenda da constituição 1.1.0 (Princípio I: app desktop opcional; restrição técnica do app) em `.specify/memory/constitution.md`

## Phase 2: Foundational

- [x] T005 `src-tauri/Cargo.toml` (crate `espia`, plugins, `webview2-com`/`windows` só no Windows, perfil release enxuto) e `tauri.conf.json` (identificador `dev.espia.app`, sem janelas no config, CSP da página local, `deep-link` com esquema `espia`, NSIS currentUser + PortugueseBR + bootstrapper do WebView2)
- [x] T006 [P] `src-tauri/src/links.rs`: `room_path`, `room_path_from_args`, `is_internal` + testes (contracts/deep-link.md)

## Phase 3: US1 — Instalar e abrir (P1) 🎯 MVP

- [x] T007 [US1] Página local `src-tauri/www/` (index.html, loader.css, loader.js): castor "Abrindo o Espia…", checagem de rede (HEAD no-cors com timeout 8 s), `location.replace(ORIGIN + to)`, estado "Sem conexão" com "Tentar de novo"
- [x] T008 [US1] `src-tauri/src/lib.rs`: janela "Espia" 1280×800 (mín. 720×480) criada no `setup`, plugin window-state (FR-009)
- [x] T009 [US1] Navegação: `on_navigation` (fora do Espia → navegador padrão), `on_new_window` → navegador padrão (FR-007)
- [x] T010 [US1] `npm run desktop:build`: conferir tamanho do instalador (SC-001), instalação sem admin e com bootstrapper do WebView2 (FR-010), RAM ociosa (SC-002) e tempo até a tela inicial (SC-003)
  - Medido 2026-10-28 (build local): instalador **1,13 MB** ✅; app aberto na tela inicial: **~209 MB privados** (Espia + 7 processos WebView2; working set 425 MB com DLLs compartilhadas) ⚠️ acima da meta de ~150 MB do SC-002 — é o piso do WebView2 com a página do Espia; abre em ~1–2 s ✅

## Phase 4: US2 — Transmitir e assistir pelo app (P1)

- [x] T011 [US2] Spike: `getDisplayMedia` no WebView2 156 (research R5)
- [x] T012 [US2] `src-tauri/src/share_bar.rs`: `ScreenCaptureStarting` → vigia 120 s → esconde a barra (FR-018) + diagnóstico em debug + teste de `title_is_bar`
- [ ] T013 [US2] Teste manual quickstart Q3 (app → site e site → app, com a barra escondida) — **precisa da pessoa usuária**

## Phase 5: US3 — Links de sala (P2)

- [x] T014 [US3] Instância única + deep link em `lib.rs` (`single-instance` com `deep-link`; `register_all()` em debug); app fechado abre `index.html?to=/s/…`
- [x] T015 [P] [US3] Site: `src/lib/desktop.ts` (`openAction` puro + `installDesktopHook`), `router.leaveGuard` em `src/lib/router.svelte.ts`, registro em `src/routes/RoomRoute.svelte`, instalação em `src/main.ts`
- [x] T016 [P] [US3] `test/lib/desktop.test.ts`
- [x] T017 [US3] FR-011: confirmar que a tela inicial aceita link colado (`parseRoomId` já trata `/s/…`, inclusive `espia://s/…`) e cobrir com teste em `test/lib/room-id.test.ts`
- [ ] T018 [US3] Teste manual quickstart Q4 — **precisa da pessoa usuária** (o registro de `espia://` vale após instalar)

## Phase 6: US4 — GitHub, CI, release e deploy (P2)

- [x] T019 [P] [US4] `.github/workflows/ci.yml`
- [x] T020 [P] [US4] `.github/workflows/release.yml` (tauri-action, tag `v*`)
- [x] T021 [P] [US4] `.github/workflows/deploy.yml` (pula sem `CLOUDFLARE_API_TOKEN`)
- [x] T022 [US4] `README.md` (o que é, baixar o app, aviso do SmartScreen, desenvolver, lançar versão)
- [x] T023 [US4] Varredura de segredos antes do primeiro push (`.dev.vars`, tokens em specs/memória)
- [x] T024 [US4] `git init`, commit, `gh repo create DiulianoVargas/espia --public`, push
- [x] T025 [US4] Tag `v0.1.0` → acompanhar o release até o instalador aparecer (SC-007)

## Phase 7: Polish

- [x] T026 Atualizar `DEFINICOES.md` (nome Espia, app desktop saiu do "fora do escopo", GitHub) e `CLAUDE`/memória
- [x] T027 `npm run deploy` do site com o gancho `espiaOpen`
- [x] T028 Renomear a pasta `Documents\screen-share` → `Documents\espia` (por último)

## Dependências

Setup → Foundational → US1 → (US2, US3 em paralelo) → US4 → Polish. T013/T018 dependem do instalador
da release (T025).
