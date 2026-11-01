# Tasks: Assistir quando quiser, telas que dormem e remover pessoas

**Input**: [spec.md](./spec.md), [plan.md](./plan.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/remove-person.md](./contracts/remove-person.md), [quickstart.md](./quickstart.md)

**Tests**: o projeto já testa a lógica pura e o Worker. Aqui entram testes de `viewModes()`, da store e de `removePerson`. As telas são conferidas pelos mocks.

Gate de toda fase: `npm run check`, `npm test`, `npm run build` e os mocks da fase abrindo.

## Phase 1: Setup

- [X] T1401 Spike R2 (quickstart M1): rodar o app de debug com `ESPIA_BROWSER_ARGS=--remote-debugging-port=9222`, entrar numa sala e ler `document.visibilityState` pelo CDP (scratchpad `app-window.mjs`) em três estados: janela visível, minimizada e escondida na bandeja (fechar no X). Anotar o resultado em `specs/014-assistir-e-remover/relatorio.md`. **Decide a T1416**: se der `visible` minimizado ou na bandeja, a T1416 é feita; senão, ela é marcada como desnecessária.

## Phase 2: Foundational (regra comum de US1, US2 e US3)

- [X] T1402 `src/lib/types.ts`:
  - `StreamView` + `watching: boolean` e `you?: boolean`;
  - `MyBroadcastView` + `snapshot: ImageBitmap | null` e `onStage: boolean`;
  - `ParticipantView` + `removable: boolean`;
  - `RoomView` + `removeDialog?: { name: string; busy: boolean; error?: string }`;
  - `RoomActions` + `watch(id: string, on: boolean)`, `pip(id: string | null)`, `toggleSelfStage()`, `askRemove(peerId: string)`, `confirmRemove()`, `cancelRemove()`.

  *Feito (2026-10-31)*: os campos entraram **opcionais** (`watching` ausente = assistindo), então os mocks antigos não precisaram mudar. Até agora só os da US1 (`watching`, `you`, `watch`, e também `audioStream`, ver nota abaixo); os da US3/US4 entram nas fases deles.

  Preencher os campos novos com valores neutros (`watching: true`, `snapshot: null`, `onStage: false`, `removable: false`) em todos os mocks (`src/mocks/room.ts`, `mobile.ts`) e nos no-ops de `MockRoom.svelte` / `MockApp.svelte`, para o `check` passar.
- [X] T1403 `src/lib/room-view.ts`: função pura `viewModes(input: { streams: { id: string }[]; layout: StageLayout; focusedId: string | null; mobile: boolean; unwatched: ReadonlySet<string>; pageVisible: boolean; pipId: string | null; selfId: string | null }): Record<string, ViewMode>` com a precedência do research R5:
  1. não assistida → `hidden`;
  2. página escondida e `id !== pipId` → `hidden`;
  3. celular: selecionada (`focusedId ?? primeira`) → `full`, outras → `hidden`;
  4. foco: focada → `full`, outras → `thumb`;
  5. grade/única → `full`.

  `selfId` nunca entra no resultado.
- [X] T1404 [P] `test/lib/room-view.test.ts`: casos de `viewModes` para cada regra e combinações:
  - não assistida + PiP → `hidden`;
  - página escondida + PiP → `full`;
  - celular + página escondida → tudo `hidden`;
  - a própria tela fora do mapa.
- [X] T1405 `src/lib/stores/room.svelte.ts`: trocar o corpo de `myViews()` por `viewModes(...)`. Adicionar o estado `unwatched = $state(new SvelteMap<string, number>())` (peerId → startedAt), `pageVisible = $state(document.visibilityState === 'visible')` com o listener `visibilitychange` (removido no `destroy`) e `pipId = $state<string|null>(null)`. O `$effect` existente que chama `mesh.setViews` passa a reagir a esses três.

**Checkpoint**: tudo igual a hoje na tela (ninguém está em `unwatched`), mas trocar de aba no site já esconde as telas (US2 no site funciona aqui).

## Phase 3: User Story 1 — Parar de assistir e voltar a assistir (P1) 🎯 MVP

**Goal**: cada tela tem "Parar de assistir" / "Assistir". Sem assistir = cartão, sem vídeo chegando, som continua.

**Independent Test**: quickstart E2E 1 (o `framesSent` de A→B para em ≤ 3 s e o `viewers` cai) + mocks `unwatched`, `unwatched-all` e `mobile/unwatched`.

- [X] T1406 [US1] `room.svelte.ts`:
  - ação `watch(id, on)`: `on=false` grava `unwatched.set(id, peer.live.startedAt)`, `on=true` faz `delete`;
  - limpar a entrada em `peer-live` (live null ou `startedAt` diferente do guardado) e em `peer-left`;
  - **não** limpar na queda/reconexão (o `welcome` com o mesmo `startedAt` mantém);
  - no `view`, `StreamView.watching = !unwatched.has(id)`.
- [X] T1407 [P] [US1] `test/lib/` (teste da regra de limpeza; extrair para uma função pura `pruneUnwatched(unwatched, peers)` em `src/lib/room-view.ts` se a store não for testável direto): entrada mantida com o mesmo `startedAt`, apagada com `startedAt` novo, live null ou pessoa fora da lista.
- [X] T1408 [US1] `src/lib/mesh.ts` + `room.svelte.ts`: `myBroadcast.viewers` = número de entradas de `mesh.theirViews()` com modo ≠ `'hidden'` (hoje é `connectedCount()`). Atualizar quando chega o callback `view`.
- [X] T1409 [US1] `src/ui/components/StreamTile.svelte`:
  - com `view.watching === false`: em vez do `<video>`, um cartão com avatar/tom, nome, `view.label` (012) e o botão primário "Assistir" (ícone `Eye`) que chama a prop nova `onwatch(true)`. O cartão pode ser focado e clicado (o clique no cartão também não seleciona; só o botão assiste);
  - com `watching` e controles: botão "Parar de assistir" (ícone `EyeOff`, `aria-label`) na barra de controles, ao lado do PiP, chamando `onwatch(false)`;
  - se o `<video>` está em PiP quando `watching` vira `false`, chama `document.exitPictureInPicture()`;
  - na miniatura (`thumb`), o cartão compacto mostra só o nome e o ícone `Eye`.
- [X] T1410 [US1] `src/ui/components/Stage.svelte`: passar `onwatch={(on) => actions.watch(s.id, on)}` em todos os `StreamTile` (única, grade, foco e faixa). Uma tela não assistida continua ocupando o lugar dela (o layout não muda).
- [X] T1411 [US1] `src/ui/pages/MobileRoomPage.svelte`: o mesmo controle na tela selecionada (botão "Parar de assistir" nos controles e cartão "Assistir").
- [X] T1412 [US1] Mocks:
  - `src/mocks/room.ts`: cenas `unwatched` (grade com 3, a 2ª `watching:false`) e `unwatched-all` (todas `false`);
  - `src/mocks/mobile.ts`: `unwatched`;
  - `MockRoom.svelte`: a ação `watch` alterna o `watching` no mock.

**Checkpoint**: US1 completa. Rodar o quickstart E2E 1.

> **Nota da implementação (2026-10-31)**: duas descobertas.
> 1. O som tocava pelo próprio `<video>`, e o `<video>` some quando o vídeo para de chegar. Agora o som toca num `<audio>` próprio (`StreamView.audioStream` = só as faixas de som) e o `<video>` é sempre mudo.
> 2. A 010 pausava o **som** junto no modo `hidden` (`peer.ts`). Agora o som vai sempre, e `hidden` corta só o vídeo.
>
> E2E (scratchpad `assistir.mjs`, servidor local, câmera falsa):
> - parar de assistir: 0 quadros codificados/decodificados e ~100 pacotes de som em 2 s;
> - "0 assistindo";
> - voltar a assistir: vídeo em ~20 ms;
> - página escondida (evento forçado, porque o headless não esconde a aba): igual;
> - nenhuma tela "falhou" depois.

## Phase 4: User Story 2 — As telas dormem fora de vista (P1)

**Goal**: minimizado, na bandeja ou em aba de trás → tudo `hidden` (menos o PiP), e as telas voltam sozinhas.

**Independent Test**: quickstart E2E 2 (site) e M3/M4/M5 (app, PiP, 2º monitor).

- [X] T1413 [US2] `StreamTile.svelte`: ouvir `enterpictureinpicture` / `leavepictureinpicture` no `<video>` e chamar a prop nova `onpip(true|false)`. `Stage.svelte` e `MobileRoomPage.svelte` ligam em `actions.pip(on ? s.id : null)`.
- [X] T1414 [US2] `room.svelte.ts`: a ação `pip(id)` atualiza `pipId`. Quando a pessoa do `pipId` sai ou para de transmitir, `pipId = null`.
- [X] T1415 [US2] Conferir que, ao voltar `visible`, o `setViews` reenvia `full`/`thumb` pelo canal `ctl` (o `PeerLink.setMyView` já ignora o modo repetido: verificar que a troca hidden→full passa). Log `[espia] visível` / `escondida` no console (uma linha, como os outros logs da store).
- [X] T1416 [US2] (**só se a T1401 mostrar `visible`**) `src-tauri/src/engine.rs`: junto do `SetMemoryUsageTargetLevel`, chamar `controller.SetIsVisible(false)` em `WindowMode::Minimized | Hidden` e `SetIsVisible(true)` em `Visible`. Teste unitário da decisão (como os de `engine_action`). Subir a versão do app para 0.1.8 em `src-tauri/tauri.conf.json` e `src-tauri/Cargo.toml`. Rodar a T1401 de novo e conferir `hidden`.

**Checkpoint**: US2 completa no site (e no app, se a T1416 rodou).

> **Resultado (2026-10-31)**: o spike mostrou que o app **nunca** ficava `hidden`, então a T1416 foi feita (app 0.1.8). O `focus()` também passou a marcar a janela visível. Medições em [relatorio.md](./relatorio.md): o app minimizado ou na bandeja para de receber vídeo e o som segue; o app **transmitindo** escondido continua enviando normal (câmera e captura nativa).

## Phase 5: User Story 3 — Minha prévia não pesa (P1)

**Goal**: prévia = imagem parada a cada 5 s; clicar mostra a própria tela grande no palco.

**Independent Test**: quickstart E2E 3 + mocks `live` e `self-stage`.

- [X] T1417 [P] [US3] ★ `src/lib/snapshot.ts`:
  - `grabSnapshot(track: MediaStreamTrack, width = 320): Promise<ImageBitmap | null>`;
  - usa `ImageCapture.grabFrame()` quando existe; senão, `MediaStreamTrackProcessor` sobre `track.clone()` (lê 1 quadro, `frame.close()`, `clone.stop()`);
  - reduz para `width` px de largura mantendo a proporção num `OffscreenCanvas` → `transferToImageBitmap()`;
  - qualquer erro → `null`, sem jogar exceção.
- [X] T1418 [US3] `room.svelte.ts`:
  - ao começar a transmitir, `snapshot = await grabSnapshot(videoTrack)` em ≤ 1 s;
  - depois, `setInterval` de 5000 ms que só pega um quadro se `pageVisible`, fecha o bitmap anterior (`close()`) e troca;
  - no fim da transmissão: limpa o intervalo, fecha o bitmap e põe `null`;
  - `myBroadcast.snapshot` e `myBroadcast.onStage` no view.
- [X] T1419 [US3] `room.svelte.ts`:
  - `selfOnStage = $state(false)` e a ação `toggleSelfStage()`;
  - com `true`, o `view.streams` ganha (no fim) `{ id: myId, name: 'Você', you: true, tone, status: 'ok', muted: true, volume: 0, stream: myBroadcast.stream, watching: true, source, label }` e `focusedId = myId`;
  - voltar a `false` em `focus(outroId)`, `focus(null)` (Esc), `toggleSelfStage()` de novo e no fim da transmissão;
  - passar `selfId: myId` para `viewModes` (fica fora de `setViews`).
- [X] T1420 [US3] `StreamTile.svelte`: prop nova `still?: ImageBitmap | null`. Quando definida, desenha o bitmap num `<canvas>` (`$effect` que faz `drawImage` ao mudar) em vez de montar o `<video>`; `null` mostra o placeholder `caption`. Com `view.you`, nunca mostra controles de som.
- [X] T1421 [US3] `src/ui/pages/RoomPage.svelte`:
  - o tile de "Você está ao vivo" passa `still={b.snapshot}` (sem `stream`), vira clicável (`onselect={actions.toggleSelfStage}`), mostra o estado `b.onStage` (borda/realce) e o rótulo "Ver grande" / "Tirar do palco" no hover;
  - `Stage.svelte`: a mensagem "Sua tela está no ar… não aparece aqui pra não virar um espelho infinito" ganha "Clique na prévia pra ver grande.";
  - `MobileRoomPage`: se mostra a própria prévia, usa `still` também.
- [X] T1422 [US3] Mocks em `src/mocks/room.ts`:
  - a cena `live` existente usa `snapshot` (um `ImageBitmap` gerado de um canvas com a cor/placeholder do mock, criado em `MockRoom.svelte`);
  - nova cena `self-stage` (`onStage: true` e a própria tela focada no palco).

**Checkpoint**: US3 completa. Rodar o quickstart E2E 3.

> **Resultado (2026-10-31)**, scratchpad `previa.mjs`, site local, câmera falsa:
> - a prévia vira `<canvas>` em ≤ 3 s, sem `<video>`, e mudou 2 vezes em 12 s;
> - clicar põe a sua tela no palco (título "Sua tela, ao vivo", muda, sem `<audio>`), sem mudar o "1 assistindo";
> - Esc ou outro clique na prévia tiram;
> - com a página escondida, 0 trocas em 11 s;
> - parar de transmitir some a prévia.
>
> No app (captura nativa da janela de teste): a prévia é um quadro de 320×200 com a cor da janela.
>
> Ajustes no caminho:
> - Esc também vale quando a sua tela é a única no palco;
> - a lista não diz "você está assistindo" na sua linha.

## Phase 6: User Story 4 — O dono tira alguém da sala (P2)

**Goal**: o dono de sala fixa remove alguém; a pessoa sai com aviso e não reconecta.

**Independent Test**: quickstart E2E 4 + testes do Worker + mocks `owner-remove`, `remove-confirm` e `home/removed`.

- [X] T1423 [P] [US4] `src/shared/protocol.ts`: `ServerMessage` + `{ t: 'removed' }` e `CLOSE.removed = 4403` (comentário: "o dono tirou você da sala: não reconecta").
- [X] T1424 [P] [US4] `src/worker/validate.ts`: `parseRemoveBody(raw: string): string | null`, que aceita só `{ "peerId": "<exatamente 16 hex minúsculos>" }`. Testes em `test/worker/validate.test.ts`.
- [X] T1425 [US4] `src/worker/room.ts`: RPC `removePerson(ownerHash, peerId): Promise<OwnerResult>`:
  - faz o `checkOwner`;
  - se a pessoa está conectada: `serializeAttachment({ joined: false })`, `send({ t: 'removed' })` e `close(CLOSE.removed, 'removido pelo dono')`;
  - apaga o fantasma, se houver;
  - `dropMembers([peerId])`, `broadcast({ t: 'peer-left', id })` e `revokeTurn` das credenciais dela;
  - devolve `'ok'` também se a pessoa não existe.
- [X] T1426 [US4] `src/worker/index.ts`: rota `POST /api/rooms/:code/remove` → o `ownerAction` aceita o caminho `/remove`, faz `parseRemoveBody` (inválido → `400 { error: 'peer' }`) e responde `204` no ok. Mesmas regras de origem, `RL_CREATE` e Bearer do contrato.
- [X] T1427 [US4] `test/worker/room.test.ts` (ou `remove.test.ts`), conforme o contrato:
  - chave errada → `403`;
  - sala rápida → `403`;
  - conectado → recebe `removed` + 4403, os outros recebem `peer-left`, o `resume` dele dá `resume-failed` e o `join` novo entra;
  - fantasma (socket fechado sem leave) → some da sala;
  - `peerId` inexistente → `204`;
  - corpo inválido → `400`.
- [X] T1428 [US4] `src/lib/signaling.ts`: `SignalingEnd` + `'removed'`. A mensagem `removed` ou o fechamento 4403 → `end('removed')`, sem reconectar. `room.svelte.ts`/`RoomRoute.svelte`: em `'removed'`, apagar a sessão guardada da sala (o `resumeToken` no `localStorage`, mesma função que o leave usa) e ir pro início com o aviso.
- [X] T1429 [US4] `src/lib/api.ts`: `removePerson(code, key, peerId): Promise<{ ok: true } | { ok: false; reason: OwnerFailure }>`, no mesmo molde de `renameRoom`.
- [X] T1430 [US4] `room.svelte.ts`:
  - `ParticipantView.removable = room.kind === 'fixed' && room.own && !p.you`;
  - as ações `askRemove(peerId)` (abre `removeDialog` com o nome), `cancelRemove()` e `confirmRemove()`: `busy` → `api.removePerson`; ok fecha (a pessoa some pelo `peer-left`); erro mostra "Não deu pra tirar. Tenta de novo." no diálogo; `403` (deixou de ser dono) fecha com toast.
- [X] T1431 [P] [US4] ★ `src/ui/components/RemoveDialog.svelte`: no molde do `DeleteRoomDialog.svelte`, com o título "Tirar {name} da sala?", o corpo "Ela pode voltar pelo link." e os botões "Tirar da sala" (perigo) / "Cancelar". Estados `busy` e `error`. Esc cancela.
- [X] T1432 [US4] `src/ui/components/Participant.svelte`: com `person.removable`, um botão "⋯"/`UserMinus` discreto no hover/foco da linha, com o `aria-label` "Remover {name} da sala", que chama a prop `onremove`. `RoomPage.svelte` liga em `actions.askRemove(p.id)` e mostra o `RemoveDialog` quando `view.removeDialog`. No celular, o mesmo na lista de pessoas do `MobileRoomPage`, se a lista existir lá.
- [X] T1433 [US4] Aviso na tela inicial: `HomeRoute.svelte` lê um aviso de navegação (o mesmo mecanismo que a 011 usa para "sala apagada", se existir; senão, um parâmetro de estado no `router.go('/', { notice: 'removed' })`) e mostra o toast "O dono te tirou da sala".
- [X] T1434 [US4] Mocks:
  - `src/mocks/room.ts`: `owner-remove` (sala fixa, `own: true`, pessoas com `removable: true`) e `remove-confirm` (`removeDialog` aberto);
  - `src/mocks/home.ts`: `removed` (toast).

**Checkpoint**: US4 completa. Rodar o quickstart E2E 4.

> **Resultado (2026-10-31)**:
> - 5 testes novos do Worker: chave errada, sala rápida e outro site → 403; corpo inválido → 400; conectado → `removed` + 4403, `peer-left`, TURN revogado, `resume` falha e o `join` novo entra; fantasma some; quem não está → 204.
> - E2E (scratchpad `remover.mjs`, 3 pessoas, Beto transmitindo câmera):
>   - só o dono tem o botão, e não na própria linha;
>   - Beto some da lista em ~230 ms e a transmissão dele termina pra Cris;
>   - Beto cai no início com "O dono te tirou da sala" e não volta sozinho;
>   - recarregar não repete o aviso;
>   - entra de novo pelo link.
> - Desvio: o botão "Remover" aparece por cima do fim da linha ao passar o mouse (ícone `UserMinus`), em vez de um "⋯". O aviso no início é um toast que some em 8 s.

## Correção do usuário (2026-10-31)

- [X] T1439 [US1] Parar de assistir corta vídeo **e som**. O som só continua em segundo plano (research R9).
  - Modo `off` (`quality.ts`: `wantsVideo`/`wantsSound`);
  - `peer.ts`: `encodeView`/`decodeView`, `hidden` + `sound: false` no fio;
  - `viewModes`: não assistida e não escolhida no celular → `off`;
  - o tile sem assistir não monta o `<audio>`.
  - E2E `assistir.mjs`: sem assistir → 0 pacotes de som; página escondida → ~100 pacotes/2 s; ao voltar, vídeo e som.

## Phase 7: Polish & Cross-Cutting

- [X] T1435 Ajustar a `spec.md`: cenário US4-6 (app e navegador são pessoas diferentes; abas do mesmo navegador já são uma só, R7) e celular sem "ver a própria tela grande" (R8).
- [X] T1436 Rodar o e2e local completo (quickstart E2E 1–4, scratchpad, `npm run dev:worker`, nunca `--use-fake-ui-for-media-stream`) e registrar o resultado em `specs/014-assistir-e-remover/relatorio.md`.
- [X] T1437 Gate final: `npm run check`, `npm test`, `npm run build`; prints dos mocks novos. Commit.
- [ ] T1438 Publicar o site (`! npm run deploy` pelo usuário, se o auto mode bloquear). Se a T1416 rodou: `npm run desktop:release` **depois** do site e publicar de novo. Roteiro manual M2–M5 com o grupo (o FPS no jogo é o SC-001).

## Dependencies & Execution Order

- **T1401** (spike) pode rodar a qualquer momento antes da T1416. Não bloqueia o resto.
- **Phase 2** (T1402–T1405) bloqueia US1, US2 e US3.
- **US1, US2 e US3** dependem só da Phase 2, mas mexem nos mesmos arquivos (`room.svelte.ts`, `StreamTile.svelte`), então fazer em sequência: US1 → US2 → US3.
- **US4** não depende das outras. A parte do Worker (T1423–T1427) pode ser feita em paralelo com qualquer fase.

## Parallel Opportunities

- T1404 (testes) ‖ T1405.
- T1407 ‖ T1409.
- T1417 (`snapshot.ts`) ‖ qualquer tarefa da US1/US2.
- T1423, T1424 e T1431 ‖ entre si e com as fases 3–5.

## Implementation Strategy

1. **MVP** = Phase 2 + US1. O site já esconde as telas na aba de trás (efeito da Phase 2), e cada um escolhe o que assiste. Dá pra publicar sozinho.
2. Depois, a US2 (PiP + app conforme o spike) e a US3 (prévia). São as que mais mexem no FPS de quem transmite.
3. A US4 por último, ou em paralelo pela parte do Worker.
4. Publicar o site a cada fase concluída é seguro (compatível com o app 0.1.7 e com abas antigas).
