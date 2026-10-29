# Implementation Plan: Espia — compartilhamento de tela entre amigos

**Branch**: `001-espia-screen-share` | **Date**: 2026-10-28 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-espia-screen-share/spec.md`

## Summary

App web (SPA Svelte 5 + Vite + TS) para salas de até 10 pessoas (uso normal 3–5) compartilharem telas via WebRTC P2P
em malha. Um Cloudflare Worker serve o site, emite credenciais temporárias de TURN e encaminha o
WebSocket para um Durable Object por sala, que faz a sinalização e mantém a lista de participantes.
A interface segue o handoff "Espia" (`design_handoff_espia/`). Entrega em duas camadas: primeiro
todas as telas com dados mockados (`?mock=<tela>&scene=<estado>`), depois a lógica real ligada aos
mesmos componentes. Inventário de telas em [screens.md](./screens.md).

## Technical Context

**Language/Version**: TypeScript 5.x, Node 24 (dev), Svelte 5 (runes)

**Primary Dependencies**: Svelte 5, Vite, `@lucide/svelte`, `@fontsource/*`; APIs nativas de WebRTC (sem biblioteca); Worker: `wrangler` 4, Durable Objects (classe SQLite, Hibernation WebSocket API), Rate Limiting binding, Workers Static Assets, Cloudflare Realtime TURN

**Storage**: Nenhum persistente. Estado da sala em memória do Durable Object; preferências em `localStorage`

**Testing**: `svelte-check`; Vitest (`src/lib`); `@cloudflare/vitest-pool-workers` (Durable Object); Playwright com captura falsa (ponta a ponta)

**Target Platform**: Navegador (Chrome/Edge desktop para transmitir; qualquer moderno para assistir) + Cloudflare Workers

**Project Type**: Web app (SPA + Worker no mesmo pacote)

**Performance Goals**: Entrar na sala < 2s após o clique; até 1080p60 com ≤ 4 espectadores, qualidade adaptativa até 9; até 10 pessoas por sala

**Constraints**: Sem servidor de mídia; upload do transmissor = N × bitrate; sem segredos no cliente

**Scale/Scope**: 1 grupo fechado de até 10 amigos; ~10 telas/estados de desktop, 3 de celular, 1 Durable Object, 2 rotas de API

## Constitution Check

| Gate | Situação |
|---|---|
| I. Simplicidade (≤ 2 passos, sem conta) | ✅ apelido + Entrar |
| II. Sem servidor de mídia | ✅ P2P mesh + TURN só como relay |
| III. Efêmero | ✅ DO em memória; some quando vazio |
| IV. Camada visual separada | ✅ `src/ui/` recebe view-models; `src/lib/` tem a lógica |
| V. Telas mockáveis | ✅ `src/mocks/` + `?mock=` (primeira entrega) |
| VI. Segredos fora do navegador | ✅ `TURN_KEY_API_TOKEN` como secret; credenciais TURN só via WebSocket e revogadas ao sair; rate limit contra varredura de códigos (security.md) |

Sem violações → Complexity Tracking vazio.

## Project Structure

### Documentation (this feature)

```text
specs/001-espia-screen-share/
├── spec.md
├── plan.md              # este arquivo
├── screens.md           # inventário de telas, estados e parâmetros de mock
├── architecture.md      # como funciona de ponta a ponta + tecnologias + custos
├── security.md          # ameaças, mitigações, privacidade, cabeçalhos
├── research.md          # decisões técnicas (R1–R14)
├── data-model.md        # entidades, view-models e stores
├── quickstart.md        # como rodar mock, dev e deploy
├── contracts/
│   ├── signaling.md     # mensagens do WebSocket (cliente ↔ Durable Object)
│   └── http-api.md      # rotas HTTP do Worker
├── checklists/requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
index.html
vite.config.ts
wrangler.jsonc                 # (fase real) Worker + DO + assets
src/
├── main.ts                    # monta App
├── App.svelte                 # decide: modo mock ou roteador real
├── ui/                        # camada visual (trocável)
│   ├── tokens.css             # tokens do handoff
│   ├── base.css               # .btn, .input, .seg, .dialog (Organic adaptado)
│   ├── components/            # Logo, LiveBadge, Avatar, Participant, StreamTile, Toast,
│   │                          # ToastStack, GoLiveModal, Stage, PermissionAlert, Spinner
│   └── pages/                 # HomePage, JoinPage, RoomPage, MobileRoomPage
├── lib/                       # lógica (sem UI)
│   ├── types.ts               # view-models compartilhados por ui/ e mocks/
│   ├── router.ts              # rotas /, /s/:id
│   ├── prefs.ts               # localStorage (apelido, goLive, volumes)
│   ├── room-id.ts             # gerar/validar ID de sala
│   ├── signaling.ts           # (fase real) cliente WebSocket + resume
│   ├── peer.ts / mesh.ts      # (fase real) RTCPeerConnection, perfect negotiation, malha
│   ├── capture.ts             # (fase real) getDisplayMedia
│   └── stores/                # (fase real) me, room, streams, toasts
├── mocks/
│   ├── MockApp.svelte         # lê ?mock & ?scene, renderiza a página com dados falsos
│   ├── MockSwitcher.svelte    # seletor de tela/cena (só no modo mock)
│   └── room.ts …              # cenas por tela
└── worker/                    # (fase real)
    ├── index.ts               # fetch: /api/rooms, /api/rooms/:c, /ws/:c, assets, cabeçalhos, rate limit
    ├── room.ts                # Durable Object Room
    ├── validate.ts            # validação de mensagens
    └── turn.ts                # gerar/revogar credenciais TURN
```

**Structure Decision**: Pacote único na raiz. `src/ui` + `src/mocks` + `src/lib` são o front;
`src/worker` é compilado pelo wrangler. O front em produção é servido pelo Worker como assets
estáticos (`assets.directory = dist`, SPA fallback).

## Complexity Tracking

Nenhuma violação.
