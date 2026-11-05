# Qualidade da transmissão, motor de mídia e transporte

Sessão grill-with-docs iniciada em 2026-11-05. Discussão aberta. As prioridades confirmadas estão registradas abaixo; as alternativas de arquitetura continuam em avaliação.

## O que o código confirma

- A captura do Windows já usa Rust, Windows.Graphics.Capture e D3D11 em `src-tauri/src/capture/video.rs`.
- A conversão produz NV12 na GPU, copia para uma textura staging e lê os pixels para buffers compartilhados. `src/lib/native-capture.ts` copia esses pixels para VideoFrame e entrega o track ao WebView2.
- O WebView2 mantém a codificação e o transporte WebRTC. `src/lib/peer.ts` prefere H.264, mas essa preferência não comprova uso de hardware em cada aparelho.
- `src/lib/mesh.ts` mantém uma conexão por pessoa. TURN mantém a mesma distribuição por par.
- `src/lib/quality.ts` limita 1080p60 a 6 Mbps por espectador. Com 5 ou 6 espectadores em visualização completa, aplica teto de 4 Mbps e reduz resolução se a escolha era 60 fps. Com 7 ou mais, aplica 720p30.
- O relatório da feature 016 registra correções e publicação da versão 0.1.9. A medição real do atraso ponta a ponta permanece pendente.

## Hipóteses que precisam de medição

O upload pode saturar com mais espectadores. A codificação por conexão pode disputar recursos com o jogo. A leitura de pixels da GPU e a ponte até o WebView2 podem consumir tempo. Nenhuma dessas hipóteses foi confirmada nesta sessão como gargalo dominante.

Comparar a mesma cena, qualidade efetiva, aparelhos e rede com 1 e 3 espectadores. Medir atraso visual ponta a ponta, fps recebido, resolução, bitrate, filas, RTT, perdas e implementação do encoder. Comparar rota direta e relay em rodadas separadas. Os indicadores getStats explicam etapas, mas não substituem atraso visual.

Se o gargalo for upload multiplicado por espectador, avaliar SFU. Se for cópia ou codificação local, avaliar um caminho nativo até o encoder de hardware. Se for a rota, avaliar TURN. Cada mudança exige comparação isolada.

## Alternativas de distribuição

| Alternativa | Upload de quem transmite | Consequência |
| --- | --- | --- |
| Conexão direta por espectador | Aproximadamente N vezes o bitrate | Evita relay quando a rota direta funciona bem |
| TURN por espectador | Aproximadamente N vezes o bitrate | Ajuda na conectividade e permite ocultar IPs dos pares com relay obrigatório |
| SFU com uma publicação | Aproximadamente o bitrate publicado | O servidor distribui as cópias; múltiplas camadas aumentam o upload |

TURN não recodifica a imagem para melhorar sua qualidade. O efeito no atraso depende da rota. A SFU exige rever a promessa atual de criptografia fim a fim entre participantes: a criptografia de transporte até a SFU não basta para manter essa promessa.

## Custo de tráfego

A documentação da Cloudflare consultada em 2026-11-05 informa US$ 0,05 por GB e franquia conjunta de 1.000 GB por mês para SFU e TURN. Workers e Durable Objects têm cobrança separada.

Para uma perna de saída faturável, GB decimais aproximados = Mbps médios × 0,45 × horas × espectadores. Áudio, protocolos e retransmissões acrescentam tráfego. Em TURN, contar cada saída da Cloudflare para um cliente TURN; uma rota com relay nas duas pontas pode acrescentar outra perna faturável. Validar pelo uso faturado, sem assumir que cada entrega custa exatamente uma cópia.

Exemplo aritmético a 6 Mbps constantes por entrega e 60 horas mensais: 3 espectadores recebem 486 GB; 9 recebem 1.458 GB. Com uma perna faturável por entrega, os valores após a franquia seriam US$ 0 e US$ 22,90. Com duas pernas de mesmo volume, seriam 972 GB e 2.916 GB, ou US$ 0 e US$ 95,80. São cenários de capacidade, não projeções do uso atual: a política atual reduz qualidade para grupos maiores.

Fontes: [preços Realtime](https://developers.cloudflare.com/realtime/sfu/platform/pricing/), [faturamento TURN](https://developers.cloudflare.com/realtime/turn/faq/), [política ICE relay](https://www.w3.org/TR/webrtc/#dom-rtcicetransportpolicy).

## Escopo e esforço de Rust

Migrar o motor significa decidir quais responsabilidades saem do WebView2. A captura já está em Rust. O WebRTC do navegador também usa código nativo internamente; mudar a linguagem de controle não garante ganho.

Estimativa preliminar de engenharia, sem orçamento fechado, para uma pessoa experiente em Windows, mídia e WebRTC, reaproveitando uma implementação WebRTC existente:

- Prova de conceito de envio nativo com uma GPU e um receptor: 1 a 2 semanas.
- Emissor Windows pronto para uso, mantendo a interface web: 6 a 12 semanas no total, incluindo integração e validação em GPUs e redes diferentes.
- Substituição ampla do motor de envio e recepção, com paridade de recursos: 3 a 6 meses ou mais no total.

As faixas são alternativas de escopo, não parcelas para somar. Um orçamento exige definir compatibilidade, bibliotecas, codecs e critérios de aceitação. O esforço inclui aceleração de hardware, sincronização de áudio e vídeo, controle de congestionamento, ICE, reconexão, troca de fonte e encerramento. Reimplementar WebRTC do zero fica fora dessas estimativas.

## Prioridades confirmadas em 2026-11-05

- Dimensionar duas transmissões simultâneas, com até sete espectadores podendo assistir às duas. O grupo costuma ter quatro ou cinco espectadores.
- Priorizar baixo atraso, adaptando resolução e fps ao conteúdo quando o PC ou a rede não sustentarem a qualidade escolhida.
- Preservar site e app, medir e melhorar por etapas. Uma migração ampla para Rust não é o próximo passo escolhido.
- Manter o uso de infraestrutura de mídia dentro da franquia gratuita. Não há autorização para assumir cobrança excedente.
- Meta de atraso ponta a ponta de até 500 ms na maior parte do tempo. Falta formalizar o percentil e as condições de ensaio. Proposta de aceitação: p95 até 500 ms em trechos estáveis, com recuperação avaliada separadamente e todos os atrasos registrados.
- Uso da sala superior a 60 horas mensais. Isso não confirma que as duas transmissões tenham sete espectadores durante todo o período.
- Preferência confirmada por conexão direta como padrão e bloqueio de SFU e TURN quando não houver saldo gratuito. Redes que dependem de relay podem ficar sem transmissão nessa condição.

## Capacidade da franquia para o cenário escolhido

Com duas publicações e sete assinaturas por publicação, há 14 entregas. Para uma SFU com uma saída faturável por entrega, a estimativa de vídeo é bitrate médio por entrega × 0,45 × 14 GB por hora de sala nesse cenário.

| Bitrate médio por entrega | Vídeo por hora de sala | Horas para 1.000 GB |
| --- | --- | --- |
| 6 Mbps | 37,8 GB | 26,5 h |
| 4 Mbps | 25,2 GB | 39,7 h |
| 2,5 Mbps | 15,75 GB | 63,5 h |

São cálculos de capacidade, não medições nem promessas de qualidade. Não incluem áudio, overhead, retransmissões ou outro consumo da franquia compartilhada. Os valores não equivalem automaticamente aos presets atuais: sete espectadores completos acionam 720p30 no modelo atual. TURN por par pode ter mais pernas faturáveis e não deve usar essa tabela sem conferir a rota.

Uso frequente pode tornar incompatíveis a distribuição integral por servidor e a restrição de custo zero. Nesse caso, avaliar conexão direta como padrão e servidor como recurso limitado. Permanecer na franquia exige uma política de controle de consumo, margem de segurança e comportamento de esgotamento; alertas de cobrança isolados não garantem custo zero.

## Integração de uma SFU no Espia

Uma prova de SFU pode reaproveitar a captura Rust e os tracks do site e do app. A mudança principal fica na distribuição de mídia atualmente controlada por mesh.ts e peer.ts. Uma publicação envia os tracks uma vez para a SFU; cada espectador solicita as publicações que quer receber.

Na alternativa gerenciada Cloudflare Realtime SFU, o Worker mantém o App Secret e faz as chamadas de criação de sessão, publicação, assinatura, renegociação e fechamento. O Durable Object da sala controla quais membros podem publicar e assinar cada track. IDs de sessão não substituem autorização de sala. A mídia flui entre clientes e SFU, sem atravessar o WebSocket da sala ou o Worker.

O protótipo mínimo usa um publicador e um receptor numa sala de teste. Depois amplia para duas publicações e sete receptores, incluindo grade, reconexão, parar de assistir, ocultar janela e consumo. Operações de uma mesma sessão SFU precisam ser serializadas. Encerrar um espectador deve encerrar sua assinatura; ocultar vídeo pode manter áudio conforme o comportamento atual.

A rota direta continua padrão. A SFU é uma alternativa a avaliar por teste, sem migração global aprovada. A política de uso sem cobrança exige contabilização agregada confiável, margem de segurança, interrupção de mídia já ativa e recusa de novas alocações ao atingir o limite. Contadores enviados pelos clientes e alertas de cobrança, sozinhos, não garantem ficar na franquia.

Hospedar uma SFU própria muda quem administra o servidor e paga o tráfego. Não elimina custo de banda, operação ou limites do upload se hospedada em casa. Nenhum serviço ou servidor novo foi provisionado.

Fontes da integração: [API SFU](https://developers.cloudflare.com/realtime/sfu/api/), [sessões e tracks](https://developers.cloudflare.com/realtime/sfu/concepts/sessions-tracks/), [negociação](https://developers.cloudflare.com/realtime/sfu/concepts/negotiation/).

## Pontos a validar no planejamento

Adendo solicitado: iniciar uma transmissão não deve fazer alguém começar a assistir automaticamente. Falta confirmar se a regra se refere aos espectadores, à prévia do próprio transmissor ou a ambos. No código atual, espectadores começam assistindo por padrão; a própria transmissão só entra no palco por ação explícita. Se o adendo se refere aos espectadores, a proposta é mostrar um cartão com Assistir e receber vídeo e som somente após esse clique. Em SFU, isso significa não criar a assinatura antes do clique, em vez de somente ocultar o player. Reiniciar uma transmissão deve voltar ao estado de aguardar clique, como proposta ainda a confirmar.

Frequência real do cenário máximo, percentil e condições da meta de atraso, contabilização e interrupção de uso de mídia, e preservação da promessa de criptografia caso a SFU avance. Esses pontos não autorizam substituir o transporte atual.

A recomendação provisória é medir o modelo atual com o grupo representativo e comparar uma prova de SFU se houver saturação por espectador. TURN obrigatório pode ser uma condição de teste, mas a escolha do transporte padrão continua aberta. Adoção de SFU exige avaliar também a promessa de criptografia entre participantes antes de registrar uma decisão de arquitetura.

Nenhuma implementação foi alterada. A direção de transporte e custo está registrada em docs/adr/0001-conexao-direta-e-franquia-de-midia.md. O glossário permanece inalterado porque não houve definição de um novo termo específico do produto.
