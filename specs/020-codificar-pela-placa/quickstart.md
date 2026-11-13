# Quickstart: validar a compressão pela placa de vídeo (020)

Referências: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/codificador.md](./contracts/codificador.md) · [contracts/qualidades.md](./contracts/qualidades.md) · [contracts/caminho.md](./contracts/caminho.md) · [contracts/medicao.md](./contracts/medicao.md)

Este guia diz como validar, não como construir. Anote todo resultado em `specs/020-codificar-pela-placa/relatorio.md` (criado na primeira medição), com data (AAAA-MM-DD), máquina e versão testada, no formato de [contracts/medicao.md](./contracts/medicao.md).

## 1. Antes de começar

### Pré-requisitos

- Node e as dependências do site instaladas (`npm install`), como nas features anteriores.
- Toolchain do Rust (Rust 1.90, edição 2024), para compilar `app/` e a sonda.
- Windows 10 ou 11, x64, com placa de vídeo NVIDIA, AMD ou Intel. A prova roda primeiro na máquina do dono.
- Chrome no computador, para ser o "amigo no navegador". Um segundo aparelho com Chrome, para as provas com mais de uma pessoa.
- Um iPhone com Safari e alguém do grupo para o teste (história 4).
- O app de hoje (0.1.10) instalado, para a leitura do que ele usa e para as comparações lado a lado.
- O jogo de referência (League of Legends nas medições de 2026-11-12) e a mesma máquina da base do G0. Sem a mesma máquina e o mesmo jogo, a comparação não vale.
- `scripts/processador.ps1` ([contracts/medicao.md](./contracts/medicao.md)), criado na etapa 1. Antes dele não existe.
- Para quem mede: saber quantos núcleos lógicos a máquina tem (a do dono tem 24).

### Rodar o site localmente

```powershell
npm run dev:worker     # vite build + wrangler dev: site e servidor locais, http://127.0.0.1:8787
npm run dev            # só o site, para abrir os mocks
```

Para testar com um celular na mesma rede, `npm run dev:lan` (como na 018).

### Regra de segurança

Teste automático nunca captura a tela de verdade. Captura real (tela, janela, câmera, som) só é testada à mão, por uma pessoa, com a janela de teste escolhida por ela. Antes do G6 a sonda gera o vídeo (barras coloridas e contador), como a do G1.

### Gates automáticos

```powershell
npm run check
npm test
npm run build
npm run app:check
npm run app:test
```

**Esperado**: tudo verde. A partir da etapa 1 inclui os testes de `tiers.rs` escritos a partir de [contracts/qualidades.md](./contracts/qualidades.md) e [contracts/caminho.md](./contracts/caminho.md).

### Como saber quem está assistindo

Toda medição precisa disso (R15). Duas conferências, juntas:

1. Cada pessoa confirma que clicou em Assistir e que a janela não está escondida.
2. No painel de diagnóstico de quem transmite (abaixo), cada pessoa recebendo vídeo aparece como um bloco "Conexão" com "Vídeo: Enviado". Confira também que a taxa ("Bitrate no intervalo") é maior que zero. **A confirmar**: se uma pessoa que parou de assistir ou escondeu a janela some do painel ou fica com taxa zero. O código mostra um bloco só para conexões com vídeo na amostra (`MediaDiagnostics.svelte`), mas não foi visto o que acontece com o canal em pausa.

## 2. Leitura do app de hoje

Só informa; não bloqueia. Responde "qual compressão o app 0.1.10 usa", que a spec deixou sem leitura. Resultado: uma linha no relatório.

O painel que o site chama de "Diagnóstico da transmissão" é o que esta feature chama de Diagnóstico de mídia. No código ele é `src/ui/components/MediaDiagnostics.svelte`, aberto na sala (`src/routes/RoomRoute.svelte`). É um painel só deste aparelho.

Passo a passo:

1. Abra o app 0.1.10 e entre numa sala. Comece a transmitir, de preferência em 1080p60.
2. Peça a pelo menos uma pessoa para clicar em Assistir. Sem ninguém recebendo, o painel mostra "Aguardando estatísticas da conexão."
3. Na janela do app de quem transmite, aperte **Ctrl+Shift+D**. O painel abre no canto de baixo à direita. Ctrl+Shift+D de novo ou Esc fecha.
4. Procure um bloco "Conexão <código>" com "Vídeo: Enviado". Leia a linha **Encoder**.
5. Anote o texto exato, a qualidade da transmissão e quantas pessoas estavam assistindo.

Como ler:

- O código do site diz que um nome do tipo `MediaFoundationVideoEncodeAccelerator` quer dizer placa de vídeo (comentário em `src/lib/peer.ts`).
- A sonda do G1, que comprime pelo processador, mostrou OpenH264 como codificador (relatório da 019). Um nome com OpenH264 indica processador.
- O painel mostra só o nome. O código lê também `powerEfficientEncoder` (verdadeiro quer dizer placa), mas o painel não o exibe hoje.
- "—" quer dizer dado indisponível. Anote assim.

**A confirmar**: se o atalho Ctrl+Shift+D funciona dentro da janela do app 0.1.10. O app carrega a interface do próprio pacote (`frontendDist`, `src-tauri/tauri.conf.json`), então deve ser a mesma, mas não foi aberto. Se não abrir, a alternativa é transmitir a mesma sala pelo site no Chrome e abrir o painel lá, sabendo que o motor de página do Chrome pode escolher outro codificador que o do app. **A confirmar** também: se o app 0.1.10 já inclui esse painel.

## 3. Gates

Regra: **gate vermelho para o que vem depois**. Se um gate falhar, parar, registrar em `relatorio.md` e replanejar com o plano B da decisão ([plan.md](./plan.md), "Gates"). Resultado de cada gate, uma linha por item.

Os comandos abaixo são os de [app/spikes/encode-probe/LEIA-ME.md](../../app/spikes/encode-probe/LEIA-ME.md), rodados em `app/spikes/encode-probe/`. A sonda nova parte dos comandos da sonda do G1 ([app/spikes/webrtc-probe/LEIA-ME.md](../../app/spikes/webrtc-probe/LEIA-ME.md)). A sonda do G1 aceita, por exemplo, `--room <código>`, `--no-live`, `--view full|thumb|hidden|off`, `--stats-secs <s>` e `--minutes <n>`, e comandos digitados como `view full`, `active off` e `quit`. A nova a reaproveita, e o `LEIA-ME.md` dela, em `app/spikes/encode-probe/LEIA-ME.md`, é quem fixa os nomes.

Variáveis usadas abaixo: `$SALA` = código da sala criada no Chrome em `http://127.0.0.1:8787`; "a sonda" = executável de `app/spikes/encode-probe/`.

### G4, uma pessoa (`app/spikes/encode-probe/`)

| Item | O que fazer | O que observar |
|---|---|---|
| (a) acha e inicia o codificador de H.264 da placa | Rodar a sonda na sala: `.\target\release\encode-probe.exe --room <código> --ruido --stats-secs 5` (sem sala: `.\target\release\encode-probe.exe --encode-only 10`). O comando precisa entrar na sala, gerar vídeo 1080p60 de teste, procurar o codificador por hardware do Windows e iniciá-lo. | No log da sonda, o nome do codificador achado e "iniciou". **Passa** se achou e iniciou. Se a máquina do dono não achar nenhum, o item falha. |
| (b) o Chrome recebe 1920×1080 em H.264 com som | No Chrome, entrar na sala e clicar em Assistir na transmissão da sonda. | `chrome://webrtc-internals`, aba da ligação, `inbound-rtp` de vídeo: `frameWidth=1920`, `frameHeight=1080`, `codecId` apontando para `video/H264`, `decoderImplementation`. Ouvir o som de teste. Anotar sim ou não. |
| (c) pelo menos 53 quadros por segundo no Chrome | Mesmo cenário. | `framesPerSecond` no `inbound-rtp` por pelo menos 1 minuto. **Passa** se ficar em 53 ou mais. Anotar o menor valor visto. |
| (d) o Diagnóstico da sonda mostra que a compressão não é a de software da biblioteca | Rodar com a impressão de estatísticas ligada: `.\target\release\encode-probe.exe --room <código> --ruido --stats-secs 5` (contraprova: o mesmo com `--backend software`). O comando precisa imprimir, como a sonda do G1 faz com `--stats-secs`, o nome do codificador de envio, e a sonda nova precisa dizer de forma clara se o quadro veio do codificador da placa ou da biblioteca. | O nome do codificador **não** é OpenH264. **Não confirmado**: o que `encoderImplementation` mostra quando o quadro chega pronto do app (`PreEncoded`); nunca foi visto. |
| (e) processador da sonda em núcleos, média de 60 s | Com o Chrome assistindo, em outro terminal: `scripts/processador.ps1 -Processo encode-probe -Nativo` ([medicao.md](./contracts/medicao.md)). | Núcleos médios e pico. Registrar com as demais colunas de medição. Referência: a sonda do G1 gastou 0,57 núcleo em software para 1 pessoa, sem jogo. |
| (f) atraso entre entregar o quadro e receber o comprimido | A sonda precisa medir e imprimir o tempo entre entregar o quadro ao codificador e receber o quadro comprimido, média e maior valor. Comando: `.\target\release\encode-probe.exe --room <código> --ruido --stats-secs 5`, lendo a linha `[codificador]`. | O valor impresso. Só registrado, não há limite escrito. |

**Passa**: (a) a (f) cumpridos. **Falhou**: volta-se à decisão R2 da 019 (biblioteca modificada ou outra biblioteca). Itens que o G4 resolve: formato do quadro, baixa latência, quadro inteiro a pedido, alvo ao vivo ([research.md](./research.md), tabela do fim).

### G5, três pessoas (`app/spikes/encode-probe/`)

| Item | O que fazer | O que observar |
|---|---|---|
| (a) três pessoas recebem o mesmo vídeo | Três Chromes (ou três abas em aparelhos diferentes) assistindo à sonda. | Os três em `webrtc-internals`: 1920×1080, H.264, quadros por segundo. Os três veem imagem e ouvem o som. |
| (b) processador com três em até 25% acima do de uma | Medir com 1 pessoa e depois com 3, cada vez 60 s: `scripts/processador.ps1 -Processo encode-probe -Nativo`. Conferir quem assiste. | Núcleos médios com 3 ≤ núcleos médios com 1 × 1,25. Falha: plano B, uma fonte por ligação (R5). |
| (c) quem entra no meio vê imagem em até 3 s | Com duas pessoas assistindo, a terceira clica em Assistir. Cronometrar do clique até a primeira imagem. | Até 3 s. Repetir algumas vezes e anotar o maior. |
| (d) os outros não congelam mais de 0,5 s nessa entrada | No mesmo momento, olhar as duas que já assistiam. | Em `webrtc-internals`, `freezeCount` e `totalFreezesDuration` das duas, antes e depois da entrada. Nenhum congelamento acima de 0,5 s. Anotar também o tamanho do quadro inteiro em 1080p (R8). Plano B: aumentar o intervalo mínimo entre quadros inteiros. |
| (e) parar de assistir e voltar funciona sem derrubar a sonda | Uma pessoa clica para parar de assistir, espera, e clica em Assistir de novo. Variante: esconde a janela. | A sonda continua rodando. Ao parar, o vídeo dela cessa (`active=false`); ao voltar, a imagem volta em poucos segundos. Os outros não são afetados. |
| (f) trocar um espectador para a reduzida e de volta funciona | No teclado da sonda, `peers`, depois `tier <início do id> reduced` e `tier <início do id> full`: comando que força um espectador para a reduzida e de volta à cheia, sem esperar a regra de banda. O mesmo efeito vem de colocar a janela de um espectador em miniatura e voltar a tela cheia. | O espectador vê a imagem trocar de tamanho e voltar; os outros não mudam. Plano B: dois canais por ligação entre apps (R7). |
| (g) forçar a troca para o caminho pelo processador mantém a transmissão | No teclado da sonda, `fail` (plano B, numa sessão nova: `fail --reabrir`): comando que simula a falha do codificador da placa (ou o desliga). | A transmissão continua para as três pessoas, com pausa de no máximo 2 s se o plano B for o usado. Anotar a duração da pausa e se alguém precisou clicar em algo (não pode). |

**Passa**: (a) a (g). **Falhou**: planos B do [plan.md](./plan.md). Só com G5 verde começa a etapa 3 (`encode/` e `publish.rs`).

### G6, com jogo (pessoa)

| Item | O que fazer | O que observar |
|---|---|---|
| Lado a lado com o app de hoje, na mesma sessão, com o número de pessoas assistindo conferido: perda de FPS do jogo não maior que a do app de hoje; processador em no máximo metade | App nativo com captura real, ou a sonda com captura. Mesma máquina, mesmo jogo, 1080p60 com som, mesmas 3 pessoas (conferidas). Primeiro o app de hoje, depois o novo (ou o inverso), sem fechar o jogo. Em cada um: anotar o FPS do jogo sem transmitir, o FPS transmitindo, e rodar `scripts/processador.ps1` (árvore para o app de hoje; `-Nativo` para o novo). | Perda de FPS no novo ≤ perda no app de hoje (base de 2026-11-12: 300 para 200 a 220). Núcleos do novo ≤ metade dos do app de hoje (base: 0,53 núcleo, leitura de um instante com "até 3 pessoas"). Anotar também a qualidade efetiva. Plano B: entrada pela memória (R3) e revisão das metas. |

Também se resolve aqui, no G6: captura e codificador no mesmo dispositivo gráfico (R3), o sinal de "placa no limite" (R10) e a janela que muda de tamanho no meio da transmissão.

## 4. Roteiro por história

A história 1 é a prova (G4 e G5, SC-001). As histórias abaixo valem depois do G5, com o app nativo ou, enquanto ele não existe, com a sonda. Cada passo diz o que prova.

### História 2: Transmitir sem derrubar o jogo (SC-002, SC-003, SC-012)

Precisa de: o jogo, 3 pessoas, G6 feito ou em andamento.

| Passo | O que fazer | Prova |
|---|---|---|
| 1 | No app de hoje, jogar sem transmitir e anotar o FPS pelo contador do jogo. | base |
| 2 | Transmitir 1080p60 para 3 pessoas (conferidas). Anotar o FPS e rodar `scripts/processador.ps1`. | SC-003 (app de hoje), SC-002 (app de hoje) |
| 3 | Repetir com o app nativo, mesma sessão, mesmo jogo, mesmas 3 pessoas. | SC-003: perda de FPS não maior; SC-002: no máximo metade dos núcleos |
| 4 | No app nativo, mudar a cena do jogo de parada para de muito movimento. | a imagem de quem assiste continua fluida, sem congelar (história 2, cenário 3) |
| 5 | Medir a memória do app nativo transmitindo: `npm run memoria -- -Nativo`. | SC-012 (meta que a 019 fixou) |
| 6 | Olhar os passos até a transmissão começar. | iguais aos de hoje, sem pergunta nova e sem opção sobre placa de vídeo (FR-014, cenário 4) |

### História 3: Mais gente assistindo não pesa mais (SC-004)

Precisa de: 7 pessoas assistindo (a sala comporta 10). Dá para fazer com a sonda se não houver sete pessoas reais, abrindo mais abas e aparelhos; anote que não eram pessoas.

| Passo | O que fazer | Prova |
|---|---|---|
| 1 | Transmitir com 1 pessoa em tela cheia. 60 s de `scripts/processador.ps1`. Anotar a qualidade efetiva. | base |
| 2 | Repetir com 3 e depois com 7 pessoas em tela cheia, conferidas. | SC-004: de 1 a 7, núcleos crescem no máximo 25% |
| 3 | Uma das pessoas sai. Olhar as que ficaram. | nada muda para as que ficaram (cenário 2) |
| 4 | Conferir no indicador de qualidade o que acontece com 5, 6 e 7 pessoas. | a regra de qualidade pelo número de espectadores continua (cenário 3, FR-011) |

Atenção: a regra da 010 reduz a qualidade com 5 ou mais pessoas em tela cheia (720p60 com 5 e 6, 720p30 com 7). A comparação de 1 a 7 mistura, portanto, qualidades diferentes. Anote a qualidade efetiva de cada medição.

### História 4: Quem assiste não percebe diferença (SC-005, SC-006, SC-007, SC-008, SC-011)

Precisa de: uma pessoa transmitindo pelo app nativo; Chrome no computador, o app, Chrome no celular e Safari no iPhone para assistir.

| Passo | O que fazer | Prova |
|---|---|---|
| 1 | Em cada um dos quatro aparelhos, clicar em Assistir. | SC-011: imagem e som nos quatro |
| 2 | Cronometrar do clique até a primeira imagem. Repetir entrando no meio da transmissão. | SC-006: até 3 s |
| 3 | No Chrome, `chrome://webrtc-internals`: quadros por segundo com 1080p60 em tela cheia. | SC-005: 53 ou mais |
| 4 | Com duas pessoas já assistindo, uma terceira entra. Olhar as duas. | SC-007: nenhum congelamento acima de 0,5 s |
| 5 | Deixar assistindo por 10 minutos. Conferir que som e imagem seguem juntos. | SC-008: até 80 ms. Método: o da sonda do G1, estatística do próprio motor e, para medida independente, duas sondas (G1, item g) |
| 6 | Colocar a tela em miniatura. | continua atualizando, em qualidade menor, gastando menos internet (cenário 3) |
| 7 | Parar de assistir, ou esconder a janela. | para de receber imagem (cenário 4) |
| 8 | Voltar a assistir. | imagem volta em poucos segundos (cenário 5, SC-006) |
| 9 | Alguém que não clicou em Assistir. | não recebe imagem nem som (cenário 6, FR-008) |

SC-005 só vale com até 4 pessoas em tela cheia: com 5 ou mais a qualidade efetiva já é menor que 1080p60 (regra da 010).

### História 5: Internet fraca de um não estraga a dos outros (SC-009)

Precisa de: 3 pessoas assistindo em tela cheia, e um jeito de limitar a internet de uma delas. Para limitar: no Chrome de quem assiste, a aba Rede das ferramentas do desenvolvedor não limita a ligação de vídeo; use o limite de banda do próprio roteador ou do celular (por exemplo, assistir pelo 4G com sinal fraco), ou um programa de limite de banda no computador dessa pessoa. O método usado fica anotado no `relatorio.md`.

| Passo | O que fazer | Prova |
|---|---|---|
| 1 | Com as 3 em qualidade cheia, limitar a internet de uma delas. | só ela passa à reduzida (cenário 1, FR-010) |
| 2 | Olhar as outras duas em `webrtc-internals`. | SC-009: pelo menos 53 quadros por segundo em 1080p60 |
| 3 | Anotar quanto tempo levou para a pessoa passar à reduzida e se a regra de 5 s abaixo de 60% se confirmou. | confirma os números de R9 |
| 4 | Tirar o limite. | ela volta sozinha à cheia (cenário 2) |
| 5 | Anotar quanto tempo levou, e se foi pela estimativa de banda ou pela tentativa a cada 30 s. | resolve "a estimativa sobe de volta recebendo a reduzida?" |
| 6 | Limitar a internet de **quem transmite**. | a qualidade baixa para todos, e o indicador de qualidade mostra a mudança (cenário 3) |
| 7 | Com a reduzida ativa por internet fraca, olhar a qualidade que ela entrega. | confirma 720p30 a 1,5 Mbps, ou leva ao plano B (reduzida sempre em 360p). Anotar o custo da segunda compressão (processador do app e FPS do jogo) |

### História 6: Computador sem placa boa continua transmitindo (SC-010)

Precisa de: um computador sem placa de vídeo capaz, ou um jeito de desligar o uso dela para o teste. Para desligar no teste: a variável de ambiente `ESPIA_SEM_PLACA`, que só vale em build de depuração do app, ou a opção equivalente da sonda (R18 de [research.md](./research.md)). Nunca aparece para a pessoa (FR-014).

| Passo | O que fazer | Prova |
|---|---|---|
| 1 | Clicar em Transmitir nesse computador, ou com a placa desligada para o teste. | começa com os mesmos passos, sem aviso técnico e sem escolha nova (cenário 1) |
| 2 | Alguém assiste. | consegue assistir |
| 3 | Olhar o indicador de qualidade. | mostra a qualidade efetiva: a escolhida, ou menor se o processador não der conta (cenário 2) |
| 4 | Abrir o Diagnóstico de mídia. | a linha "Compressão: processador" (FR-018) |
| 5 | Repetir 10 vezes. | SC-010: a transmissão começa em todas, em pelo menos 720p30 |
| 6 | Com uma transmissão pela placa em andamento, forçar a falha do codificador. | continua pelo outro caminho, com no máximo uma pausa curta, sem transmitir de novo (cenário 3, G5 item g) |
| 7 | Comparar o painel antes e depois da falha. | a linha passa de "placa de vídeo" para "processador" |

## 5. O que só a pessoa faz

| O quê | Por que | Onde se anota |
|---|---|---|
| Medições com jogo (G6, história 2) | só uma pessoa joga e lê o contador de FPS | `relatorio.md`, no formato de [medicao.md](./contracts/medicao.md) |
| Conferir quantos estão assistindo em cada medição | a ressalva do dono sobre o G0 | idem |
| Leitura do app de hoje no Diagnóstico de mídia | precisa do app 0.1.10 transmitindo e de alguém assistindo | `relatorio.md`, uma linha |
| Provas com grupo (histórias 3, 4 e 5): 3 a 7 pessoas | só o grupo tem as pessoas e os aparelhos | `relatorio.md` |
| Safari no iPhone | depende de alguém do grupo com o aparelho; na 019 não foi feito e o risco foi aceito; aqui volta a ser pedido | `relatorio.md` |
| Chrome no celular | idem | `relatorio.md` |
| Outras marcas de placa (AMD, Intel) | só o grupo sabe quais placas tem; uma prova por marca | `relatorio.md` |
| Escolher como limitar a internet de uma pessoa (história 5) | método ainda não definido | `relatorio.md` |
