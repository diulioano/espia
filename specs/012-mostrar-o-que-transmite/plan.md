# Implementation Plan: Mostrar o que cada pessoa está transmitindo

**Branch**: `012-mostrar-o-que-transmite` (trabalho direto na `main`, como nas anteriores) | **Date**: 2026-10-30 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/012-mostrar-o-que-transmite/spec.md`

## Summary

Um texto curto (≤ 60 caracteres) acompanha o "ao vivo" de cada pessoa: o **nome do programa** no app ("Hollow Knight"), o **tipo** no site ("Tela inteira", "Janela", "Aba do navegador") ou a **câmera** ("Câmera · C920"). Ele é calculado por uma função pura no navegador de quem transmite (research R1) e vai num campo opcional `label` da mensagem `live`, que o servidor já repassa a todos e guarda só enquanto a pessoa está ao vivo (R2). Quem assiste vê o texto no cartão da transmissão, nas miniaturas, na lista "Na sala" e nos chips do celular. Quem transmite vê o mesmo texto no "Você está ao vivo" (R4). O título da janela nunca sai do aparelho (Q1: A). Não precisa de app novo nem de API nativa.

## Technical Context

**Language/Version**: TypeScript 6, Svelte 5, Cloudflare Workers (compat `2026-10-01`)

**Primary Dependencies**: as de hoje. Nenhuma nova.

**Storage**: nada novo. O `label` vive dentro do `live` da pessoa (attachment + `roster` da 011), que já é apagado quando ela para ou sai.

**Testing**: Vitest (função pura `liveLabel`), Vitest + `@cloudflare/vitest-plugin` (validação e repasse do `label`), mocks e roteiro manual de captura real ([quickstart.md](./quickstart.md)).

**Target Platform**: site (Chrome/Edge/celular) e app desktop 0.1.7 sem mudança.

**Project Type**: SPA + Worker

**Performance Goals**: SC-001: o texto aparece em até 2 s (vai junto do `live` que já existe).

**Constraints**:
- nunca o título da janela (FR-002);
- nada na prévia nem em Suas salas (FR-010);
- compatível com clientes antigos nos dois sentidos.

**Scale/Scope**:
- **compartilhado**: `protocol.ts` (+`label`, `LIVE_LABEL_MAX`);
- **Worker**: `validate.ts` (limpar `label`), `room.ts` (pôr no `LiveInfo`);
- ★ `src/lib/live-label.ts`;
- `room.svelte.ts`: calcular nos três caminhos (site, app, câmera), reenviar quando o site troca de superfície, montar as views;
- `types.ts`;
- **telas**: `StreamTile`, `Participant`, `RoomPage` ("Você está ao vivo"), `MobileRoomPage` (chips);
- **mocks**: `room.ts`, `mobile.ts`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio / restrição | Situação |
|---|---|
| I. Simplicidade | ✅ Nada novo pra fazer; nenhum campo a mais no seletor (Q2: A, Q3: A). |
| II. Sem servidor de mídia | ✅ Só um texto na sinalização. |
| III. Sem contas e nada sobre as pessoas (2.0.0) | ✅ O texto vive só enquanto a pessoa está ao vivo, dentro da sala, e some com ela. Não vai pra prévia nem pra Suas salas. Nunca o título da janela. |
| IV. Camada visual separada | ✅ O texto é decidido em `src/lib` (`liveLabel`) e chega pronto nas views. |
| V. Telas mockáveis | ✅ As cenas existentes ganham o texto, incluindo um comprido e um ausente. |
| VI. Segredos fora do navegador | ✅ Nada de segredo. O servidor limpa o texto como o apelido (security A4). |
| Restrição do app desktop | ✅ Sem API nativa nova: o nome do programa já chega à página pela ponte de captura. |

Sem violações. **Depois do design**: igual. ✅

## Project Structure

### Documentation (this feature)

```text
specs/012-mostrar-o-que-transmite/
├── spec.md
├── plan.md               # este arquivo
├── research.md           # R1–R4
├── data-model.md         # LiveInfo.label, liveLabel(), view-models
├── quickstart.md
├── contracts/signaling-delta.md
├── checklists/requirements.md
└── tasks.md              # /speckit-tasks
```

### Source Code (repository root)

```text
src/shared/protocol.ts          # LiveInfo.label?, live.label?, LIVE_LABEL_MAX
src/worker/validate.ts          # cleanLiveLabel (mesma limpeza do apelido, ≤ 60)
src/worker/room.ts              # live → LiveInfo com label
src/lib/live-label.ts           # ★ liveLabel(input) puro
src/lib/stores/room.svelte.ts   # label nos 3 caminhos + configurationchange + views
src/lib/types.ts                # StreamView.label, ParticipantView.liveLabel, MyBroadcastView.label
src/ui/components/StreamTile.svelte, Participant.svelte
src/ui/pages/RoomPage.svelte, MobileRoomPage.svelte
src/mocks/room.ts, mobile.ts
test/lib/live-label.test.ts     # ★
test/worker/room.test.ts        # + bloco 012
```

**Structure Decision**: o mesmo projeto SPA + Worker. Só um arquivo novo de lógica (`live-label.ts`).

## Ordem de entrega

1. Protocolo + servidor (validação, repasse) + testes do Worker.
2. `liveLabel` + testes.
3. Mocks com o texto em todas as cenas, depois as telas (tile, lista, chips, "Você está ao vivo").
4. Store real: calcular nos três caminhos e reenviar na troca de superfície.
5. Roteiro manual de captura real (site, app, câmera), depois publicar (`npm run deploy`; o app 0.1.7 pega sozinho).

## Complexity Tracking

Sem violações da constituição.
