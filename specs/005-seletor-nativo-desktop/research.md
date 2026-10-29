# Research — 005 Seletor de tela nativo no app desktop

Data: 2026-10-29. Contexto: a [spec](./spec.md) pede grade de miniaturas, captura sem o seletor do Windows, selo "tocando som", som só do programa da janela e tela inteira com o som do PC **menos o Discord**. Tudo isso só no app desktop.

## R1 — A página dentro do WebView2 consegue escolher a fonte do `getDisplayMedia`?

- **Decisão**: não. A captura sai do `getDisplayMedia` e passa a ser feita pelo **app** (Rust). A página recebe os quadros prontos.
- **Por quê**: o evento `ScreenCaptureStarting` do WebView2 só tem `Cancel`, `Handled`, `OriginalSourceFrameInfo` e `GetDeferral`, e nada que escolha a fonte ([doc](https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2screencapturestartingeventargs)). Os ids `chromeMediaSourceId` só valem para extensões ou Electron. A opção `--auto-select-desktop-capture-source` (já usada nos testes de debug) vale para o processo inteiro e é fixada ao abrir o app, então não serve para uma escolha por clique.
- **Alternativas rejeitadas**:
  - **Electron** (`desktopCapturer` + `setDisplayMediaRequestHandler`): o instalador passaria de ~100 MB (SC-506 pede no máximo +5 MB). Além disso, o som de tela inteira no Electron é o loopback do PC inteiro, com o Discord junto.
  - **Recriar o WebView2 com outro `--auto-select…` a cada escolha**: derruba a sala (WebSocket e malha P2P) a cada transmissão.
  - **Fazer o WebRTC inteiro no Rust**: duplica toda a malha (`mesh.ts`/`peer.ts`). É grande demais.

## R2 — Listar janelas e monitores

- **Decisão**:
  - **Janelas**: `EnumWindows`, filtrando as visíveis, com título, sem `WS_EX_TOOLWINDOW`, não "cloaked" (`DwmGetWindowAttribute(DWMWA_CLOAKED)`, que esconde as janelas UWP suspensas e as de outros desktops virtuais), sem dono e fora do processo do Espia.
  - **Programa**: `QueryFullProcessImageNameW` + `FileDescription` da versão do executável (cai para o nome do `.exe`).
  - **UWP**: janela do `ApplicationFrameHost` → processo da `CoreWindow` filha.
  - **Monitores**: `EnumDisplayMonitors` + `GetMonitorInfoW` (principal = `MONITORINFOF_PRIMARY`), com o nome "Tela 1/2…" pela ordem.
- **Ícone**: `WM_GETICON` (`ICON_BIG`), depois `GetClassLongPtrW(GCLP_HICON)`, depois o ícone do executável. Vira PNG 32×32 por WIC e é guardado em cache por processo.
- **Alternativa rejeitada**: o crate `xcap`. Traz dependências de imagem pesadas, e precisamos do HWND e do PID de qualquer jeito.

## R3 — Miniaturas a cada 2 s

- **Decisão**:
  - **Janelas**: `PrintWindow(hwnd, PW_RENDERFULLCONTENT)` num DIB, reduzido para 320×180 (cabendo na proporção) e codificado em JPEG por **WIC**, que é nativo do Windows e não aumenta o binário. Vai para a página como `data:image/jpeg;base64` (a CSP do site já libera `img-src data:`).
  - **Monitores**: `BitBlt` do DC da tela, pelo mesmo caminho.
  - **Janela minimizada** ou captura preta: miniatura nula, e a página mostra o ícone grande no lugar (edge case da spec).
- **Por quê**: `PrintWindow` com `PW_RENDERFULLCONTENT` captura janelas desenhadas por DirectX e não abre uma sessão de captura (nem borda amarela) por item. Dez miniaturas a cada 2 s custam pouca CPU.
- **Alternativa rejeitada**: uma sessão WGC por janela para as prévias. É pesado e, no Windows 10, desenha a borda amarela em todas as janelas.

## R4 — Captura de vídeo da fonte escolhida

- **Decisão**: **Windows.Graphics.Capture** (WGC), criado por `IGraphicsCaptureItemInterop::CreateForWindow/CreateForMonitor`, com `Direct3D11CaptureFramePool` (BGRA) e cursor ligado.
  - Cada quadro é reduzido para a resolução pedida (720/1080 de altura, mantendo a proporção) e convertido para **NV12** na GPU (`ID3D11VideoProcessor`).
  - Depois é copiado para uma textura de staging e escrito no buffer compartilhado (R5).
  - O FPS é limitado no Rust: descarta os quadros que chegam antes de 1/fps.
  - **Borda amarela**: no Windows 11, `IsBorderRequired = false` (após `GraphicsCaptureAccess.RequestAccessAsync(Borderless)`). No Windows 10 a borda aparece, como já aparece hoje com o Chrome.
  - Fim da fonte (janela fechada) → `GraphicsCaptureItem.Closed` → evento `ended` para a página, que encerra a transmissão.
- **Requisito**: Windows 10 1903+. Em versões mais antigas, o `capture_supported` responde `false` e a página usa o fluxo da v0.1.2 (FR-511).

## R5 — Levar os quadros do Rust até a página

- **Decisão**: **buffers compartilhados do WebView2** (`ICoreWebView2Environment12::CreateSharedBuffer` + `ICoreWebView2_17::PostSharedBufferToScript`, WebView2 1.0.1661+) ([doc](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_17)).
  - Três slots NV12 (anel), postados uma vez no início com `additionalDataAsJson` (largura, altura, tamanho, índice).
  - A cada quadro, o Rust escreve no próximo slot livre e avisa por um **`tauri::ipc::Channel`**: `{slot, seq, ts}`, ~60 mensagens pequenas por segundo.
  - A página faz `new VideoFrame(buffer, {format: 'NV12', codedWidth, codedHeight, timestamp})`, que copia de forma síncrona e libera o slot. Depois escreve num `MediaStreamTrackGenerator({kind: 'video'})` (Chromium, disponível no WebView2).
  - O track resultante entra no `MediaStream` que hoje sai do `getDisplayMedia`. A malha P2P não muda.
- **Custo**: 1080p NV12 = 3,1 MB por quadro, ou ~187 MB/s a 60 fps de cópia de memória dentro do mesmo PC. O WebRTC continua codificando como hoje.
- **Mudança de qualidade no ar** (`constrainCapture`): o `applyConstraints` não age num track gerado. A página chama `capture_set_quality`, e o Rust recria os slots no novo tamanho e posta de novo.
- **Alternativas rejeitadas**:
  - **Quadros pelo IPC do Tauri** (`Channel` binário): passa por requisição de protocolo customizado e não aguenta ~187 MB/s.
  - **Codificar H.264 no Rust (Media Foundation) e decodificar com WebCodecs**: codifica duas vezes, tem mais latência, mais código e depende de codec de hardware. Fica de plano B se a cópia NV12 pesar em PCs fracos.
  - **WebSocket local**: precisaria abrir uma porta e liberar o `connect-src` na CSP.

## R6 — Som da janela escolhida (só o programa dela)

- **Decisão**: **loopback de processo do WASAPI** (`ActivateAudioInterfaceAsync` com `VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK` e `AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS`).
  - Modo **`INCLUDE_TARGET_PROCESS_TREE`**, apontando para a **raiz da árvore** do processo da janela (sobe pelos pais enquanto o executável for o mesmo). Isso cobre os navegadores, cujo som sai de um processo filho.
  - O formato é PCM float32, 48 kHz, estéreo, em pedaços de 10 ms enviados por `Channel` binário (3,8 KB por pedaço, o que é tranquilo).
  - A página monta um `AudioData` e escreve num `MediaStreamTrackGenerator({kind: 'audio'})`.
- **Versão do Windows**: a doc lista o build 20348 como mínimo ([doc](https://learn.microsoft.com/en-us/windows/win32/api/audioclientactivationparams/ns-audioclientactivationparams-audioclient_process_loopback_params)). O exemplo oficial "Application Loopback" roda no Windows 10 2004 (19041). Se a ativação falhar, a transmissão vai **sem som** e o app avisa ("Seu Windows não deixa separar o som da janela"). Nunca cai para o som do PC inteiro.

## R7 — Tela inteira com som do PC menos o Discord (Q1: B)

- **Decisão**: loopback de processo em modo **`EXCLUDE_TARGET_PROCESS_TREE`** apontando para a raiz da árvore do Discord ([doc](https://learn.microsoft.com/en-us/windows/win32/api/audioclientactivationparams/ne-audioclientactivationparams-process_loopback_mode)).
  - **Raízes do Discord**: processos `Discord.exe`, `DiscordPTB.exe` e `DiscordCanary.exe` cujo pai não tem o mesmo nome.
  - **Sem Discord aberto**: exclui uma árvore inexistente (PID 0 não serve, então usa o PID do próprio Espia). O resultado é o som do PC inteiro.
  - **Uma raiz**: exclui essa raiz.
  - **Mais de uma raiz** (por exemplo, Discord e Discord PTB juntos): o modo só aceita uma exclusão, então a regra de segurança da FR-509 vale: **sem som**, com o aviso "Mais de um Discord aberto — tela inteira vai sem som."
  - **Discord abre ou reinicia no meio**: um vigia confere as raízes a cada 1 s. Se elas mudarem, o app recria o cliente de áudio com a nova exclusão. Pode haver um corte de ~50 ms no som, e é aceito. O Discord leva vários segundos entre abrir e entrar numa chamada, então 1 s não deixa vazar voz.
- **Alternativa rejeitada**: incluir só a árvore do processo em primeiro plano. Não é o que a pessoa escolheu (Q1: B).
- **Novo `AudioKind`**: `'pc'` ("com som do PC"), aceito pelo Worker. Quem assiste vê "· com som".

## R8 — Selo "tocando som"

- **Decisão**: `IMMDeviceEnumerator` → dispositivo de saída padrão → `IAudioSessionManager2::GetSessionEnumerator`. Para cada sessão em `AudioSessionStateActive` com pico (`IAudioMeterInformation::GetPeakValue`) acima de 0,001, o app pega o PID, sobe até a raiz da árvore e marca as janelas da mesma raiz. É recalculado junto com as miniaturas (a cada 2 s).
- **Limite**: só o dispositivo de saída padrão. Quem manda um programa para outra saída não vê o selo, mas o som da janela continua indo (o loopback de processo não depende de dispositivo).

## R9 — Expor isso à página com segurança

- **Decisão**: comandos Tauri (`#[tauri::command]`) liberados **só** para a origem `https://espia.espia.workers.dev` (e `http://127.0.0.1:*` em debug), na janela `main`, por uma capability com `remote.urls`. A lista de comandos é registrada no `build.rs` (`AppManifest::commands`). Nenhum plugin de sistema de arquivos ou shell entra.
  - Do lado do site, a dependência é `@tauri-apps/api` (só `core`: `invoke` + `Channel`, alguns KB). A detecção é `'__TAURI_INTERNALS__' in window` + `capture_supported()`. Um app antigo ou um navegador comum → `false` → fluxo v0.1.2.
  - Os buffers compartilhados vão só para o frame principal da janela `main`. O Rust confere a origem da página (`webview.url()`) antes de postar.
- **Constituição**: exige a emenda 1.2.0, que troca "nenhuma API nativa exposta à página" por "só a ponte de captura (contracts/native-capture.md), só para a origem do site".

## R10 — Tamanho do instalador

- **Decisão**: só os crates `windows` (com mais features) e `webview2-com` (já usados), além de `base64`. Não entra crate de imagem nem de codec (o WIC faz o JPEG).
- **Estimativa**: +300–600 KB no binário. A meta SC-506 (≤ +5 MB) tem folga. A tarefa de build mede o tamanho real.

## R11 — Testes

- **Rust** (`cargo test`):
  - o filtro de janelas, por uma função pura sobre os atributos;
  - as raízes do Discord, por uma função pura sobre a tabela de processos;
  - a escolha do modo de loopback;
  - o cálculo de tamanho e proporção.
- **Vitest**:
  - `native-capture.ts`, com a ponte falsa: montagem do track, fallback e mudança de qualidade;
  - a regra de som nova em `audio-source.ts`;
  - o Worker aceitando `'pc'`.
- **Mocks** (princípio V): `?mock=room&scene=pickerwindows`, `pickerscreens`, `pickerempty`, `pickerfallback`, com miniaturas falsas.
- **Manual** ([quickstart.md](./quickstart.md)): captura real, Discord, dois monitores e o tamanho do instalador. A captura real não é automatizada (ela capturaria a tela real; ver a memória do projeto).
