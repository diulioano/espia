# Sinalização de mídia v2

Sem novos endpoints HTTP. Alterar os contratos existentes em src/shared/protocol.ts e validar em src/worker/validate.ts.

## Entrada e retomada

Mensagens existentes join e resume passam a carregar `mediaProtocol:2`. O servidor valida antes de registrar membro, retomar identidade ou fornecer iceServers. Ausência ou versão diferente fecha com novo código `4426`, reason `atualização necessária`; o cliente apresenta Atualize o Espia e não entra em loop de reconexão. Confirmar ausência de colisão ao implementar.

Welcome inclui `mediaProtocol:2` e `mediaPolicy:{kind:'direct-only',reason:'unverified-budget'}`. O servidor mantém política nos caminhos join, resume e refresh; iceServers contém somente STUN. A supressão de emissão e renovação TURN é pré-requisito dos fundamentos, antes de construir qualquer welcome v2; configuração de segredos não muda isso. Cliente novo diante de welcome sem v2 interrompe setup em vez de cair no modo legado. Expiração/refresh do envelope ICE pode permanecer para compatibilidade estrutural, sem emissão TURN.

## Transmissão e reconexão

Live conserva os campos existentes e passa a carregar `requestId`, UUID v4 canônico de 36 caracteres gerado por captura, e `broadcastSeq`, inteiro seguro não negativo. O servidor associa ambos ao membro e confirma `{t:'live-ack',requestId,broadcastSeq,live:LiveInfo}` ao emissor. O início canônico startedAt é único e crescente por peer. Repetições e mudanças de qualidade do mesmo requestId/broadcastSeq preservam a chave; uma nova captura cria novo UUID e seq estritamente maior. RequestId não é credencial de autorização.

Unlive carrega `{t:'unlive',requestId,broadcastSeq}`. O servidor guarda apenas o maior broadcastSeq, seu requestId, lastStartedAt e status live/ended no estado existente do membro, preservando-os em hibernação, retomada e reinício. Seq menor ou mesma seq com UUID diferente é obsoleta e não produz peer-live/unlive nem muda estado. Repetição do mesmo live ativo devolve o mesmo ACK; depois de ended é rejeitada e não reanuncia mídia. Stop do estado atual é idempotente; stop de seq anterior não afeta captura nova. Nova captura requer seq maior e UUID novo. Primeira seq é 0; nova seq é maior confirmada +1, sem overflow. lastStartedAt preserva o último início após unlive; novo início usa max(relógio atual,lastStartedAt+1) com validação de inteiro seguro. A marca terminal não é removida em unlive; só acompanha a remoção da identidade do membro pelo ciclo de vida existente.

Welcome de retomada devolve ao próprio membro `broadcastOrder` com essa marca e estado, ou null se nunca transmitiu. Cliente confirma a retomada antes de calcular o próximo broadcastSeq. Nada disso é histórico durável de participantes; não armazenar lista de UUIDs encerrados. A revision de ctl continua independente e restrita à conexão.

O emissor aguarda confirmação canônica da própria transmissão antes de aceitar grants. Uma confirmação só atualiza o início pendente com o mesmo requestId/broadcastSeq; resultados supersedidos não reabrem captura. Ao confirmar, solicita view-sync no ctl para resolver corrida entre transportes. Retomada devolve o mesmo live sem gerar outra chave. Unlive invalida grants e escolhas. O servidor serializa e verifica a marca de ordem antes de mudar estado; atualizar qualidade não pode produzir outro startedAt.

## Transição de versão

Servir cliente e Worker compatíveis como uma entrega. A primeira mensagem de cliente antigo e attachments antigos sem mediaProtocol devem ser tratados como incompatíveis também em hibernação e resume, sem passar pela emissão de credenciais. Drenar sockets preexistentes por procedimento de release; não afirmar que mudar HTML atualiza página já aberta.

Uma atualização não pode deixar pares antigos enviando automaticamente para novos receptores. A matriz novo/novo deve passar; antigo/novo e novo/antigo devem resultar em atualização explícita, nunca degradação silenciosa do consentimento. Clientes que falsifiquem capabilities não são prova de compatibilidade; a regra é aplicada também no sender novo.
