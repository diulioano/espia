---
description: "Tarefas da 002: som da transmissão sem vazar o Discord"
---

# Tasks: Som da transmissão sem vazar o Discord

**Input**: `/specs/002-audio-sem-discord/` (spec.md, plan.md, research.md, data-model.md, contracts/signaling-delta.md, quickstart.md)

**Tests**: incluídos. O plano e o quickstart pedem testes da regra de decisão (tabela do data-model) e do contrato do `live`.

**Organização**: por história. `[P]` = pode rodar em paralelo (arquivos diferentes, sem dependência pendente). Caminhos relativos à raiz.

## Phase 1: Setup

Nada a configurar: projeto, Vitest e Playwright de scratchpad já existem (001).

## Phase 2: Foundational (bloqueia todas as histórias)

**Objetivo**: o tipo de som (`AudioKind`) existe no protocolo, no Worker e na regra pura. Sem isso nenhuma história anuncia ou decide o som.

- [ ] T001 Adicionar `export type AudioKind = 'tab' | 'window' | 'none'` e trocar `LiveInfo.audio: boolean` → `audio: AudioKind` e a mensagem do cliente `{ t: 'live'; …; audio: AudioKind }` em `src/shared/protocol.ts`
- [ ] T002 Em `src/worker/validate.ts` (case `'live'`): aceitar `audio` em `['tab','window','none']`; aceitar também `boolean` legado e converter `true → 'tab'`, `false → 'none'`; qualquer outro valor → `null` (vira `error bad-message`). Comentar que o boolean é compatibilidade temporária (contracts/signaling-delta.md)
- [ ] T003 [P] Testes de contrato em `test/worker/room.test.ts` (describe "Som no live"): `audio: 'window'` → os outros recebem `peer-live` com `live.audio === 'window'`; `audio: true` → `'tab'`; `audio: false` → `'none'`; `audio: 'system'` → `error` com `code: 'bad-message'`. Ajustar o teste existente "live/unlive avisam os outros", que hoje manda `audio: true`, para `'tab'`
- [ ] T004 [P] Criar `src/lib/audio-source.ts` com `decideAudio(input: { wanted: boolean; surface: 'browser' | 'window' | 'monitor' | undefined; audioLabel: string | null }): { kind: AudioKind; drop: boolean; warn: boolean }`, seguindo a tabela do data-model.md. Regras: wanted=false → `none`, `drop` se veio faixa, `warn=false`; browser+faixa → `tab`; window+faixa com rótulo que não indica sistema → `window`; window+rótulo de sistema (regex `/system/i`) → `none, drop, warn`; monitor → `none`, `drop` se veio faixa, `warn=true`; surface indefinida com faixa → `none, drop`; browser/window sem faixa → `none, warn=false`. Exportar também `audioLabelText(kind, reason?: 'monitor')` com "com som da aba" / "com som da janela" / "sem som" / "sem som (tela inteira)"
- [ ] T005 [P] Testes da tabela inteira do data-model.md em `test/lib/audio-source.test.ts` (uma linha da tabela = um caso) e de `audioLabelText`
- [ ] T006 Rodar `npm run check` e `npm test`: tudo verde antes de seguir (T001 quebra os tipos de quem usa `live.audio`; corrigir os pontos que o typecheck apontar em `src/lib/stores/room.svelte.ts` e `src/mocks/room.ts` com o mínimo, `audio ? 'tab' : 'none'`, até as histórias refinarem)

**Checkpoint**: protocolo novo no ar, regra testada, nada visível mudou ainda.

## Phase 3: User Story 1 — Janela leva só o som dela (P1) 🎯 MVP

**Goal**: transmitir a janela do jogo leva o som do jogo e nada do Discord; aba continua com o som da aba.

**Independent Test**: quickstart M1 e M2 (Discord tocando; transmitir janela e aba; do outro lado só o som escolhido).

- [ ] T007 [US1] Em `src/lib/capture.ts` (`startCapture`): trocar `systemAudio: 'include'` → `systemAudio: 'exclude'` e adicionar `windowAudio: 'window'` às opções de `getDisplayMedia`, com comentário apontando research.md R2/R3
- [ ] T008 [US1] Em `src/lib/capture.ts`: nova função `applyAudioRule(stream: MediaStream, wanted: boolean): { kind: AudioKind; warn: boolean; reason?: 'monitor' }` que lê `stream.getVideoTracks()[0].getSettings().displaySurface` e a faixa de áudio (`label`), chama `decideAudio`, e quando `drop` faz `track.stop()` + `stream.removeTrack(track)` **antes** de retornar. Logar `console.debug('[espia] áudio:', surface, label, kind)` (quickstart T-Q1)
- [ ] T009 [US1] Em `src/lib/stores/room.svelte.ts` (`startBroadcast`): chamar `applyAudioRule(stream, prefs.audio)` logo depois de `startCapture` e **antes** de `mesh.setLocal`; guardar `kind` em `myBroadcast.audioKind` (adicionar `audioKind: AudioKind` e `audioReason?: 'monitor'` em `MyBroadcastView` de `src/lib/types.ts`); `sendLive()` passa a mandar `audio: myBroadcast.audioKind`
- [ ] T010 [US1] Em `src/lib/stores/room.svelte.ts`: reavaliar a regra quando a faixa de vídeo disparar `configurationchange` ou o stream disparar `addtrack`/`removetrack` (FR-205, research R4). Reaplicar `applyAudioRule`; se a faixa de áudio mudou, chamar `mesh.setLocal(stream, quality)` de novo; se o `kind` mudou, atualizar `myBroadcast.audioKind` e reanunciar com `sendLive()`

**Checkpoint**: M1/M2 do quickstart passam (janela e aba com o som certo, sem Discord).

## Phase 4: User Story 2 — Tela inteira sem som, explicado (P1)

**Goal**: tela inteira nunca manda som; o modal explica antes e um aviso explica depois.

**Independent Test**: quickstart M3 (tela inteira com Discord tocando → nenhum som; card "sem som (tela inteira)" + toast com a dica).

- [ ] T011 [US2] Em `src/lib/stores/room.svelte.ts`: quando `applyAudioRule` devolver `warn` (no início ou numa reavaliação do T010), mostrar um toast `kind: 'live'` (não `'error'`, que fica até clicar; o FR-208 pede que suma sozinho) com o texto exato "Tela inteira vai sem som. Pra levar o som, transmita a janela do jogo ou uma aba." (FR-208); no máximo uma vez por transmissão (flag zerada em `stopBroadcast`)
- [ ] T012 [P] [US2] Em `src/ui/components/GoLiveModal.svelte`: trocar a explicação de "Compartilhar áudio" ("O som do jogo ou vídeo vai junto. Funciona melhor com uma aba.") por "Vai o som da janela ou aba que você escolher — Discord e outros apps ficam de fora. Tela inteira vai sem som." (FR-206)
- [ ] T013 [P] [US2] Em `src/mocks/room.ts`: adicionar a cena `livescreen` ("Ao vivo · tela inteira sem som"), com `myBroadcast.audioKind: 'none'`, `audioReason: 'monitor'` e o toast do T011 visível; ajustar a cena `live` existente para `audioKind: 'window'` (constituição V: estados mockáveis)

**Checkpoint**: M3 passa; o modal mostra o texto novo; `?mock=room&scene=livescreen` mostra o estado.

## Phase 5: User Story 3 — Saber o que está indo de som (P2)

**Goal**: rótulos de som no card de quem transmite e no subtítulo de quem assiste batem com o que sai.

**Independent Test**: quickstart M1–M5, conferindo os rótulos em cada caso.

- [ ] T014 [P] [US3] Em `src/ui/pages/RoomPage.svelte` (card "Você está ao vivo"): trocar `{b.audio ? 'com áudio' : 'sem áudio'}` por `audioLabelText(b.audioKind, b.audioReason)` (FR-207); conferir se `src/ui/pages/MobileRoomPage.svelte` mostra o mesmo texto em algum lugar e alinhar
- [ ] T015 [US3] Em `src/lib/stores/room.svelte.ts` (`buildView`, montagem de `streams`): `quality` passa a ser `` `${res}p · ${fps} fps · ${live.audio === 'none' ? 'sem som' : 'com som'}` `` (FR-209)
- [ ] T016 [P] [US3] Em `src/mocks/room.ts`: atualizar os `quality` das transmissões mockadas para incluir "· com som" / "· sem som", para o mock bater com a sala real

**Checkpoint**: rótulos corretos em todos os casos M1–M5.

## Phase 6: Polish & validação

- [ ] T017 E2E no scratchpad (Playwright + Edge, **só abas de teste** via `--auto-select-tab-capture-source-by-title`, conferindo a cor do vídeo antes de qualquer screenshot; nunca `--use-fake-ui-for-media-stream`): (a) aba com som → quem assiste recebe faixa de áudio e subtítulo "· com som", card "com som da aba"; (b) modal com som desligado → nenhuma faixa de áudio chega, "sem som"
- [ ] T018 `npm run check`, `npm test`, `npx vite build` verdes; `npm run deploy`; `npm run smoke https://espia.espia.workers.dev`
- [ ] T019 Teste manual com a galera, roteiro M1–M5 de `specs/002-audio-sem-discord/quickstart.md`, com o Discord ligado; registrar o rótulo do som de janela (T-Q1) em `specs/002-audio-sem-discord/research.md` R3 e ajustar a regex do T004 se precisar
- [ ] T020 [P] Atualizar `specs/001-espia-screen-share/spec.md` (FR-010/FR-013 e o edge case de áudio da linha 140) e `DEFINICOES.md` (linha "Áudio do sistema/aba junto com a tela") apontando para a 002

## Dependencies & Execution Order

- **Phase 2** (T001–T006) bloqueia tudo. T003, T004 e T005 em paralelo depois do T001 (T003 depende também do T002).
- **US1** (T007–T010) depende só da Phase 2. É o MVP: já elimina o vazamento em janela e aba.
- **US2** (T011–T013) depende do T008/T009 (usa `warn`/`audioReason`). O T007 sozinho já impede o som na tela inteira.
- **US3** (T014–T016) depende do T009 (`audioKind` no view-model).
- **Polish** depois das histórias. O T019 precisa do deploy (T018).

## Parallel Example

```text
Depois do T001/T002:
  T003 testes de contrato (test/worker/room.test.ts)
  T004 regra pura (src/lib/audio-source.ts)
  T005 testes da regra (test/lib/audio-source.test.ts)
US2 + US3 juntos (arquivos diferentes):
  T012 GoLiveModal · T013/T016 mocks · T014 RoomPage
```

## Implementation Strategy

1. **MVP = Phase 2 + US1 (T001–T010)**: o Discord para de vazar em janela e aba, e a tela inteira já vai sem som pelo T007. Dá para fazer deploy aqui se precisar de correção urgente.
2. **+ US2**: explica a tela inteira (texto e aviso) e fecha a rede de segurança.
3. **+ US3**: rótulos para quem transmite e quem assiste.
4. **Polish**: e2e, deploy e teste com a galera (M1–M5), que é o que de fato valida o SC-201 (zero vozes do Discord).
