# Tasks: Mostrar o que cada pessoa está transmitindo

**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/signaling-delta.md](./contracts/signaling-delta.md), [quickstart.md](./quickstart.md)

**Testes**: função pura `liveLabel` e Worker (validação/repasse/vazamento), como o plano pede. Captura real só manual.

## Phase 1: Foundational

- [X] T1201 `src/shared/protocol.ts`: `export const LIVE_LABEL_MAX = 60`; `LiveInfo.label?: string` (comentário: specs/012, nunca o título da janela); `ClientMessage` `live` ganha `label?: string`.
- [X] T1202 `src/worker/validate.ts`: no `case 'live'`, aceitar a chave `label` em `hasOnly`. `cleanLiveLabel(raw)` usa a mesma limpeza do `cleanNickname` (NFC, remove `UNSAFE`, junta espaços, trim) com 1–`LIVE_LABEL_MAX` caracteres. Inválido ou ausente → a mensagem `live` sai **sem** `label` (nunca null por causa dele).
- [X] T1203 `src/worker/room.ts`: no `case 'live'`, montar `LiveInfo` com `...(msg.label ? { label: msg.label } : {})`. Confirmar que `preview()` e `summary()` continuam sem o `label`.
- [X] T1204 Testes em `test/worker/room.test.ts`, bloco `describe('Descrição da transmissão (specs/012)')`:
  - `live` com `label: 'Hollow Knight'` → `peer-live.live.label` pros outros, e no `welcome` de quem entra depois;
  - `'a‮b'` vira `'ab'`;
  - 61 caracteres ou número → `live` aceito sem `label`;
  - sem `label` (cliente antigo) continua funcionando;
  - prévia e `status` sem o texto.

## Phase 2: User Story 1 — Ver o que cada um está mostrando (P1) 🎯

- [X] T1205 [P] [US1] Criar `src/lib/live-label.ts` com `liveLabel(input)`, exatamente a tabela de `data-model.md`. Corta em 60 com "…".
- [X] T1206 [P] [US1] `test/lib/live-label.test.ts`: todas as linhas da tabela.
- [X] T1207 [US1] `src/lib/types.ts`: `StreamView.label?`, `ParticipantView.liveLabel?`, `MyBroadcastView.label?`.
- [X] T1208 [US1] `src/ui/components/StreamTile.svelte`: no chip do nome, depois do nome, ` · {label}` em texto 65% com reticências (tile e miniatura).
- [X] T1209 [US1] `src/ui/components/Participant.svelte`: a linha de baixo vira "transmitindo · {label}" / "transmitindo pra sala · {label}", cortada com reticências.
- [X] T1210 [US1] `src/ui/pages/MobileRoomPage.svelte`: chips "Ao vivo agora" com o `label` numa segunda linha (12px, 65%, cortada).
- [X] T1211 [US1] Mocks:
  - `src/mocks/room.ts`: `STREAMS` com `label` (Lu "Hollow Knight", Bia "Aba do navegador", Tomás "Visual Studio Code");
  - um da `crowd` com label comprido ("Microsoft Flight Simulator 2024 — Edição Premium Deluxe" → cortado);
  - um sem label (versão antiga);
  - participantes ao vivo com `liveLabel`;
  - `myBroadcast.label` nas cenas `live*`;
  - `src/mocks/mobile.ts` herda.
- [X] T1212 [US1] `src/lib/stores/room.svelte.ts`:
  - `buildView()`: `StreamView.label = p.live.label` e `ParticipantView.liveLabel = p.live?.label` (pra mim, `myBroadcast.label`);
  - `sendLive()` manda `label: b.label`.
- [X] T1213 [US1] `src/lib/stores/room.svelte.ts`, calcular o `label` ao começar:
  - site: `liveLabel({ kind: 'browser', surface: video.getSettings().displaySurface })`;
  - app: janela → `{ kind: 'native-window', app: source.app }`; tela → `{ kind: 'native-monitor', title: source.title, monitors: sources.filter(monitor).length }`;
  - câmera: `{ kind: 'camera', title, facing }`.
  - No `configurationchange` do site: recalcular; se mudou, atualizar `myBroadcast.label` e `sendLive()`.

## Phase 3: User Story 2 — Eu vejo o que estou mostrando (P2)

- [X] T1214 [US2] `src/ui/pages/RoomPage.svelte`: o cartão "Você está ao vivo" mostra `b.label` (quando houver) no começo da linha de estatísticas, no lugar do `· câmera {sourceTitle}`.

## Phase 4: Polish

- [X] T1215 `npm run check`, `npm test`, `npm run build`. Prints dos mocks `room` grid/focus/single/live/livecam/crowd e `mobile` watching.
- [X] T1216 `specs/001-espia-screen-share/contracts/signaling.md`: link para o delta da 012.
- [X] T1217 Commit "012: mostrar o que cada pessoa está transmitindo"; publicar (`! npm run deploy`, pela pessoa usuária) e fazer o roteiro manual do quickstart (site, app, câmera).

## Dependencies

T1201 → T1202–T1204 (Worker) e T1207 (tipos) → telas e mocks (T1208–T1211) → store (T1212–T1213) → T1214 → Polish. T1205/T1206 são independentes.

## Publicação

- 2026-10-30: commit `54afd3c`, versão `b6a164a7`. Falta o roteiro manual com captura real (site, app, câmera).
