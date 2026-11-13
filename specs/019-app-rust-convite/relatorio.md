# Relatório da feature 019

## US1 — Convite com o endereço oficial (T043, concluída)

Publicação: `npm run deploy` em 2026-11-11, versão `fb38d279-ab88-4a59-a5f9-45533a1c16e6`.

| Cenário | Resultado | Evidência |
|---|---|---|
| 1. Sala aberta pelo endereço oficial: convite copiado e mostrado começam com `espia.diuliano.com` | Passou | Página aberta em `https://espia.diuliano.com/s/cfr-dv2`. Convite mostrado: `espia.diuliano.com/s/cfr-dv2`. Copiado: `https://espia.diuliano.com/s/cfr-dv2`. Observação: no endereço oficial, o código anterior também produziria este resultado; não prova a mudança sozinho. |
| 2. Sala aberta pelo endereço antigo | Passou | Após o deploy (versão `fb38d279-ab88-4a59-a5f9-45533a1c16e6`), aberta em `espia.espia.workers.dev/s/cfr-dv2`, o convite mostrado é `espia.diuliano.com/s/cfr-dv2` e o copiado é `https://espia.diuliano.com/s/cfr-dv2`. Antes do deploy, o mesmo teste mostrava `espia.espia.workers.dev/s/cfr-dv2` (linha de base). |
| 3. App já instalado (0.1.10) | Passou | Janela do app desktop mostra `espia.diuliano.com/s/cfr-dv2`; copiado: `https://espia.diuliano.com/s/cfr-dv2`. Versão instalada confirmada pela pessoa. |
| 4. Link antigo de sala abre a mesma sala | Passou | Link antigo abriu a mesma sala, como antes. |
| 5. Criar sala fixa copia sozinho com o endereço oficial | Passou | Sala fixa `hg2-pc4` criada; a cópia automática ao colar trouxe `https://espia.diuliano.com/s/hg2-pc4`. |

Prova SC-001 e SC-002: os cinco cenários passaram após o deploy. História 1 concluída.

## G0 — Linha de base do app de hoje (T008, não medida)

Registrado em 2026-11-11. A medição não foi feita: a pessoa não conseguiu reproduzir a sessão com o jogo e os espectadores.

| Item | Valor | Origem |
|---|---|---|
| 1. FPS do jogo sem transmitir | cerca de 200 | Relato de memória, na máquina de um amigo. Não medido. |
| 2. FPS do jogo transmitindo | 140 a 160 | O mesmo relato, com 3 pessoas assistindo, às vezes 4. Resolução e taxa de quadros da transmissão não registradas. |
| 3. Processador do app transmitindo | não medido | |
| 4. Memória numa sala e na bandeja | não medido | |
| 5. Codificador que o motor usa | não medido | |

O relato dá uma perda de 20 a 30% no FPS do jogo. É um pouco pior que a ordem de grandeza anotada na spec (200 para 160 a 180). O jogo era o CS2. Máquina e data do relato não foram registradas, então ele serve só de ordem de grandeza, como os números antigos da spec.

**Situação do gate**: G0 não passou pelo critério da T008 (cinco itens, na mesma sessão). O que isso muda:

- **SC-004 e SC-005** não dependem deste registro: a comparação é feita lado a lado, na mesma máquina e na mesma sessão, na medição da história 3 (quickstart, "Medição lado a lado"). Essa medição passa a ser a única base das duas metas.
- **G1, item (f)**: a saída "processador em até metade do G0" fica sem base. O item só passa se a placa de vídeo codificar, ou depois de o item 3 acima ser medido.
- **SC-006** tem metas absolutas (150 MB e 40 MB) e não usa o G0.

## Gates G1, G2 e G3 — achados das sondas, antes de a pessoa rodar

Registrado em 2026-11-11. As três sondas compilam (`cargo build --release`). Nenhum gate foi rodado ainda pela pessoa (T010, T012, T014): o que está aqui vem da leitura do código das crates e de execuções sem site, sem Chrome e sem perfil real.

### G1 — ligação (`app/spikes/webrtc-probe/`, `libwebrtc` 0.4.0, `webrtc-sys` 0.3.53)

| Item | Situação | Observação |
|---|---|---|
| (a) compila no Windows | Passou | Executável de 21,4 MB. Exige a flag `+crt-static` no link (sem ela, erro LNK2038); o app de verdade precisa da mesma flag. |
| (b) a (e), (g), (h), (j) | Não rodado | Dependem do site local, do Chrome e do iPhone. Comandos no `LEIA-ME.md` da sonda. |
| (e) API de parâmetros por ligação | Existe na crate | `RtpSender::set_parameters` com `max_bitrate`, `max_framerate`, `scale_resolution_down_by`, `active`; estatísticas com `quality_limitation_reason` e `encoder_implementation`. Não exercitado contra o Chrome. |
| (f) placa de vídeo codifica | **Não, na versão oficial** | Conferido no `build.rs` da `webrtc-sys` 0.3.53: os codificadores da placa (NVIDIA, VAAPI) só entram no Linux; no ramo do Windows o código está comentado. Resta o H.264 em software (openh264), um codificador por ligação. |
| (i) tamanho do executável | 21,4 MB | Memória do processo: não medida. |

**Risco do item (f)**: sem a placa de vídeo, o item só passa se o processador do processo ficar em até metade do G0, e o G0 não foi medido. Codificar 1080p60 em software para 3 pessoas tende a pesar mais que o app de hoje, o que iria contra o motivo da migração. Saídas, por ordem de custo:

1. Medir mesmo assim (T010), porque é a única forma de saber.
2. Caminho que a crate oferece e a pesquisa não previa: `VideoEncoderBackend::PreEncoded`. O app codifica o H.264 uma vez com o codificador da placa do Windows (Media Foundation) e entrega os quadros já codificados. Não testado. Custo: escrever o codificador, e a qualidade por espectador (miniatura 360p, teto por ligação) deixa de sair de graça, porque seria uma codificação para todos, ou duas (cheia e miniatura).
3. Plano B de R2: versão modificada da crate com codificador do Windows.
4. Plano C de R2: `str0m` com codificador próprio (exige replanejar as fases de mídia).

**Licenças do binário do WebRTC** (resolve parte do "não confirmado" da T007): o pacote baixado traz um `LICENSE.md` de cerca de 197 KB que lista webrtc, abseil-cpp, boringssl, dav1d, ffmpeg, libaom, libjpeg_turbo, libsrtp, libvpx, libyuv, openh264, opus, protobuf, rnnoise, zlib e outras. A seção do ffmpeg traz os textos da GPL v2 e v3; o `args.gn` do pacote tem `ffmpeg_branding = "Chrome"` e `rtc_use_h264 = true`. Não confirmado: se o ffmpeg está de fato dentro do `webrtc.lib` e sob qual licença. Conferir antes do arquivo de licenças do instalador (T093).

### G2 — interface (`app/spikes/ui-probe/`, Slint 1.17.0)

Execução de 15 s, sozinha (sem o motor do G1 no mesmo processo):

| Medida | Valor |
|---|---|
| Memória privada do processo | 95 a 104 MB |
| Quadros entregues ao palco (1080p) | 57 a 60 por segundo |
| Quadros descartados | 0 a 14 por janela de 5 s |
| Executável | 13,4 MB |

**Risco**: a meta do gate é 150 MB com o motor do G1 carregado. A interface sozinha, com um vídeo 1080p por imagem RGBA e duas miniaturas, já ocupa cerca de 100 MB, então sobram cerca de 50 MB para o motor de vídeo. A comparação lado a lado com o site não foi feita. Limitações já vistas: sombra sem espalhamento, cantos elípticos só por caminho, só fontes TTF/OTF (as do site foram convertidas de woff, com as somas das tabelas conferidas).

### G3 — dados do app de antes (`app/spikes/webview-data/`, `rusty-leveldb` 3.0.3)

A sonda lê uma pasta de teste criada por ela mesma (dois formatos de texto, com acento) e recusa abrir a pasta viva. Nada foi conferido contra dados reais do motor de página: o formato das chaves continua "não confirmado" até a T014. Comando da T014 no `LEIA-ME.md` da sonda.

## G3 — dados do app de antes, cópia do perfil real (T014)

Data: 2026-11-11. Comando: `cargo run --release -- C:\teste-g3\leveldb` em `app/spikes/webview-data/`, com o app de antes fechado e uma cópia da pasta `leveldb`.

Resultado da sonda (linha `RESULTADO:`, sem valores): salas fixas lidas; chaves de dono = 1 (1 com 64 hex); apelido lido.

**Passou** o critério de leitura (salas fixas, chave de dono e apelido de um perfil real). **Conferido**: a única sala própria deste aparelho é a que tem chave de dono; a leitura bate. O cenário 5 da T086 pede 2 salas próprias: antes dele, criar mais uma sala fixa neste aparelho (ou ajustar o cenário para 1).

Decisão R10: a leitura do formato Chromium funciona em dados reais. Plano B de exportação manual não é necessário pelo G3, mas fica pendente o número de chaves.

### G3 — segunda leitura (2026-11-11)

Depois de a pessoa criar mais salas fixas neste aparelho, nova cópia e nova leitura: salas fixas lidas; chaves de dono = 4 (4 com 64 hex); apelido lido. Confere com o que a pessoa relatou: 3 salas próprias novas mais a chave que já existia. O perfil tem o que o cenário 5 da T086 pede (pelo menos 2 salas próprias).

**G3 passou. R10 confirmado**: o `localStorage` do motor de página é lido de uma cópia da pasta com `rusty-leveldb`; o plano B (exportação manual) não é necessário.

## Fase 2 — estado em 2026-11-12

Feitas 25 das 30 tarefas (T009, T011, T013 a T036). Gate: `cargo check --workspace --all-targets` sem avisos e `cargo test --workspace` com 397 testes verdes; `npm run check` verde.

- **Telas com dados falsos** (T025 a T033): 69 cenas do site portadas (home 13, join 5, room 47, notfound 4), mais as do app; o teste de paridade confere os ids contra `src/mocks/*.ts`. Conferidas por captura de tela (`--screenshot`), não lado a lado com o site por uma pessoa.
- **App andando sem mídia** (T034 a T036): tela inicial real, entrada e sala sem vídeo nem som. Provado com servidor local falso (27 testes) e, em parte, contra o site local (`npm run dev:worker`): criar sala fixa, entrar por link `espia://`, ver chegada e saída, Suas salas com os números do servidor. Queda de rede e retomada só foram provadas com o servidor falso. Ninguém clicou na janela de verdade.

Abertas:

| Tarefa | Depende de |
|---|---|
| T008 (G0), T010 (G1), T012 (G2) | pessoa |
| T037 ligação entre pessoas | decisão de R2 depois do G1 (a placa de vídeo não codifica na `libwebrtc` oficial no Windows) |
| T038 conferência do esqueleto | T037 e pessoa |

Diferenças conhecidas das telas em relação ao site: sombra dos avisos mais dura; títulos que quebram em duas linhas com espaçamento um pouco maior; sem a animação de abrir dos diálogos; Esc não fecha o menu "⋯"; a dica do apelido diz "Fica só neste computador" (o site diz "navegador"); o seletor mantém os textos do site com a palavra "compartilhar"; a área de transferência usa a crate `arboard`, que não está na lista de acessos da constituição (confirmar ou trocar).

Disco: `C:` com 26 GB livres (95% usado). As pastas `target` de `app/` e das sondas somam vários GB; `cargo clean` em `app/spikes/*` libera espaço depois que os gates forem rodados.

## G1 — resultado parcial (log da sonda, 2026-11-12, sala `pwd-ebn`)

Logs: três execuções da sonda contra `npm run dev:worker` (`--view thumb`, `--view hidden` e `--stats-secs 5`), com o Chrome do outro lado. O G1 **não passou**: só parte dos itens tem evidência, e o item (f) não pode ser avaliado sem o G0.

| Item | Situação | Evidência |
|---|---|---|
| (a) compila no Windows | Passou | Executável de 21,4 MB, `+crt-static`. |
| (b) Chrome recebe vídeo 1080p60 H.264 e som | **Não provado** | O log é só da sonda. Falta o `chrome://webrtc-internals` do Chrome (`frameWidth`, `framesPerSecond`, `decoderImplementation`). A sonda codificou H.264 com `encoderImplementation='OpenH264'` a 1920x1080, 59 a 60 quadros por segundo, mas o que o Chrome decodificou não consta. |
| (c) sonda recebe vídeo e som do Chrome | Parcial | Som: 48.000 amostras por segundo, de forma contínua. Vídeo: 1 quadro por segundo antes de a sonda começar a enviar; 20 a 28 quadros por segundo depois. A diferença precisa de explicação (o Chrome parece reduzir o envio quando a sonda não manda vídeo de volta). Não provado com o Chrome do outro lado. |
| (d) troca `ViewMessage` pelo canal `ctl` | Passou | `ctl → view v2` e `ctl ← view v2 (decodeView ok)` nos modos `thumb`, `hidden` e `full`; `view-sync` respondido. |
| (e) muda parâmetros por ligação e lê o motivo de limite | Parcial | `set_parameters` aplicado e confirmado pelo motor (`active=true bitrate=6000000 fps=60 scale=1.00`). `qualityLimitationReason` aparece (`none`). Não testado: mudar bitrate, fps ou escala com o comando ao vivo e ver o efeito. |
| (f) placa de vídeo, ou processador em até metade do G0 | **Não pode passar ainda** | Codificação por software (OpenH264). Processador do processo com codificação ativa: 62,8% de um núcleo, que é 2,6% da máquina de 24 núcleos. Sem o G0 medido, não há comparação. |
| (g) som e imagem em até 80 ms por 10 minutos | Parcial | Minuto 1 de 10: −12 ms. A execução parou em 118 s. Faltam 9 minutos. |
| (h) Safari no iPhone recebe | Não rodado | |
| (i) memória e tamanho do executável | Passou (com a ressalva) | Memória privada: 9 a 12 MB ociosa; 26 MB recebendo; 80 MB com codificação de 1080p60 ativa. Executável 21,4 MB. |
| (j) aviso de firewall do Windows | Não anotado | Precisa de observação manual na primeira execução. |

Outros dados do log:
- Quadros de vídeo recebidos da sonda: 1 congelamento registrado em 115 s, com o codificador já ativo.
- Avisos do OpenH264 sobre taxa de quadros (`Actual input framerate 0.000000`): aparecem na primeira fase da codificação e podem ser normais, mas não foram investigados.
- A linha `quiquit` no log foi um erro de digitação de quem rodou a sonda; não afeta o resultado.

Próximo passo para fechar o G1: (1) rodar o Chrome com `chrome://webrtc-internals` e registrar (b) e a parte do vídeo que chega ao Chrome; (2) rodar a sonda por 10 minutos com `--stats-secs 5` para o (g); (3) medir o G0 (T008) para avaliar (f); (4) testar Safari no iPhone (h); (5) anotar o aviso de firewall (j).

### G1 — registro do lado do Chrome (captura de `getDisplayMedia`, 2026-11-12)

O Chrome capturou a janela de teste com `getDisplayMedia`, pedindo 1920x1080 e 30 quadros por segundo (ideal). Dois registros:

- **08:56:43** (stream `0bd55ec8…`): faixa de som `f8f132cd-d495-49c4-ae83-ab38fcd51672` e faixa de vídeo `750956fe-8397-418f-9197-fa1b18a76780`. **São as mesmas faixas que a sonda registrou** em `chegou faixa de VÍDEO / SOM do outro lado` na execução `--stats-secs 5`. Isso confirma que a sonda recebeu exatamente o que o Chrome capturou.
- **08:51:12** (outra captura, faixas `857a57d5…` e `4dd6f0c3…`): não corresponde a nenhuma execução da sonda no log.

Consequências:
- (c) fica mais firme: a origem do vídeo e do som recebidos pela sonda é a captura de uma janela real no Chrome.
- A captura pede 30 quadros por segundo. Por isso o vídeo que o Chrome manda **não passa de 30**, e uma janela parada manda muito menos. Isso pode explicar os 1 a 28 quadros por segundo do (c). Não é a mesma coisa que o (b), que mede o vídeo **da sonda** chegando ao Chrome: continua sem evidência.
- (b) ainda pede o `chrome://webrtc-internals` na aba de recepção do Chrome, mostrando `frameWidth=1920`, `frameHeight=1080`, `framesPerSecond` e `codecId` H.264.

### G1 — crash ao sair de 'full' (corrigido em 2026-11-12)

Sintoma: a sonda caía quando a pessoa desfocava a tela do Chrome (o site manda `view hidden`). O log da execução com `--stats-secs 5` terminava exatamente na linha `ctl ← view v2 modo=hidden`, sem a linha `set_local →` seguinte.

Causa: `set_local` chamava `sender.set_track(None)` no canal de vídeo ativo. Essa remoção de faixa derrubava o processo. Só o site faz a parada certa, com `active=false` (`sendParams`).

Correção (`app/spikes/webrtc-probe/src/main.rs`, `set_local`): a faixa não é mais removida. A parada do envio fica com `apply_encodings`, que já manda `active=false` para `hidden` e `off`. Mesma mudança no som.

Verificação: a sonda compilou em release (`webrtc-probe.exe`, 22.456.320 bytes). A pessoa confirmou que não cai mais ao desfocar a tela do Chrome. Os itens (g), (h), (i) e (j) continuam pendentes, e o (g) de 10 minutos precisa ser repetido com esta versão, já que a execução anterior foi interrompida pelo crash.

### G1 — (b) do lado do Chrome, `chrome://webrtc-internals` (2026-11-12, sala `pwd-ebn`)

Evidência do Chrome recebendo a sonda:
- `inbound-rtp (kind=video)`: `frameHeight=1080`, `contentType=screenshare`, `[codec]=H264 (profile-level-id=42001f)`, `decoderImplementation=ExternalDecoder (D3D11VideoDecoder)`, `powerEfficientDecoder=true`.
- `inbound-rtp (kind=audio)`: Opus, recebido.
- `data-channel (label=ctl, state=open)`: canal de controle aberto.
- `transport (iceState=connected, dtlsState=connected)`; `remote-outbound-rtp` para áudio e vídeo.

Ainda não confirmado: `framesPerSecond` e `frameWidth` do vídeo recebido. A tabela não mostra; é preciso abrir os gráficos da linha `inbound-rtp (kind=video)`. O item (b) só passa com a taxa perto de 60 e a largura de 1920.

### G1 — (b) taxa ao vivo (2026-11-12)

Leitura ao vivo, com a sonda em `full` e o Chrome assistindo: **57,95 quadros por segundo** no `inbound-rtp` de vídeo do Chrome. Perto dos 60 do G1. O valor some ao desfocar a janela porque o site manda `hidden` e a sonda para de enviar; isso é o comportamento esperado.

Ainda não registrado: o `freezeCount` no mesmo instante (na leitura anterior, 7 congelamentos em 13,2 s acumulados). É preciso saber se os congelamentos continuam aumentando com a sonda em `full`.

**Critério de taxa do (b), definido pela pessoa**: abaixo de 53 quadros por segundo é preocupante; de 58 para 60 não há diferença perceptível. Com 57,95 ao vivo, o critério de taxa passa. Resta o `freezeCount` com a sonda em `full`.

### G1 — congelamentos do (b), leituras da pessoa (2026-11-12)

- Antes: `freezeCount=7`, `totalFreezesDuration=13,211 s`.
- Depois de uma retomada (alt+tab e volta para `full`): `freezeCount=8`, `totalFreezesDuration=26,499 s`. Um único congelamento de cerca de 13,3 s. Momento exato não anotado (retomada ou regime estável).
- Teste seguinte, em `full`: `totalFreezesDuration` continua em 26,499 s, ou seja, sem congelamento novo nesse teste. `freezeCount` atual não anotado.

Pendente: anotar o `freezeCount` e o horário do congelamento de 13 s. Se o número de congelamentos não sobe em uma execução longa em `full`, o (b) fecha.

- Leitura seguinte, em `full`: `freezeCount=12` (subiu de 8, ou seja, 4 congelamentos novos). `totalFreezesDuration` lido como 26,499 s, sem mudança. Inconsistente com a contagem: a duração precisa ser lida de novo no mesmo instante que a contagem.
- Consequência: o (b) **não fecha** enquanto os congelamentos subirem em regime estável em `full`.

**Correção**: a leitura de `freezeCount=8` após a retomada não foi do mesmo período que a de 12. A pessoa confirmou que o 12 já estava antes do teste em `full`. Não há evidência de congelamentos novos nesse teste. Estado atual: `freezeCount=12`, `totalFreezesDuration=26,499 s`, origem dos congelamentos (retomada ou regime estável) não determinada.

Próxima prova do (b): com a sonda em `full` e o Chrome assistindo, anotar `freezeCount` e `totalFreezesDuration` no início e no fim de 2 a 3 minutos sem interação. Se os dois não mudarem, o (b) fecha.

### G1 — (b), leitura de nova ligação (09:47, 2026-11-12)

Estatísticas de outra ligação (`ssrc=2433073671`; os contadores zeram a cada conexão, por isso não se somam às leituras de 12 congelamentos):
- `freezeCount=1`, `totalFreezesDuration=0,183 s`: um congelamento curto.
- `pauseCount=1`, `totalPausesDuration=22,585 s`: período em `hidden`.
- `framesDecoded=24.037`, 1920x1080, `D3D11VideoDecoder`.
- Taxa média estimada: cerca de 56 quadros por segundo (a partir de `totalInterFrameDelay` = 425,529 s para 24.036 intervalos). Estimativa, não leitura direta.

Pendente para fechar o (b): nova leitura desta mesma ligação depois de 2 a 3 minutos em `full`. Se `freezeCount` continuar em 1 e `totalFreezesDuration` quase parado, o (b) fecha.

### G1 — item (b): PASSOU (2026-11-12, decisão da pessoa)

Provas:
- Chrome recebe 1920x1080 em H.264 da sonda, decodificado pela placa (`D3D11VideoDecoder`).
- Taxa ao vivo: 57,95 quadros por segundo em `full` (limite combinado: 53).
- Congelamentos na ligação de 09:47: `freezeCount=1`, 0,183 s no total.

Ressalva: a prova de 2 a 3 minutos com leituras de início e fim não foi feita formalmente; a estabilidade vem de uma leitura única.

Itens do G1 ainda abertos: (c) som e vídeo de volta do Chrome, com a leitura de 10 minutos; (f) processador frente ao G0; (g) sincronia de 10 minutos; (h) iPhone; (j) aviso de firewall.

### G1 — execução de ~10 minutos sem jogo (2026-11-12, arquivo da pessoa)

- Sonda em 1920x1080 a 60 quadros por segundo, H.264 por software (OpenH264), enquanto o Chrome estava em `full` (de cerca de 35 s a 440 s). Depois, `hidden`.
- Processador do processo em `full`: 53 a 62% de um núcleo, ou 2,2 a 2,6% da máquina. Em `hidden`: 1 a 6%. Sem jogo, sem comparação com o G0.
- Memória privada no fim: 71 MB.
- Envio real de 170 a 197 kbps, abaixo do alvo de 4.300 a 6.000 kbps: imagem de teste quase estática.
- Sincronia (g): não medida. Em todos os minutos, `sem estimatedPlayoutTimestamp dos dois lados (som=false, imagem=false)`; a sonda não recebeu estatísticas do Chrome nessa execução.
- Pendente: (f) com o jogo (G0); (g) em nova execução com a leitura do Chrome; (h) iPhone; (j) aviso de firewall.

### G1 — (j) e (h)

- (j) aviso de firewall do Windows: **não apareceu** na primeira execução. Passou.
- (h) Safari no iPhone recebe: **não reproduzido** pela pessoa. Pendente. O critério do G1 exige o teste; sem ele, o G1 não passa, e a decisão sobre a ausência do teste precisa ser da pessoa.

### G1 — (g) sincronia, três execuções de 10 minutos (2026-11-12)

Critério: som e imagem não se afastam mais de 80 ms.

- Leituras válidas (som e imagem atualizando): até **±27 ms**, média de cerca de 19 ms. Dentro do critério.
- Os valores acima de 40 mil ms não são desincronia: o `estimatedPlayoutTimestamp` da imagem ficou congelado num valor fixo (por exemplo `4003477098195`) enquanto o som continuava. O vídeo que chegava ao probe parou de atualizar. Não é leitura válida.
- Peer `1a18b6` (terceira execução): leituras válidas de 3 a 10 minutos, máximo **19 ms**. É a medição contínua mais limpa.
- Peer `b7989d` (Chrome): o vídeo ficou parado por vários minutos nas execuções 2 e 3 (de 120 s a 780 s e de 300 s a 600 s), com o som normal. Causa não confirmada: pode ser o Chrome não atualizar a janela capturada quando está em segundo plano. Precisa de confirmação.

Situação do (g): **passou para o peer `1a18b6`** (10 minutos, máximo 19 ms). A medição com o peer `b7989d` não foi contínua por causa da parada do vídeo, que está em aberto.

**Causa confirmada pela pessoa**: o Chrome estava em segundo plano durante as execuções. A janela capturada deixou de atualizar, e o vídeo que chegava à sonda parou (ver acima). Efeito do teste, não defeito da sonda. Para medições futuras: deixar o Chrome em primeiro plano durante a execução inteira.

Situação do G1 em 2026-11-12:
- (a) compila: passou.
- (b) Chrome recebe 1080p H.264 e taxa acima de 53: passou (decisão da pessoa, com a ressalva de prova de 2 a 3 minutos sem formalizar).
- (c) sonda recebe vídeo e som do Chrome: passou (identidade das faixas confirmada; vídeo estável com o Chrome em primeiro plano).
- (d) troca de `ViewMessage`: passou.
- (e) mudança de parâmetros: aplicado e confirmado pelo motor; testado só `set_parameters`, sem variar bitrate/fps ao vivo.
- (f) processador frente ao G0: **pendente** (precisa de G0 com jogo).
- (g) sincronia: passou para o peer `1a18b6` (10 minutos, máximo 19 ms).
- (h) iPhone: **pendente** (decisão da pessoa).
- (i) memória e tamanho: anotados (71 a 80 MB com codificação; 21,4 MB executável).
- (j) aviso de firewall: não apareceu. Passou.

### G1 — decisões da pessoa (2026-11-12)

- **(h) iPhone/Safari: risco aceito pela pessoa.** Não reproduzido. Fica registrado como risco conhecido: se o Safari não receber o vídeo na qualidade máxima, o problema aparece com os espectadores de iPhone, e a decisão deve ser revisitada.
- **Uso esperado: a maioria transmite na qualidade máxima (1080p60).** Por isso o (f) é o item que mais importa: o caso medido (1080p60 por software, cerca de 2,4% da máquina sem jogo) é o caso típico, e a comparação com o G0 com jogo é obrigatória antes de fechar o G1.

## G0 — medições com o app de hoje (0.1.10), 2026-11-12, parcial

Situação: **transmitindo** (a pessoa confirmou). Jogo: **League of Legends** (não CS2; o relato de 200 para 140–160 era de outro jogo, do amigo). Máquina: a mesma da pessoa.

| Item | Valor | Origem |
|---|---|---|
| (1) FPS do jogo sem transmitir | cerca de 240 | Estimativa da pessoa ("eu acredito"). Não medido com contador. |
| (2) FPS do jogo transmitindo | cerca de 180 a 200 | Estimativa da pessoa. Não medido com contador. Número de pessoas e qualidade não anotados. |
| (3) Processador do app transmitindo | não medido | Pendente: Gerenciador de Tarefas, somando `Espia` e os processos do motor. |
| (4) Memória numa sala transmitindo | 146 a 267 MB (pico 267), típica de 185 a 245 MB | `scripts/memoria.ps1 -Watch`, 12:50 a 13:12, 6 processos do motor. Acima da meta de 150 MB. |
| (4) Memória na bandeja | não medido | Pendente. |
| (5) Codificador | não medido | Pendente: Diagnóstico de mídia do site. |

Observação: o relato anterior (200 para 140–160 com CS2 e 3 a 4 pessoas) e esta estimativa (240 para 180–200) não são da mesma sessão nem do mesmo número de pessoas. Só a medição com contador e processador vale para o G0.

- Atualização (2026-11-12): **FPS do LoL sem transmitir: 300**, lido pela pessoa no contador do jogo (leitura direta, não estimativa). Falta o valor transmitindo 1080p60 para 3 pessoas, com o mesmo contador, e o processador do app nessa transmissão.
- Atualização (2026-11-12): **LoL transmitindo: 200 a 240 FPS**, relatado pela pessoa (confirmar se foi lido no contador do jogo, e com quantas pessoas e em que qualidade). Perda em relação aos 300 sem transmitir: de 20% a 33%. Pendente: processador do app nessa transmissão.
- Confirmado pela pessoa (2026-11-12): os 200 a 240 FPS transmitindo vieram do contador do jogo. Número de pessoas e qualidade da transmissão ainda não confirmados (o G0 pede 1080p60 para 3 pessoas).
- **Correção (2026-11-12)**: a faixa de 200 a 240 FPS transmitindo foi com **1 pessoa** assistindo, em 1080p60. Não é a condição do G0 (3 pessoas). Para o G0 vale só a medição com 3 pessoas em 1080p60. Estado: FPS sem transmitir = 300; FPS transmitindo com 1 pessoa = 200 a 240; com 3 pessoas = pendente; processador do app = pendente.

## G0 — medição com 3 pessoas (2026-11-12, relato da pessoa)

Condição: LoL, app 0.1.10 transmitindo **1080p60 com som para 3 pessoas**, com pessoas reais.

- FPS sem transmitir: **300** (contador do jogo).
- FPS transmitindo para 3 pessoas: **200 a 220**, conforme a cena (contador do jogo).
- Perda: de 27% a 33% do FPS.
- Processador do app: pendente, a pessoa vai enviar print do Gerenciador de Tarefas.
- Processador do app com 3 pessoas em 1080p60 (print do Gerenciador de Tarefas, 2026-11-12, leitura instantânea): **Espia 0,6%** (25,5 MB) + **WebView2 Manager (6) 1,6%** (109,6 MB). Total: **2,2% do processador da máquina** e **135 MB** pelo Gerenciador. Rede: 0,1 MB/s. Limitação: uma leitura, não média da transmissão. Ver nota sobre a diferença para o `memoria.ps1` (outra métrica).

## G1 — fechamento (2026-11-12, decisão da pessoa)

| Item | Resultado |
|---|---|
| (a) compila | Passou |
| (b) Chrome recebe 1080p60 H.264 e som | Passou (1920x1080, H.264, decodificado pela placa; 57,95 quadros por segundo ao vivo) |
| (c) sonda recebe vídeo e som do Chrome | Passou |
| (d) `ViewMessage` pelo `ctl` | Passou |
| (e) parâmetros por ligação | Parcial: `set_parameters` confirmado pelo motor; mudança ao vivo de bitrate, fps e escala não testada |
| (f) processador até metade do G0 | **Não passou**: sonda 2,4% da máquina (sem jogo) contra limite de 1,1% (metade de 2,2% do app de hoje com 3 pessoas e LoL). Não é comparação de mesma condição. |
| (g) sincronia 10 minutos | Passou (peer 1a18b6, máximo 19 ms) |
| (h) iPhone/Safari | Não testado: **risco aceito pela pessoa** |
| (i) memória e tamanho | Anotado: 71 a 80 MB com codificação; 21,4 MB |
| (j) aviso de firewall | Não apareceu. Passou |

**Decisão da pessoa**: fechar o G1 com o (f) reprovado e aceito como risco. O risco é que o app nativo não corte o processador pela metade (SC-005) com codificação por software. O plano B de R2 (codificador do Windows, ou `str0m`) continua disponível se a medição de processador em condição igual mostrar que a meta não é atingível.

Pendências que não bloqueiam o fechamento: medição com a mesma condição (sonda com jogo, ou app de hoje sem jogo) para comparar em igualdade; mudança de bitrate, fps e escala ao vivo no (e).

**Correção da métrica do (f) (2026-11-12)**: a porcentagem da máquina depende do número de núcleos (o computador da pessoa tem 24 threads lógicas; o de muitos usuários tem 6 a 12). A comparação correta é em núcleos equivalentes (porcentagem × threads ÷ 100):
- App de hoje, 3 pessoas, 1080p60, LoL aberto: 2,2% × 24 = **0,53 núcleo**.
- Sonda, 1080p60, sem jogo: 2,4% × 24 = **0,57 núcleo**.
- Critério (metade do G0): **0,27 núcleo**. A sonda não passa, e a conclusão do (f) é a mesma em qualquer processador.

## Encaminhamento do item (f) do G1 (2026-11-12)

O risco aceito no item (f) virou a feature [020-codificar-pela-placa](../020-codificar-pela-placa/spec.md): comprimir o vídeo uma vez pela placa de vídeo e mandar o mesmo vídeo a todos, pela conexão direta. A investigação está em [investigacao.md](../020-codificar-pela-placa/investigacao.md). As histórias de transmitir da 019 (fase 4) esperam a prova da 020.

**Ressalva do G0 (2026-11-12, dita pela pessoa)**: na medição com "3 pessoas", algumas podiam não estar assistindo no momento da leitura (janela escondida ou sem Assistir). Quem não assiste não recebe vídeo, então o 0,53 núcleo e os 200 a 220 FPS podem ser de 1 ou 2 espectadores, não de 3. A base do G0 fica como "até 3 espectadores, número real não confirmado". A comparação que vale continua sendo a lado a lado, na mesma sessão, conferindo no app quantos estão assistindo.
