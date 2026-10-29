# Tasks: Transmissão leve

**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/native-capture-delta.md](./contracts/native-capture-delta.md), [quickstart.md](./quickstart.md)

**Testes**: o plano pede Vitest para as regras puras (`quality.ts`, `cpu-governor.ts`, descarte de som) e `cargo test` para o Rust. O resto é o diagnóstico R1 e o roteiro T1–T6.

**Ordem de publicação**: o site pode subir antes do app 0.1.7 (contrato compatível nos dois sentidos).

## Phase 1: Setup — diagnóstico (R1)

- [x] T1001 Criar no scratchpad `diag-010.mjs` (playwright-core + `connectOverCDP('http://127.0.0.1:9222')`, como os scripts da 007). Na página do Espia ele deve:
  - a cada 5 s, gravar `Performance.getMetrics` (`JSHeapUsedSize`, `JSHeapTotalSize`, `Nodes`, `JSEventListeners`) num CSV;
  - via `page.evaluate`, injetar um coletor de `getStats()` de todas as `RTCPeerConnection` (guardar referências com um *wrap* de `RTCPeerConnection` antes da página carregar, `addInitScript`) e gravar, a cada 1 s, `outbound-rtp` de vídeo: `bytesSent` (delta → Mbps), `targetBitrate`, `frameHeight`, `framesPerSecond`, `qualityLimitationReason`, `pliCount`, `nackCount`, `keyFramesEncoded`;
  - comandos `heap <arquivo>` (`HeapProfiler.takeHeapSnapshot` para arquivo) e `trace <s>` (`Tracing.start` com categorias `v8,devtools.timeline,webrtc,media,disabled-by-default-v8.cpu_profiler` → arquivo JSON).
- [x] T1002 Rodar o diagnóstico no build de debug (`$env:ESPIA_BROWSER_ARGS='--remote-debugging-port=9222'; npm run desktop:dev`):
  - transmitir a janela de teste "espia-som-teste" (som, como na 007) pelo seletor nativo em 1080p60, com 1 aba do Edge assistindo no site;
  - 5 min com som e 5 min sem som;
  - duas fotos do heap com 2 min de intervalo (com som);
  - um trace de 10 s.
  - Anotar em `specs/010-transmissao-leve/relatorio.md`: o crescimento do `JSHeapUsedSize` com e sem som (o heap do JS cresce, ou só a memória nativa da página?); os 5 tipos que mais cresceram entre as fotos; a divisão da CPU no trace (eval/IPC, `VideoFrame`, compressão); e o envio real por conexão.
  - Se a causa não for a do R2 (H1–H3), registrar em `research.md` e ajustar as tarefas da US1 antes de seguir.

## Phase 2: Foundational

- [x] T1003 [P] Em `src/lib/quality.ts`:
  - `export function stepDown(q: { resolution; fps }): { resolution; fps }` e `stepUp(q, chosen)`, conforme `data-model.md`: "1080p60 → 1080p30 → 720p30; 720p60 → 720p30; 720p30 → 720p30"; `stepUp` é o inverso e nunca passa de `chosen`;
  - `effectiveQuality(chosen, fullViewers, cpuSteps = 0)`: aplica a regra de espectadores e depois `cpuSteps` vezes `stepDown`; o `maxBitrate` sai da tabela `BITRATE` da qualidade final, limitado pelo teto de espectadores;
  - manter `isAdjusted` como está.
- [x] T1004 [P] Testes em `test/lib/quality.test.ts`:
  - `stepDown` em cada qualidade, `stepUp` limitado por `chosen`;
  - `effectiveQuality` com `cpuSteps` 0, 1 e 2 combinado com 3, 5 e 7 espectadores (vale o mais baixo);
  - `isAdjusted` verdadeiro quando a CPU baixou.

**Checkpoint**: `npm test` passa.

## Phase 3: User Story 1 — Transmitir por horas sem a memória subir (P1) 🎯 MVP

**Goal**: com som pela ponte nativa, a memória cresce no máximo 3 MB/min; som sincronizado.

**Independent Test**: quickstart T1 e T2.

- [x] T1005 [P] [US1] *(Trocado depois do diagnóstico: os pedaços continuam em 10 ms, e o som e os avisos de quadro saem do canal do Tauri para o canal de mensagens do WebView2, `PostWebMessageAsJson`, em `src-tauri/src/capture/mod.rs` (`SoundSink`, `FrameSink`). Ver `relatorio.md`.)* ~~Em `src-tauri/src/capture/audio.rs`, `CHUNK_FRAMES` de 480 para **1920** (40 ms), com o comentário citando specs/010. Atualizar o teste Rust que depender do tamanho e acrescentar um que confere `CHUNK_FRAMES * CHANNELS * 4 == 15_360` bytes.
- [x] T1006 [US1] Em `src/lib/native-capture.ts`, no canal `sound`, conforme `contracts/native-capture-delta.md`:
  - antes de criar o `AudioData`, descartar o pedaço se `audioWriter.desiredSize !== null && audioWriter.desiredSize <= 0` (fila) ou se ele chegou com mais de **200 ms** de atraso em relação ao relógio (`performance.now()` comparado ao início da sessão + `audioTs`);
  - se a causa no relatório for deriva (H3), usar o relógio como carimbo de tempo (`audioTs = (performance.now() - t0) * 1000`);
  - aceitar pedaços de qualquer tamanho múltiplo de 2 canais (o app 0.1.6 manda 10 ms);
  - *(feito: descarte por fila acima de 200 ms, `shouldDropSound`; o som e os avisos de quadro chegam por `bridge.onHostMessage` (chrome.webview "message"), com os canais do Tauri mantidos para apps antigos)*
  - contar `somDescartado` no log `[espia] captura nativa:` de 10 s.
  - Extrair a decisão de descarte para uma função pura exportada, `shouldDropSound(desiredSize, lateMs)`.
- [x] T1007 [P] [US1] Testes em `test/lib/native-capture.test.ts`:
  - `shouldDropSound` (fila cheia, atraso de 199 e de 201 ms, `desiredSize` nulo);
  - com a ponte falsa que já existe: pedaços de 10 ms e de 40 ms viram `AudioData` com `numberOfFrames` 480 e 1920;
  - com o *writer* cheio, os pedaços são descartados e contados.
- [x] T1008 [US1] Medir com `diag-010.mjs` e `npm run memoria` no debug (T1 com a janela de teste, 15 min). Se o crescimento passar de 3 MB/min, aplicar o plano B do R2 (som por anel de buffer compartilhado) e registrar. Anotar em `relatorio.md`.
- [ ] T1009 [US1] `npm run check`, `npm run build`, `npm test` e deploy do site (`! npm run deploy`, pela pessoa usuária). Compatível com o app 0.1.6.

**Checkpoint**: SC-1001 no debug.

## Phase 4: User Story 2 — O upload nunca passa do teto (P2)

**Goal**: envio por conexão dentro do teto desde o primeiro quadro.

**Independent Test**: quickstart T3.

- [x] T1010 [US2] Em `src/lib/peer.ts`, `attach()`: para `kind === 'video'`, criar o transceiver com `sendEncodings: [sendParams(quality, this.theirView)]` (`active`, `maxBitrate`, `maxFramerate`, `scaleResolutionDownBy`), para que o teto valha antes da negociação (research R3, U1/U3). Passar a `quality` atual ao `attach`. O `applyEncodings` continua reaplicando quando volta a `stable`.
- [x] T1011 [US2] Em `src/lib/peer.ts`, `sendStats(): Promise<{ bytesSent, limitation, pli, keyFrames } | null>`, lendo `outbound-rtp` de vídeo do `this.video?.sender.getStats()`. Em `src/lib/mesh.ts`, `sendStats()` que junta por pessoa. O store (T1015) usa isso a cada 1 s, e um `console.debug('[espia] envio', …)` a cada 10 s soma Mbps por pessoa, os limites e o `pliCount`.
- [x] T1012 [US2] *(Feito no debug com 3 abas assistindo: log `[espia] envio` a cada 10 s, ~30 fps por pessoa, sem limite por CPU, 1,3–2,7 Mbps no total, porque a janela de teste é simples. O pico de 45 Mbps só se confere com o jogo real, no T1018.)* Medir T3 com `diag-010.mjs` (3 abas assistindo, entrando, saindo e trocando de miniatura para tela cheia). Se ainda houver pico acima de 21 Mbps, olhar `pliCount`/`keyFramesEncoded` (U2) e registrar em `relatorio.md`.

**Checkpoint**: SC-1003.

## Phase 5: User Story 3 — Menos CPU e ajuste automático (P3)

**Goal**: CPU da página ≤ ~22 % com 3 pessoas em 1080p60; ajuste automático sem aviso.

**Independent Test**: quickstart T4–T6.

- [x] T1013 [P] [US3] *(Descartado depois de ler o código: a captura de janela do Windows só entrega quadro quando a imagem muda, então há pouco quadro repetido para pular, e comparar por amostras arriscaria perder mudança pequena, como texto. O ritmo já nunca passa do fps. O ganho de CPU do lado do app veio da troca de canal da US1: 60 avisos por segundo deixaram de executar script na página.)* ~~ Em `src-tauri/src/capture/video.rs`, no laço de quadros:
  - (a) quadro igual: antes de copiar o *staging* para o slot, calcular um *hash* barato (ex.: FNV-1a de 1 a cada 64 bytes do plano Y) e, se for igual ao último enviado e o último envio foi há menos de 1 s, pular a cópia e o aviso à página (manda pelo menos 1 quadro por segundo, para quem entra ver a imagem);
  - (b) ritmo: trocar `interval * 9 / 10` por um agendamento de prazo (`next_due += interval`, sem acumular atraso), para nunca passar de `fps`.
  - Testes Rust da função de *hash* e da regra "pular se igual e < 1 s".~~
- [x] T1014 [P] [US3] Criar `src/lib/cpu-governor.ts` conforme `data-model.md` (`CpuGovernor` com `steps`, `limitedFor`, `freeFor`; `tick(limited, canGoDown, canGoUp): 'down' | 'up' | null`; `reset()`; 10 s para descer e 60 s para subir) e `test/lib/cpu-governor.test.ts`:
  - desce depois de 10 ticks no limite e não antes;
  - um tick folgado zera a contagem;
  - sobe depois de 60 folgados;
  - não passa de 2 degraus;
  - `reset`.
- [x] T1015 [US3] Em `src/lib/stores/room.svelte.ts`:
  - `cpuSteps = $state(0)` e um `CpuGovernor`;
  - o `quality` derivado passa a ser `effectiveQuality(this.myBroadcast, fullViewers, this.cpuSteps)`;
  - enquanto `myBroadcast` existe, um `setInterval` de 1 s: `mesh.sendStats()` → `limited` = alguma pessoa em tela cheia com `limitation === 'cpu'` → `governor.tick(limited, stepDown(q) ≠ q, cpuSteps > 0)` → em `'down'`/`'up'`, ajustar `cpuSteps` (o efeito que já existe chama `applyQuality`);
  - sem pessoas em tela cheia, não chamar `tick`;
  - zerar (`reset` + `cpuSteps = 0`) ao trocar a qualidade na mão e em `stopBroadcast`;
  - **sem toast**: o indicador `adjusted` já mostra a mudança;
  - o mesmo timer faz o log de envio do T1011.
- [ ] T1016 [US3] **precisa da pessoa usuária** (jogo real e 3 pessoas). T4–T6 do `quickstart.md` com o build de debug ou o 0.1.7. Anotar a CPU antes e depois em `relatorio.md`. Se não chegar a ~22 %, registrar o que o trace de R1 mostrou e decidir com a pessoa.

**Checkpoint**: SC-1004 e SC-1005.

## Phase 6: Polish

- [x] T1017 Versão 0.1.7 em `package.json`, `package-lock.json` (topo), `src-tauri/Cargo.toml` e `src-tauri/tauri.conf.json`; `cargo test`; `npm run tauri build`; medir o tamanho do instalador. *(Feito: `Espia_0.1.7_x64-setup.exe`, 1,62 MiB.)*
- [ ] T1018 **precisa da pessoa usuária**. Roteiro T1–T6 com o app 0.1.7 instalado e o site publicado. Números em `relatorio.md`.
- [x] T1019 [P] Atualizar o contrato da 005 (`specs/005-seletor-nativo-desktop/contracts/native-capture.md`) com o pedaço de som de 40 ms e o descarte, apontando para o delta da 010.

## Dependencies & Execution Order

- **Phase 1**: T1001 → T1002. O resultado pode mudar as tarefas da US1.
- **Phase 2**: T1003 → T1004 (`quality.ts`). Independente da Phase 1.
- **US1**: depende da Phase 1. T1005 ‖ T1006 → T1007 → T1008 → T1009.
- **US2**: depende da Phase 2 (`sendParams` com a qualidade efetiva). T1010 → T1011 → T1012. Pode andar em paralelo com a US1.
- **US3**: T1013 (Rust) e T1014 (governor) em paralelo; T1015 depende de T1003, T1011 e T1014; T1016 por último.
- **Polish**: depois das três histórias.

### Parallel examples

- T1003+T1004 (quality) ‖ T1001 (script de diagnóstico).
- US1: T1005 (Rust) ‖ T1006 (página).
- US3: T1013 (Rust) ‖ T1014 (governor).

## Implementation Strategy

1. **MVP = diagnóstico + US1**: acaba com o vazamento de ~750 MB/h. Sobe o site; o app 0.1.6 já ganha o descarte na página.
2. **US2**: upload no teto (pouca mudança, efeito grande na internet de quem joga).
3. **US3**: CPU e ajuste automático.
4. 0.1.7 e o roteiro.
