# Tasks: Reduzir atraso da imagem

**Input**: documentos de `specs/016-reduzir-atraso-imagem/`
**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/latency.md](./contracts/latency.md), [quickstart.md](./quickstart.md)
**Branch atual**: `main`. **Created**: 2026-11-04.

**Organization**: fases por história, em ordem US1 (P1), US2 (P1), US3 (P2). Os pontos de aceitação exigidos pela spec são obrigatórios. Não há fase autônoma de TDD; as regressões automatizadas do plano fazem parte da verificação das mudanças de comportamento.

## Format: `[ID] [P?] [Story] Description`

- `[P]`: trabalho em arquivos diferentes, que pode ocorrer junto de outra tarefa indicada na seção de paralelismo, após seus pré-requisitos.
- `[US1]`, `[US2]`, `[US3]`: vínculo com histórias da spec.
- Caminhos são relativos à raiz do repositório. Os arquivos novos estão indicados; não recriar o projeto, adicionar infraestrutura de produção ou publicar alterações nesta execução.
- Todos os itens começam desmarcados. Só concluir tarefas de medição com evidência real; indisponibilidade de aparelhos, Discord ou instrumento A/V deve constar como pendência, nunca como aprovação.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: preparar a régua reproduzível e o registro de ensaio, sem mudar comportamento da transmissão.

- [X] T001 Criar `specs/016-reduzir-atraso-imagem/relatorio.md` com inventário do ambiente e seção AcceptanceRun: build/revisão, aparelhos, versões de navegador/WebView2, origem, qualidade efetiva, som, espectadores, rota e intervalos de perturbação; respeitar "Sem conteúdo de mídia ou identidade das pessoas" e "Estados: baseline → after-change → pass/fail"; registrar dados ausentes como pendências.
- [ ] T002 [P] Criar `scripts/latency-source.html` com contador monotônico em milissegundos, movimento contínuo e flash/beep sincronizado iniciado por clique, sem dependência de rede ou gravação; conferir que funciona como janela/aba transmitida e referência para câmera conforme `specs/016-reduzir-atraso-imagem/quickstart.md`.
- [X] T003 Definir em `specs/016-reduzir-atraso-imagem/relatorio.md` o procedimento de coleta numérica a cada segundo, referência visual comum, cálculo nearest-rank de p95, máximo e contagem ≥ 5000 ms; registrar o instrumento A/V com resolução ≤ 20 ms, tratamento de lacunas e trechos estáveis definidos antes do ensaio, sem subtrair relógios de aparelhos diferentes.
- [ ] T004 Validar o setup de `specs/016-reduzir-atraso-imagem/quickstart.md` e registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` os resultados iniciais de `npm run check`, `npm test`, `npm run build`, acesso do app debug à origem local e disponibilidade de 1/3 espectadores; conferir `src-tauri/capabilities/capture.json` antes de atribuir resultados ao caminho nativo, sem ampliar permissões de produção, encerrar apps alheios ou fazer deploy.

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: observar a cadeia existente e preparar a linha de base. Ajuste de execução de 2026-11-04: T005–T007 bloqueiam código; T008 permanece obrigatório para aceitação, mas suas medições externas não bloqueiam correções locais reproduzíveis. Há apenas um ambiente conectado, sem espectadores reais/Discord/instrumento A/V. Não marcar T008 como concluída por testes simulados.

- [X] T005 Implementar `src/lib/media-stats.ts` (novo) com normalização por sessão/link/stats.id/ssrc dos deltas de encode, envio, jitter e decode, contadores e rota; aplicar "Todos os tempos normalizados em ms", "Delta só é válido com mesma identidade, tempo crescente e denominador positivo. Reset, ausência, inatividade e zero de amostras retornam null" e "Ring buffer de 60 amostras por link; coleta a cada segundo, no máximo uma consulta pendente"; verificar reset, campos ausentes e denominador zero em `test/lib/media-stats.test.ts` (novo), sem somar essas métricas como atraso ponta a ponta.
- [X] T006 Integrar coleta opcional inbound/outbound em `src/lib/peer.ts` e agregação em `src/lib/mesh.ts` usando T005; manter a API de stats usada pelo CPU governor, correlação local por identidade e descarte de resultado de link removido; garantir "exportação usa índice local de link, nunca nickname, código de sala ou identificador persistente" e rota "somente host/srflx/relay; sem endereço"; verificar adaptadores sem métricas em `test/lib/peer-latency.test.ts` (novo).
- [X] T007 Integrar a coleta em `src/lib/stores/room.svelte.ts` também para quem apenas assiste, reaproveitando o ciclo existente sem duplicar consultas de envio; liberar timers/rings/referências ao fechar/remover link ou encerrar sessão e disponibilizar inspeção/exportação manual local de números técnicos sem novo fluxo visual nem transmissão ao servidor; verificar que o governor continua com 10 s/60 s e que consulta pendente não bloqueia teardown.
- [ ] T008 Executar a linha de base T1/T2 do `specs/016-reduzir-atraso-imagem/quickstart.md` como baseline de aceitação: 30 min com 1/3 espectadores, som ligado/desligado, app e browser e comparação equivalente com Discord; correlacionar contador/flash-beep com T005–T007 em `specs/016-reduzir-atraso-imagem/relatorio.md`, separando reprodução, hipótese e ausência de evidência; atualizar `specs/016-reduzir-atraso-imagem/research.md` se a etapa responsável divergir da hipótese inicial.

**Checkpoint**: régua e diagnóstico disponíveis; linha de base registrada. Se não for possível executar algum ensaio, manter T008 pendente e resolver a limitação antes de atestar aceitação; não fabricar a reprodução dos 10 s.

## Phase 3: User Story 1 — Assistir perto do momento atual (Priority: P1) — MVP

**Goal**: corrigir o offset de abertura e favorecer mídia atual em condições estáveis, sem novos passos para assistir.

**Independent Test**: T1/T2 do quickstart, com contador em 30 min, 1/3 espectadores e comparação alternada com Discord; p95 ≤ 1000 ms, nenhuma amostra ≥ 5000 ms e diferença de p95 ≤ 500 ms. Inclui cenário de entrada/troca de modo de outro espectador.

### Implementation and validation

- [X] T009 [P] [US1] Criar a parte de vídeo de CaptureTiming em `src/lib/capture-timing.ts` com relógio monotônico injetável; aplicar sessionToken "identidade local única da execução, não enviada ao servidor", videoOffsetUs "null até primeiro frame válido; depois arrivalUs − nativeTsUs", lastNativeTsUs "timestamp relativo mais recente aceito" e lastVideoUs "último timestamp entregue ao gerador"; "Rejeitar timestamps/seq não finitos, negativos, regressivos ou de geração inválida. Mudança de resolução conserva época da sessão"; verificar startup de 2 s/10 s e monotonicidade em `test/lib/capture-timing.test.ts` (novo).
- [X] T010 [US1] Integrar T009 ao vídeo em `src/lib/native-capture.ts`, removendo a referência anterior ao invoke, copiando somente frame válido e mantendo uma escrita pendente; verificar startup demorado, mudança de geração, frame inválido, writer rejeitado e liberação de recursos em `test/lib/native-capture.test.ts`, mantendo gen/slot/seq/ts, três slots NV12 e formatos da ponte v1; não recarimbar cada mensagem atrasada como atual.
- [X] T011 [P] [US1] Aplicar a preferência `jitterBufferTarget=0` aos receivers de áudio/vídeo em ontrack de `src/lib/peer.ts`, com detecção de propriedade e tratamento de erro sem impedir recepção/negociação; verificar suporte, ausência e setter que lança em `test/lib/peer-latency.test.ts`, sem fallback não padronizado, edição de SDP ou reset periódico de ICE.
- [ ] T012 [US1] Conferir a integração de T010/T011 com `specs/016-reduzir-atraso-imagem/contracts/latency.md`, validando no `test/lib/native-capture.test.ts` os formatos base64/ArrayBuffer/lista e duração calculada pelo tamanho recebido, não por pacote fixo; registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` o comportamento com app instalado e app reconstruído, sem elevar versão mínima.
- [ ] T013 [US1] Executar T1 após a correção em `specs/016-reduzir-atraso-imagem/quickstart.md`, incluindo movimento contínuo, 1080p60 quando suportado e controle 720p30, 1/3 espectadores, som/sem som e entrada/mudança de visualização de um terceiro; registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` p95 ≤ 1000 ms, nenhuma amostra ≥ 5000 ms e manutenção do limite pelos demais espectadores (FR-001/002/007, SC-001).
- [ ] T014 [US1] Executar T2 de `specs/016-reduzir-atraso-imagem/quickstart.md` nas ordens Espia→Discord e Discord→Espia, sem transmissões simultâneas e com qualidade equivalente; registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` a diferença de p95 por par, aprovando SC-002 somente se ≤ 500 ms; revisar `specs/016-reduzir-atraso-imagem/research.md` e `specs/016-reduzir-atraso-imagem/plan.md` com evidências se T013/T014 falharem.

**Checkpoint**: MVP da US1 demonstrado por T013/T014. Isso não comprova recuperação após engasgo nem conclui a feature inteira.

## Phase 4: User Story 2 — Recuperar a imagem depois de um engasgo (Priority: P1)

**Goal**: descartar trabalho velho e voltar automaticamente ao momento atual após perturbações, com som sincronizado e encerramento terminal.

**Independent Test**: T3/T5 do quickstart; cinco repetições por tipo de perturbação de 10 s, retorno a atraso ≤ 1000 ms e A/V ≤ 200 ms em até 5 s após normalização, sem intervenção; cinco encerramentos sem reativação.

### Implementation and validation

- [X] T015 [P] [US2] Ampliar CaptureTiming em `src/lib/capture-timing.ts` com audioNextUs "próximo timestamp calculado por amostras, null sem áudio", lastAudioArrivalUs "chegada do último pacote de áudio válido", audioQueuedUs "duração das escritas pendentes do gerador atual", writerGeneration "invalida completions do writer de áudio anterior" e stopped "terminal; qualquer evento posterior é ignorado"; aplicar "Primeiro áudio começa no relógio local atual; duração = frames/48000 em microssegundos, com 2 canais. Aceitar somente chunks íntegros e não vazios" e "Lacuna ou desvio do áudio > 200 ms: descartar backlog controlado e reancorar no tempo local, sem reproduzir toda a lacuna"; verificar em `test/lib/capture-timing.test.ts` lacunas, retorno de áudio e timestamps crescentes.
- [X] T016 [US2] Integrar T015 aos writers em `src/lib/native-capture.ts`, garantindo "audio none → audio active: novo writerGeneration, audioNextUs reancorado. Completions do writer anterior não alteram fila ou estado do novo" e teardown idempotente; verificar em `test/lib/native-capture.test.ts` áudio interrompido, fila cheia, bursts, escrita pendente/rejeitada e stop, fechando objetos ainda pertencentes ao app e não reutilizando objetos consumidos pelo motor.
- [X] T017 [P] [US2] Implementar NativePublishState e mailbox interna em `src-tauri/src/capture/mod.rs`: sessionEpoch "token cancelável de sessão", latestFrame "zero ou um descriptor gen/slot/seq/ts + instante local de enfileiramento", pcmQueue "chunks com instante local; duração total ≤ 200 ms", uiScheduled "no máximo uma closure pendente de publicação de mídia", controls "eventos de estado separados; ended é terminal" e droppedFrames/droppedAudioUs "contadores técnicos"; substituir frame pendente pelo novo, remover PCM mais antigo, verificar idade de 200 ms antes de postar e não manter locks em chamadas WebView2; verificar mailbox/cancelamento em testes Rust no mesmo arquivo.
- [X] T018 [P] [US2] Limitar backlog de captura PCM em `src-tauri/src/capture/audio.rs` antes da publicação, retendo no máximo os 200 ms mais recentes em vez de drenar todo o passado para envio; preservar PCM f32/48 kHz/estéreo e exclusão do Discord, contabilizar descarte e verificar bursts e chunk parcial em testes Rust no mesmo arquivo.
- [X] T019 [US2] Conectar o publicador T017 aos sinks de `src-tauri/src/capture/mod.rs` e à emissão em `src-tauri/src/capture/video.rs`, validando geração/seq/slot e descriptor mais recente antes da postagem; invalidar descriptors da geração anterior em mudança de tamanho e preservar entrega dos controles sem coalescê-los como quadros; verificar pausa de UI, slot sobrescrito e retomada sem alterar layout NV12 nem contrato v1.
- [X] T020 [US2] Invalidar a época nativa antes de parar produtores em `src-tauri/src/capture/mod.rs` e a sessão JS em `src/lib/native-capture.ts`; garantir "Stop limpa mídia pendente e invalida closures antigas antes de encerrar os produtores" e "running → stopped: invalidar token, remover listeners/timers, encerrar writers/tracks e liberar buffers. Rejeições assíncronas são tratadas; não reativam sessão"; integrar cancelamento de startup em `src/lib/stores/room.svelte.ts` para fechar recurso aberto tardiamente; verificar start/write/setQuality tardios em `test/lib/native-capture.test.ts` e testes Rust da captura.
- [X] T021 [P] [US2] Implementar PeerMediaEpoch em `src/lib/peer.ts`, serializando/coalescendo mídia desejada por revisão com "Cada setLocal incrementa revision; operações consultam revisão/closed antes e após awaits", "Parada remove faixas após qualquer aplicação que já estivesse em andamento. Close é terminal"; verificar em `test/lib/peer-lifecycle.test.ts` (novo) attach/replaceTrack tardios, stop/start e close sem reaplicar faixa antiga ou quebrar perfect negotiation.
- [ ] T022 [US2] Executar T3 em `specs/016-reduzir-atraso-imagem/quickstart.md`, cinco vezes para rede insuficiente, transmissor ocupado e espectador ocupado por 10 s, incluindo pausa/retorno de áudio e renderer; registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` recuperação ≤ 5 s, atraso ≤ 1000 ms, A/V ≤ 200 ms e ausência de ação manual; não confundir limite da mailbox nativa com ausência de fila no renderer (FR-003/004/005/008, SC-003/005).
- [ ] T023 [US2] Executar T5 de `specs/016-reduzir-atraso-imagem/quickstart.md`, cinco encerramentos com callbacks atrasados, nova transmissão, saída da sala e close; registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` zero reativação de imagem/som encerrados e liberação dos recursos (FR-009, SC-006); se T022/T023 falharem, atualizar `specs/016-reduzir-atraso-imagem/research.md` e `specs/016-reduzir-atraso-imagem/plan.md` antes de adotar outra política de recuperação.

**Checkpoint**: imagem e som recuperam juntos, sem callbacks ressuscitando transmissões. Ausência de timestamp PCM no transporte antigo exige validação A/V real; não relaxar limites para mascarar essa limitação.

## Phase 5: User Story 3 — Entrar e trocar a visualização sem carregar atraso (Priority: P2)

**Goal**: conservar proximidade do momento atual ao entrar, voltar ou alterar visualização/qualidade, mantendo som e PiP.

**Independent Test**: T4 do quickstart, cinco repetições de cada transição; atraso ≤ 1000 ms em ≤ 5 s, contado após a primeira imagem no início/retorno e após a ação nas demais; outros espectadores mantêm SC-001.

### Implementation and validation

- [X] T024 [US3] Reaplicar de forma segura a preferência de recepção de T011 nas ativações full/thumb de `src/lib/peer.ts`, integrando `src/lib/mesh.ts` sem novo ctl; verificar em `test/lib/peer-latency.test.ts` a preservação de hidden (som continua), off (sem som/vídeo) e receiver sem suporte, sem reiniciar toda a malha.
- [X] T025 [US3] Integrar retorno de visibilidade/PiP, entrada e reconexão em `src/lib/stores/room.svelte.ts` com reaplicação de modos e invalidação de resultados de peer/sessão substituídos, mantendo "Quem só assiste também é coletado. Fechamento, remoção do peer e fim da sessão liberam ring e referências"; verificar limpeza e modos em `test/lib/view-modes.test.ts` e `test/lib/peer-lifecycle.test.ts`.
- [X] T026 [US3] Conferir a integração de mudança de qualidade entre `src/lib/native-capture.ts` e `src/lib/stores/room.svelte.ts`, mantendo época de captura, buffer generation válido e "Uma operação supersedida não deve prender a nova nem reaplicar faixa antiga" de PeerMediaEpoch; verificar troca rápida de qualidade e stop durante troca em `test/lib/native-capture.test.ts` e `test/lib/peer-lifecycle.test.ts`, preservando tetos e governor da 010.
- [ ] T027 [US3] Validar os caminhos reais de montagem/recriação de tiles em `src/ui/components/Stage.svelte` e `src/ui/components/StreamTile.svelte`, sem inserir chamadas WebRTC na UI, conferindo autoplay, som em elemento separado, mute/volume, PiP e mock `/?mock=room&scene=grid`; registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` a manutenção dos controles sem passos ou configurações adicionais.
- [ ] T028 [US3] Executar T4 de `specs/016-reduzir-atraso-imagem/quickstart.md` cinco vezes por transição (entrada, queda/retorno, miniatura↔destaque, qualidade, ocultar/restaurar), incluindo um espectador com dificuldade e outros estáveis; registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` os prazos por evento e o atraso dos demais, aprovando somente se SC-004 e FR-006/007/008 forem atendidos.

**Checkpoint**: três histórias aprovadas individualmente, sem usar mudança de estado da conexão ou track.muted como prova de imagem atual.

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: verificar cobertura de plataformas, recursos, compatibilidade e evidências finais.

- [ ] T029 Executar a matriz de `specs/016-reduzir-atraso-imagem/quickstart.md` para tela/janela/câmera, app↔browser, browser↔browser, app instalado/site novo, app reconstruído, Chrome/Edge e recepção Firefox/Safari/celular, incluindo fallback sem jitterBufferTarget; registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` combinações/aparelhos testados, resultados e lacunas, sem concluir suporte não executado.
- [ ] T030 Verificar recursos e isolamento com `src/lib/quality.ts`, `src/lib/cpu-governor.ts` e `scripts/memoria.ps1`, incluindo tetos de upload, histerese 10 s/60 s, janela de diagnóstico limitada, fim de captura e regressão funcional de sala com 10 pessoas; registrar em `specs/016-reduzir-atraso-imagem/relatorio.md` que um espectador ruim não causa rebaixamento global nem mantém timers/writers vivos após teardown.
- [X] T031 Executar gates finais de `package.json` (`npm run check`, `npm test`, `npm run build`, `npm run desktop:build`) e `cargo test --manifest-path src-tauri/Cargo.toml` para as mudanças Rust, conferindo mocks e contratos preservados; registrar comandos, resultados e problemas preexistentes separadamente em `specs/016-reduzir-atraso-imagem/relatorio.md`, sem publicar instalador/site ou inventar aprovação.
- [ ] T032 Consolidar em `specs/016-reduzir-atraso-imagem/relatorio.md` a comparação baseline/depois e tabela de evidências FR-001–009/SC-001–006, atualizar `specs/016-reduzir-atraso-imagem/quickstart.md` com instruções efetivamente verificadas e `specs/016-reduzir-atraso-imagem/research.md` com conclusões/limitações; manter tarefas de aceitação pendentes quando faltar evidência e revisar `specs/016-reduzir-atraso-imagem/plan.md` se for necessária outra solução antes de declarar conclusão.

## Dependencies & Execution Order

### Phase Dependencies

```text
Setup T001–T004
  → Foundational T005 → T006 → T007; T008 baseline externo pendente para aceitação
  → US1 T009–T014 (MVP de atraso estável)
  → US2 T015–T023 (recuperação e encerramento)
  → US3 T024–T028 (transições)
  → Polish T029–T032 (regressão e evidências finais)
```

As histórias são verificáveis separadamente, mas compartilham arquivos e evoluem a mesma captura; a ordem de integração é deliberadamente sequencial. Não há três frentes independentes editando peer.ts, native-capture.ts ou room.svelte.ts ao mesmo tempo.

### Dependencies within phases

- Setup: T001 e T002 podem começar juntos; T003 depende de T001; T004 depende de T001–T003.
- Foundational: T005 → T006 → T007 → T008. Instrumentar sem alterar o comportamento de captura até registrar baseline.
- US1: T009 e T011 podem iniciar juntos após T007 (ajuste de execução acima); T010 depende de T009; T012 depende de T010/T011; T013 → T014 após integração. T010 e T011 também usam arquivos diferentes.
- US2: T015, T017, T018 e T021 podem iniciar após US1; T016 depende de T015; T019 depende de T017/T018; T020 depende de T016/T019. T021 deve terminar antes de T022; T022 depende de toda implementação da US2; T023 segue T022. Respeitar a serialização das edições em mod.rs entre T017/T019/T020.
- US3: T024 → T025 → T026 → T027 → T028, depois da US2; não paralelizar edições sobrepostas.
- Polish: T029 → T030 → T031 → T032. Gates devem ser repetidos somente se correções posteriores invalidarem o resultado anterior.

### Traceability

| Requisito / critério | Tarefas principais |
|---|---|
| FR-001/002, SC-001 | T009–T013 |
| SC-002 | T014 |
| FR-003/004, SC-003 | T015–T022 |
| FR-005, SC-005 (A/V) | T015/T016/T018/T022 |
| FR-006, SC-004 | T024–T028 |
| FR-007 | T013/T028/T030 |
| FR-008, SC-005 (sem ação manual) | T022/T027/T028 |
| FR-009, SC-006 | T020/T021/T023 |
| Compatibilidade, privacidade e gates | T005–T007/T012/T029–T032 |

## Parallel Examples

### US1

Após T007, implementar CaptureTiming e sua verificação em `src/lib/capture-timing.ts` / `test/lib/capture-timing.test.ts` (T009), ao mesmo tempo que o adapter de receiver e sua verificação em `src/lib/peer.ts` / `test/lib/peer-latency.test.ts` (T011). T010 pode prosseguir enquanto T011 termina. A integração T012 aguarda ambos.

### US2

Após US1, T015 (timing TS), T017 (mailbox em mod.rs), T018 (backlog em audio.rs) e T021 (lifecycle em peer.ts) usam arquivos distintos. T016 pode acompanhar T017/T018/T021 após T015. A conexão dos sinks T019 aguarda a mailbox e o descarte PCM. Não executar T019 e T020 em paralelo com T017.

### US3

Não há paralelismo de edição seguro nesta fase: a cadeia compartilha peer/store e depende das proteções anteriores. Durante T027, uma sessão de navegador e outra de app podem observar controles em paralelo, em aparelhos e salas de ensaio separados; consolidar os resultados antes de T028, evitando concorrência de transmissões na comparação de desempenho.

## Implementation Strategy

### MVP First

Concluir a parte local de Setup e Foundational, implementar US1 e validar T008/T013/T014 quando os aparelhos estiverem disponíveis. T002/T004 mantêm pendências de captura/debug reais. Esse incremento entrega redução de atraso estável mensurável; não representa a feature completa. Nenhum checkpoint implica autorização ou necessidade de deploy.

### Incremental Delivery

Adicionar recuperação (US2), validar perturbações e encerramento, depois transições (US3) e matriz final. Preservar protocolo e ausência de configuração manual em todos os incrementos. Se a preferência de buffering e a ponte compatível não cumprirem a spec, registrar a etapa responsável e revisar o design antes de escolher reset de reprodução ou outro formato.

## Notes

- 32 tarefas: Setup 4, Foundational 4, US1 6, US2 9, US3 5, Polish 4.
- Sete tarefas marcadas `[P]`, com pré-requisitos e arquivos distintos explicitados acima.
- Não alterar Worker, ctl, permissões de produção ou ponte v1 como atalho para atender às metas.
- Contador visual comprova atraso ponta a ponta; getStats e jitterBufferTarget não o substituem.
- Gerar esta lista não executa as tarefas nem valida o desempenho do produto.



## Progresso da execução — 2026-11-04

19 tarefas concluídas, 13 pendentes. Implementação local e gates automatizados passaram; as tarefas T002/T004/T008/T012/T013/T014/T022/T023/T027/T028/T029/T030/T032 permanecem abertas pelos ensaios reais e pela comparação final ainda indisponíveis. O detalhamento e os resultados estão em [relatorio.md](./relatorio.md). Nenhuma história foi declarada aprovada ponta a ponta nem houve deploy.

Publicação autorizada em 2026-11-04: site e instalador **0.1.9** disponíveis em produção, versão Cloudflare 60d9b5e3-4064-454e-8dff-1887bf16ac71. Entrega de arquivos e sinalização verificadas; medições reais continuam pendentes. Ver [relatorio.md](./relatorio.md).
