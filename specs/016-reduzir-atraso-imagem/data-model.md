# Data Model: Reduzir atraso da imagem

Estado exclusivamente transitório. Nenhuma tabela, migração, preferência local ou entidade de servidor nova.

## CaptureTiming

Uma instância por startNative, com relógio monotônico injetável para testes.

| Campo | Unidade / regra |
|---|---|
| sessionToken | identidade local única da execução, não enviada ao servidor |
| videoOffsetUs | null até primeiro frame válido; depois arrivalUs − nativeTsUs |
| lastNativeTsUs | timestamp relativo mais recente aceito |
| lastVideoUs | último timestamp entregue ao gerador |
| audioNextUs | próximo timestamp calculado por amostras, null sem áudio |
| lastAudioArrivalUs | chegada do último pacote de áudio válido |
| audioQueuedUs | duração das escritas pendentes do gerador atual |
| writerGeneration | invalida completions do writer de áudio anterior |
| stopped | terminal; qualquer evento posterior é ignorado |

### Regras e transições

- `starting → running`: primeiro frame de geração válida define offset. Não utilizar o instante anterior ao invoke como época do vídeo.
- Rejeitar timestamps/seq não finitos, negativos, regressivos ou de geração inválida. Mudança de resolução conserva época da sessão.
- Vídeo entregue com timestamps crescentes; trabalho antigo é descartado, nunca acumulado para preservar todos os quadros.
- Primeiro áudio começa no relógio local atual; duração = frames/48000 em microssegundos, com 2 canais. Aceitar somente chunks íntegros e não vazios.
- `audio none → audio active`: novo writerGeneration, audioNextUs reancorado. Completions do writer anterior não alteram fila ou estado do novo.
- Lacuna ou desvio do áudio > 200 ms: descartar backlog controlado e reancorar no tempo local, sem reproduzir toda a lacuna. Registrar contagem de descarte; não chamar conteúdo comprovadamente antigo de atual.
- `running → stopped`: invalidar token, remover listeners/timers, encerrar writers/tracks e liberar buffers. Rejeições assíncronas são tratadas; não reativam sessão.

O timestamp local estimado não é uma medição ponta a ponta. Na ponte legada não há timestamp por pacote PCM; a precisão de A/V deve ser validada pelo evento flash/beep.

## NativePublishState

Uma instância interna por captura Rust, compartilhada pelos produtores e publicador de UI.

| Campo | Regra |
|---|---|
| sessionEpoch | token cancelável de sessão |
| latestFrame | zero ou um descriptor gen/slot/seq/ts + instante local de enfileiramento |
| pcmQueue | chunks com instante local; duração total ≤ 200 ms |
| uiScheduled | no máximo uma closure pendente de publicação de mídia |
| controls | eventos de estado separados; ended é terminal |
| droppedFrames / droppedAudioUs | contadores técnicos |

Cada frame novo substitui o pendente. PCM excedente remove primeiro o mais antigo. O publicador retira lote limitado, verifica cancelamento e idade (200 ms) antes de postar e reagenda só se houver trabalho novo. Locks não permanecem adquiridos durante chamadas WebView2.

Stop limpa mídia pendente e invalida closures antigas antes de encerrar os produtores. Eventos de controle não são substituídos por quadros. Mudança de geração invalida descriptors dos slots antigos. O limite é da fila controlada pelo app: não alegar limite de fila do renderer sem ensaio.

## PeerMediaEpoch

Por PeerLink: closed, desiredLocalStream, desiredQuality, revision e fila serial de aplicação. Cada setLocal incrementa revision; operações consultam revisão/closed antes e após awaits. Uma operação supersedida não deve prender a nova nem reaplicar faixa antiga. Parada remove faixas após qualquer aplicação que já estivesse em andamento. Close é terminal.

Transições: idle → attaching → live → stopping → idle; qualquer estado → closed. Nova transmissão usa outra revisão. Renegociação conserva perfect negotiation existente.

## MediaStatsSample

Por sessão/link e por objeto RTCStats: timestamp monotônico, referência da amostra anterior, deltas opcionais de encode/fila de envio/jitter/decode, bytes/frames/perdas/freezes e tipo de candidato selecionado (somente host/srflx/relay; sem endereço).

- Todos os tempos normalizados em ms; contadores originais acumulados respeitam suas unidades.
- IDs/ssrc servem só para correlação em memória; exportação usa índice local de link, nunca nickname, código de sala ou identificador persistente.
- Delta só é válido com mesma identidade, tempo crescente e denominador positivo. Reset, ausência, inatividade e zero de amostras retornam null.
- Ring buffer de 60 amostras por link; coleta a cada segundo, no máximo uma consulta pendente. É um diagnóstico recente, não o arquivo de aceitação de 30 min.
- Quem só assiste também é coletado. Fechamento, remoção do peer e fim da sessão liberam ring e referências.

## AcceptanceRun

Registro manual local de ensaio: build/revisões, versões de navegador/WebView2, aparelhos, origem, qualidade efetiva, som, número de espectadores, tipo de rota, início/fim das perturbações e amostras numéricas de atraso/A/V. Sem conteúdo de mídia ou identidade das pessoas.

Estados: baseline → after-change → pass/fail. Dados ausentes não contam como aprovação; o registro é evidência de validação, não estado do produto.
