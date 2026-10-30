# Implementation Plan: Salas fixas e chamada que sobrevive a quedas

**Branch**: `011-salas-fixas` (trabalho direto na `main`, como na 004–010) | **Date**: 2026-10-30 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/011-salas-fixas/spec.md`

## Summary

1. **Causa da queda de 29/10 (R1)**: uma publicação reinicia o Durable Object da sala e corta os WebSockets **sem rodar `webSocketClose`**. Como quem está na sala só fica guardado no attachment do socket, a sala reinicia "vazia", diz que não existe, e ninguém consegue voltar. Correção: guardar a lista de quem está na sala (`roster`) no storage do DO. Ao reiniciar, quem não tem socket vira "fantasma" e retoma a mesma identidade. Com isso, a US1 deixa de ser só tela.
2. **Queda não é sair (US1, 🎯 MVP)**: tolerância por pessoa 20 → 60 s. Sala rápida segurada 5 min quando todos caem (R2). Na tela, aviso "Reconectando…", lista esmaecida, "De volta!" e a tela de falha depois de 60 s, que some sozinha se a ligação voltar (R8).
3. **Sala fixa (US2)**: no storage do próprio DO (`fixed`), com expiração por alarme 30 dias depois do último `join` (R3). Criar devolve `ownerKey` uma vez. O servidor guarda só o hash (R4).
4. **Suas salas (US3)**: lista em `localStorage`, no máximo 5 (pedido de 30/10, R9), e uma consulta `POST /api/rooms/status` que devolve só números (R7).
5. **Dono (US4)**: `PATCH`/`DELETE /api/rooms/:code` com a chave. Renomear avisa a sala ao vivo. Apagar expulsa com 4410 e deixa uma lápide sem nome por 30 dias, para mostrar N1/N3 (R5, R6).
6. **Constituição 2.0.0**: o princípio III muda (a sala fixa guarda dados no servidor), então é uma versão MAJOR.

## Technical Context

**Language/Version**: TypeScript 6, Svelte 5 (runes), Cloudflare Workers (compat `2026-10-01`)

**Primary Dependencies**: as de hoje. Durable Objects com storage SQLite, alarmes, Hibernation WebSocket API, Rate Limiting binding. Nenhuma dependência nova.

**Storage**: storage do DO (chaves novas `roster`, `holdUntil`, `fixed`, `tombstone`, ver [data-model.md](./data-model.md)). No navegador, `localStorage` (`espia:rooms`, `espia:owner:<code>`, ver [contracts/local-storage.md](./contracts/local-storage.md)).

**Testing**:
- Vitest + `@cloudflare/vitest-plugin` 1.4.10. Usa `evictDurableObject(stub, { webSockets: 'close' })` para simular a publicação, e `runDurableObjectAlarm` / relógio para os 30 dias.
- Vitest puro para `src/lib` e `src/shared`.
- Mocks e roteiro ponta a ponta em [quickstart.md](./quickstart.md).

**Target Platform**: o site (Chrome/Edge para transmitir, qualquer navegador moderno para assistir, inclusive celular) e o app 0.1.7 sem mudança.

**Project Type**: SPA + Worker, no mesmo repositório

**Performance Goals**:
- SC-001: todos voltam em até 30 s depois de uma publicação;
- SC-005: Suas salas com números em até 2 s;
- SC-008: criar sala fixa em menos de 15 s.

**Constraints**:
- nenhum apelido sai do servidor por `status`;
- nenhum histórico de pessoas depois que a sala esvazia;
- compatível com abas abertas na versão antiga e com o app 0.1.7 (R11);
- sem API nativa nova.

**Scale/Scope**:
- **Worker** (~+250 linhas):
  - `room.ts`: `roster`, reinício, `holdUntil`, `fixed`, lápide, `summary`/`rename`/`remove`;
  - `index.ts`: rotas novas;
  - `validate.ts`: nome e lista de códigos.
- **Compartilhado**: `protocol.ts` (+mensagens, `CLOSE.deleted`, `RoomStatus`), ★ `room-name.ts`.
- **Lógica** (`src/lib`):
  - `api.ts`: criar fixa, status, renomear, apagar;
  - ★ `saved-rooms.ts`: lista, limite, ordem, grupos e textos;
  - `prefs.ts`: chaves de dono;
  - `signaling.ts`: `'deleted'`, `retryNow`, aviso de volta;
  - `room.svelte.ts`: `link`, `downSince`, nome e dono, ações;
  - `room-view.ts`: cabeçalho com nome;
  - `types.ts`.
- **Telas** (`src/ui`):
  - ★ `SavedRoom.svelte`, ★ `SavedRoomsPanel.svelte`, ★ `NameDialog.svelte` (criar/renomear), ★ `DeleteRoomDialog.svelte`, ★ `RoomMenu.svelte` (popover "⋯"), ★ `ReconnectPill.svelte`, ★ `ReconnectFail.svelte`, ★ `NotFoundPage.svelte`;
  - mudam `HomePage`, `JoinPage` (selo "Sala fixa"; o "não existe" passa para a `NotFoundPage`), `RoomPage`, `MobileRoomPage` e `Participant`/lista (esmaecer).
- **Rotas**: `HomeRoute` (lista, status, diálogos) e `RoomRoute` (fim `deleted`, `NotFoundView`, lembrar a sala).
- **Mocks**: `home` +6 cenas, `join` +1, `room` +9, ★ `notfound` (4 × saved), `mobile` +2.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio / restrição | Situação |
|---|---|
| I. Simplicidade (2 passos, sem conta) | ✅ Sala rápida continua igual. Sala fixa sem conta: o dono é uma chave no aparelho. Entrar por Suas salas tem menos passos que hoje. |
| II. Sem servidor de mídia | ✅ Nada muda na mídia. Continua P2P. |
| III. Efêmero e sem contas | ❌ **Violado**: a sala fixa guarda nome, último uso e hash da chave no servidor, e o navegador passa a guardar a lista de salas e as chaves (fora da lista "apelido, qualidade, volume"). → **Emenda 2.0.0** abaixo. |
| IV. Camada visual separada | ✅ Componentes novos recebem view-models (`SavedRoomView`, `NotFoundView`, `link`). A lógica fica em `src/lib`. Cores só de `tokens.css` (os valores do design já são tokens ou `rgba` do tema, como nas telas de hoje). |
| V. Telas mockáveis | ✅ As 23 cenas do design ganham mock (FR-034). |
| VI. Segredos fora do navegador / CSPRNG / limite | ✅ `ownerKey` e códigos com CSPRNG no servidor. O servidor guarda só o hash. `status` conta no limite de consultas, e criar/renomear/apagar no de criação. A `ownerKey` é um segredo **do aparelho**, não do servidor (igual ao `resumeToken` de hoje). |
| Restrição do app desktop (só as APIs listadas) | ✅ Nenhuma API nativa nova. |

### Emenda proposta — 2.0.0 (MAJOR: redefine o princípio III)

> ### III. Sem contas e nada sobre as pessoas
> Não existem contas, login nem perfis. O servidor pode guardar **a sala**: código, nome de uma sala
> fixa, último uso e o hash da chave do dono. Nada disso identifica uma pessoa. O servidor MUST NOT guardar apelidos, quem
> entrou nem quando, depois que a sala esvazia. Uma sala rápida existe enquanto houver alguém nela, ou
> por até 5 min depois que todos caíram. Uma sala fixa some depois de 30 dias sem uso ou quando o dono apaga.
> No navegador guardamos só preferências locais (apelido, qualidade, volume por pessoa), as salas fixas
> em que se entrou naquele aparelho e as chaves de dono das que ele criou, em `localStorage`.

É feita na primeira tarefa, antes do código (como na 1.4.0/1.5.0). O `DEFINICOES.md` (linha "Vida da sala") e a `security.md` (dados guardados e ameaça "roubo da chave do dono") são atualizados junto.

**Depois do design (Phase 1)**: os contratos não criam violação nova. A lápide guarda só motivo e prazo, e `status` não leva apelidos. ✅

## Project Structure

### Documentation (this feature)

```text
specs/011-salas-fixas/
├── spec.md
├── plan.md                    # este arquivo
├── research.md                # R1–R11
├── data-model.md              # storage do DO, localStorage, view-models
├── quickstart.md              # testes, mocks, roteiro ponta a ponta
├── contracts/
│   ├── http-api-delta.md      # POST fixa, status, PATCH, DELETE, 410
│   ├── signaling-delta.md     # welcome.room, room-renamed, room-deleted, 4410, tempos
│   └── local-storage.md       # espia:rooms, espia:owner:<code>
├── checklists/requirements.md
└── tasks.md                   # /speckit-tasks
```

### Source Code (repository root)

```text
src/
├── shared/
│   ├── protocol.ts            # + RoomInfo, room-renamed/deleted, CLOSE.deleted, RoomStatus
│   └── room-name.ts           # ★ normalizeRoomName (navegador e Worker)
├── worker/
│   ├── index.ts               # + rotas status / PATCH / DELETE, POST com kind
│   ├── room.ts                # + roster, reinício, holdUntil, fixed, lápide, summary/rename/remove
│   └── validate.ts            # + nome, lista de códigos
├── lib/
│   ├── api.ts                 # + createFixedRoom, fetchStatuses, renameRoom, deleteRoom
│   ├── saved-rooms.ts         # ★ lista (limite 5), ordem, grupos, textos
│   ├── prefs.ts               # + chaves de dono
│   ├── signaling.ts           # + 'deleted', retryNow, onBack
│   ├── room-view.ts           # cabeçalho com nome da sala fixa
│   ├── stores/room.svelte.ts  # + link/downSince, room, ações do dono
│   └── types.ts
├── routes/HomeRoute.svelte, RoomRoute.svelte
├── ui/components/             # ★ SavedRoom, SavedRoomsPanel, NameDialog, DeleteRoomDialog, RoomMenu, ReconnectPill, ReconnectFail
├── ui/pages/                  # ★ NotFoundPage; HomePage, JoinPage, RoomPage, MobileRoomPage
└── mocks/                     # home, join, room, mobile + ★ notfound
test/
├── worker/room.test.ts        # + publicação, hold, fixa, status, dono
└── lib/                       # ★ room-name, saved-rooms; signaling
wrangler.jsonc, vitest.config.ts  # GRACE_MS 60000, ★ HOLD_MS 300000 (testes: curtos)
```

**Structure Decision**: o mesmo projeto único SPA + Worker de sempre. Nada de pasta nova além de `src/shared/room-name.ts` e `src/lib/saved-rooms.ts`.

## Ordem de entrega

1. Emenda 2.0.0 + `DEFINICOES.md`/`security.md`.
2. **US1**, servidor: `roster` + reinício + `holdUntil` + `GRACE_MS` 60 s, com os testes de publicação. *Isso sozinho já resolve a queda de 29/10 e pode ir pro ar antes do resto.*
3. **US1**, tela: mocks `room` reconnecting/back/reconnectfail e `mobile` reconnecting, depois a lógica `link` no store.
4. **US2**: protocolo/API da sala fixa, depois mocks `home` create, `join` fixed, `room` fixed/created/guest, `notfound` expired/quick, depois tudo real.
5. **US3**: `saved-rooms.ts` + `status`, depois mocks `home` first/few/many/loading/gone e `mobile` home, depois tudo real.
6. **US4**: `PATCH`/`DELETE`, depois mocks `room` ownermenu/rename/delete, `home` menu e `notfound` deleted/kicked, depois tudo real.
7. Roteiro do quickstart e publicação (avisar a galera: a primeira publicação ainda derruba as salas abertas, R11).

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Princípio III (dados da sala guardados no servidor e no navegador) | Link que não muda (US2) e Suas salas (US3), pedidos do grupo em 29/10 | Sala só em memória não sobrevive vazia nem a publicações. Guardar a lista de salas só no navegador sem nada no servidor não deixa o link valer com a sala vazia. |
