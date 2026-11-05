# Contrato de escolha e autorização de mídia

## Interface

StreamView deve distinguir available, opening, playing, stopped e unavailable. available usa nome/descrição/Assistir; stopped pode dizer Você parou de assistir; unavailable explica falha e permite Tentar novamente apenas para escolha existente. Não mostrar Você parou de assistir antes do primeiro clique. Nenhuma miniatura ao vivo ou áudio hidden antes de Assistir.

Preservar eventos watch(id,on), porém validar que a chave da transmissão ainda é a mesma no momento do clique. A store resolve identidade e revisões; UI não usa WebRTC. Foco, volume, grade e restaurar janela não emitem watch(true). O botão Assistir do toast deve chamar a mesma escolha, em vez de apenas selecionar destaque. Timers de primeiro quadro e tentativas automáticas só existem para escolhas ativas. Testar eventos de teclado e evitar propagação do botão para seleção automática de cartão.

## Canal ctl v2

Mensagem JSON proposta: `{ t:'view', version:2, transmission:{peerId,startedAt}, revision, mode:'off'|'hidden'|'thumb'|'full' }`.

- PeerId deve identificar o transmissor local do destinatário, e startedAt deve ser sua transmissão local atual.
- Revision é inteiro seguro não negativo e crescente no contexto da conexão; descartar duplicatas, atrasadas, versões antigas e mensagens malformadas. Nova conexão exige novo grant.
- Mensagem inválida ou ausente nunca promove off para envio. Ao perder a conexão de controle, retirar grants e tracks antes de qualquer retomada.
- Enviar estado vigente no onopen; perder controle durante opening não pode liberar mídia.
- Ao iniciar nova transmissão local, limpar grants antes de ligar qualquer track. Não reaproveitar full da transmissão anterior.
- O ctl pode negociar conexão sem áudio/vídeo. Criar transceivers sem track ou recvonly conforme negociação; o primeiro add/replaceTrack de envio depende do grant.
- Depois do live-ack, o transmissor envia `{t:'view-sync',version:2,transmission:{peerId,startedAt}}`; o receptor responde com a escolha vigente para essa chave, inclusive off. Isso elimina dependência de ordem entre confirmação pelo WebSocket e grant pelo ctl. Não criar grant automaticamente no receptor.
- Off não depende apenas de encoding.active=false aplicado depois da negociação. Remover tracks de ambos os senders, invalidar revisões e limpar reprodução. Retomar exige a mesma chave e escolha ainda válida.

O contrato v1 hidden+sound:false não autoriza esta versão. `theirViews()` e contagem consideram grants válidos e fluxo de vídeo; não contar apenas conexão estabelecida.

## Provas de fronteira

Nenhuma ligação inicial de track antes do grant; clique seguido de Parar enquanto replaceTrack aguarda; grant antigo após unlive/live; onopen depois de saída; reconexão mesma transmissão; mobile troca de selecionada; nova transmissão não selecionada. Testar envio e recepção reais além de reducer.
