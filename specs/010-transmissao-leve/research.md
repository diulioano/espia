# Research: Transmissão leve (010)

Base: [relatório da 009](../009-memoria-do-app/relatorio.md) (app 0.1.6, seletor nativo, 1080p60, 3 pessoas).

## R1 — Diagnóstico antes de corrigir

- **Decision**: build de debug com `ESPIA_BROWSER_ARGS=--remote-debugging-port=9222` (infra já usada na 005–007) e Playwright `connectOverCDP` num script do scratchpad. Transmissão nativa com som (janela de teste "espia-som-teste", como na 007), sem ninguém assistindo e depois com 1 aba de navegador assistindo. Medir:
  1. **Memória**: duas fotos do heap (`HeapProfiler.takeHeapSnapshot`) com 2 min de intervalo, comparando o que cresceu (tipo e quem segura). Mais `Performance.getMetrics` (`JSHeapUsedSize`) a cada 5 s, com e sem som, para separar o heap do JavaScript da memória nativa da página.
  2. **CPU**: um *trace* de 10 s (`Tracing.start`, categorias `v8`, `devtools.timeline`, `webrtc`, `media`) para ver quanto vai para: eval de mensagens do canal, `new VideoFrame` (cópia), conversão e compressão por pessoa.
  3. **Upload**: `RTCPeerConnection.getStats()` de cada conexão (`outbound-rtp`: `bytesSent`, `targetBitrate`, `qualityLimitationReason`, `framesPerSecond`, `frameHeight`, `nackCount`, `pliCount`, `keyFramesEncoded`) a cada 1 s.
- **Rationale**: a medição da 009 mostra **onde** está o problema (a página, com som), mas não **o quê**. Corrigir sem saber pode errar o alvo.

## R2 — Som da ponte nativa: o caminho de hoje (suspeito principal da US1)

- Rust (`capture/audio.rs`): pedaços de **480 quadros** (10 ms, 2 canais, f32) = **3.840 bytes**, `Channel::send(InvokeResponseBody::Raw)` **100 vezes por segundo**.
- Tauri 2.12 (`ipc/channel.rs`): `Raw` ≥ 1.024 bytes vai pelo caminho *fetch*. Para **cada** pedaço:
  - `webview.eval("…invoke('plugin:__TAURI_CHANNEL__|fetch', …).then(…runCallback…)")`, ou seja, um script novo compilado e executado por pedaço;
  - um pedido IPC (protocolo `ipc.localhost`) para buscar os bytes;
  - a resposta vira `ArrayBuffer` → `soundBytes` → `Float32Array` → `AudioData` → `audioWriter.write(chunk)`, **sem esperar** (`.catch(() => {})`).
- O vídeo usa buffers compartilhados (3 slots, sem cópia no IPC) e só manda um JSON pequeno por quadro, que é direto por `eval`. **Ele descarta quadros quando ocupado** (`busy`). O som não descarta.
- **Hipóteses (R1 confirma qual)**:
  - **H1**: a fila do `WritableStream` do gerador de som cresce, porque ninguém espera o `write` e não há descarte. Isso causaria crescimento linear, que bate com o padrão medido.
  - **H2**: custo por mensagem no motor (100 `eval` com texto único + 100 pedidos IPC por segundo): scripts compilados ou respostas guardadas por mais tempo que o esperado.
  - **H3**: carimbo de tempo do som (`audioTs`, contado pelos quadros) adianta em relação ao relógio de som do motor, e o motor guarda o excesso.
- **Decision (correção, conforme R1)**:
  - **Sempre**: pedaços de **40 ms** (1.920 quadros, 15.360 bytes) em vez de 10 ms → 4× menos mensagens, `eval` e pedidos IPC. O atraso extra (~30 ms) é aceitável para transmissão de jogo.
  - **Sempre**: na página, **descartar** pedaço de som se a fila do gerador não estiver vazia (`writer.desiredSize <= 0`) ou se ele chegar atrasado mais de 200 ms em relação ao relógio de parede; contar descartes no log de 10 s que já existe.
  - **Se H3**: carimbo de tempo pelo relógio de parede (`performance.now()`), não pela soma dos quadros.
  - **Se H2 e o lote de 40 ms não bastar**: som por um anel de buffer compartilhado, como o vídeo. Mais trabalho, fica como plano B.
- **Contrato**: muda o tamanho do pedaço e a regra de descarte dentro da ponte de captura existente ([contracts/native-capture-delta.md](./contracts/native-capture-delta.md)). Nenhum comando novo, nenhuma emenda à constituição.

### R2 — Resultado (2026-10-29)

Medido (ver `relatorio.md`): a causa é o **`eval` por mensagem** do canal do Tauri (H2), não a fila (H1) nem a deriva (H3). O lote de 40 ms só dividiria o problema por 4, e o base64 por `eval` piorou (~70 MB/min). **Decisão final**: som (10 ms, base64) e avisos de quadro vão pelo canal de mensagens do WebView2 (`ICoreWebView2::PostWebMessageAsJson` → `chrome.webview` "message"). Nenhum script é executado por mensagem. Página estável em 4 min com som. O descarte por fila (> 200 ms) ficou como proteção.

## R3 — Upload acima do teto (US2)

- **Fato**: a coluna "Rede" do Gerenciador soma **envio e recebimento**. Os 45 Mbps podem incluir vídeo de outra pessoa que você estava assistindo. R1 mede só o `bytesSent` de cada conexão, que é a régua da SC-1003.
- **Hipóteses**:
  - **U1**: `setEncoding` retorna cedo quando ainda não há `encodings` (antes da primeira negociação) e só aplica o teto quando a sinalização assenta. Nesse meio tempo, o motor usa o próprio padrão, que pode ser alto para tela.
  - **U2**: perdas na rede → muitos quadros-chave (`pliCount`, `keyFramesEncoded`), que são grandes.
  - **U3**: troca de miniatura → tela cheia aplica o teto um pouco depois.
- **Decision**:
  - Aplicar o teto **no momento de adicionar a faixa**, via `addTransceiver(track, { sendEncodings: [{ maxBitrate, maxFramerate, scaleResolutionDownBy }] })`, em vez de depois da negociação (U1 e U3);
  - reaplicar sempre que `signalingState` volta a `stable` (já existe);
  - registrar `pliCount` e `keyFramesEncoded` no log de 10 s (U2: se for perda, o teto continua valendo na média e o pico é curto; nada a mudar além de medir).

## R4 — CPU por quadro na captura nativa (US3, FR-1004)

Caminho de hoje por quadro (1080p60):
1. GPU: BGRA → NV12 (shader) → textura *staging* → `Map` → **cópia 1** para o slot do buffer compartilhado (Rust).
2. Página: `new VideoFrame(new Uint8Array(slot…))` → **cópia 2** (~3 MB por quadro, ~180 MB/s em 60 fps).
3. Motor: para cada pessoa, o compressor converte e redimensiona o quadro (por pessoa).

- **Decision (ordem, medir cada passo com o trace de R1)**:
  - **a)** A cópia 2 não dá para evitar: o slot é um buffer compartilhado com o app e é reescrito depois. Então o caminho viável é **copiar menos vezes**:
    - capturar no tamanho de envio (já é);
    - **não mandar quadros repetidos**: se a imagem não mudou (a WGC entrega por mudança, mas em tela cheia o Windows repete quadros), o Rust compara um *hash* barato de amostras do quadro e pula a cópia 1, o aviso e a cópia 2. Em cenas paradas (menu, desktop) isso corta boa parte da CPU.
  - **b)** **Ritmo exato**: o Rust já limita a `fps`. Garantir que nunca entra mais de 1 quadro por intervalo (hoje 90 % do intervalo) e que a página descarta antes da cópia (`busy`), não depois.
  - **c)** Quadros com o `contentHint` certo ('motion' em 60 fps) já existem; nada a mudar.
- **Meta**: SC-1004 (≤ ~22 % com 3 pessoas). Se a) e b) não chegarem lá, o ajuste automático (R5) é a rede de segurança. Nenhuma mudança no modelo sem servidor de mídia.
- **Alternatives**: comprimir uma vez só e repassar (exige servidor de mídia, princípio II, fora).

## R5 — Ajuste automático de qualidade por CPU (FR-1005 a FR-1008)

- **Decision**: a cada 1 s, ler `getStats()` dos envios em tela cheia. "No limite" = **alguma** conexão em tela cheia com `qualityLimitationReason === 'cpu'`. Contadores: 10 s seguidos no limite → desce um degrau; 60 s seguidos sem nenhuma em `'cpu'` → sobe um degrau, até a escolhida.
- **Degraus** (pura, testável): `1080p60 → 1080p30 → 720p30`; `720p60 → 720p30`; mínimo `720p30`.
- **Combinação**: `effectiveQuality(chosen, fullViewers)` ganha um terceiro parâmetro, `cpuStep`, e vale o mais baixo. Aplica pelo caminho que já existe (`applyQuality`: captura nativa `setQuality`, `applyConstraints` no navegador, `mesh.setQuality`, aviso `quality` para a sala).
- **Sem aviso** (spec, Clarifications): o indicador existente (`adjusted` em `room.svelte.ts`) já marca quando a qualidade efetiva difere da escolhida.
- **Rationale**: o próprio motor mede se está limitando por CPU. Sem medir a CPU do PC, sem API nativa nova.
- **Troca na mão**: zera `cpuStep` e os contadores.

## R6 — Fora de escopo

- Trocar a compressão de vídeo usada (decisão anterior da pessoa usuária: manter o padrão do navegador).
- Servidor de mídia (princípio II).
