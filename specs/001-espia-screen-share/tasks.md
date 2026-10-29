---
description: "Tarefas do Espia, quebradas por tela"
---

# Tasks: Espia — compartilhamento de tela entre amigos

**Input**: `/specs/001-espia-screen-share/` (spec.md, plan.md, screens.md, data-model.md, contracts/)

**Organização**: primeiro todas as telas mockadas (US1), depois a lógica real por história.
`[P]` = pode rodar em paralelo (arquivos diferentes). Caminhos relativos à raiz do repositório.

## Phase 1: Setup

- [x] T001 Criar projeto Vite + Svelte 5 + TS na raiz (`package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.ts`)
- [x] T002 [P] Instalar `@lucide/svelte`, `svelte-check`; scripts `dev`, `build`, `check`
- [x] T003 [P] Copiar tokens do handoff para `src/ui/tokens.css` e criar `src/ui/base.css` (`.btn*`, `.input`, `.seg`, `.dialog`, foco)

## Phase 2: Foundational

- [x] T004 View-models em `src/lib/types.ts` (data-model.md)
- [x] T005 Derivações puras do header/palco em `src/lib/room-view.ts`
- [x] T006 `App.svelte`: se `?mock` → `MockApp` (import dinâmico); senão roteador real (placeholder por enquanto)
- [x] T007 `src/mocks/MockApp.svelte` + `src/mocks/registry.ts` (telas × cenas) + índice para `?mock` sem valor
- [x] T008 `src/mocks/MockSwitcher.svelte` (troca tela/cena via `history.replaceState`)

**Checkpoint**: `/?mock` abre o índice.

## Phase 3: US1 — Sala mockada (P1) 🎯 primeira entrega

**Goal**: `?mock=room&scene=<cena>` mostra todas as cenas de `Room.dc.html`.
**Independent Test**: percorrer as 10 cenas pelo seletor e comparar com o design.

- [x] T009 [P] [US1] `src/ui/components/Logo.svelte`
- [x] T010 [P] [US1] `src/ui/components/LiveBadge.svelte` (sm/md)
- [x] T011 [P] [US1] `src/ui/components/Avatar.svelte` (tons, anel ao vivo)
- [x] T012 [P] [US1] `src/ui/components/Participant.svelte`
- [x] T013 [P] [US1] `src/ui/components/Spinner.svelte`
- [x] T014 [US1] `src/ui/components/StreamTile.svelte` (stage/grid/thumb, ok/loading/failed, chip de nome, controles com auto-hide 2s, mudo, slider, PiP, tela cheia, `<video>` quando houver `stream`)
- [x] T015 [P] [US1] `src/ui/components/Toast.svelte` + `ToastStack.svelte` (máx. 3, animações)
- [x] T016 [P] [US1] `src/ui/components/GoLiveModal.svelte` (segmentados, switch, aviso, Esc/backdrop fecham)
- [x] T017 [P] [US1] `src/ui/components/PermissionAlert.svelte` (E3)
- [x] T018 [US1] `src/ui/components/Stage.svelte` (conectando / vazio / único / grade 2×2 ou 3×2 / foco + faixa; Esc volta)
- [x] T019 [US1] `src/ui/pages/RoomPage.svelte` (sidebar, header com convite, palco, footer, card "Você está ao vivo", contador)
- [x] T020 [US1] `src/mocks/room.ts`: as 10 cenas (connecting, empty, copied, modal, denied, single, grid, focus, streamfail, live) com interações locais (focar, Esc, copiar, abrir/fechar modal, mudo/volume)
- [x] T021 [US1] Responsivo da sala: < 1024px sidebar vira faixa horizontal (primeira versão; revisar com o design)
- [ ] T021b [US1] Revisão visual lado a lado com `Room.dc.html` em 1440×900, cena por cena

**Checkpoint**: sala mockada completa.

## Phase 4: US1 — Demais telas mockadas

- [x] T022 [P] [US1] `src/ui/pages/HomePage.svelte` (D1) + `src/mocks/home.ts` (`default`, `empty`, `error`)
- [x] T023 [P] [US1] `src/ui/components/TextField.svelte` (label, dica, erro)
- [x] T024 [P] [US1] `src/ui/components/AvatarStack.svelte`
- [x] T025 [US1] `src/ui/pages/JoinPage.svelte` (D2, E2, cheia, M1 responsivo) + `src/mocks/join.ts` (`default`, `nopreview`, `notfound`, `full`, `mobile`)
- [x] T026a [US1] Grade até 9 transmissões (2×1, 2×2, 3×2, 3×3) + cena `crowd` (10 pessoas) no mock da sala
- [x] T026 [US1] `src/ui/pages/MobileRoomPage.svelte` (M2, M3, conectando, falha) + `src/mocks/mobile.ts`; M1 em `?mock=join&scene=mobile`
- [x] T026b [US1] Design v1.1 (2026-10-28): `src/ui/components/Mascot.svelte` (castor, 6 humores, só CSS, `prefers-reduced-motion`); logo vira o castor; castor na home (peek), entrada (happy), E2/cheia (sad), palco vazio (sleepy), conectando (loading), alerta E3 (sad, agora no canto superior direito junto com os toasts), celular vazio/conectando

**Checkpoint**: SC-001 — todas as telas do handoff abrem via `?mock`.

## Phase 4b: Fundação da fase real (bloqueia US2+)

> Referências: [architecture.md](./architecture.md), [security.md](./security.md), [contracts/](./contracts/).

- [x] T046 [Spike] Codec e CPU: Chrome/Edge Windows, 1 transmissor → 4 espectadores em 1080p60 **e** → 9 espectadores em 720p30, comparar H.264 × VP9 × VP8 (CPU, fps, nitidez de texto). Resultado fixa `setCodecPreferences` (research R11) — *dispensado pelo usuário (2026-10-28): usar o codec padrão do navegador*
- [x] T047 [Spike] TURN: gerar credencial, forçar `iceTransportPolicy: 'relay'` entre 2 abas, medir; testar revogação no meio da chamada — *feito: relay funciona (11–12 candidatos); revogação não bloqueia na hora, ver security.md A7*
- [x] T048 Responder D-1 (exposição de IP): opção A, conexão direta
- [x] T049 [P] Fontes auto-hospedadas (`@fontsource/caprasimo`, `@fontsource/figtree`) — remover `@import` do Google Fonts de `tokens.css`
- [x] T050 [P] `wrangler` + `wrangler.jsonc`: assets SPA (`run_worker_first` para `/api/*`, `/ws/*`), DO `Room` (`new_sqlite_classes`), 3 bindings de rate limit, var `TURN_KEY_ID`, secret `TURN_KEY_API_TOKEN`; scripts `dev:worker`, `deploy` — *var/secret do TURN ficam para T038*
- [x] T051 [P] `public/_headers` + cabeçalhos no Worker (security.md §5)
- [x] T052 `src/shared/protocol.ts`: tipos das mensagens (signaling.md) usados pelo cliente e pelo Worker
- [x] T053 `src/worker/validate.ts`: validação de cada mensagem + higienização de apelido (security A4/A5)

## Phase 5: US2 — Criar sala e convidar (P1)

- [x] T027 [P] [US2] `src/lib/room-id.ts` (gerar código `xxx-xxx` com CSPRNG; normalizar/extrair código de link ou texto colado)
- [x] T028 [P] [US2] `src/lib/prefs.ts` (apelido, goLive, volumes em `localStorage` com try/catch)
- [x] T029 [US2] `src/lib/router.ts` (`/`, `/s/:código`, history API)
- [x] T030 [US2] `src/worker/index.ts`: `POST /api/rooms`, `GET /api/rooms/:c`, `/ws/:c`, checagem de `Origin`, rate limit, 400/403/429
- [x] T031 [US2] `src/worker/room.ts` (DO): `reserve`, `preview`, `join`, `room-full`, `room-not-found`, `peer-joined/left`, tons, `from` definido no servidor, hibernação com `serializeAttachment`, auto-resposta de `ping`, limites de tamanho/frequência
- [x] T032 [US2] DO: tolerância de 20 s + `resume` com `resumeToken`
- [x] T033 [US2] `src/lib/signaling.ts` (conectar, fila, backoff, `resume`) + `src/lib/stores/room.svelte.ts` → `RoomView`
- [x] T034 [US2] Ligar HomePage (criar/colar link) e RoomPage (lista real, copiar convite, toasts join/leave, sair)
- [x] T035 [P] [US2] Testes do DO com `@cloudflare/vitest-plugin` (`npm test`, 23 testes) + `npm run smoke` contra servidor rodando: 11ª pessoa recusada, `from` forjado ignorado, flood/64 KB fecham, `Origin` estranho 403, resume dentro/fora dos 20 s

## Phase 6: US3 — Entrar por link (P1)

- [x] T036 [US3] JoinPage real: `GET /api/rooms/:c` → prévia, `notfound` (404), `full`; erro de rede → esconde a prévia
- [x] T037 [US3] Estado `connecting` real até o `welcome`

## Phase 7: US4 — Go Live (P1)

- [x] T038 [US4] `src/worker/turn.ts`: gerar credenciais (TTL 6 h) no `welcome`, `turn-refresh`, revogar no `peer-left`
- [x] T039 [US4] `src/lib/peer.ts`: `RTCPeerConnection` + canal `ctl`, perfect negotiation, trickle ICE, `setConfiguration` ao renovar TURN, `restartIce`
- [x] T040 [US4] `src/lib/mesh.ts`: um peer por participante (quem chega inicia), adicionar/remover a transmissão em todos, bitrate por resolução/fps, `contentHint`, `degradationPreference`, codec do spike
- [x] T041 [US4] `src/lib/capture.ts` + `stores/broadcast.svelte.ts`: `getDisplayMedia` (prefs, `systemAudio`, `selfBrowserSurface`), `NotAllowedError` → E3, `track.onended`, `live`/`unlive`
- [x] T042 [US4] Ligar GoLiveModal, card "Você está ao vivo" (prévia real, nº de espectadores), contador, Parar transmissão

## Phase 8: US5 — Assistir (P1)

- [x] T043 [US5] `stores/streams.svelte.ts`: `peer-live` → loading; primeiro frame → ok; `failed`/10 s → failed; retry
- [x] T044 [US5] Foco/grade/Esc reais; volume por pessoa persistido; começar mudo
- [x] T061 [US5] Teto de qualidade por nº de espectadores (`applyConstraints` + `setParameters`), reenvio de `live` com a qualidade efetiva, aviso no modal com > 5 pessoas (FR-034/035)
- [x] T045 [US5] Canal `ctl` `view: full|thumb|hidden` e ajuste do sender por espectador (research R10)
- [x] T054 [US5] PiP e tela cheia com o `<video>` real

## Phase 9: US6 — Avisos (P2)

- [x] T055 [US6] `stores/toasts.svelte.ts` (fila máx. 3, 4 s, erro persistente, "Assistir" foca)

## Phase 10: US7 — Celular (P3)

- [x] T056 [US7] Detecção `canShare`; < 640px usa MobileRoomPage; Go Live desabilitado

## Phase 11: Polish

- [ ] T057 [P] Vitest para `room-id.ts`, `room-view.ts`, `join-view.ts`, `prefs.ts`, `validate.ts`
- [ ] T058 Playwright: 3 contextos Chromium com captura falsa — criar, entrar, Go Live, ver, parar, sair
- [ ] T059 Checklist de segurança (security.md §6) + `npm audit`
- [ ] T060 Teste manual com 3 PCs + 1 celular (inclui 4G para forçar TURN); `npm run deploy` para `*.workers.dev`; alerta de uso no painel Cloudflare

## Dependências

- Phase 2 bloqueia tudo. Phases 3–4 (mocks) independem do backend.
- Phase 4b bloqueia US2+. T046/T047 (spikes) podem rodar antes de tudo e informam T040/T038.
- US2 → US3 → US4 → US5. US6 e US7 dependem de US2.
