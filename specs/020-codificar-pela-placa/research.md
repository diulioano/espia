# Research: Transmitir sem pesar no jogo, codificando uma vez pela placa de vídeo

**Feature**: [spec.md](./spec.md) · **Plan**: [plan.md](./plan.md) · **Date**: 2026-11-12

O levantamento que originou a feature está em [investigacao.md](./investigacao.md). Tudo o que está marcado "não confirmado" continua assim até a prova ou a tarefa que o resolve (ver o fim deste arquivo).

## Contexto

- A 019 escolheu a crate `libwebrtc` 0.4.0 para a ligação de vídeo (R2 da 019). O gate G1 mostrou que ela comprime no processador, uma vez por espectador, e que no Windows ela não usa a placa de vídeo.
- O dono aceitou em 2026-11-12: duas qualidades (cheia e reduzida) no lugar da qualidade sob medida por espectador; 3 segundos para a primeira imagem; meio segundo de congelamento no máximo quando alguém entra; 53 quadros por segundo como piso de 1080p60.
- Ressalva do dono sobre a base de comparação: na medição do app de hoje com "3 pessoas", parte delas podia não estar assistindo. A comparação que vale é lado a lado, conferindo quantas assistem.

## Fatos do código de hoje

- **Vídeo já codificado na `libwebrtc` 0.4.0** (`src/video_source.rs`, `src/video_frame.rs`, `src/rtp_sender.rs`):
  - `NativeVideoSource::new_encoded(resolução)` cria uma fonte que recebe quadros já comprimidos.
  - `capture_encoded_frame(&EncodedVideoFrame { codec, payload, timestamp_us, frame_type, resolution, .. })` entrega um quadro.
  - `take_keyframe_request()` devolve e limpa o pedido de quadro inteiro.
  - `take_rate_control_request()` devolve o último alvo de banda e de quadros por segundo (só o mais recente).
  - `RtpSender::set_video_encoder_backend(VideoEncoderBackend::PreEncoded)` liga o repasse num canal de envio.
- **Repasse por ligação** (`webrtc-sys` 0.3.53, `src/passthrough_video_encoder.cpp`): cada canal de envio tem o seu repassador.
  - Ele começa "quebrado": segura os quadros parciais e pede um quadro inteiro até chegar um.
  - Ele percebe quadro pulado pelo número de sequência e pela identidade da fonte, e nesse caso também segura e pede quadro inteiro.
  - Não faz simulcast (`supports_simulcast = false`) e não muda o tamanho da imagem.
- **Captura de hoje** (`src-tauri/src/capture/video.rs`): `Windows.Graphics.Capture` entrega uma textura; o processador de vídeo da placa reduz e converte para NV12 numa textura; depois há uma cópia para a memória, que é o que esta feature dispensa.
- **Regras de qualidade** (`app/crates/espia-core/src/quality.rs`, porte de `src/lib/quality.ts`): `effective_quality` (escolhida, teto pelo número de espectadores, degraus do processador), `send_params` (miniatura 360p, 15 quadros por segundo, 500 kbps; `hidden` e `off` não enviam).
- **Sonda do G1** (`app/spikes/webrtc-probe/`): entra numa sala, negocia, troca `ViewMessage`, mede processador e memória. Tirar a faixa de um canal ativo com `set_track(None)` derrubava o processo; parar é `active=false`.
- **Bindings do Windows**: a crate `windows` 0.62 (já usada em `src-tauri/` e prevista na 019) traz `MFTEnumEx`, `MFT_ENUM_FLAG_HARDWARE`, `IMFDXGIDeviceManager`, `MFCreateDXGISurfaceBuffer`, `MF_SA_D3D11_AWARE`, `METransformNeedInput` e as propriedades `CODECAPI_AVLowLatencyMode`, `CODECAPI_AVEncCommonRateControlMode`, `CODECAPI_AVEncCommonMeanBitRate`, `CODECAPI_AVEncMPVGOPSize` e `CODECAPI_AVEncVideoForceKeyFrame` (conferido no código da crate).

## R1 — Por onde o vídeo comprimido entra

**Decisão**: usar o caminho de vídeo já codificado da `libwebrtc` 0.4.0 (`new_encoded`, `capture_encoded_frame`, `VideoEncoderBackend::PreEncoded`). A biblioteca continua cuidando de empacotar, enviar, reenviar pacote perdido e estimar a banda. O app passa a ser o dono da compressão.

**Motivo**: mantém a R2 da 019 (mesma biblioteca, sem versão modificada) e tira a compressão do processador.

**Alternativas consideradas**:
- Versão modificada da biblioteca com codificador do Windows (plano B da 019): exige manter um fork e montar o WebRTC.
- Outra biblioteca (`str0m`) com codificador próprio (plano C da 019): recepção, sincronia e reenvio por nossa conta.
- Uma compressão pela placa por espectador: esbarra no limite de compressões simultâneas das placas e multiplica o custo.

**Risco / não confirmado**: o caminho nunca foi exercitado aqui. Formato esperado do quadro (H.264 com códigos de início, parâmetros de sequência junto de cada quadro inteiro) é o padrão do empacotador do WebRTC, mas não foi lido no repassador.

**Gate / plano B**: G4. Se falhar, volta-se à decisão R2 da 019.

## R2 — Quem comprime: o codificador de H.264 da placa, pelo Windows

**Decisão**: usar o codificador de H.264 por hardware exposto pelo Windows (Media Foundation), achado com `MFTEnumEx` e `MFT_ENUM_FLAG_HARDWARE`, pela crate `windows` 0.62.

**Motivo**: uma interface só cobre NVIDIA, AMD e Intel, sem kit de cada fabricante e sem dependência nova. O codificador faz parte do driver da placa.

**Alternativas consideradas**:
- Kits dos fabricantes (NVENC, AMF, oneVPL): mais controle, mas três implementações e três licenças.
- FFmpeg: pesado no instalador e com licença a tratar.

**Risco / não confirmado**: o comportamento muda por marca e por driver (atraso, quadro inteiro a pedido, mudança de banda ao vivo). Só a máquina do dono será provada primeiro.

**Gate / plano B**: G4. Se o codificador do Windows não servir numa marca, essa marca cai no caminho pelo processador (R11) até haver solução específica.

## R3 — O quadro vai da captura ao codificador sem passar pelo processador

**Decisão**: entregar ao codificador a textura NV12 que a captura já produz na placa, usando o mesmo dispositivo gráfico (`IMFDXGIDeviceManager`, `MFCreateDXGISurfaceBuffer`). Na prova G4 e G5 o quadro é gerado no processador e enviado para uma textura, porque ali se prova a ligação e não a captura.

**Motivo**: a cópia da placa para a memória é parte do peso de hoje.

**Alternativas consideradas**: entregar o quadro pela memória (mais simples, mas mantém a cópia).

**Risco / não confirmado**: se a captura e o codificador podem dividir o mesmo dispositivo sem travar um ao outro; o custo no FPS de jogo que usa a placa no limite.

**Gate / plano B**: G6. Plano B: entrada pela memória, medindo o custo.

## R4 — Como o codificador é configurado

**Decisão**:
- baixa latência ligada, sem quadros B;
- perfil Constrained Baseline, o mesmo que o site negocia hoje;
- taxa constante com alvo que pode mudar ao vivo;
- quadro inteiro só a pedido (intervalo automático desligado ou muito longo);
- quadros por segundo e tamanho iguais aos da qualidade efetiva.

**Motivo**: é o que uma transmissão ao vivo precisa: atraso baixo, formato que Chrome e Safari já recebem, banda controlada.

**Alternativas consideradas**: perfil High (melhor imagem na mesma banda, mas não é o que a negociação anuncia; fica para depois da prova).

**Risco / não confirmado**: se cada marca respeita a baixa latência e a mudança de alvo ao vivo; se o quadro inteiro traz junto os parâmetros de sequência.

**Gate / plano B**: G4 (itens de formato e de atraso). Plano B: acrescentar os parâmetros de sequência antes de cada quadro inteiro no próprio app.

## R5 — Uma fonte por qualidade, repartida por todas as ligações

**Decisão**: para cada qualidade em uso existe uma fonte de vídeo codificado e uma faixa. O canal de vídeo de cada ligação aponta para a faixa da qualidade daquele espectador. O mesmo quadro comprimido é entregue uma vez à fonte e a biblioteca o repassa a cada ligação.

**Motivo**: é "comprimir uma vez". O repassador de cada ligação já foi desenhado para entrar no meio do fluxo (segura até o próximo quadro inteiro e o pede).

**Alternativas consideradas**: uma fonte por ligação recebendo cópias do mesmo quadro (funciona, mas multiplica cópias e pedidos de quadro inteiro).

**Risco / não confirmado**: várias ligações na mesma fonte não foi visto funcionando.

**Gate / plano B**: G5. Plano B: uma fonte por ligação, alimentada com o mesmo quadro.

## R6 — Duas qualidades: cheia e reduzida

**Decisão**:
- **Cheia**: a qualidade efetiva da transmissão (`effective_quality` da 010).
- **Reduzida**, com dois ajustes conforme quem precisa dela:
  - só miniaturas: 360p, 15 quadros por segundo, 500 kbps (os números de `send_params` de hoje);
  - pelo menos um espectador em tela cheia com internet fraca: 720p, 30 quadros por segundo, 1,5 Mbps.
- Cada qualidade só é comprimida enquanto alguém a recebe. Sem ninguém em tela cheia, a cheia para. Sem ninguém assistindo, nada é comprimido.

**Motivo**: cobre miniatura e internet fraca com no máximo duas compressões pela placa, longe do limite de compressões simultâneas.

**Alternativas consideradas**:
- Só uma qualidade: a miniatura receberia a cheia e gastaria a internet de todos.
- Três qualidades: mais uma compressão para um caso raro.

**Risco / não confirmado**: o custo da segunda compressão na placa; se 720p30 a 1,5 Mbps atende a internet fraca típica do grupo.

**Gate / plano B**: medição na história 5. Plano B: a reduzida fica sempre em 360p.

## R7 — Trocar a qualidade de um espectador e parar de enviar

**Decisão**:
- Trocar de qualidade é trocar a faixa do canal de vídeo daquela ligação para a faixa da outra qualidade. O repassador percebe a fonte nova, segura e pede quadro inteiro.
- Parar de enviar (`hidden`, `off`, quem não clicou em Assistir) é `active=false` no canal. Nunca tirar a faixa de um canal ativo.

**Motivo**: a regra do Assistir (FR-008) continua igual à do site, e o defeito achado no G1 fica evitado.

**Alternativas consideradas**: dois canais de vídeo por ligação (um por qualidade), ligando um e desligando o outro. Muda a negociação com o site, que espera um canal de vídeo por pessoa.

**Risco / não confirmado**: trocar a faixa de um canal de repasse em uso.

**Gate / plano B**: G5. Plano B: dois canais por ligação, só entre apps, mantendo um canal com o site.

## R8 — Quadro inteiro a pedido, sem atrapalhar os outros

**Decisão**: a cada quadro o app lê `take_keyframe_request()` de cada fonte. Havendo pedido, o próximo quadro daquela qualidade sai inteiro. Pedidos próximos são juntados: no máximo um quadro inteiro a cada 300 ms por qualidade. Não há quadro inteiro periódico.

**Motivo**: quem entra no meio vê a imagem logo (SC-006), e uma fila de pedidos não derruba a banda dos outros (SC-007).

**Alternativas consideradas**: quadro inteiro a cada N segundos (gasta banda sem necessidade e não atende quem entra entre dois).

**Risco / não confirmado**: tamanho do quadro inteiro em 1080p e o efeito dele em quem já assiste.

**Gate / plano B**: G5 mede o congelamento dos outros. Plano B: aumentar o intervalo mínimo.

## R9 — Internet de cada espectador

**Decisão**: a banda de cada ligação é lida das estatísticas da própria ligação (banda de saída disponível), uma vez por segundo. As regras ficam numa função pura em `espia-core`:
- espectador em tela cheia cuja banda fica abaixo de 60% da taxa da cheia por 5 segundos, ou abaixo de 90% por 15 segundos, passa a receber a reduzida;
- volta à cheia quando a banda fica em 90% ou mais da taxa da cheia por 15 segundos; se a estimativa não subir sozinha, uma tentativa de volta a cada 30 segundos;
- se **todos** os que recebem a cheia estão abaixo de 90%, o problema é a internet de quem transmite: ninguém é rebaixado, e a taxa da cheia baixa para 90% da menor banda. Fora desse caso a taxa da cheia é a nominal, para o mais fraco não puxar os outros.

O alvo único de `take_rate_control_request()` não decide nada, porque guarda só o pedido da última ligação que falou; ele é registrado no diagnóstico.

**Motivo**: com um vídeo para todos, o mais fraco não pode puxar os outros (FR-010), e a regra precisa ser testável sem rede.

**Alternativas consideradas**: usar o alvo único da fonte (não distingue espectadores); baixar a cheia para o mais fraco (pune todos).

**Risco / não confirmado**: se a estimativa de banda sobe de volta enquanto a ligação recebe só a reduzida. Os números (60%, 90%, 5 s, 15 s, 30 s) são ponto de partida.

**Gate / plano B**: história 5. Plano B: volta só por tentativa periódica.

## R10 — As regras da 010 continuam

**Decisão**: a qualidade efetiva continua saindo de `effective_quality` (escolhida, número de espectadores em tela cheia, degraus). O ajuste "o computador não dá conta" passa a olhar o codificador da placa: quadros que ele não conseguiu comprimir a tempo contam como "limitado".

**Motivo**: o envio pela internet continua crescendo com a plateia, porque o mesmo vídeo sobe uma vez para cada pessoa (FR-011).

**Alternativas consideradas**: tirar o teto por número de espectadores (o custo de processador some, mas o de internet não).

**Risco / não confirmado**: qual sinal o codificador dá quando a placa está no limite.

**Gate / plano B**: G6. Plano B: usar o atraso entre entregar o quadro e receber o comprimido.

## R11 — Computador sem placa capaz, ou placa que falha

**Decisão**: o caminho pelo processador é o que o G1 já provou: quadros crus e o codificador de software da biblioteca, um por espectador, na qualidade escolhida, com o ajuste automático por processador que já existe (regra da 010). Não há teto próprio deste caminho (decisão do dono em 2026-11-12). O app escolhe sozinho:
- não achou codificador por hardware, ou ele não inicia: caminho pelo processador desde o começo;
- três erros seguidos, ou nenhum quadro comprimido em 2 segundos: troca para o caminho pelo processador no meio da transmissão.

**Motivo**: FR-012 a FR-014. Quem não tem placa capaz continua transmitindo, sem pergunta e sem termo técnico.

**Alternativas consideradas**: codificador de software do Windows no mesmo caminho de repasse (uma compressão para todos também sem placa). Fica como melhoria, porque não foi provado e o caso é minoria.

**Risco / não confirmado**: trocar um canal de repasse para o codificador de software no meio da transmissão, sem renegociar.

**Gate / plano B**: G5 (item de troca). Plano B: a transmissão é parada e reaberta sozinha, com pausa de até 2 segundos, sem a pessoa clicar em nada.

## R12 — O que a pessoa vê

**Decisão**: nenhuma opção nova. O Diagnóstico de mídia do app ganha uma linha "Compressão: placa de vídeo" ou "Compressão: processador", com o nome do codificador. O indicador de qualidade que já existe continua mostrando a qualidade efetiva.

**Motivo**: FR-014 e FR-018; constituição I.

**Alternativas consideradas**: chave "usar placa de vídeo" nas opções (contra a simplicidade).

## R13 — Onde o código vive

**Decisão**:
- `app/crates/espia-core/src/tiers.rs`: regras puras. Qualidade de cada espectador, ajuste da reduzida, junção de pedidos de quadro inteiro, escolha e troca do caminho de compressão.
- `app/crates/espia-media/src/encode/`: `mod.rs` (interface do codificador), `hardware.rs` (codificador da placa), `software.rs` (caminho pelo processador).
- `app/crates/espia-media/src/publish.rs`: fontes e faixas por qualidade, e a ligação de cada canal à faixa certa.
- `espia-ui` e o binário `espia`: a linha do diagnóstico entra na lista de linhas que o painel já recebe (`DiagLine`), com cena de dados falsos. Não muda `view_model.rs` (R18).
- `app/spikes/encode-probe/`: a prova, a partir da sonda do G1.
- `scripts/processador.ps1`: medição do processador em núcleos (R15).

**Motivo**: a regra IV da constituição continua garantida pelo grafo de crates, e as regras ficam testáveis sem placa e sem rede.

## R14 — Provas antes de construir

**Decisão**: três gates, na ordem. Gate vermelho para o que vem depois.

| Gate | Prova |
|---|---|
| **G4** | Vídeo de teste 1080p60 comprimido pela placa chega a uma pessoa no Chrome, com som, e o processador da sonda é registrado |
| **G5** | O mesmo vídeo para três pessoas: processador perto do de uma; uma entra no meio; uma para e volta a assistir; uma troca de qualidade; o caminho troca para o processador |
| **G6** | Captura real ligada ao codificador, com jogo, lado a lado com o app de hoje |

Antes deles, uma leitura que só informa: qual compressão o app de hoje usa (Diagnóstico de mídia do site).

**Motivo**: FR-001 e FR-002. A ideia está confirmada só no código-fonte.

## R15 — Como medir

**Decisão**:
- Processador em **núcleos** (porcentagem da máquina × núcleos lógicos ÷ 100), média de 60 segundos, somando a árvore de processos do app. Um script novo, `scripts/processador.ps1`, faz a conta para o app de hoje (árvore do `Espia.exe`) e para o app nativo ou a sonda (um processo).
- FPS do jogo pelo contador do próprio jogo.
- Em toda medição anota-se quantas pessoas estão **assistindo** naquele momento, lido na sala.

**Motivo**: porcentagem depende da máquina, uma leitura de um instante engana, e a base do G0 ficou em dúvida por não se saber quantos assistiam.

## R16 — Licença

**Decisão**: o codificador da placa é parte do Windows e do driver; o app não embarca codificador novo. O codificador de software continua sendo o que já vem dentro do binário do WebRTC.

**Risco / não confirmado**: se usar o codificador do sistema muda alguma obrigação de patente do H.264 para um app gratuito. Segue junto da pendência de licenças da 019 (arquivo de licenças do instalador).

## R17 — O que muda na 019

**Decisão**:
- A R2 da 019 continua (mesma biblioteca), com o adendo de que a compressão é do app.
- A tarefa T037 da 019 (ligação entre pessoas sem mídia) não muda.
- As tarefas de transmitir da 019 (fase 4) passam a usar `publish.rs` e `encode/` daqui, e só começam depois do G5.
- As metas SC-004 e SC-005 da 019 são as mesmas SC-003 e SC-002 daqui.

## R18 — Regras fechadas na revisão do desenho

Pontos que as decisões acima deixavam em aberto e que os contratos apontaram. Fechados em 2026-11-12:

| Ponto | Regra |
|---|---|
| Miniatura quando a reduzida está no ajuste de internet fraca | Recebe o mesmo vídeo de 720p30. Só existe uma reduzida. Gasta mais internet da miniatura enquanto durar; aceito para não ter uma terceira compressão. |
| Reduzida de internet fraca quando a cheia já é 720p30 ou menor | Mesmo tamanho e mesmos quadros por segundo da cheia, com taxa de 1,5 Mbps ou metade da taxa da cheia, o que for menor. |
| Espectador em tela cheia com internet fraca conta no teto por número de espectadores? | Conta. O teto é pelo modo de ver, não pelo que a pessoa recebe. |
| Pedido de quadro inteiro que chega dentro dos 300 ms | É adiado até a janela passar. Nunca descartado. |
| "Todos abaixo" (R9) | Se todos os que recebem a cheia estão abaixo de 90% da taxa nominal, ninguém desce. Um espectador sozinho nunca desce. (Revisto em 2026-11-12, na análise.) |
| Base dos 60% e dos 90% | Sempre a taxa nominal da qualidade efetiva, não a taxa já baixada. |
| Taxa da cheia | A nominal. Só no caso "todos abaixo": a cada segundo, o menor valor entre a nominal e 90% da menor banda entre quem recebe a cheia. Sobe de volta pela mesma conta. (Revisto em 2026-11-12: antes a conta valia sempre, e um espectador mediano baixava a taxa de todos.) |
| Banda entre 60% e 90% | Abaixo de 90% por 15 s seguidos, o espectador desce para a reduzida, salvo no caso "todos abaixo". (Novo em 2026-11-12.) |
| Teto no caminho pelo processador | Não há. Vale a qualidade escolhida, com o teto por número de espectadores e o ajuste automático por processador da 010. (Decisão do dono em 2026-11-12.) |
| Aviso quando a placa não ajuda (T056 e FR-013 da 019) | Sem aviso. A pessoa vê só o indicador de qualidade e a linha "Compressão" do diagnóstico. (Decisão do dono em 2026-11-12.) |
| Tentativa de volta a cada 30 s que não dá certo | Vale de novo a regra de descer (60% por 5 s). |
| Depois de trocar para o processador, a placa volta? | Não na mesma transmissão. A transmissão seguinte tenta a placa de novo. |
| Os 2 s sem quadro comprimido | Só contam enquanto alguma qualidade está ativa. |
| Como forçar o caminho pelo processador e a falha, para testar | Na sonda: opções de linha de comando. No app: variável de ambiente `ESPIA_SEM_PLACA`, só em build de depuração. Nunca uma opção na tela. |
| Intervalo de leitura de `scripts/processador.ps1` | 1 segundo. |
| Nome do painel | O site chama de "Diagnóstico da transmissão". É o mesmo painel que a spec chama de Diagnóstico de mídia. |
| Onde entra a linha "Compressão" | Na lista de linhas que o painel do app já recebe (`DiagLine`). Não muda `view_model.rs`. |
| Alvo de taxa que o codificador recusa mudar ao vivo | O codificador é reaberto com a taxa nova e o próximo quadro sai inteiro. Se isso se repetir mais de uma vez por minuto, conta como erro para a troca de caminho. |
| Janela capturada que muda de tamanho | As qualidades ativas são reabertas no tamanho novo e o próximo quadro sai inteiro. |

## O que ficou sem confirmação

| Item | Onde | Resolve |
|---|---|---|
| Qual compressão o app de hoje usa | investigacao.md | leitura no Diagnóstico de mídia, antes do G4 |
| O vídeo comprimido pela placa chega certo ao Chrome | R1, R4 | G4 |
| Formato do quadro que o repassador espera | R1, R4 | G4 |
| Baixa latência, quadro inteiro a pedido e alvo ao vivo no codificador da máquina do dono | R2, R4 | G4 |
| Várias ligações na mesma fonte | R5 | G5 |
| Trocar a faixa de um canal de repasse em uso | R7 | G5 |
| Congelamento dos outros quando alguém entra | R8 | G5 |
| Trocar para o caminho pelo processador sem renegociar | R11 | G5 |
| Captura e codificador no mesmo dispositivo gráfico; custo no FPS | R3, R10 | G6 |
| A estimativa de banda sobe de volta recebendo a reduzida | R9 | história 5 |
| Custo da segunda compressão | R6 | história 5 |
| Outras marcas de placa (AMD, Intel) | R2 | conferência com o grupo |
| Safari no iPhone recebe o vídeo comprimido pela placa | spec, FR-015 | conferência com o grupo |
| Patente do H.264 com o codificador do sistema | R16 | junto da pendência de licenças da 019 |
