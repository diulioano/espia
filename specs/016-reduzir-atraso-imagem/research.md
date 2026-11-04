# Research: Reduzir atraso da imagem

Data: 2026-11-04. Pesquisa estática e documentação primária; sem reprodução de transmissão real nesta fase.

## R1 — Medir antes de atribuir os 10 s a uma etapa

**Decision**: usar contador visível comum à origem e ao destino como medida de atraso ponta a ponta. Correlacionar com métricas por sessão/peer a cada segundo, inclusive quando o cliente só assiste. Criar scripts/latency-source.html com contador e flash/beep gerados localmente.

**Rationale**: peer.ts:249–273 só expõe dados básicos de envio; room.svelte.ts:261–266 coleta durante a própria transmissão. Não há observabilidade inbound. FIRST_FRAME_TIMEOUT_MS=10000 é timeout de falha de primeiro quadro, não uma espera de 10 s adicionada à imagem. Separar correlação de causalidade.

**Alternatives considered**: reduzir esse timeout, usar RTT como atraso de imagem ou atribuir a TURN sem medir — rejeitados porque não medem a idade da imagem apresentada.

## R2 — Alinhar relógios da captura nativa

**Decision**: extrair CaptureTiming; mapear o ts relativo do vídeo para performance.now no primeiro frame válido, sem incluir o tempo anterior ao capture_start. Preservar o mapeamento em troca de qualidade. Áudio começa no mesmo domínio local; reancorar em lacuna > 200 ms ou após remover/recriar sua faixa, mantendo timestamps estritamente crescentes e descartando trabalho antigo.

**Rationale**: native-capture.ts:278 fixa clockZeroUs antes do invoke, mas video.rs:398–423 prepara slots/GPU/permissão antes de criar started. O vídeo usa clockZeroUs + msg.ts (:339) e o áudio começa em performance.now (:362). O descompasso de épocas é comprovável no código; seu efeito real no motor e sua relação com o relato precisam de medição. audioTs avança só por amostras e removeAudio não o reinicia (:288–294,362–365).

O mapeamento inicial pela chegada é uma estimativa: não identifica atraso já acumulado antes do primeiro callback. Não recarimbar cada pacote atrasado como se tivesse sido capturado agora. Comparar evolução do ts com o relógio local, limitar filas no produtor e validar com contador.

**Alternatives considered**: usar o instante da resposta do invoke — os eventos podem anteceder a resposta; relógio civil — sujeito a ajustes; nova API nativa de sincronização — desnecessária para a primeira correção e ampliaria o contrato.

## R3 — Limitar o trabalho antes do IPC

**Decision**: manter os comandos e payloads atuais; adicionar mailbox interna de vídeo com apenas o último aviso e um único agendamento de UI pendente. Limitar PCM a 200 ms, descartando o mais antigo. Separar eventos de controle e invalidar o publicador por sessão antes do stop. Não bloquear a captura aguardando a UI.

**Rationale**: capture/mod.rs:173–202 agenda with_webview/PostWebMessageAsJson por quadro/pacote. audio.rs:262–286 drena todos os pacotes WASAPI antes de publicar todos os chunks. Não há limite nessa publicação. O busy da página limita escritas do gerador, não o IPC anterior. SlotTable.stale compara apenas seq já recebido: um backlog FIFO crescente pode parecer atual mesmo com slot sobrescrito.

Não reutilizar o slot da mensagem descartada; publicar o descriptor mais recente e copiá-lo imediatamente no JS. Checar sessão, geração e idade novamente na closure de publicação. Eventos ended/audio/warn não podem ser coalescidos como frames. Essa limitação do agendamento nativo não equivale a backpressure completo do renderer: o ensaio com pausa da página deve verificar também essa fronteira.

**Alternatives considered**: fila sem limite — mantém o passado; descartar somente no JS — tarde para reduzir trabalho nativo; alterar o layout dos buffers/ACK por quadro — reservado a revisão do contrato se as medições demonstrarem necessidade.

A documentação de [PostWebMessageAsJson](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2#postwebmessageasjson) descreve a entrega por evento à página. O limite de fila proposto é uma decisão do Espia, não uma garantia atribuída à API.

## R4 — Preferência de baixa latência com fallback

**Decision**: quando disponível, definir receiver.jitterBufferTarget=0 para áudio e vídeo de mídia ao vivo; tratar exceção como ausência de suporte. Aplicar em ontrack e na ativação da visualização. Não adotar playoutDelayHint não padronizado nem modificar SDP.

**Rationale**: [WebRTC, jitterBufferTarget](https://www.w3.org/TR/webrtc/#dom-rtcrtpreceiver-jitterbuffertarget) define uma preferência em milissegundos, sujeita aos limites do motor e a ajuste gradual. O valor solicitado não comprova atraso real nem assegura recuperação em 5 s. A sincronização A/V pode fazer o motor usar o maior target das faixas; testar ambas juntas e a reprodução atual em elementos separados.

**Alternatives considered**: reset periódico de ICE, renegociar toda a malha, seek/currentTime em MediaStream, acelerar playbackRate — rejeitados como política inicial sem evidência de benefício e com risco para áudio, PiP e espectadores saudáveis.

## R5 — Estatísticas opcionais e lifecycle

**Decision**: normalizar deltas por (sessão, peer, stats.id/ssrc). Coletar envio, recepção e rota com payload mínimo; ausências, denominadores zero, mudança de objeto e contadores regressivos produzem null/reset. Janela circular de 60 amostras por link em memória; exportação manual apenas de números técnicos, sem mídia ou dados pessoais.

**Rationale**: [WebRTC Stats](https://www.w3.org/TR/webrtc-stats/) define contadores acumulados. Calcular encodeMs=1000×ΔtotalEncodeTime/ΔframesEncoded, sendQueueMs=1000×ΔtotalPacketSendDelay/ΔpacketsSent e jitterMs=1000×ΔjitterBufferDelay/ΔjitterBufferEmittedCount. RTT, perdas, freezes, framesDecoded e totalDecodeTime complementam diagnóstico, sem somar métricas sobrepostas como se fossem atraso ponta a ponta.

setLocal faz múltiplos awaits sem geração (peer.ts:180–187); proteger cada operação com identidade de transmissão e serialização, garantindo que stop prevaleça. Novos timers e promessas precisam de cleanup em close/remove/unlive, e callbacks não podem publicar dados de um link substituído.

**Alternatives considered**: novo governor global por banda/atraso de um espectador — prejudicaria os outros; novo polling independente duplicando envio — custo desnecessário; gravar métricas no servidor — desnecessário e fora da privacidade estabelecida.

## R6 — Compatibilidade e critérios de saída

**Decision**: não mudar Worker, ctl, versão 1 da ponte, comandos, layouts ou decisões de DEFINICOES.md. Preservar formatos base64/ArrayBuffer/lista, CPU governor 10/60 s e regras full/thumb/hidden/off. Testar atraso e A/V com elementos de reprodução existentes antes de considerar qualquer alteração neles.

**Rationale**: as alterações propostas cabem nos mecanismos existentes. Nenhuma causa do relato foi confirmada por execução, e nenhuma meta foi medida. O primeiro incremento deve corrigir os defeitos testáveis sem adicionar infraestrutura.

**Alternatives considered**: migrar para SFU, impor atualização ou escolher outro codec sem medir — fora do escopo desta correção.

## Resultado da pesquisa

Não restam decisões técnicas indefinidas para gerar as tarefas. Permanecem ensaios obrigatórios de eficácia, detalhados no quickstart. Se a abordagem compatível não cumprir a spec, revisar o plano com evidências; não converter hipóteses em conclusão nem declarar a feature pronta.

## Resultado da implementação local — 2026-11-04

A regressão startNative confirmou offset de timestamps de 2 s e 10 s quando a abertura é atrasada artificialmente, além de áudio 9,99 s atrasado após uma lacuna de 10 s. Os testes passam após mapeamento pelo primeiro frame e reancoragem de áudio. Essa é evidência de defeitos locais; a associação com o relato ponta a ponta permanece não confirmada.

A mailbox foi isolada em src-tauri/src/capture/publish.rs para testar limites sem WebView/COM. NativeSession recebe AbortSignal opcional apenas no adaptador TypeScript, sem novo comando nativo. A abertura nativa é serializada para evitar que uma captura cancelada libere buffers de outra sessão. stop/close e mudanças de qualidade invalidam trabalho supersedido.

O modelo CaptureTiming é realizado por uma instância por captura, junto à identidade AbortController/época do store; não há identificador de pessoa. A reancoragem usa o desvio entre o próximo timestamp e o relógio monotônico, sem manter um campo redundante de chegada. O formato legado do PCM continua sem timestamps; a mailbox não controla a fila interna do renderer. A aceitação no cenário real ainda exige o quickstart.
