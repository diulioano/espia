# Data Model: Transmitir sem pesar no jogo, codificando uma vez pela placa de vídeo

**Feature**: [spec.md](./spec.md) · **Plano**: [plan.md](./plan.md) · **Decisões**: [research.md](./research.md)

**Nada é guardado em disco nem no servidor.** Tudo o que está neste documento vive na memória do aparelho de quem transmite, enquanto a transmissão existe, e some com ela. Não há arquivo novo, campo novo no protocolo, mensagem nova nem mudança no Worker. Nada sobre as pessoas é guardado (constituição III). A única coisa que vira texto fora da memória é o registro das medições, que fica em `relatorio.md`, escrito pela pessoa.

O que não foi confirmado fica marcado "não confirmado" ou "a confirmar", com o gate ou a história que resolve (tabela do fim de [research.md](./research.md)).

## Mapa das entidades

| Entidade | Onde vive | Quem escreve | Quem lê |
|---|---|---|---|
| Transmissão publicada | memória, `espia-media::publish` | publicador, a partir das regras de `espia-core::tiers` | tela (só o indicador de qualidade) |
| Qualidade | memória, `espia-core::quality` e `tiers` | funções puras | publicador |
| Espectador de uma transmissão | memória, `espia-core::tiers` | regras puras, a cada `ViewMessage` e a cada leitura de banda | publicador |
| Quadro comprimido | memória, em trânsito | codificador | fonte de vídeo da biblioteca |
| Pedido de quadro inteiro | memória, em trânsito | biblioteca (`take_keyframe_request`) | publicador, que junta os pedidos |
| Caminho de compressão | memória, `espia-core::tiers` | regras puras de escolha e troca | publicador |
| Linha do diagnóstico | memória, view-model | publicador | painel do Diagnóstico de mídia |
| Medição | texto em `relatorio.md` | pessoa, com `scripts/processador.ps1` | pessoa |

## Transmissão publicada

A tela, janela ou câmera que a pessoa está mostrando, do ponto de vista de quem comprime e envia. É a entidade "Transmissão" da spec, mais o que o publicador precisa para repartir o vídeo.

| Campo | Valores | Regra |
|---|---|---|
| Qualidade escolhida | `GoLive`: resolução 720 ou 1080, quadros por segundo 30 ou 60 | Vem das opções de hoje. Não muda no meio da transmissão. |
| Qualidade efetiva | `Quality { resolution, fps, max_bitrate }` | Sai de `effective_quality(escolhida, espectadores em tela cheia, degraus do processador)`. Muda ao vivo. |
| Caminho de compressão | `placa` ou `processador` (ver abaixo) | Um por transmissão. |
| Qualidades ativas | conjunto de `cheia`, `reduzida`; vazio, um ou os dois | Só no caminho pela placa. Ver "Qualidade". |
| Espectadores | lista de "Espectador de uma transmissão" | Um por pessoa que clicou em Assistir. |
| Indicador de qualidade | a qualidade efetiva, como hoje | O que a pessoa vê. Nada além disso (R12). |

Regras:

- Sem espectador recebendo, `qualidades ativas` fica vazio: nada é comprimido nem enviado (como hoje).
- No caminho pela placa existem no máximo duas qualidades ao mesmo tempo (FR-006), cada uma com uma fonte de vídeo e uma faixa (R5). O canal de vídeo de cada ligação aponta para a faixa da qualidade que aquele espectador recebe.
- No caminho pelo processador não há "cheia" e "reduzida": a biblioteca comprime por espectador, como na sonda do G1, e as regras de hoje (`send_params`) valem por pessoa.
- A transmissão de câmera segue as mesmas regras da de tela.

## Qualidade

Uma qualidade é um tamanho de imagem, uma taxa de quadros e uma taxa de envio, mais "quando está ativa". No caminho pela placa há duas.

### Cheia

| Campo | Valor | De onde vem |
|---|---|---|
| Resolução, quadros por segundo, taxa | a qualidade efetiva da transmissão | `effective_quality` (R6, R10) |
| Ativa quando | pelo menos um espectador recebe a cheia | R6, FR-007 |

Sem ninguém recebendo a cheia, ela para (nenhuma compressão dela roda).

### Reduzida

A reduzida tem **dois ajustes**, escolhidos por quem precisa dela (R6):

| Ajuste | Resolução | Quadros por segundo | Taxa | Usado quando |
|---|---|---|---|---|
| Miniatura | 360p | 15 | 500 kbps | Só há espectadores em miniatura precisando da reduzida. Os números são os de `send_params` para `ViewMode::Thumb`. |
| Internet fraca | 720p | 30 | 1,5 Mbps | Há pelo menos um espectador em tela cheia com internet fraca recebendo a reduzida. |

| Campo | Regra |
|---|---|
| Ativa quando | pelo menos um espectador recebe a reduzida (R6, FR-007). Sem ninguém, ela para. |
| Qual ajuste | "Internet fraca" se existe ao menos um espectador de tela cheia na reduzida; senão "Miniatura". Nunca os dois ao mesmo tempo, porque só há uma reduzida. |

Regras fechadas na R18 de [research.md](./research.md):

- No ajuste "Internet fraca", os espectadores em miniatura também passam a receber 720p30, porque só existe uma reduzida. É consequência de "no máximo duas qualidades". Regra fechada na R18.
- Quando a cheia já é 720p30 (o mínimo de `effective_quality`), a reduzida de internet fraca tem o mesmo tamanho e a mesma taxa de quadros, só uma taxa de envio menor. O research não trata o caso.

## Espectador de uma transmissão

Pessoa que clicou em Assistir. A cada momento recebe a cheia, a reduzida ou nada.

| Campo | Valores | Regra |
|---|---|---|
| Modo de visão | `full`, `thumb`, `hidden`, `off`, ou sem modo | Vem do `ViewMessage` (`ViewMode` em `quality.rs`). Sem modo: não recebe vídeo (`wants_video(None)` é falso). |
| Banda disponível | bits por segundo | Banda de saída disponível da própria ligação, lida das estatísticas uma vez por segundo (R9). |
| O que recebe | `cheia`, `reduzida` ou `nada` | Calculado pelas regras de [contracts/qualidades.md](./contracts/qualidades.md). |
| Contadores de banda | segundos seguidos abaixo de 60%, segundos seguidos em 90% ou mais, segundos desde a última tentativa | Só existem para espectador em tela cheia. |

### O que recebe, por modo

| Modo de visão | Recebe |
|---|---|
| `full`, banda boa | cheia |
| `full`, banda fraca (transição abaixo) | reduzida |
| `thumb` | reduzida |
| `hidden` | nada (`active=false`; o som continua, como hoje) |
| `off` | nada (`active=false`; sem som) |
| sem modo (não clicou em Assistir) | nada |

### Transições de um espectador em tela cheia

```text
cheia
  ──banda < 60% da taxa da cheia por 5 s seguidos, ou < 90% por 15 s seguidos,
    e nem todos os que recebem a cheia estão abaixo de 90%──▶  reduzida
  ──TODOS os que recebem a cheia estão abaixo de 90%──▶  continua cheia
        (ninguém desce; a taxa da cheia baixa para 90% da menor banda)

reduzida
  ──banda >= 90% da taxa da cheia por 15 s seguidos──▶  cheia
  ──30 s na reduzida sem ter voltado (tentativa)──▶  cheia, em teste
  (se a banda continua baixa, vale de novo a regra de descer)

qualquer estado
  ──modo vira thumb──▶  reduzida (miniatura)
  ──modo vira hidden, off ou some──▶  nada (active=false; faixa fica no canal)
  ──modo volta a full──▶  cheia, e a regra de banda recomeça do zero
```

Regras:

- Os limites são 60% e 5 segundos, ou 90% e 15 segundos, para descer; 90% e 15 segundos para subir; tentativa a cada 30 segundos (R9, R18). São ponto de partida, a confirmar na história 5.
- "Taxa da cheia" é a taxa da qualidade efetiva (`max_bitrate` de `Quality`).
- Banda entre 60% e 90%: quem está na cheia desce depois de 15 segundos seguidos; quem está na reduzida continua nela até a regra de volta.
- Quem está em `thumb` não passa pela regra de banda: recebe a reduzida de qualquer jeito.
- O alvo único de `take_rate_control_request()` **não decide nada**: guarda só o pedido da última ligação que falou. Ele só é registrado no diagnóstico (R9).
- Toda troca de qualidade é trocar a faixa do canal de vídeo daquela ligação, e o repassador pede quadro inteiro da faixa nova (R7). Parar de enviar é `active=false`; nunca tirar a faixa de um canal ativo.
- Se o computador de **quem transmite** é que tem internet fraca, todos os espectadores da cheia ficam abaixo ao mesmo tempo, ninguém desce e a taxa da cheia baixa para todos, como hoje (US5, cenário 3).

## Quadro comprimido

Um quadro de vídeo já comprimido, entregue pelo codificador à fonte de vídeo da biblioteca. É o que o app passa a produzir no lugar da biblioteca (R1).

| Campo | Valores | Observação |
|---|---|---|
| Codec | H.264 | O que Chrome e Safari recebem. Perfil Constrained Baseline (R4). |
| Parte | inteiro ou parcial | `frame_type` de `EncodedVideoFrame`. Inteiro abre o fluxo de quem entra; parcial só serve a quem já está recebendo. |
| Dados | bytes do quadro | Formato esperado: H.264 com códigos de início e parâmetros de sequência junto de cada quadro inteiro. **Não confirmado** (G4). |
| Tamanho em bytes | número | Só informativo; vai para o registro do G5 (tamanho do quadro inteiro em 1080p, R8). |
| Tamanho da imagem | largura × altura | `resolution` de `EncodedVideoFrame`. Igual ao da qualidade a que pertence. |
| Tempo de captura | microssegundos | `timestamp_us`. Marca o instante em que o quadro foi entregue ao codificador. Serve também para medir o atraso (G4 item f). |
| Qualidade | cheia ou reduzida | Cada quadro pertence a uma só qualidade e vai só à fonte dela. |

Regras:

- Um quadro é comprimido uma vez e entregue uma vez à fonte; a biblioteca o repassa a cada ligação (R5).
- O primeiro quadro de uma qualidade que acaba de ficar ativa é inteiro.
- Quadro inteiro só sai a pedido: não há quadro inteiro periódico (R8).

## Pedido de quadro inteiro

Pedido de um quadro inteiro novo de uma qualidade, feito pela biblioteca quando alguém entra no meio, troca de faixa ou perde um quadro.

| Campo | Valores | Regra |
|---|---|---|
| Qualidade | cheia ou reduzida | Cada qualidade tem a sua fila. Um pedido da cheia não força quadro inteiro da reduzida. |
| Chegada | instante da leitura | A cada quadro o app lê `take_keyframe_request()` de cada fonte (R8). |
| Último quadro inteiro | instante | Base da junção. |

Regras (R8):

- Havendo pedido, o próximo quadro daquela qualidade sai inteiro.
- Pedidos próximos são **juntados**: no máximo **um quadro inteiro a cada 300 ms por qualidade**.
- O pedido que chega dentro dos 300 ms é adiado até a janela passar, nunca descartado (R18). Descartar deixaria o espectador que pediu sem imagem.
- O efeito em quem já assiste (SC-007: não congelar mais de meio segundo) é medido no G5. Plano B: aumentar o intervalo mínimo.

## Caminho de compressão

Quem comprime a transmissão: a placa de vídeo ou o processador. Escolhido sozinho pelo app, por transmissão, sem pergunta (FR-014).

| Campo | Valores |
|---|---|
| Caminho | `placa` ou `processador` |
| Estado | `escolhendo`, `placa`, `processador` |
| Erros seguidos | contador (zera quando um quadro sai) |
| Tempo sem quadro | segundos desde o último quadro comprimido, enquanto alguma qualidade está ativa |
| Nome do codificador | texto, vindo do Windows (placa) ou o da biblioteca (processador) |

```text
escolhendo
  ──achou e iniciou o codificador da placa──▶  placa
  ──não achou codificador por hardware──▶  processador
  ──achou, mas não iniciou──▶  processador

placa
  ──3 erros seguidos──▶  processador          (no meio da transmissão)
  ──nenhum quadro comprimido em 2 s──▶  processador

processador
  (estado final nesta transmissão; a transmissão seguinte tenta a placa de novo, R18)
```

Regras (R11):

- No caminho pelo processador não há teto próprio: vale a qualidade escolhida, com o teto por número de espectadores e o ajuste automático por processador (`CpuGovernor`). Decisão do dono em 2026-11-12 (R18).
- A troca no meio da transmissão não pode exigir que a pessoa transmita de novo. Plano B se a troca sem renegociar não funcionar: parar e reabrir sozinho, com pausa de até 2 segundos.
- A troca nunca vira pergunta, aviso técnico ou opção. A pessoa só vê o indicador de qualidade e a linha do diagnóstico.
- Regras e casos de teste: [contracts/caminho.md](./contracts/caminho.md).

## Linha do diagnóstico

Uma linha nova no Diagnóstico de mídia, só neste aparelho (R12, FR-018).

| Campo | Valores |
|---|---|
| Texto | `Compressão: placa de vídeo` ou `Compressão: processador` |
| Nome do codificador | texto do codificador em uso. Placa: o que o Windows informa. Processador: o da biblioteca. |
| Quando aparece | durante a transmissão, no painel do Diagnóstico de mídia |
| Valor desconhecido | `—`, como as outras linhas do painel |

Regras:

- Não é opção: ninguém muda o caminho por aqui.
- Hoje o painel do app mostra as linhas como pares rótulo/valor (`DiagLine`, em `app/crates/espia-ui/ui/components/room-state.slint`) e o `view_model.rs` não traz o diagnóstico. Onde exatamente a linha entra (par rótulo/valor ou texto único; no `RoomView` ou no shell) **ainda não está decidido**. Ela ganha cena de dados falsos antes da lógica real (constituição V).
- O painel de hoje já mostra "Encoder" com o nome que a biblioteca informa. A linha nova diz **qual dos dois caminhos** está em uso.

## Medição

Um registro de uma medição, escrito pela pessoa em `relatorio.md`. Contrato: [contracts/medicao.md](./contracts/medicao.md).

| Campo | Valores | Regra |
|---|---|---|
| Data | AAAA-MM-DD | Obrigatória. |
| Máquina | processador, número de núcleos lógicos, placa de vídeo | A máquina do dono tem 24 núcleos lógicos. |
| Jogo | nome, ou "sem jogo" | |
| Qualidade | resolução e quadros por segundo da transmissão | Anotar a **efetiva**, não só a escolhida: com 5 ou mais pessoas em tela cheia ela cai (regra da 010). |
| Pessoas assistindo | número, e como foi conferido | Obrigatório. Vale quem está assistindo **naquele momento**, não quem entrou na sala. |
| FPS do jogo sem transmitir | número | Contador do próprio jogo. |
| FPS do jogo transmitindo | número ou faixa | Contador do próprio jogo. |
| Processador em núcleos | média de 60 s, e pico | `porcentagem da máquina × núcleos lógicos ÷ 100`. Para o app de hoje soma a árvore de processos do `Espia.exe`. |
| Aplicativo medido | app de hoje, app nativo ou sonda | |

Regras:

- Processador em núcleos, não em porcentagem: a porcentagem depende de quantos núcleos a máquina tem, e uma leitura de um instante engana (R15).
- Sem o número de pessoas assistindo conferido, a medição não vale como comparação (ressalva do dono sobre o G0, em research.md, "Contexto").
- A base de hoje (0,53 núcleo; 300 para 200 a 220 FPS; LoL) é uma leitura de um instante, com "até 3 pessoas". A comparação que vale é lado a lado, na mesma sessão.

## De onde vem cada número

| Número | Onde é usado | Decisão | Fonte no código |
|---|---|---|---|
| 360p, 15 quadros por segundo, 500 kbps | reduzida, ajuste miniatura | R6 | `send_params`, ramo `ViewMode::Thumb` de `quality.rs` (escala `resolução / 360`, 15, 500.000) |
| 720p, 30 quadros por segundo, 1,5 Mbps | reduzida, ajuste internet fraca | R6 | Não existe em `quality.rs`; é constante nova em `tiers.rs` |
| Cheia = qualidade efetiva | cheia | R6, R10 | `effective_quality` |
| 1 a 4, 5 a 6, 7 ou mais espectadores em tela cheia | teto da cheia | R10 (regra da 010) | `effective_quality`, `count_full_viewers` |
| 2,5 / 4 / 4,5 / 6 Mbps | taxa da cheia por faixa | R10 | `bitrate()` em `quality.rs` |
| 720p30 mínimo | menor cheia possível | R10, R11 | `step_down` |
| 10 s para descer, 60 s para subir um degrau | ajuste por processador | R10 (regra da 010) | `DOWN_AFTER`, `UP_AFTER` em `cpu_governor.rs` |
| 300 ms por qualidade | quadro inteiro | R8 | constante nova em `tiers.rs` |
| 60% por 5 s | descer para a reduzida | R9 | constantes novas em `tiers.rs` |
| 90% por 15 s | voltar para a cheia | R9 | constantes novas em `tiers.rs` |
| tentativa a cada 30 s | voltar para a cheia | R9 | constante nova em `tiers.rs` |
| 3 erros seguidos | troca para o processador | R11 | constante nova em `tiers.rs` |
| 2 s sem quadro | troca para o processador | R11 | constante nova em `tiers.rs` |
| pausa de até 2 s | plano B da troca | R11 | sem código |
| 60 s de média | medição | R15 | `scripts/processador.ps1` |
| 53 quadros por segundo, 3 s, 0,5 s, 25%, 80 ms | critérios de sucesso | spec (SC-004 a SC-009) | sem código |

Nenhuma constante nova está escrita ainda: `tiers.rs` não existe.

## Pontos fechados na revisão

Os pontos que este documento marcava como abertos (miniatura no ajuste de internet fraca, reduzida quando a cheia já é 720p30, pedido de quadro inteiro dentro dos 300 ms, volta da placa depois da troca, contagem dos 2 s, forma da linha do diagnóstico) foram fechados na R18 de [research.md](./research.md). Onde o texto acima diz "ver Pontos em aberto", vale a R18.

O que continua **não confirmado** é o que depende de prova, e está na tabela do fim de [research.md](./research.md).
