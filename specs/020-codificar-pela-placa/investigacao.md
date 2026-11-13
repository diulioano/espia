# Investigação: codificar o vídeo uma vez, pela placa de vídeo

**Data**: 2026-11-12 · **Spec**: [spec.md](./spec.md) · **Origem**: fechamento do G1 da [019](../019-app-rust-convite/relatorio.md), item (f)

Pergunta: vale a pena o app nativo codificar a transmissão uma vez só, na placa de vídeo, e mandar o mesmo vídeo a todos os espectadores (a ideia "como o Discord faz")?

## Resposta curta

Vale, e é mais que interessante: sem isso a migração para o app nativo não entrega o que prometeu. O caminho existe na biblioteca que a 019 já escolheu, sem precisar de versão modificada. Nada dele foi testado ainda, então o primeiro passo é uma prova pequena, antes de construir.

## Por que não dá para ficar como está

Números medidos em 2026-11-12, na mesma máquina (24 núcleos lógicos), em núcleos equivalentes:

| Caso | Processador | Condição |
|---|---|---|
| App de hoje (0.1.10) | 0,53 núcleo | 1080p60 para **até 3** pessoas (algumas podiam não estar assistindo na hora), LoL aberto, leitura de um instante |
| Sonda do app nativo, codificando no processador | 0,57 núcleo | 1080p60 para **1** pessoa, sem jogo |
| Meta da 019 (SC-005) | 0,27 núcleo | metade do app de hoje |

- A sonda gasta para 1 pessoa o que o app de hoje gasta para até 3. Ressalva do dono: na medição do app de hoje, parte das 3 pessoas podia não estar assistindo, e quem não assiste não recebe vídeo. Se só 1 assistia, os dois números seriam equivalentes e a inferência abaixo perde força.
- Na conexão direta, cada espectador tem a sua própria codificação. Com 3 pessoas, a sonda faria 3 codificações. Se o custo crescer na mesma proporção, seriam cerca de 1,7 núcleo. Isso é estimativa, não medição.
- Conclusão: codificando no processador, o app nativo ficaria mais pesado que o app de hoje, não mais leve.

A diferença sugere que o app de hoje já codifica pela placa de vídeo. O código do site reforça isso: `src/lib/peer.ts` pede ao motor o H.264 pela placa (`preferHardwareCodec`), e o app de hoje carrega esse mesmo código. Ainda assim é inferência: o item 5 do G0 (qual codificador o motor usa) nunca foi lido. Dá para confirmar no Diagnóstico de mídia do site, durante uma transmissão pelo app de hoje.

## O que foi conferido no código

| Fato | Onde | Situação |
|---|---|---|
| A biblioteca não usa a placa de vídeo para codificar no Windows | `build.rs` da `webrtc-sys` 0.3.53: codificadores da placa só no Linux | Confirmado (G1) |
| A biblioteca aceita vídeo **já codificado** pelo app | `libwebrtc` 0.4.0: `NativeVideoSource::new_encoded`, `capture_encoded_frame`, `VideoEncoderBackend::PreEncoded` | Confirmado no código-fonte. Não testado |
| Ela avisa quando um espectador precisa de um quadro inteiro novo | `take_keyframe_request` | Confirmado no código-fonte. Não testado |
| Ela informa quanta banda a rede aguenta | `take_rate_control_request` (guarda só o pedido mais recente) | Confirmado no código-fonte. Não testado |
| Aceita H.264, que é o que Chrome e Safari recebem hoje | `EncodedVideoCodec::H264` | Confirmado no código-fonte |
| A captura de hoje já deixa o quadro pronto na placa de vídeo | `src-tauri/src/capture/video.rs`: reduz e converte na placa, e só depois copia para a memória | Confirmado |

O último ponto importa: hoje o quadro sai da placa, vai para a memória e a página o codifica. Com o codificador da placa, o quadro pode ir direto da captura para a codificação, sem passar pelo processador.

## A ideia do Discord, em duas partes

Pelo que é público sobre o Discord (não conferido nesta investigação), a transmissão dele junta duas coisas:

1. **Um servidor recebe o vídeo uma vez e repassa a todos.** No Espia isso não entra: o princípio II da constituição e o ADR 0001 mantêm a conexão direta, sem servidor de mídia.
2. **O vídeo é codificado pela placa de vídeo.** Isso cabe no Espia e é o que esta feature propõe.

O que dá para aproveitar da primeira parte sem servidor: **codificar uma vez só** e mandar o mesmo vídeo a cada espectador pela conexão direta.

## O que se ganha

- **Processador**: a codificação sai do processador. É o que o jogo mais disputa em jogos como LoL e CS2.
- **Custo que não cresce com a plateia**: uma codificação para 1 ou para 7 espectadores.
- **Limite das placas**: placas GeForce têm limite de codificações simultâneas no sistema inteiro (8, segundo a nota da NVIDIA para o SDK 12.2; versões anteriores citam 3). Uma codificação por espectador esbarraria nisso com plateia cheia e outro programa gravando. Uma ou duas codificações ficam longe do limite.
- **Sem biblioteca modificada**: o plano B da 019 (versão alterada da biblioteca) deixa de ser necessário.

## O que se perde ou muda

| Mudança | Consequência | Como tratar |
|---|---|---|
| Não há mais uma codificação por espectador | A qualidade sob medida para cada um (miniatura, teto por ligação) deixa de sair de graça | Duas codificações: uma cheia e uma reduzida. Quem vê em miniatura recebe a reduzida |
| Um espectador pede um quadro inteiro novo | Todos recebem esse quadro, que é maior | Aceitável; limitar a frequência |
| Um espectador com rede fraca | Não dá para baixar a qualidade só para ele dentro da mesma codificação | Ele passa a receber a reduzida; os outros continuam na cheia |
| Envio pela internet | **Não muda**: o mesmo vídeo ainda sobe uma vez por espectador | A regra de qualidade por número de espectadores (010) continua valendo |
| A placa de vídeo passa a trabalhar na codificação | Em jogo que usa a placa no limite, pode custar FPS | Medir com jogo. Não medido |
| Computador sem placa capaz, ou placa ocupada | Sem codificação pela placa | Voltar sozinho para a codificação no processador, com qualidade menor |

## O que não foi confirmado

| Item | Como se confirma |
|---|---|
| Qual codificador o app de hoje usa | Diagnóstico de mídia do site, com o app de hoje transmitindo |
| O vídeo codificado pela placa chega certo ao Chrome, ao app e ao Safari | Prova com um espectador de cada |
| Várias ligações podem receber o mesmo vídeo já codificado | Prova com 3 espectadores, um deles entrando no meio |
| Parar de mandar a quem não assiste (regra do Assistir) funciona nesse caminho | Prova: espectador sai de Assistir e a ligação para de receber |
| Comportamento do codificador das placas NVIDIA, AMD e Intel (atraso, quadro inteiro a pedido, controle de banda) | Prova em pelo menos uma placa de cada marca que o grupo tem |
| Custo no FPS de jogo que usa a placa no limite | Medição com jogo, lado a lado com o app de hoje |
| Estimativa de 1,7 núcleo para 3 espectadores codificando no processador | Só se interessar: rodar a sonda atual com 3 espectadores |
| Limite de codificações simultâneas nas placas e drivers do grupo | Nota da NVIDIA; conferir a versão atual antes de depender do número |

## Subtrabalhos sugeridos

Em ordem. Cada um termina com uma medição ou um "passou/falhou", e o seguinte só começa se o anterior passar.

| # | Subtrabalho | Entrega | Para se der errado |
|---|---|---|---|
| 1 | Ler o codificador do app de hoje | Uma linha no relatório | Só informa; não bloqueia |
| 2 | Prova: vídeo gerado, codificado pela placa, um espectador no Chrome | Chrome recebe 1080p60; processador da sonda medido | Parar: voltar à decisão R2 da 019 (biblioteca modificada ou outra biblioteca) |
| 3 | Prova: o mesmo vídeo para 3 espectadores, um entrando no meio, um saindo de Assistir | Os 3 recebem; processador quase igual ao de 1 | Avaliar uma codificação pela placa por espectador |
| 4 | Captura real ligada direto ao codificador, medida com jogo | FPS do jogo e processador, lado a lado com o app de hoje | Rever as metas SC-004 e SC-005 da 019 |
| 5 | Segunda codificação, reduzida, para miniatura e rede fraca | Miniatura e rede fraca atendidas sem afetar os outros | Miniatura recebe a cheia, com custo de rede |
| 6 | Computador sem placa capaz: codificação no processador | Transmite sem pergunta nova, em qualidade menor | Avisar que o computador não dá conta |
| 7 | Conferência com o grupo: Chrome, app, celular e iPhone | Roteiro manual da pessoa | Corrigir por aparelho |

Os subtrabalhos 2 e 3 são a prova que decide. São pequenos: reaproveitam a sonda do G1 (`app/spikes/webrtc-probe/`).

## Fontes

- Código: `~/.cargo/registry/src/*/libwebrtc-0.4.0/src/{video_source.rs,video_frame.rs,rtp_sender.rs}` e `webrtc-sys-0.3.53/{build.rs,src/passthrough_video_encoder.cpp,include/livekit/encoded_video_frame_buffer.h}`.
- Código do projeto: `src-tauri/src/capture/video.rs`.
- Medições: [relatorio.md da 019](../019-app-rust-convite/relatorio.md), seções G0 e G1.
- Limite de codificações nas placas GeForce: [nota da NVIDIA, SDK 12.2](https://docs.nvidia.com/video-technologies/video-codec-sdk/12.2/nvenc-application-note/index.html) (8 por sistema) e [SDK 11.1](https://docs.nvidia.com/video-technologies/video-codec-sdk/11.1/nvenc-application-note/index.html) (3).
