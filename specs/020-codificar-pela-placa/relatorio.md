# Relatório: 020 — codificar pela placa

**Modelo de bloco de medição**

- data (AAAA-MM-DD):
- máquina (processador, núcleos lógicos, placa de vídeo):
- jogo:
- qualidade efetiva:
- pessoas assistindo:
- como isso foi conferido:
- FPS do jogo sem transmitir:
- FPS do jogo transmitindo:
- aplicativo medido:
- núcleos (média de 60 s e pico):
- atraso entre entregar o quadro e receber o comprimido (só G4):
- passou ou falhou:

## Leitura do app de hoje

_(a preencher)_

## G4, uma pessoa

**Como foi medido (2026-11-12, roteiro automático rodado pelo Claude a pedido do dono):** sonda `encode-probe --room <sala> --ruido --stats-secs 5` contra o site local; Chrome de verdade (um perfil novo por janela, janelas lado a lado e visíveis) entrou na sala e clicou em Assistir; os números do Chrome vêm de `getStats()` da própria página (os mesmos de `chrome://webrtc-internals`), lidos uma vez por segundo. Nenhuma captura de tela: o vídeo é o gerado pela sonda. Máquina: AMD Ryzen 9 7900 (24 núcleos lógicos), NVIDIA GeForce GTX 1660 SUPER mais a AMD integrada. Pessoas assistindo conferidas pelo comando `peers` da sonda. Ninguém olhou a imagem nem ouviu o som: "imagem" aqui é quadro decodificado e "som" é energia de áudio medida pelo Chrome.

- (a) acha e inicia o codificador de H.264 da placa — critério: achou e iniciou — resultado: **passou.** Achou 3 (dois `AMDh264Encoder`, que não iniciam no dispositivo da NVIDIA, e `NVIDIA H.264 Encoder MFT`, que iniciou).
- (b) o Chrome recebe 1920×1080 em H.264 com som (`chrome://webrtc-internals`) — critério: frameWidth 1920, frameHeight 1080, codec H.264, som ouvido — resultado: **passou, com ressalva.** 1920×1080, `video/H264` (`profile-level-id=42001f`, `packetization-mode=1`), 0 pacote perdido. Som: Opus chegando; o site começa com o som do cartão silenciado, e depois de clicar em "Ativar som" a energia de áudio passou a subir. `decoderImplementation` não é exposto ao `getStats()` da página. Ninguém ouviu o bip.
- (c) pelo menos 53 quadros por segundo no Chrome — critério: 53 ou mais por pelo menos 1 minuto — resultado: **passou em regime, falhou no começo.** Nos primeiros ~30 s depois do Assistir: 5 congelamentos (4,9 s somados), 24 pedidos de quadro inteiro, 9 segundos abaixo de 53. Causa: o codificador manda 6 Mbps desde o primeiro quadro, e a banda estimada da ligação nova começa em ~0,75 Mbps e leva ~30 s para chegar a 6 Mbps (o pedido de taxa da biblioteca é só impresso). Depois disso: 60 s seguidos com mínimo de 59,2 e média de 60,0 quadros por segundo, 0 congelamento.
- (d) o Diagnóstico da sonda mostra que a compressão não é a de software da biblioteca — critério: não é OpenH264; origem da compressão é a placa — resultado: **passou.** `origem=placa`, `encoderImplementation='LiveKit pre-encoded passthrough'`. Depois do `fail`, a contraprova mostra `OpenH264`.
- (e) processador da sonda em núcleos, média de 60 s, registrado — critério: medido com `scripts/processador.ps1 -Processo encode-probe -Nativo` — resultado: 0,23 e 0,22 núcleo médio (pico 0,41 e 0,44), 1 pessoa, 53 amostras cada. Inclui gerar o quadro e o ruído, que o app não gasta.
- (f) atraso entre entregar o quadro e receber o comprimido registrado — critério: média e maior valor registrados — resultado: médio 5,3 ms, maior 25,4 ms (8.400 quadros). Com três pessoas: médio 4,7 ms, maior 45,5 ms.

## G5, três pessoas

**Como foi medido:** mesma sessão do G4, com mais dois Chromes entrando depois. Ver a nota do G4.

- (a) três pessoas recebem o mesmo vídeo — critério: 1920×1080, H.264, imagem e som nas três — resultado: **passou.** Os três Chromes em 1920×1080, H.264, ~60 quadros por segundo; `peers` lista as três ligações em `cheia (placa)`; um só codificador da cheia. Som: mesma ressalva do G4 (b).
- (b) processador com três em até 25% acima do de uma — critério: duas medições de 60 s; núcleos com três ≤ núcleos com uma × 1,25 — resultado: **falhou.** 1 pessoa: 0,23 e 0,22 (média 0,225). 3 pessoas: 0,32 e 0,32 (média 0,32). Razão 1,42, limite 1,25. Em valor absoluto são +0,05 núcleo por pessoa a mais. O plano B escrito (uma fonte por ligação) não ajuda: o custo que cresce é o de enviar, não o de comprimir.
- (c) quem entra no meio vê imagem em até 3 s — critério: até 3 s do clique até a primeira imagem — resultado: **passou.** 312 ms do clique ao primeiro quadro decodificado. Ressalva: o recém-chegado congela nos ~30 s seguintes pelo mesmo motivo do G4 (c).
- (d) os outros não congelam mais de 0,5 s nessa entrada — critério: `freezeCount` e `totalFreezesDuration` sem congelamento acima de 0,5 s; tamanho do quadro inteiro em 1080p anotado — resultado: **passou.** Chrome 1 e Chrome 2: 0 congelamento na entrada do terceiro, mínimo de 58,8 quadros por segundo nos 6 s em volta. Quadro inteiro em 1080p com ruído: 40 a 46 KB em média (62 KB nos primeiros). 77 pedidos da biblioteca viraram 41 quadros inteiros (janela de 300 ms).
- (e) parar de assistir e voltar funciona sem derrubar a sonda — critério: a sonda segue rodando; a imagem volta em poucos segundos — resultado: **passou.** Parado: `peers` mostra `nada (active=false)`, som cortado, 0 quadro decodificado em 2 s. Voltou em 106 ms. Sonda viva; os outros dois sem congelamento.
- (f) trocar um espectador para a reduzida e de volta funciona, sem mexer nos outros — critério: a imagem troca de tamanho e volta; os outros não mudam — resultado: **passou.** Chrome 2 foi a 640×360 a 15 quadros por segundo e voltou a 1920×1080; os outros dois ficaram em 1920×1080, 0 congelamento. Reduzida criada em 439 ms, troca de faixa em 0 ms, sem renegociar.
- (g) forçar a troca para o caminho pelo processador mantém a transmissão, sem ninguém clicar em nada — critério: transmissão continua para as três, com pausa de no máximo 2 s; duração da pausa anotada — resultado: **passou.** `fail`: as três ligações trocaram para `OpenH264` sem renegociar e sem clique; vídeo de volta em ~100 a 160 ms (medida da sonda). Nos 15 s seguintes cada Chrome decodificou ~900 quadros em 1920×1080; um segundo com queda (14,9 no Chrome 1, 51 a 52 nos outros). Plano B (`fail --reabrir`) não foi rodado com Chrome.

## G6, com jogo

- prévia (sonda com captura, T016) — resultado:
- G6 (T024) — resultado:

## História 2

_(a preencher)_

## História 3

_(a preencher)_

## História 4

_(a preencher)_

## História 5

_(a preencher)_

## História 6

_(a preencher)_

## Conferência com o grupo

_(a preencher)_

## Decisões

_(a preencher)_
