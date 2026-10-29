---
description: "Tarefas da 006: transmitir câmera ou placa de captura"
---

# Tasks: Transmitir câmera ou placa de captura

**Input**: [spec.md](./spec.md), [plan.md](./plan.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/signaling-live.md](./contracts/signaling-live.md), [quickstart.md](./quickstart.md)

**Tests**: incluídos (plano R9). Vitest para a lógica pura e o Worker. E2E no Edge só com câmera **falsa** (`--use-fake-device-for-media-stream`). Hardware real fica no roteiro C1–C7.

**Formato**: `- [ ] T6xx [P?] [US?] descrição com caminho`.

---

## Phase 1: Setup

- [x] T601 Emenda MINOR 1.2.0 → 1.3.0 em `.specify/memory/constitution.md`:
  - **princípio I**: "voz como recurso próprio continua fora do escopo; o som de uma câmera transmitida (inclusive microfone) é permitido enquanto a câmera estiver no ar";
  - **restrições técnicas**: "Navegadores alvo para transmitir: Chrome/Edge desktop; **câmera também no celular**".

  Atualizar o Sync Impact Report (motivo: spec 006, Q1: C e Q3: B) e `Last Amended`.

---

## Phase 2: Foundational (bloqueia todas as histórias)

- [x] T602 [P] Em `src/shared/protocol.ts`:
  - `AudioKind = 'tab' | 'window' | 'pc' | 'device' | 'none'` e `AUDIO_KINDS` com `'device'`;
  - `export type LiveSource = 'screen' | 'camera'` e `LIVE_SOURCES`;
  - `LiveInfo.source: LiveSource`;
  - a mensagem cliente `live` com `source?: LiveSource`.
- [x] T603 Em `src/worker/validate.ts`, aceitar `source` em `live` (adicionar a `hasOnly`). Ausente → `'screen'`; valor fora de `LIVE_SOURCES` → `null` (bad-message). Em `src/worker/room.ts`, guardar e repassar `source` no `LiveInfo` (`peer-live` e `welcome`).
- [x] T604 [P] Testes em `test/worker/room.test.ts`, conforme a tabela de `contracts/signaling-live.md`:
  - sem `source` → repassado como `'screen'`;
  - `'camera'` repassado;
  - `source: 'x'` → `bad-message`;
  - `audio: 'device'` aceito;
  - `'system'` continua recusado.
- [x] T605 [P] Em `src/lib/audio-source.ts`, `audioLabelText('device')` → `'com som do dispositivo'`. Teste em `test/lib/audio-source.test.ts`.
- [x] T606 [P] Criar `src/lib/devices.ts`, só com funções puras e sem navegador:
  - `isCaptureCard(label)`: regex `capture|hdmi|elgato|avermedia|cam link|game|usb3? video`;
  - `audioRole(label, pairedWithCapture)`: `'game'` se pareado com placa ou o nome tiver `capture|hdmi|line|elgato|avermedia|cam link|game`; `'voice'` se tiver `mic|microfone|headset|fone` ou for `default`/`communications`; senão `'voice'`;
  - `suggestAudio(video, audios)`: mesmo `groupId`, senão o nome com mais tokens em comum (≥ 1 token de 3+ letras), senão `null`;
  - `audioConstraints(id, role)`: tabela "Constraints por tipo de som" de `data-model.md` (game: echoCancellation/noiseSuppression/autoGainControl `false`, `channelCount: 2`; voice: `true`, `channelCount: 1`);
  - `videoConstraints(id, resolution, fps)`: `ideal`;
  - `PREVIEW_LIMIT = 4` e `previewIds(devices, focusedId)`: até 4 ids, priorizando o escolhido e os não ocupados.
- [x] T607 [P] Testes em `test/lib/devices.test.ts` cobrindo cada função: Elgato HD60 X + "Digital Audio Interface (HD60 X)" pelo `groupId`; webcam com "Microfone (C920)" → voz; "Line In" → jogo; nada parecido → null; limite de 4 prévias com a escolhida sempre dentro.
- [x] T608 Tipos em `src/lib/types.ts`:
  - `ShareSurface = 'window' | 'monitor' | 'camera'`;
  - `CameraView` (`permission: 'unknown' | 'prompt' | 'granted' | 'denied'`, `devices: { id, label, kind: 'camera' | 'capture', preview: MediaStream | null, busy: boolean }[]`, `audios: { id, label }[]`, `videoId`, `audioId`, `audioRole: 'voice' | 'game'`, `mobile: boolean`, `facing: 'user' | 'environment'`);
  - `RoomView.camera: CameraView`;
  - `MyBroadcastView.source?: 'screen' | 'camera'`;
  - `StreamView.source?: 'screen' | 'camera'`;
  - `GoLivePrefs.lastVideoLabel?`, `lastAudioLabel?` e `lastAudioRole?`;
  - `RoomActions`: `allowCamera()`, `selectCamera(id)`, `selectAudio(id | null)`, `setAudioRole(role)`, `setFacing(f)`.

  Ajustar `src/lib/prefs.ts` para ler e guardar os campos novos e aceitar `surface: 'camera'`.

**Checkpoint**: protocolo + lógica pura prontos; `npm run check` e `npm test` verdes.

---

## Phase 3: User Story 1 — Transmitir a webcam (P1) 🎯 MVP

**Goal**: aba Câmeras com permissão por clique, lista com prévias e transmissão sem outro seletor.
**Independent Test**: C1/C2 (webcam real) e o E2E com câmera falsa: a outra aba recebe o padrão de teste e vê "câmera de <apelido>".

### Visual (mock primeiro)

- [x] T609 [P] [US1] Em `src/ui/components/SourceCard.svelte`:
  - prop opcional `stream: MediaStream | null`, que mostra `<video autoplay muted playsinline>` no lugar da miniatura (com `srcObject` por `$effect`);
  - prop `busy`: cartão desabilitado com o chip "Em uso por outro programa";
  - ícone de webcam/placa quando sem prévia ("toque para ver").
- [x] T610 [US1] Em `src/ui/components/GoLiveModal.svelte`:
  - terceira aba **"Câmeras"** (ícone `Camera`) com a contagem quando `permission === 'granted'`;
  - painel por `camera.permission`:
    - `prompt`/`unknown`: texto "Pra mostrar suas câmeras, o navegador vai pedir permissão." + botão **"Permitir câmeras"** (`allowCamera`);
    - `denied`: "Câmera bloqueada. Libere nas permissões do site e abra de novo.";
    - `granted` sem dispositivos: "Nenhuma câmera ou placa encontrada";
    - `granted` com dispositivos: grade de `SourceCard` com as prévias;
  - Transmitir só com `videoId`.

  Funciona igual no modo `native` e no `browser` (a aba existe nos dois).
- [x] T611 [US1] Mocks em `src/mocks/room.ts` e `MockRoom.svelte`:
  - `pickercamperm`;
  - `pickercams` (3 dispositivos: "HD Pro Webcam C920", "Elgato HD60 X" (capture), "OBS Virtual Camera", com prévias falsas por `<canvas>.captureStream()` quando disponível, senão ícone);
  - `pickercamnone`, `pickercamdenied`;
  - `livecam` (card ao vivo com `source: 'camera'`).

  Ações do mock: permitir (vai para `granted`), escolher, transmitir.

### Lógica real

- [x] T612 [US1] Criar `src/lib/camera.ts`:
  - `cameraPermission()` via `navigator.permissions.query({ name: 'camera' })`, com try/catch → `'unknown'`;
  - `requestCamera()`: `getUserMedia({ video: true })` e parar as faixas na hora;
  - `listDevices()`: `enumerateDevices()` → `videoinput`/`audioinput` com `label` não vazio;
  - `watchDevices(cb)`: `devicechange`;
  - `openPreview(id)`: 320×180, 15 fps; `NotReadableError` → `busy`;
  - `startCamera({ videoId, audioId, audioRole, resolution, fps, facing? })`: `getUserMedia` com `videoConstraints` + `audioConstraints` e `contentHint: 'motion'`.
- [x] T613 [US1] Em `src/lib/stores/room.svelte.ts`:
  - estado da câmera (permissão, dispositivos, prévias, escolha);
  - ao abrir o modal: só consulta a permissão (sem pedir). `allowCamera` → `requestCamera` → lista;
  - prévias com `previewIds` (no máximo 4), fechadas ao fechar o modal, ao trocar de aba para fora de Câmeras e ao transmitir (FR-604);
  - `devicechange` atualiza a lista com o modal aberto;
  - `startBroadcast` com `surface === 'camera'`: fecha as prévias, chama `startCamera`, usa o caminho atual (mesh, `constrainCapture` com `applyConstraints`, `ended` → parar) e manda `live` com `source: 'camera'` e `audio: 'device' | 'none'`;
  - `buildView` monta `camera` e `myBroadcast.source`.
- [x] T614 [US1] Quem assiste: em `buildView`, `StreamView.source = p.live.source`. Em `src/ui/components/StreamTile.svelte` (placeholder e chip) e no card "Você está ao vivo" de `src/ui/pages/RoomPage.svelte`/`MobileRoomPage.svelte`, usar "câmera de <nome>" e "câmera · <dispositivo>" quando `source === 'camera'` (FR-607).
- [x] T615 [US1] Erros: `NotAllowedError` no `allowCamera` → `permission = 'denied'`; `NotReadableError` ao transmitir → toast "Essa câmera está sendo usada por outro programa" e o modal reabre; dispositivo desconectado → `ended` → parar (FR-608).

**Checkpoint**: MVP. Webcam transmitida no site e no app.

---

## Phase 4: User Story 2 — Placa de captura com o som do console (P1)

**Goal**: escolher a entrada de som (sugerida pela placa) como "Som de jogo", sem tratamento.
**Independent Test**: C3 (placa + console). E2E: com "jogo", `getSettings().echoCancellation === false`.

- [x] T616 [US2] No `GoLiveModal.svelte` (aba Câmeras, com "Compartilhar áudio" ligado):
  - um `<select>` "Entrada de som" com todas as `audios` + "Sem som";
  - um segmentado **"Som de voz | Som de jogo"** (mesmo estilo de pílula do 720p/1080p);
  - o texto sob o interruptor: "Vai o som de <entrada> junto com a câmera." ou "Só a imagem vai pra sala.".
- [x] T617 [US2] No store, ao escolher um vídeo: `audioId = suggestAudio(...)?.id ?? null` e `audioRole = audioRole(label, isCaptureCard(video.label))`, a menos que a pessoa já tenha trocado nesta abertura. `selectAudio`/`setAudioRole` gravam a escolha manual.
- [x] T618 [P] [US2] Mock `pickercammic` (Elgato escolhida, som "Digital Audio Interface (HD60 X)" como jogo) + ações do mock para trocar entrada/perfil.

---

## Phase 5: User Story 3 — Câmera com a minha voz (P2)

**Goal**: microfone como "Som de voz", com cancelamento de eco, e som só junto da câmera.
**Independent Test**: C4 (duas pessoas com caixas, sem eco perceptível). E2E: com "voz", `echoCancellation === true`.

- [x] T619 [US3] Com `audioRole === 'voice'` e uma entrada escolhida, mostrar sob o seletor a dica fixa: "Se você está no Discord, a galera pode te ouvir duas vezes. Fone de ouvido evita eco." (US3 cenário 4).
- [x] T620 [US3] Garantir a FR-606b no store:
  - parar a câmera para o som (o `stopCapture` para todas as faixas);
  - não existe caminho para transmitir som sem vídeo: `startCamera` exige `videoId`;
  - se a faixa de vídeo terminar, parar também a de áudio.

  O mudo inicial para quem assiste já é a regra atual (conferir que vale para `'device'`).

---

## Phase 6: User Story 4 — Celular transmitindo a câmera (P3)

**Goal**: botão Transmitir no celular abrindo só a aba Câmeras (frontal/traseira), em 720p30.
**Independent Test**: C5 (celular → PC vê a imagem; apagar a tela encerra).

- [x] T621 [US4] Em `src/lib/capture.ts`, trocar `canShare` por `canShareScreen()` (getDisplayMedia) e `canShareCamera()` (getUserMedia). Em `RoomView`/`JoinView`, `canShare = canShareScreen || canShareCamera` e um campo novo `canShareScreen`. Revisar `JoinPage.svelte`, `Stage.svelte`, `RoomPage.svelte` e `MobileRoomPage.svelte` para o texto "só assistir" sumir quando há câmera.
- [x] T622 [US4] No `GoLiveModal.svelte` com `camera.mobile` (ou sem `canShareScreen`): só a aba Câmeras, sem a grade de dispositivos, com um segmentado **"Frontal | Traseira"** (`setFacing`). Resolução fixa em 720p e FPS em 30 (as opções escondidas), com o aviso "Transmitir pelo celular gasta internet e bateria.".
- [x] T623 [US4] No store, no celular, `startCamera` com `facingMode` em vez de `deviceId`, em 720/30. O track `ended`, ou `mute` por mais de 3 s (tela apagada), → parar a transmissão.
- [x] T624 [P] [US4] Mock `mobilecam` (MobileRoomPage com o modal só de Câmeras, Traseira escolhida).

---

## Phase 7: Polish & Cross-Cutting

- [x] T625 [P] Preferências: lembrar `lastVideoLabel`, `lastAudioLabel` e `lastAudioRole` ao transmitir. Ao abrir com a permissão já dada, remarcar pelo nome (FR-609).
- [x] T626 [P] Acessibilidade: `aria-label` nas prévias e "Permitir câmeras" com foco inicial. Sem rolagem horizontal em 390 px.
- [x] T627 Gate: `npm run check`, `npm test` e `npm run build` verdes. As 7 cenas de mock abrem e mostram todos os estados.
- [x] T628 E2E no Edge (scratchpad) **só** com `--use-fake-device-for-media-stream` e abas de teste:
  - transmitir a câmera falsa → a outra aba recebe 1 faixa de vídeo com o padrão de teste e "câmera de …";
  - com "jogo" → `echoCancellation: false`; com "voz" → `true`;
  - fechar o modal → 0 faixas de prévia vivas.

  **Nunca** usar `--use-fake-ui-for-media-stream` sem a câmera falsa, nem capturar a tela real.
- [x] T629 Publicar o site (`npm run deploy`, pela pessoa usuária). O app 0.1.3 já pega a câmera, sem build novo.
- [ ] T630 Roteiro manual C1–C7 do `quickstart.md` (webcam, app, placa + console, voz sem eco, celular, câmera ocupada, cabo desconectado).
- [ ] T631 Commit e relatório `specs/006-transmitir-camera/relatorio.md` no formato do da 005.

---

## Dependencies & Execution Order

- **Setup (T601)** → **Foundational (T602–T608)** → histórias.
- **US1 (T609–T615)** é o MVP; US2, US3 e US4 dependem dela (aba, store e `camera.ts`).
- **US2 (T616–T618)** antes de **US3 (T619–T620)**: o seletor de som e o perfil são da US2.
- **US4 (T621–T624)** depende só de US1.
- **Polish** depois. T629 antes de T630.

## Parallel Opportunities

- **Foundational**: T602/T605/T606/T607 em paralelo; T603 depois de T602; T604 depois de T603.
- **US1**: o visual (T609, T611) em paralelo com `camera.ts` (T612); juntam em T610/T613.
- **US2/US4**: os mocks (T618, T624) em paralelo com a lógica.

## Implementation Strategy

1. **MVP** = T601–T615: webcam sem som, no site e no app. Dá para publicar e testar com a galera.
2. **+ US2**: placa com som de jogo (o caso do console).
3. **+ US3**: microfone como voz, com a dica de fone.
4. **+ US4**: celular.
5. **Polish**: prefs, E2E com câmera falsa, deploy, roteiro C1–C7 e relatório.
