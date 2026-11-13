# Contrato: as regras de qualidade (R5 a R10)

**Feature**: [spec.md](../spec.md) · **Modelo**: [data-model.md](../data-model.md) · **Decisões**: [research.md](../research.md)

Tudo aqui vira funções puras em `app/crates/espia-core/src/tiers.rs`: sem rede, sem placa, sem relógio (cada passo de tempo é uma leitura por segundo, como em `cpu_governor.rs`). Os testes são escritos a partir das tabelas deste contrato. Os nomes de função ainda não existem; as tabelas dizem o que cada uma precisa responder.

Valem só para o caminho pela placa. No caminho pelo processador a biblioteca comprime por espectador e as regras de hoje (`send_params`) valem por pessoa ([caminho.md](./caminho.md)).

Os números de qualidade usados nos exemplos vêm de `app/crates/espia-core/src/quality.rs`.

## 1. Quem recebe o quê

Entrada: o modo de visão de cada espectador (`ViewMode`: `Full`, `Thumb`, `Hidden`, `Off`, ou sem modo) e se a banda dele está fraca (regra 5). Saída: `cheia`, `reduzida` ou `nada`.

| Modo de visão | Banda | Recebe |
|---|---|---|
| `Full` | boa | cheia |
| `Full` | fraca | reduzida |
| `Thumb` | qualquer | reduzida |
| `Hidden` | qualquer | nada (`active=false`; o som continua) |
| `Off` | qualquer | nada (`active=false`; sem som) |
| sem modo | qualquer | nada |

Quem não clicou em Assistir não tem modo e não recebe nada (FR-008).

Casos de teste:

| Entrada | Saída |
|---|---|
| `Full`, banda boa | cheia |
| `Full`, banda fraca | reduzida |
| `Thumb`, banda boa | reduzida |
| `Thumb`, banda fraca | reduzida (a banda não muda nada para miniatura) |
| `Hidden` | nada |
| `Off` | nada |
| sem modo | nada |

## 2. Quando cada qualidade fica ativa

A cheia fica ativa se pelo menos um espectador recebe a cheia. A reduzida, se pelo menos um recebe a reduzida (R6, FR-007).

| Espectadores | Cheia | Reduzida |
|---|---|---|
| nenhum | inativa | inativa |
| só `Hidden`, `Off` ou sem modo | inativa | inativa |
| só `Thumb` | inativa | ativa |
| só `Full`, bandas boas | ativa | inativa |
| `Full` e `Thumb` | ativa | ativa |
| `Full` com banda fraca e outro `Full` com banda boa | ativa | ativa |

Casos de teste:

| Entrada | Saída |
|---|---|
| 0 espectadores | nenhuma ativa |
| 2 `Thumb` | só a reduzida |
| 3 `Full`, bandas boas | só a cheia |
| 1 `Full`, 1 `Thumb` | as duas |
| 1 `Full` que passa a `Hidden`, mais nada | nenhuma ativa |
| 1 `Thumb` e 1 `Hidden` | só a reduzida |

## 3. Qual ajuste a reduzida usa

| Quem recebe a reduzida | Ajuste |
|---|---|
| só espectadores em `Thumb` | 360p, 15 quadros por segundo, 500 kbps (os números de `send_params` para `Thumb`) |
| pelo menos um espectador `Full` com banda fraca | 720p, 30 quadros por segundo, 1,5 Mbps |

Só existe uma reduzida, então todos os que a recebem recebem o mesmo ajuste. Quando há `Full` fraco e `Thumb` ao mesmo tempo, a miniatura recebe os 720p30. É a consequência de "no máximo duas qualidades"; o research não escreve isso explicitamente (ver "Pontos em aberto").

A cheia é sempre a qualidade efetiva: `effective_quality(escolhida, espectadores em tela cheia, degraus do processador)` (R6, R10). Quem está `Full` com banda fraca continua contando como espectador em tela cheia para o teto, porque `count_full_viewers` conta pelo modo e não pelo que a pessoa recebe. É a leitura mais direta do código. Regra fechada na R18.

Casos de teste:

| Entrada | Saída |
|---|---|
| 2 `Thumb` | reduzida em 360p15, 500 kbps |
| 1 `Full` fraco | reduzida em 720p30, 1,5 Mbps |
| 1 `Full` fraco e 1 `Thumb` | reduzida em 720p30, 1,5 Mbps |
| 1 `Full` fraco e depois volta à cheia, 2 `Thumb` ficam | reduzida volta a 360p15, 500 kbps |
| escolhida 1080p60, 4 espectadores em `Full` | cheia 1080p60, 6 Mbps |
| escolhida 1080p60, 5 espectadores em `Full` | cheia 720p60, 4 Mbps |
| escolhida 1080p60, 7 espectadores em `Full` | cheia 720p30, 2,5 Mbps |
| escolhida 720p30, 1 `Full` fraco | cheia 720p30, 2,5 Mbps; reduzida 720p30, 1,25 Mbps (mesmo tamanho; metade da taxa da cheia, R18) |

## 4. Parar de enviar

- Parar de enviar a uma ligação é `active=false` no canal de vídeo dela (R7).
- **Nunca** tirar a faixa de um canal ativo. Na sonda do G1, `set_track(None)` num canal ativo derrubava o processo.
- A faixa continua no canal. Voltar a enviar é `active=true`, e o repassador pede quadro inteiro.
- O som segue `wants_sound`: só `Off` o corta.

Casos de teste:

| Entrada | Saída |
|---|---|
| `Full` vira `Hidden` | `active=false`, faixa mantida, som continua |
| `Full` vira `Off` | `active=false`, faixa mantida, sem som |
| `Hidden` vira `Full` | `active=true`, pedido de quadro inteiro da cheia |
| `Thumb` vira `Off` | `active=false`; se era o último da reduzida, a reduzida para |
| qualquer saída | nenhuma ação "tirar faixa" para um canal com `active=true` |

## 5. Trocar a qualidade de um espectador

- Trocar é apontar o canal de vídeo daquela ligação para a faixa da outra qualidade (R7). Os outros espectadores não são tocados.
- O repassador percebe a fonte nova, segura os quadros e pede quadro inteiro da qualidade nova.
- A qualidade de destino precisa estar comprimindo (ativa) quando o canal passa a apontar para ela. Se não estava, ela é ativada antes ou junto, e o primeiro quadro dela é inteiro. A ordem exata entre ativar e apontar é um detalhe do `publish.rs`, não uma regra pura.

Casos de teste:

| Entrada | Saída |
|---|---|
| espectador A passa de cheia para reduzida; B e C ficam na cheia | só o canal de A muda de faixa; pedido de quadro inteiro da reduzida |
| A passa de cheia para reduzida e a reduzida estava inativa | reduzida passa a ativa |
| A passa de reduzida para cheia e era o único na reduzida | reduzida passa a inativa |
| A passa de cheia para reduzida e era o único na cheia | cheia passa a inativa (com mais ninguém `Full`) |

## 6. Quadro inteiro a pedido

Entrada: os pedidos que a biblioteca fez (lidos com `take_keyframe_request()` de cada fonte a cada quadro), o instante de cada um e o instante do último quadro inteiro de cada qualidade. Saída: se o próximo quadro daquela qualidade sai inteiro (R8).

Regras:

- Havendo pedido, o próximo quadro da qualidade sai inteiro.
- Pedidos próximos são juntados: **no máximo um quadro inteiro a cada 300 ms por qualidade**.
- Cada qualidade tem a sua conta. Um pedido da cheia não faz a reduzida emitir quadro inteiro.
- Não há quadro inteiro periódico.
- O primeiro quadro de uma qualidade que acabou de ficar ativa é inteiro.

O pedido que chega dentro dos 300 ms é **adiado** até a janela passar, nunca descartado (R18). Descartar deixaria o espectador que pediu sem imagem.

Casos de teste (instantes em ms; qualidade única):

| Entrada | Saída |
|---|---|
| pedido em 0, sem quadro inteiro antes | quadro inteiro já no próximo quadro |
| pedidos em 0, 100 e 200 | um quadro inteiro em 0 e um segundo, juntando os dois pedidos seguintes, em 300 (adiado) |
| pedidos em 0 e em 300 | dois quadros inteiros, em 0 e 300 |
| pedidos em 0 e em 299 | o segundo sai em 300, não em 299 |
| 60 s sem pedido | nenhum quadro inteiro além do primeiro |
| pedido na cheia, nenhum na reduzida | só a cheia emite quadro inteiro |
| reduzida acaba de ficar ativa | primeiro quadro dela é inteiro |

## 7. Internet de cada espectador (R9)

Entrada: a banda de saída disponível de cada ligação, uma leitura por segundo, e a taxa da cheia (`max_bitrate` da qualidade efetiva). Saída: quais espectadores `Full` estão com banda fraca, e a taxa que a cheia deve ter.

Regras:

| Regra | Valor |
|---|---|
| Descer para a reduzida | banda abaixo de **60%** da taxa da cheia por **5 s** seguidos, ou abaixo de **90%** por **15 s** seguidos |
| Voltar para a cheia | banda em **90%** ou mais da taxa da cheia por **15 s** seguidos |
| Tentativa de volta | uma a cada **30 s**, se a estimativa não subiu sozinha |
| Todos abaixo | se **todos** os que recebem a cheia estão abaixo de 90%, ninguém desce e a taxa da cheia baixa para 90% da menor banda |
| Taxa da cheia | a nominal, salvo no caso "todos abaixo" |

Valores são ponto de partida; confirmam na história 5. O alvo único de `take_rate_control_request()` não decide nada: guarda só o pedido da última ligação que falou, e é só registrado no diagnóstico.

Os exemplos usam taxa da cheia de 6 Mbps (1080p60): 60% são 3,6 Mbps e 90% são 5,4 Mbps.

Casos de teste (uma leitura por segundo; os outros espectadores com banda boa, salvo quando dito):

| Entrada | Saída |
|---|---|
| A em cheia com 4 Mbps por 60 s | A passa à reduzida aos 15 s (abaixo de 90%) |
| A com 3,5 Mbps por 4 s e depois 6 Mbps | A continua na cheia (contadores zerados) |
| A com 3,5 Mbps por 5 s seguidos | A passa à reduzida |
| A na reduzida com 5,4 Mbps por 14 s | A continua na reduzida |
| A na reduzida com 5,4 Mbps por 15 s | A volta à cheia |
| A na reduzida com 5,0 Mbps (entre 60% e 90%) por 60 s | A continua na reduzida, sem tentativa antes dos 30 s |
| A na reduzida, banda sem subir por 30 s | tentativa: A volta à cheia em teste |
| A, B e C na cheia, os três com 3,0 Mbps por 5 s | ninguém desce; taxa da cheia baixa para 2,7 Mbps (90% de 3,0) |
| só A na cheia, com 3,0 Mbps por 5 s | A não desce (sozinho nunca desce); taxa da cheia baixa para 2,7 Mbps |
| A com banda fraca, B e C boas | só A desce; B e C continuam na cheia na taxa cheia |
| A `Thumb` com 0,2 Mbps | A continua na reduzida e não entra na conta de "todos abaixo" |

Quem transmite com internet fraca: todos os espectadores da cheia ficam abaixo ao mesmo tempo, ninguém desce e a taxa da cheia baixa para todos, e o indicador de qualidade que já existe mostra a mudança (US5, cenário 3).

Fechado na R18 de [research.md](../research.md):

- "Todos abaixo": todos os que recebem a cheia abaixo de 90% da nominal. Um espectador sozinho nunca desce.
- Os 60% e os 90% são sempre da taxa nominal da qualidade efetiva, não da taxa já baixada.
- Taxa da cheia: a nominal. Só no caso "todos abaixo", a cada segundo: o menor valor entre a nominal e 90% da menor banda entre quem recebe a cheia. Sobe de volta pela mesma conta.
- Tentativa de volta que não dá certo: vale de novo a regra de descer.
- Se a estimativa de banda sobe de volta enquanto a ligação recebe só a reduzida continua **não confirmado** (história 5; plano B: volta só por tentativa periódica).

Casos de teste a mais:

| Entrada | Saída |
|---|---|
| 1 espectador `Full`, banda em 40% da nominal por 10 s | não desce; taxa da cheia = 90% da banda dele |
| taxa nominal 6 Mbps; bandas de 5, 4 e 8 Mbps | taxa da cheia = 6 Mbps (o de 8 está bem); os de 5 e 4 descem aos 15 s |
| taxa nominal 6 Mbps; bandas de 9, 10 e 8 Mbps | taxa da cheia = 6 Mbps |
| taxa nominal 6 Mbps, cheia já baixada para 3 Mbps; um espectador com 3,2 Mbps de banda | conta como abaixo: o limite continua 3,6 Mbps (60% da nominal) |

## 8. Convivência com `effective_quality` (R10)

- A qualidade efetiva continua saindo de `effective_quality`: escolhida, número de espectadores em tela cheia, degraus do processador. O número de espectadores continua baixando a qualidade, porque o mesmo vídeo sobe uma vez para cada pessoa (FR-011).
- O ajuste "o computador não dá conta" passa a olhar o codificador da placa: quadros que ele não conseguiu comprimir a tempo contam como "limitado" para o `CpuGovernor` (10 s limitado baixa um degrau, 60 s de folga sobe um, `cpu_governor.rs`).
- O indicador de qualidade continua mostrando a qualidade efetiva.
- Qual sinal o codificador dá quando a placa está no limite é **não confirmado** (G6). Plano B: usar o atraso entre entregar o quadro e receber o comprimido.

Casos de teste (já cobertos pelos testes de `quality.rs`; repetir no que `tiers.rs` calcular):

| Entrada | Saída |
|---|---|
| escolhida 1080p60, 3 espectadores `Full`, 1 degrau do processador | cheia 1080p30, 4,5 Mbps |
| escolhida 1080p60, 3 espectadores `Full`, 2 degraus | cheia 720p30, 2,5 Mbps |
| escolhida 720p30, 5 espectadores `Full` | cheia 720p30, 2,5 Mbps (nunca sobe acima do escolhido) |
| 2 espectadores `Thumb`, 4 `Full` | só os 4 `Full` contam para o teto |
| a placa fica "limitada" por 10 s | um degrau a menos na cheia |

## Pontos fechados na revisão

Miniatura no ajuste de internet fraca, reduzida com a cheia já em 720p30, contagem de `Full` fraco no teto por espectadores, pedido de quadro inteiro dentro dos 300 ms e os detalhes de "todos abaixo" foram fechados na R18 de [research.md](../research.md). O contrato acima já segue essas regras; onde o texto diz "ver Pontos em aberto", vale a R18.

Continua **não confirmado**, por depender de prova: o sinal de "limitado" no codificador da placa (G6) e a subida da estimativa de banda recebendo a reduzida (história 5).
