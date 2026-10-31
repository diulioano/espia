# Implementation Plan: Assistir quando quiser, telas que dormem e remover pessoas

**Branch**: `014-assistir-e-remover` (trabalho direto na `main`, como na 004–013) | **Date**: 2026-10-31 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/014-assistir-e-remover/spec.md`

## Summary

1. **Uma regra só para o que cada um recebe (R1, R5)**: a função pura `viewModes()` decide `full` / `thumb` / `hidden` por tela a partir de assistir/não assistir, página visível, PiP, celular e layout. O `hidden` da 010 já pausa o envio daquele par e mantém o som, então US1 e US2 economizam dos dois lados sem mexer no WebRTC.
2. **US1**: `unwatched` por transmissão (`startedAt`) na store. O tile vira o cartão "Assistir" e não monta `<video>`. "N assistindo" passa a contar quem me vê com modo ≠ `hidden`.
3. **US2**: `visibilitychange` (nunca `blur`). No app, um **spike** confere se minimizar ou ir pra bandeja deixa a página `hidden`. Se não deixar, o `engine.rs` passa a ligar `SetIsVisible(false)` no WebView2 (só Rust, app 0.1.8, sem emenda) (R2).
4. **US3**: prévia = `ImageCapture.grabFrame()` a cada 5 s → `ImageBitmap` de 320 px num `<canvas>`. Clicar põe a própria transmissão no palco como uma tela "Você", muda e fora das mensagens (R6).
5. **US4**: `POST /api/rooms/:code/remove` com a chave do dono, mesmo caminho de renomear/apagar. O DO fecha a pessoa com `removed` + 4403, sem fantasma, e o cliente não reconecta (R7, [contracts/remove-person.md](./contracts/remove-person.md)).

## Technical Context

**Language/Version**: TypeScript 6, Svelte 5 (runes), Cloudflare Workers; Rust (Tauri 2) só se o spike R2 pedir.

**Primary Dependencies**: as de hoje. Nenhuma nova.

**Storage**: nada novo (ver [data-model.md](./data-model.md)).

**Testing**:
- Vitest (`src/lib`, `src/shared`);
- `@cloudflare/vitest-plugin` (Worker);
- e2e scratchpad com Playwright + Edge;
- roteiro manual em [quickstart.md](./quickstart.md).

**Target Platform**: o site (desktop e celular) e o app 0.1.7. Se o spike pedir, o app 0.1.8 vem com a mudança no `engine.rs`.

**Project Type**: SPA + Worker (+ app Tauri), no mesmo repositório.

**Performance Goals**:
- SC-001: com o Espia minimizado, o FPS do jogo não cai por causa de telas recebidas;
- SC-002/003: pausar e voltar em ≤ 3 s;
- SC-005: prévia com ≤ 1 quadro a cada 3 s.

**Constraints**:
- som nunca cortado (SC-004);
- `blur` não esconde (FR-007);
- nada guardado sobre quem foi removido (princípio III);
- cliente e app antigos continuam funcionando.

**Scale/Scope**:
- **Lógica**:
  - `room-view.ts` ★`viewModes()`;
  - `room.svelte.ts` (`unwatched`, `pageVisible`, `pipId`, `selfOnStage`, `snapshot`, remover);
  - ★`snapshot.ts` (`grabFrame` + fallback);
  - `signaling.ts` (fim `'removed'`, limpa sessão);
  - `api.ts` (`removePerson`);
  - `mesh.ts`/`peer.ts` (`theirViews` para "assistindo");
  - `types.ts`.
- **Telas**:
  - `StreamTile` (cartão "Assistir", botão "Parar de assistir", modo `still` em canvas, eventos de PiP);
  - `Stage` (tela "Você");
  - `RoomPage` (prévia clicável);
  - `MobileRoomPage`;
  - `Participant` (menu "Remover da sala");
  - ★`RemoveDialog.svelte`;
  - `HomePage`/`HomeRoute` (aviso de removido).
- **Worker**: `index.ts` (rota `/remove`), `room.ts` (`removePerson`), `validate.ts` (`peerId`), `protocol.ts` (`removed`, `CLOSE.removed`).
- **Mocks**: `room` +5 cenas, `home` +1, `mobile` +1.
- **App (condicional)**: `engine.rs` `SetIsVisible`.

## Constitution Check

| Princípio / restrição | Situação |
|---|---|
| I. Simplicidade | ✅ Assistir continua automático (0 passos a mais). Os controles novos são opcionais. Textos informais. |
| II. Sem servidor de mídia | ✅ Só muda quem recebe o quê, ainda P2P. |
| III. Sem contas e nada sobre as pessoas | ✅ Remover não guarda nada; a escolha de assistir fica só na memória da página; a prova de dono é a da 011. |
| IV. Camada visual separada | ✅ A regra (`viewModes`, `snapshot`, remoção) fica em `src/lib`. Os tiles recebem `watching`, `you`, `snapshot` e emitem `watch`/`pip`/`toggleSelfStage`. |
| V. Telas mockáveis | ✅ 7 cenas novas (FR-018). |
| VI. Segredos / limite | ✅ `/remove` usa `RL_CREATE` e o Bearer do dono, e o servidor compara só o hash. |
| App: só as APIs listadas | ✅ Nenhum comando novo para a página. `SetIsVisible` é interno do Rust, como o `MemoryUsageTargetLevel` da 009. |

**Gate**: passa. Sem emenda.

## Project Structure

### Documentation (this feature)

```text
specs/014-assistir-e-remover/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/remove-person.md
├── checklists/requirements.md
└── tasks.md            # /speckit-tasks
```

### Source Code (repository root)

```text
src/
├── shared/protocol.ts            # removed, CLOSE.removed
├── worker/{index,room,validate}.ts
├── lib/
│   ├── room-view.ts              # ★ viewModes()
│   ├── snapshot.ts               # ★ quadro parado da própria transmissão
│   ├── signaling.ts · api.ts · mesh.ts · peer.ts · types.ts
│   └── stores/room.svelte.ts
├── ui/
│   ├── components/{StreamTile,Stage,Participant}.svelte · ★ RemoveDialog.svelte
│   └── pages/{RoomPage,MobileRoomPage,HomePage}.svelte
├── routes/{RoomRoute,HomeRoute}.svelte
└── mocks/{room,home,mobile}.ts
src-tauri/src/engine.rs           # só se o spike R2 pedir
test/                             # room-view, store, worker room
```

**Structure Decision**: a mesma estrutura de sempre. Um arquivo novo de lógica (`snapshot.ts`) e um componente novo (`RemoveDialog`).

## Ordem sugerida

1. **US1 + US2 juntos na regra** (`viewModes` + testes), depois as telas da US1. O spike do app (M1) roda em paralelo, porque decide se há app 0.1.8.
2. **US3** (prévia + palco).
3. **US4** (Worker primeiro, com testes; depois o cliente e as telas).
4. **Publicação**: o site sozinho já entrega tudo no navegador. O app 0.1.8 só entra se o spike pedir (publicar o site antes, como sempre).

## Complexity Tracking

Nenhuma violação.
