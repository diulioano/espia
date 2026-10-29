---
description: "Tarefas da 005: seletor de tela nativo no app desktop"
---

# Tasks: Seletor de tela nativo no app desktop

**Input**: [spec.md](./spec.md), [plan.md](./plan.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/native-capture.md](./contracts/native-capture.md), [quickstart.md](./quickstart.md)

**Tests**: incluídos, como pede o plano (R11): `cargo test` para as funções puras do Rust e Vitest para a ponte e o protocolo. A captura real fica no roteiro manual N1–N8, porque não pode ser automatizada sem capturar a tela real.

**Formato**: `- [ ] T5xx [P?] [US?] descrição com caminho`. `[P]` = pode rodar em paralelo (arquivo diferente, sem dependência pendente).

---

## Phase 1: Setup

- [x] T501 Emenda MINOR 1.1.0 → 1.2.0 em `.specify/memory/constitution.md`. A restrição técnica do app desktop passa a ser: "Tauri 2 + WebView2, só Windows, carrega o site publicado; expõe à página **apenas** a ponte de captura (`specs/005-seletor-nativo-desktop/contracts/native-capture.md`: listar fontes, miniaturas, iniciar/parar captura de vídeo e som), **apenas** para a origem do site, na janela principal. Qualquer outra API nativa exige emenda." Atualizar o Sync Impact Report (motivo: pedido de 2026-10-29) e `Last Amended`.
- [x] T502 [P] Em `src-tauri/Cargo.toml`, acrescentar ao `windows` (0.62) as features `Graphics_Capture`, `Graphics_DirectX_Direct3D11`, `Foundation`, `Win32_Graphics_Direct3D11`, `Win32_Graphics_Direct3D`, `Win32_Graphics_Dxgi`, `Win32_Graphics_Dxgi_Common`, `Win32_Graphics_Gdi`, `Win32_Graphics_Dwm`, `Win32_Graphics_Imaging`, `Win32_Media_Audio`, `Win32_Media_Audio_Endpoints`, `Win32_System_Com`, `Win32_System_Com_StructuredStorage`, `Win32_System_Threading`, `Win32_System_ProcessStatus`, `Win32_System_Diagnostics_ToolHelp`, `Win32_System_WinRT`, `Win32_System_WinRT_Graphics_Capture`, `Win32_System_WinRT_Direct3D11`, `Win32_Storage_FileSystem`, `Win32_System_Variant` e `Win32_UI_Shell`. Acrescentar o crate `base64 = "0.22"` e `serde = { version = "1", features = ["derive"] }`. Confirmar com `cargo check` em `src-tauri/`.
- [x] T503 [P] Adicionar `@tauri-apps/api` (versão alinhada ao `@tauri-apps/cli` 2.12) em `dependencies` do `package.json`. Importar só de `@tauri-apps/api/core` (`invoke`, `Channel`).
- [x] T504 Criar `src-tauri/capabilities/capture.json`:
  - `identifier: "capture"`, `windows: ["main"]`;
  - `remote.urls: ["https://espia.espia.workers.dev/*"]`;
  - `permissions` com os 5 comandos do contrato.

  Em `src-tauri/build.rs`, registrar `tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&["capture_supported","capture_list","capture_start","capture_set_quality","capture_stop"]))`. Para debug, criar uma segunda capability `capture-dev.json` com `remote.urls: ["http://127.0.0.1:*/*"]`, incluída só em builds de debug (por exemplo, por `tauri.conf.json` > `app.security.capabilities` ou condicional no build).

---

## Phase 2: Foundational (bloqueia todas as histórias)

- [x] T505 [P] Em `src/shared/protocol.ts`, `AudioKind = 'tab' | 'window' | 'pc' | 'none'` e `AUDIO_KINDS` com `'pc'`. Conferir que `src/worker/validate.ts` aceita `'pc'` via `AUDIO_KINDS`, sem outra mudança.
- [x] T506 [P] Teste em `test/worker/room.test.ts`: `live` com `audio: 'pc'` é aceito e repassado. `'system'` continua recusado.
- [x] T507 [P] Em `src/lib/audio-source.ts`, `audioLabelText('pc')` → `'com som do PC (sem Discord)'`. Criar também `nativeAudioText({ surface, audio, selectedTitle })` com exatamente os textos de `data-model.md`:
  - "Só o som de *<title>* — nada de notificação vazando."
  - "Só o som da janela que você escolher — Discord e notificações ficam de fora."
  - "Todo o som do computador vai junto — menos o Discord."
  - "Só a imagem vai pra sala."

  Testes em `test/lib/audio-source.test.ts`.
- [x] T508 [P] Tipos em `src/lib/types.ts`:
  - `CaptureSource` (`id: string` no formato `"w:<hwnd>"` / `"m:<hmonitor>"`; `kind: 'window' | 'monitor'`; `title` não vazio; `app`; `icon: string | null` em `data:image/png` 32×32; `thumb: string | null` em `data:image/jpeg` ≤ 320×180; `playing: boolean`, sempre `false` em monitor; `primary: boolean`);
  - `PickerState` (`mode: 'native' | 'browser'`, `tab`, `sources`, `selectedId: string | null`, `audioText`, `canStart`);
  - `GoLivePrefs.lastSourceKey?: string`;
  - `NativeStartResult` (`{ width, height, audio: AudioKind, audioWarn?: 'unsupported' | 'many_discords' }`).
- [x] T509 Esqueleto Rust em `src-tauri/src/capture/mod.rs`:
  - os 5 comandos `#[tauri::command]`, com stubs que devolvem os erros do contrato;
  - um estado `Mutex<Option<Session>>` gerenciado pelo Tauri;
  - a função `origin_ok(webview)`, que confere se `webview.url()` está na origem do site (ou em `127.0.0.1` em debug) e é chamada no início de **todo** comando;
  - `capture_supported` real: `{ ok, version: 1, win11 }`, com `ok` = Windows ≥ 10 1903 e `GraphicsCaptureSession::IsSupported()`.

  Registrar em `src-tauri/src/lib.rs` (`.manage(...)`, `.invoke_handler(tauri::generate_handler![...])`) e declarar `mod capture` (só `cfg(windows)`; fora do Windows, `capture_supported` devolve `ok: false`).
- [x] T510 Módulo `src/lib/native-capture.ts`, seguindo o princípio IV (nada de UI):
  - `detectNative(): Promise<boolean>` chama `capture_supported` só se `'__TAURI_INTERNALS__' in window`, com try/catch, e exige `version === 1`;
  - `listSources(thumbs)`;
  - `pollSources(onUpdate, intervalMs = 2000)`, que devolve uma função de parar;
  - uma interface `Bridge` injetável (invoke, Channel e `chrome.webview`), para teste com ponte falsa.
- [x] T511 [P] Teste `test/lib/native-capture.test.ts` (ponte falsa): sem `__TAURI_INTERNALS__` → `false`; erro ou `version` diferente de 1 → `false`; o polling chama a cada 2 s e para quando a função de parar é chamada.
- [x] T512 Em `src/lib/stores/room.svelte.ts`, ao abrir o modal, rodar `detectNative()` uma vez por sessão e montar `picker: PickerState` no `RoomView`. No modo `native`, iniciar o `pollSources` ao abrir e pará-lo ao fechar ou transmitir (FR-504). Tirar `selectedId` quando a fonte some da lista (US1 cenário 5). Adicionar `selectSource(id)` aos eventos do modal.

**Checkpoint**: a ponte existe, o site ainda funciona igual e os testes passam.

---

## Phase 3: User Story 1 — Escolher a janela direto no seletor do app (P1) 🎯 MVP

**Goal**: grade de janelas com miniaturas; Transmitir captura sem o seletor do Windows (por enquanto sem som).
**Independent Test**: no app, com 3 janelas abertas, escolher uma e transmitir. A sala mostra aquela janela e nenhum seletor do Windows aparece (N1–N3, sem a parte do som).

### Visual (mock primeiro, princípio V)

- [x] T513 [P] [US1] Criar o componente `src/ui/components/SourceCard.svelte` conforme `tela-esperada.png`:
  - miniatura 16:9 (com `thumb` nulo, ícone grande no centro sobre o padrão listrado);
  - selo "tocando som" com ícone de volume no canto inferior esquerdo (prop `playing`);
  - ✓ em círculo laranja e borda laranja quando `selected`;
  - rodapé com ícone 20px, título (uma linha, reticências) e nome do programa.

  Botão acessível (`aria-pressed`), só com os tokens de `src/ui/tokens.css`.
- [x] T514 [US1] Em `src/ui/components/GoLiveModal.svelte`, com `picker.mode === 'native'`:
  - contagem nas abas ("Janelas 6", "Telas inteiras 2");
  - "Prévias atualizam a cada 2s" à direita das abas;
  - grade de `SourceCard` com 3 colunas no desktop, 2 abaixo de ~720px e 1 em 390px, rolagem vertical e sem rolagem horizontal;
  - estado vazio ("Nenhuma janela aberta pra mostrar");
  - Transmitir desabilitado sem `canStart`;
  - texto do som vindo de `picker.audioText`.

  Com `mode === 'browser'`, manter **exatamente** o layout da 004 (cartão "Você escolhe a janela no próximo passo").
- [x] T515 [US1] Mocks em `src/mocks/room.ts` e `src/mocks/MockRoom.svelte`:
  - cenas `pickerwindows` (6 janelas como em `tela-esperada.png`: Hollow Knight escolhida e tocando, YouTube — Firefox tocando, VS Code, Spotify tocando, Planilha do campeonato, Downloads com `thumb: null`);
  - `pickerempty`;
  - `pickerfallback` (modo browser).

  Miniaturas falsas são `data:` SVG listrados com cores do token. O `MockRoom` deve permitir clicar e selecionar. Registrar em `src/mocks/registry.ts`.

### Listagem real

- [x] T516 [P] [US1] Criar `src-tauri/src/capture/sources.rs`:
  - a função pura `is_listable(attrs: &WinAttrs, own_pid) -> bool`: visível, título não vazio, sem `WS_EX_TOOLWINDOW`, não cloaked, sem dono e com PID diferente do Espia;
  - `enum_windows()` com `EnumWindows` + `DwmGetWindowAttribute(DWMWA_CLOAKED)`;
  - nome do programa (`QueryFullProcessImageNameW` + `FileDescription` da versão do executável, ou o nome do `.exe`);
  - UWP: `ApplicationFrameHost` → PID da `CoreWindow` filha;
  - ícone (`WM_GETICON ICON_BIG` → `GCLP_HICON` → ícone do exe) → PNG 32×32 por WIC → `data:image/png;base64`, em cache por PID.

  `#[cfg(test)]` com casos do filtro.
- [x] T517 [P] [US1] Criar `src-tauri/src/capture/thumbs.rs`: `PrintWindow(hwnd, PW_RENDERFULLCONTENT)` num DIB, redução para caber em 320×180 mantendo a proporção (função pura `fit(w,h,maxw,maxh)` com testes) e JPEG qualidade ~70 por WIC → `data:image/jpeg;base64`. Janela minimizada (`IsIconic`) ou imagem toda preta → `None`.
- [x] T518 [US1] Em `src-tauri/src/capture/mod.rs`, `capture_list({ thumbs })` real (só janelas por enquanto), rodando fora da thread principal (`tauri::async_runtime::spawn_blocking`) e devolvendo `CaptureSource[]` serializado (`serde`, camelCase).

### Captura de vídeo

- [x] T519 [US1] Criar `src-tauri/src/capture/video.rs`:
  - dispositivo D3D11 (BGRA support) + `IDirect3DDevice` WinRT;
  - `GraphicsCaptureItem` por `IGraphicsCaptureItemInterop::CreateForWindow`;
  - `Direct3D11CaptureFramePool::CreateFreeThreaded` (BGRA, 2 buffers);
  - `IsCursorCaptureEnabled = true`; no Win11, `GraphicsCaptureAccess::RequestAccessAsync(Borderless)` + `IsBorderRequired = false`;
  - em `FrameArrived`: descarta se < 1/fps desde o último, converte para NV12 no tamanho alvo (altura 720/1080, mantendo a proporção, largura e altura pares) com `ID3D11VideoProcessor`, copia para uma textura de staging e faz `Map`;
  - `item.Closed` → manda `{t:'ended', reason:'source_closed'}`.

  A função pura `target_size(src_w, src_h, res) -> (w,h)` tem testes.
- [x] T520 [US1] Em `src-tauri/src/capture/video.rs`, o anel de buffers compartilhados:
  - pelo `webview.with_webview` → `ICoreWebView2Controller` → `CoreWebView2` (`ICoreWebView2_17`) e o ambiente (`ICoreWebView2Environment12`), criar 3 buffers do tamanho NV12 (`w*h*3/2`);
  - postar com `PostSharedBufferToScript(buffer, COREWEBVIEW2_SHARED_BUFFER_ACCESS_READ_ONLY, json)`, com `{"espia":"frame-slot","gen","slot","width","height","format":"NV12"}`;
  - escrever cada quadro no próximo slot (respeitando o pitch do `Map`) e mandar `{t:'frame',gen,slot,seq,ts}` pelo `Channel`;
  - nunca reescrever um slot antes de anunciar os outros dois.

  Os chamados ao WebView2 vão na thread da UI (`run_on_main_thread`). Conferir `origin_ok` antes de postar.
- [x] T521 [US1] Em `src-tauri/src/capture/mod.rs`, `capture_start` (sem som ainda: `audio: 'none'`) e `capture_stop` com o ciclo `idle → starting → live → stopped` de `data-model.md`:
  - uma sessão por vez (`start` encerra a anterior);
  - falha de vídeo → erro `capture_failed`; fonte inexistente → `source_gone`;
  - recarregar ou navegar a página (`on_page_load`) → `stop`.
- [x] T522 [US1] Em `src/lib/native-capture.ts`, `startNative(prefs, sourceId): Promise<{ stream, result }>`:
  - ouve `chrome.webview` `sharedbufferreceived`, guarda por `(gen, slot)` e libera as gerações antigas (`releaseBuffer`);
  - cria `new MediaStreamTrackGenerator({ kind: 'video' })` e, a cada `frame`, faz `new VideoFrame(slotBuffer, { format: 'NV12', codedWidth, codedHeight, timestamp: ts })` → `writer.write(frame)` no mesmo tick, descartando `seq` atrasado em mais de 2;
  - `ended` → `generator.stop()` (dispara `ended` no track);
  - `contentHint` como na 004 (60 fps → `'motion'`, senão `'detail'`).

  Também criar `stopNative()` e logar `[espia] captura nativa:` (fonte, tamanho, fps, quadros descartados por segundo).
- [x] T523 [US1] Em `src/lib/capture.ts` e `src/lib/stores/room.svelte.ts`, com o picker no modo `native`, `startBroadcast` usa `startNative` em vez do `getDisplayMedia` e **não** aplica `applyAudioRule` (o som é decidido no app). O `AudioKind` vem de `result.audio`. `ended` → `stopBroadcast(true)`. Parar a transmissão → `capture_stop`. No modo `browser`, o código fica como está.
- [x] T524 [P] [US1] Testes em `test/lib/native-capture.test.ts` com a ponte falsa e stubs de `MediaStreamTrackGenerator`/`VideoFrame`: buffers de `gen` antiga ignorados, `seq` atrasado descartado, `ended` encerra o track e `selectedId` some quando a fonte some.

**Checkpoint**: MVP. Grade real no app, transmitir uma janela sem o seletor do Windows, site intacto.

---

## Phase 4: User Story 2 — Som só da janela escolhida + selo "tocando som" (P1)

**Goal**: selo nos cartões, só o som do programa da janela e nada do Discord.
**Independent Test**: N4. Com uma chamada do Discord, transmitir a janela de um vídeo com som; quem assiste ouve o vídeo e nenhuma voz.

- [x] T525 [P] [US2] Criar `src-tauri/src/capture/procs.rs`: snapshot de processos (`CreateToolhelp32Snapshot`: pid, ppid, exe) e a função pura `tree_root(pid, table) -> pid`, que sobe enquanto o pai tiver o mesmo executável. Testes com tabelas falsas (navegador com processo de áudio filho, processo órfão, ciclo).
- [x] T526 [US2] Em `src-tauri/src/capture/audio.rs`, sessões "tocando": `IMMDeviceEnumerator` → saída padrão → `IAudioSessionManager2::GetSessionEnumerator`. Cada sessão em `AudioSessionStateActive` com `IAudioMeterInformation::GetPeakValue() > 0.001` → PID → `tree_root`. Em `capture_list`, marcar `playing` nas janelas com a mesma raiz.
- [x] T527 [P] [US2] Em `src-tauri/src/capture/audio.rs`, a função pura `audio_plan(kind, wants_audio, window_root, discord_roots, own_pid) -> AudioPlan`, com **todas** as linhas da tabela "AudioPlan" de `data-model.md`. Testes cobrindo cada linha, inclusive "janela do próprio Discord vai com o som dela".
- [x] T528 [US2] Em `src-tauri/src/capture/audio.rs`, loopback de processo:
  - `ActivateAudioInterfaceAsync(VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK, IAudioClient, PROPVARIANT{AUDIOCLIENT_ACTIVATION_PARAMS{PROCESS_LOOPBACK, {pid, mode}}})`;
  - `Initialize` em `AUDCLNT_STREAMFLAGS_LOOPBACK | EVENTCALLBACK`, formato float32 48 kHz 2 canais;
  - uma thread de captura junta pedaços de 480 quadros (10 ms) e envia como `ArrayBuffer` no `Channel` `sound`.

  Falha na ativação → a sessão segue sem som, com `audioWarn: 'unsupported'`. **Nunca** cair para o loopback do dispositivo inteiro.
- [x] T529 [US2] Em `capture_start`, montar a `AudioPlan` (janela: `include(tree_root(pid da janela))`), iniciar o áudio depois do vídeo e devolver `audio`/`audioWarn` conforme o contrato.
- [x] T530 [US2] Em `src/lib/native-capture.ts`, o canal `sound`: `MediaStreamTrackGenerator({ kind: 'audio' })` e, para cada `ArrayBuffer`, `new AudioData({ format: 'f32', sampleRate: 48000, numberOfChannels: 2, numberOfFrames: 480, timestamp, data })`, com timestamp contínuo (`+10_000` µs). Sem som → o stream sai sem track de áudio. Tratar a mensagem `{t:'audio', kind, warn}` (som mudou no ar), atualizando `audioKind` e mandando `live` de novo.
- [x] T531 [US2] Em `src/lib/stores/room.svelte.ts` + `GoLiveModal.svelte`, o texto do som pelo `nativeAudioText` com o título da janela escolhida (FR-508). `audioWarn: 'unsupported'` → toast "Seu Windows não deixa separar o som da janela — foi só a imagem."
- [x] T532 [P] [US2] Testes Vitest: `AudioData` com timestamps contínuos, stream sem track de áudio quando `audio: 'none'` e a mensagem `audio` atualizando o kind.

**Checkpoint**: US1 + US2 cobrem o pedido P1 inteiro.

---

## Phase 5: User Story 3 — Tela inteira no seletor do app, com som do PC menos o Discord (P2)

**Goal**: aba Telas inteiras com monitores reais e som do PC sem o Discord (Q1: B).
**Independent Test**: N5–N7, com dois monitores, Spotify tocando e o Discord em chamada.

- [x] T533 [P] [US3] Em `src-tauri/src/capture/sources.rs`, `EnumDisplayMonitors` + `GetMonitorInfoW`: id `"m:<hmonitor>"`, título "Tela N" pela ordem, `app: "Principal"` no monitor com `MONITORINFOF_PRIMARY` e `""` nos outros, `primary`. Em `thumbs.rs`, miniatura por `BitBlt` do DC da tela no retângulo do monitor. Incluir em `capture_list`.
- [x] T534 [US3] Em `src-tauri/src/capture/video.rs`, `CreateForMonitor` no `capture_start` quando o id começar com `m:`.
- [x] T535 [P] [US3] Criar `src-tauri/src/capture/discord.rs` com a função pura `discord_roots(table) -> Vec<pid>`: executáveis `Discord.exe`, `DiscordPTB.exe` e `DiscordCanary.exe`, sem diferenciar maiúsculas, cujo pai não tem o mesmo executável. Testes: nenhum, um com filhos, dois (Discord + PTB) e reinício (PID novo).
- [x] T536 [US3] Em `src-tauri/src/capture/audio.rs`, o modo `EXCLUDE_TARGET_PROCESS_TREE` conforme a `AudioPlan`:
  - 0 raízes → exclui o PID do Espia;
  - 1 raiz → exclui essa raiz;
  - 2+ raízes → sem som e `audioWarn: 'many_discords'`.

  O `AudioKind` devolvido é `'pc'`.
- [x] T537 [US3] Vigia do Discord em `src-tauri/src/capture/discord.rs`: uma thread que, a cada 1 s durante uma sessão de tela com som, recalcula `discord_roots`. Se mudou, recria o cliente de áudio com a nova `AudioPlan` e, se o kind mudar, manda `{t:'audio', kind, warn}`. Para junto com a sessão.
- [x] T538 [US3] No modal (modo native), a aba Telas inteiras tem o interruptor de som **destravado** (diferente da 004) e o texto "Todo o som do computador vai junto — menos o Discord." Toast de `many_discords`: "Mais de um Discord aberto — tela inteira vai sem som." O card ao vivo mostra "com som do PC (sem Discord)" (T507).
- [x] T539 [P] [US3] Mock `pickerscreens` em `src/mocks/room.ts` (2 telas, "Tela 1 · Principal" escolhida, som ligado) e `livepc` (card ao vivo com `audioKind: 'pc'`).

**Checkpoint**: as duas abas no app são reais. Roteiro N5–N7.

---

## Phase 6: User Story 4 — Site continua com o seletor do navegador (P2)

**Goal**: nenhuma regressão fora do app; app antigo + site novo = fluxo v0.1.2.
**Independent Test**: no Chrome, o seletor é idêntico ao da v0.1.2. Testes atuais verdes (SC-505).

- [x] T540 [US4] Revisar `GoLiveModal.svelte`, `capture.ts` e `room.svelte.ts`: no modo `browser`, o caminho é byte a byte o comportamento da 004 (`getDisplayMedia` com `displaySurface`, `applyAudioRule`, tela inteira com som travado). Qualquer falha de `detectNative`, `capture_list` ou `capture_start` com `capture_failed` → `mode = 'browser'` naquela abertura, sem erro bloqueante (FR-511). `source_gone` → recarrega a lista e mantém o modo native.
- [x] T541 [P] [US4] Testes em `test/lib/native-capture.test.ts`: `capture_list` lançando → modo browser; app sem o comando (erro "command not found") → modo browser.
- [ ] T542 [US4] E2E rápido no Edge (só uma aba de teste escolhida pelo título, como na 004 T407): o seletor do site mostra o cartão "próximo passo" e os 18 casos da 004 continuam ok.

---

## Phase 7: User Story 5 — Escolha lembrada (P3)

**Goal**: o seletor volta na última aba e com a última fonte marcada, se ela ainda existir.
**Independent Test**: transmitir a janela X, parar e abrir de novo. X vem marcada.

- [x] T543 [US5] Em `src/lib/prefs.ts`, salvar `lastSourceKey` ao transmitir: `"<app>|<title>"` para janela e `"m:<n>"` para monitor. O HWND muda quando o programa reabre, então a chave não usa ele. Em `room.svelte.ts`, na primeira lista depois de abrir, pré-selecionar a fonte cuja chave bate. Se nenhuma bater, nada vem marcado.
- [x] T544 [P] [US5] Teste em `test/lib/native-capture.test.ts` (ou `prefs`): chave igual → marcada; janela fechada → nada marcado.

---

## Phase 8: Polish & Cross-Cutting

- [x] T545 Qualidade no ar: em `room.svelte.ts`, o `constrainCapture(q)` no modo native chama `capture_set_quality({ resolution, fps })`. No Rust, recria o processador e os 3 slots com `gen + 1` e posta de novo. Em `native-capture.ts`, troca para a nova geração. Cobrir a troca de geração no teste da ponte falsa.
- [x] T546 [P] Jogo em tela cheia exclusiva: se os primeiros 30 quadros de uma janela vierem todos pretos (amostra do plano Y do NV12 em `video.rs`), mandar pelo canal `frames` a mensagem nova `{t:'warn', reason:'black'}` e acrescentá-la ao tipo `FrameMsg` em `contracts/native-capture.md`. A página mostra o toast "A tela tá saindo preta — tenta deixar o jogo em modo janela ou sem bordas."
- [x] T547 [P] Rodar `/security-review` focado na capability remota, no `origin_ok`, nos buffers compartilhados e nos comandos. Corrigir o que aparecer.
- [x] T548 Gate: `npm run check`, `npm test` e `npm run build` verdes; `cargo test` e `cargo clippy` em `src-tauri/` sem erros. Mocks `pickerwindows`, `pickerscreens`, `pickerempty`, `pickerfallback` e `livepc` abrem e batem com `tela-esperada.png`.
- [ ] T549 Medição de desempenho: a 1080p60 transmitindo uma janela de jogo, anotar a CPU do Espia e os quadros descartados por segundo (log `[espia] captura nativa:`). Se a cópia NV12 pesar (> ~10% de CPU ou descarte constante), registrar e abrir o plano B (H.264 + WebCodecs, research R5) como tarefa nova.
- [x] T550 Subir a versão para `0.1.3` (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`). Rodar `npm run desktop:build` e anotar o tamanho de `Espia_0.1.3_x64-setup.exe` em relação aos 1,13 MB da v0.1.2. A referência é ~+5 MB, flexível (SC-506).
- [x] T551 Publicar o site (`npm run deploy`) **antes** de distribuir o app: o app carrega o site publicado.
- [ ] T552 Roteiro manual N1–N8 do `quickstart.md` com a galera (Discord ligado, dois monitores, Windows 10 se der).
- [ ] T553 Commit + tag `v0.1.3` (o workflow Release gera o instalador). Escrever `specs/005-seletor-nativo-desktop/relatorio.md` no formato do da 004: o que foi feito, diferenças em relação ao design, testes, tamanho e o que depende da pessoa usuária.

---

## Dependencies & Execution Order

- **Setup (T501–T504)** → **Foundational (T505–T512)** → histórias.
- **US1 (T513–T524)** é o MVP e base das demais: US2, US3 e US5 usam a sessão e a grade dela.
- **US2** depende de US1 (T521/T522).
- **US3** depende de US1 (vídeo) e de US2 (T525 `procs.rs`, T527/T528 áudio).
- **US4** pode rodar logo depois da Foundational (é revisão e fallback), mas o fechamento (T542) vem depois de US1.
- **US5** depende de US1.
- **Polish** depois de todas. T551 (deploy) antes de T552/T553.

Ordem sugerida: T501–T512 → T513–T515 (visual) → T516–T524 → T525–T532 → T533–T539 → T540–T542 → T543–T544 → T545–T553.

## Parallel Opportunities

- **Setup**: T502, T503 em paralelo; T504 depois de T502.
- **Foundational**: T505, T506, T507 e T508 em paralelo; T509 (Rust) em paralelo com T510/T511 (TS).
- **US1**: o visual (T513–T515) em paralelo com o Rust (T516, T517); T519 em paralelo com T522 (lados diferentes do contrato).
- **US2**: T525 e T527 (funções puras) em paralelo; T532 em paralelo com T531.
- **US3**: T533, T535 e T539 em paralelo.

Exemplo (US1): uma frente faz `SourceCard.svelte` + mocks (T513, T515), outra faz `sources.rs` + `thumbs.rs` (T516, T517). Elas se encontram em T518/T514.

## Implementation Strategy

1. **MVP = Setup + Foundational + US1**: a grade real e a transmissão de janela sem o seletor do Windows, ainda sem som. Validar N1–N3 (sem som). Dá para publicar como 0.1.3-beta só para teste interno.
2. **+ US2**: fecha todo o P1 (som só da janela + selo). É a primeira versão que dá para entregar à galera.
3. **+ US3**: telas inteiras com som do PC menos o Discord.
4. **+ US4/US5 + Polish**: robustez, conforto, medição, release 0.1.3 e relatório.
