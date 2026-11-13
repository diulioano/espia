# Sonda 020 (T009 a T012): compressão pela placa de vídeo no app nativo do Espia

Crate isolada (fora do workspace de `app/`), copiada da sonda do G1 (`app/spikes/webrtc-probe/`) e estendida. Entra numa sala do site local como se fosse mais um cliente, com `libwebrtc` `=0.4.0`, mas o vídeo agora pode ser **comprimido uma vez pela placa de vídeo** (codificador de H.264 da Media Foundation, crate `windows` 0.62) e entregue já comprimido a todas as ligações (`NativeVideoSource::new_encoded`, `VideoEncoderBackend::PreEncoded`). Serve para os gates **G4** (uma pessoa) e **G5** (três pessoas) da feature 020 (tarefas T014 e T015).

**A sonda nunca captura tela, câmera ou microfone.** O vídeo é gerado (barras coloridas, contador de 16 bits, barra branca andando, clarão de 1 s) e o som é um tom de 440 Hz com um bip de 1 kHz por segundo. Só a pessoa, no roteiro manual, captura de verdade (G6, fora desta sonda).

O comportamento da sonda do G1 continua inteiro com `--backend software` (ou `auto`, `hardware`, `nvenc`).

## Compilar

```powershell
cd app\spikes\encode-probe
cargo build --release -j 6
.\target\release\encode-probe.exe --help
```

- O `.cargo\config.toml` liga `+crt-static` (o `webrtc.lib` pré-compilado usa a biblioteca C estática; sem isso o `link.exe` falha com LNK2038).
- Na primeira compilação o `webrtc-sys` baixa o pacote pronto (~90 MB) de `github.com/livekit/rust-sdks/releases`. Precisa de internet só nessa vez.
- Use `-j 6` e **nunca dois builds ao mesmo tempo** (o computador já reiniciou com builds pesados em paralelo).
- Não rode `cargo build` com a sonda aberta: o Windows não deixa sobrescrever o `.exe` em uso.
- O executável se chama `encode-probe.exe`, o nome que `scripts\processador.ps1 -Processo encode-probe -Nativo` espera.

Teste rápido, sem sala e sem rede (só a placa): `.\target\release\encode-probe.exe --encode-only 10` (ver "Teste sem sala").

## Preparação comum

1. Em outro terminal, na raiz do repositório: `npm run dev:worker` (site e servidor locais, `http://127.0.0.1:8787`). Para parar: feche esse terminal (ou encerre `workerd` e o `node` do `wrangler`).
2. No Chrome, abrir `http://127.0.0.1:8787`, criar uma sala e anotar o código (ex. `2zr-w9e`). Sem Chrome, uma sala sai de `curl.exe -s -X POST -H "Origin: http://127.0.0.1:8787" http://127.0.0.1:8787/api/rooms`.
3. A sonda manda `Origin` igual ao endereço de `--base` e `mediaProtocol: 2`.
4. Comandos ao vivo: digite uma linha no terminal da sonda e Enter. Os do G1: `bitrate N`, `fps N`, `scale X`, `active on|off`, `reset`, `view full|thumb|hidden|off`, `stats`, `quit`. Os novos (T012): `peers`, `tier <início do id> full|reduced|auto`, `fail`, `fail --reabrir`. Parar: `quit` + Enter, ou Ctrl+C.
5. Variáveis usadas abaixo (PowerShell): `$S = ".\target\release\encode-probe.exe"` e `$SALA = "<código>"`. O `scripts\processador.ps1` roda **da raiz do repositório**.

## O que é novo em relação à sonda do G1

| Opção | O que faz |
|---|---|
| `--backend placa` (padrão) | uma fonte `new_encoded` e uma faixa **para a cheia** (1920x1080, 60 qps, 6 Mbps) usadas por todas as ligações. O canal de vídeo de cada ligação recebe `set_video_encoder_backend(PreEncoded)` e a faixa. Os outros valores de `--backend` são o caminho do G1 |
| `--sem-placa` | começa já no caminho pelo processador (a biblioteca comprime, como no G1). Sem codificador por hardware a sonda faz isso sozinha e diz por quê |
| `--reduzida-fraca` | a reduzida usa 720p, 30 qps, 1,5 Mbps (em vez de 360p, 15 qps, 500 kbps) |
| `--ruido` | põe ruído novo a cada quadro no vídeo da placa. Sem isso as barras paradas comprimem a ~0,3 Mbps e a banda estimada fica longe da de um jogo. Use nas medições de processador e de banda |
| `--sem-aud` | tira o delimitador de acesso (NAL 9) da frente de cada quadro. Só se a biblioteca ou o Chrome reclamarem (ver "O que foi achado") |
| `--encode-only <s>` | sem sala: gera, comprime pela placa e imprime o resumo (T010) |
| `peers` | lista cada ligação: modo em que me vê, qual vídeo recebe (cheia, reduzida, nada), faixa do canal, se está forçada |
| `tier <id> full\|reduced\|auto` | força um espectador (início do id, 3 letras bastam) para a cheia ou a reduzida, sem esperar regra de banda; `auto` volta à regra do ViewMessage |
| `fail` | simula a falha do codificador da placa: para os codificadores e troca **todas as ligações** para o caminho pelo processador (troca de faixa e de backend de cada canal, **sem renegociar**) |
| `fail --reabrir` | plano B: para e reabre a transmissão sozinha (`unlive` e `live`), já pelo processador |

**Regra do Assistir (pelo `ViewMessage`)**: `full` recebe a cheia; `thumb` recebe a reduzida (troca de faixa no canal daquela ligação); `hidden` e `off` são `active=false` (a faixa fica no canal; nunca `set_track(None)`); sem `ViewMessage`, nada (nem se cria o canal).

**Reduzida**: segunda fonte, segunda faixa e segundo codificador, criados só quando alguém precisa (~0,4 s para abrir) e **parados quando ninguém a recebe** (`reduzida parada: ninguém a recebe`). A cheia fica no ar a transmissão inteira (decisão da sonda: reabrir a cheia a cada primeiro espectador comeria o orçamento de 3 s do item G5 (c)).

**Quadro inteiro**: a cada quadro a sonda lê `take_keyframe_request()`; com pedido, o próximo quadro sai inteiro, no máximo um a cada 300 ms por qualidade (pedido dentro da janela é adiado, nunca descartado); não há quadro inteiro periódico. `take_rate_control_request()` só é impresso.

**Em toda troca de caminho** a sonda imprime a pausa em milissegundos (`[troca de caminho] ... o vídeo voltou N ms depois do comando`).

## O que a sonda imprime

Todas as linhas começam com o tempo desde o início. Além das do G1 (`[envio vídeo]`, `[envio som]`, `[recebe vídeo]`, `[recebe som]`, `[rota]`, `[recepção na sonda]`, `[proc]`, `[sync]`):

- `[codificador] <nome> '<codificador>' WxH a N qps | entregues X recebidos Y (descartados na fila Z) | atraso entregar→receber médio A ms, maior B ms | quadros inteiros K (tamanho médio T KB; pedidos da biblioteca P, emitidos E) | erros N (recusados pela fonte R) | NAL 9 tirados | taxa pedida pela biblioteca ...`: a cada `--stats-secs`, uma linha por qualidade no ar (cheia e, se existir, reduzida). Os contadores são **desde o início** daquele codificador.
- `[envio vídeo] origem=placa|biblioteca codificador='...' | ... | encoderImplementation='...'`: `origem=placa` com `encoderImplementation='LiveKit pre-encoded passthrough'` é o vídeo vindo do codificador da placa; `origem=biblioteca` com `OpenH264` é o caminho pelo processador.
- `[recepção na sonda] ... maior intervalo entre quadros N ms` (sonda que assiste): o maior intervalo entre dois quadros desde a última linha. É a medida independente da pausa de uma troca.
- `[codec negociado] ... pt=103 H264/90000 fmtp='...profile-level-id=...'`: o codec que a resposta aceitou (o primeiro da seção de vídeo).
- `[codificador] cheia: a biblioteca pede alvo de N bps a M qps (só impresso)`: `take_rate_control_request`.
- `[troca de caminho] ...`: comandos `fail` e `fail --reabrir`.

## Itens do G4 (T014): uma pessoa

Preparação: sala criada no Chrome, `npm run dev:worker` rodando. Comece sempre pela sonda transmitindo e depois clique em Assistir no Chrome.

### (a) Acha e inicia o codificador de H.264 da placa

```powershell
& $S --room $SALA --stats-secs 5
```

Observar no log da sonda: `codificadores de H.264 por hardware achados (MFTEnumEx, NV12 -> H.264): N`, uma linha `achado` por codificador, `INICIOU: <nome>` e `caminho pela placa: cheia 1920x1080 a 60 qps ... codificador '<nome>'`. **Passa** se achou e iniciou. Na máquina do dono (GTX 1660 SUPER + AMD integrada) achou 3: dois `AMDh264Encoder` (não iniciam, ver "O que foi achado") e `NVIDIA H.264 Encoder MFT` (inicia). Se não achar nenhum a sonda diz que não achou, segue pelo processador e o item **falha**. Prova sem sala: `& $S --encode-only 10`.

### (b) O Chrome recebe 1920×1080 em H.264 com som

```powershell
& $S --room $SALA --ruido
```

No Chrome (na sala), clicar em Assistir na transmissão "Sonda 020". Abrir `chrome://webrtc-internals`, aba da ligação, `inbound-rtp` de vídeo: `frameWidth=1920`, `frameHeight=1080`, `codecId` apontando para `video/H264`, `decoderImplementation`. Ouvir o bip a cada segundo. Anotar sim ou não. No log da sonda, `[codec negociado] ... H264/90000 fmtp='...profile-level-id=...'` mostra o que a resposta aceitou. **Observação**: o quadro sai do codificador com SPS `profile_idc=66`, `constraint_flags=0xC0`, `level_idc=42` (Constrained Baseline, nível 4.2), enquanto a SDP anuncia `profile-level-id=42001f`; uma sonda que assistia (FFmpeg) decodificou sem problema. Se o Chrome não mostrar imagem, a primeira suspeita é o delimitador de acesso (NAL 9) na frente de cada quadro: rode de novo com `--sem-aud`.

### (c) Pelo menos 53 quadros por segundo no Chrome, por pelo menos 1 minuto

Mesmo comando do item (b) e mesmo cenário (um Chrome assistindo, um minuto). Em `webrtc-internals`, `framesPerSecond` do `inbound-rtp` de vídeo. **Passa** se ficar em 53 ou mais; anotar o menor valor visto. Conferência cruzada no log da sonda: `[envio vídeo] ... 1920x1080 a N qps`.

### (d) A compressão não é a de software da biblioteca

```powershell
& $S --room $SALA --ruido --stats-secs 5
```

Observar a linha `[envio vídeo]`: `origem=placa`, `codificador='NVIDIA H.264 Encoder MFT'` e `encoderImplementation='LiveKit pre-encoded passthrough'` (**não** é OpenH264). `powerEfficientEncoder=false` aparece mesmo assim: a biblioteca não informa isso para o repassador, então esse campo não prova nada. Contraprova: `& $S --room $SALA --backend software` mostra `origem=biblioteca` e `encoderImplementation='OpenH264'`.

### (e) Processador da sonda em núcleos, média de 60 s

Com o Chrome assistindo (comando do item (b)), em **outro** terminal, na raiz do repositório:

```powershell
scripts\processador.ps1 -Processo encode-probe -Nativo
```

(60 s por padrão; `-Duracao N` muda.) Registrar núcleos médios e pico, mais as colunas do formato de medição. **Atenção**: o número da sonda inclui o processador gasto para gerar o quadro NV12, subir para a textura e o ruído de `--ruido`, que o app de verdade não gasta (lá a captura já entrega a textura). Medido em 2026-11-12 neste computador, com uma sonda assistindo e `--ruido`: **0,41 núcleo médio**, pico 0,54 (6 amostras). Referência do G1: 0,57 núcleo em software para 1 pessoa.

### (f) Atraso entre entregar o quadro e receber o comprimido

```powershell
& $S --room $SALA --ruido --stats-secs 5
```

Anotar, da linha `[codificador] cheia ...`, `atraso entregar→receber médio X ms, maior Y ms`. Medido em 2026-11-12 (1080p60, uma sonda assistindo): médio 3,8 a 5,3 ms, maior 11 a 16 ms (com uma passada de 39 ms no `--encode-only`, não investigada). Sem sala: `& $S --encode-only 10` imprime o mesmo (médio 4,6 ms, maior 8,7 ms). O atraso inclui a espera na fila da sonda, o envio do quadro para a textura e a compressão. **Só registrado, não há limite escrito.**

## Itens do G5 (T015): três pessoas

Três Chromes (ou três aparelhos) assistindo à mesma sonda. Duas sondas extras servem de "pessoa" quando faltar gente: `& $S --room $SALA --no-live --view full --stats-secs 5` (assiste e imprime `[recebe vídeo]` e `[recepção na sonda]`).

### (a) Três pessoas recebem o mesmo vídeo

```powershell
& $S --room $SALA --ruido
```

Os três Chromes clicam em Assistir. Em `webrtc-internals` de cada um: 1920x1080, H.264, quadros por segundo; os três veem imagem e ouvem o som. No terminal da sonda, `peers` deve listar as três ligações com `vídeo que recebe: cheia (placa)`. Conferência: o log mostra `canal de vídeo criado ... fonte=cheia` para cada uma e **um só** `[codificador] cheia` (uma compressão para todos).

### (b) Processador com três em até 25% acima do de uma

Com 1 pessoa assistindo e depois com 3 (confira com `peers`), cada vez 60 s, na raiz do repositório:

```powershell
scripts\processador.ps1 -Processo encode-probe -Nativo
```

Duas medições de cada. **Passa** se núcleos com 3 ≤ núcleos com 1 × 1,25. Falha: plano B, uma fonte por ligação (R5). Use `--ruido` na sonda nas duas medições (mesma carga).

### (c) Quem entra no meio vê imagem em até 3 s

Com duas pessoas assistindo, a terceira clica em Assistir e se cronometra do clique até a primeira imagem; repetir algumas vezes e anotar o maior. Com uma sonda como terceira pessoa: `& $S --room $SALA --no-live --view hidden --stats-secs 1`, digitar `view full` nela e olhar quando aparece a primeira linha `[recebe vídeo] 1920x1080`. No terminal da sonda que transmite, `[codificador] cheia ... pedidos da biblioteca P, emitidos E` mostra o quadro inteiro pedido na entrada (pedidos dentro de 300 ms são juntados).

### (d) Os outros não congelam mais de 0,5 s nessa entrada

No momento da entrada do item (c), olhar as duas que já assistiam: em `webrtc-internals`, `freezeCount` e `totalFreezesDuration` de cada uma, antes e depois da entrada; nenhum congelamento acima de 0,5 s. Com sondas assistindo: `[recebe vídeo] ... congelamentos N` e `[recepção na sonda] ... maior intervalo entre quadros N ms` (a linha que cobre a entrada). Anotar também o **tamanho do quadro inteiro em 1080p**: `tamanho médio T KB` em `[codificador] cheia` (com `--ruido`, ~119 KB a 6 Mbps; sem ele, ~10 KB). Plano B se falhar: aumentar o intervalo mínimo entre quadros inteiros (a constante é `KEYFRAME_WINDOW` em `src/media.rs`).

### (e) Parar de assistir e voltar funciona sem derrubar a sonda

No Chrome: clicar para parar de assistir, esperar, clicar em Assistir de novo; variante: esconder a janela. Pela sonda, simulando: na sonda que assiste, `view hidden`, esperar, `view full`. No terminal da sonda que transmite: `peers` mostra `vídeo que recebe: nada (vídeo desligado: active=false)` e depois `cheia (placa)`. A sonda continua rodando; ao parar, o vídeo cessa (`active=false`, a faixa fica no canal); ao voltar a imagem volta em poucos segundos; os outros não são afetados.

### (f) Trocar um espectador para a reduzida e de volta funciona

No terminal da sonda que transmite, com as três ligações:

```text
peers
tier <início do id do espectador> reduced
peers
tier <início do id do espectador> full
```

(`tier <id> auto` devolve a regra do ViewMessage.) O espectador vê a imagem cair para 640x360 a 15 qps e voltar para 1920x1080; os outros não mudam (confira no `webrtc-internals` deles e em `peers`). O log mostra `faixa trocada cheia -> reduzida em N ms (sem renegociar)`, `reduzida criada em ~390 ms` na primeira vez e `reduzida parada: ninguém a recebe` quando o último sai dela. O mesmo efeito vem de o espectador pôr a janela em miniatura (`ViewMessage` `thumb`). Plano B se falhar: dois canais por ligação entre apps (R7).

### (g) Forçar a troca para o caminho pelo processador mantém a transmissão

Com as três pessoas assistindo, no terminal da sonda que transmite:

```text
fail
```

Sem ninguém clicar em nada, a transmissão continua para as três. A sonda imprime, por ligação, `[troca de caminho] <id>: o vídeo voltou N ms depois do comando` (resolução de ~50 ms; só vê o quadro codificado de novo, então tende a superestimar); quem assiste confirma com `[recepção na sonda] ... maior intervalo entre quadros N ms` ou olhando o Chrome. Depois, `[envio vídeo] origem=biblioteca ... encoderImplementation='OpenH264'`. Medido em 2026-11-12 com uma sonda assistindo: a pausa vista pela sonda foi de **110 ms**, e o maior intervalo entre quadros em quem assistia ficou em 31 ms (sem pausa perceptível). O plano B, parar e reabrir sozinho, é `fail --reabrir` (comece uma sessão nova: depois de `fail` o caminho já é o do processador). Medido: `live-ack` 83 ms depois do comando, vídeo de volta em **203 ms**, maior intervalo em quem assistia de 127 a 218 ms. A tarefa admite pausa de até 2 s no plano B.

## Teste sem sala

```powershell
& $S --encode-only 10
```

Gera NV12 no ritmo real (`--size`, `--gen-fps`), comprime pela placa por 10 s, pede um quadro inteiro a cada 2 s e muda a taxa uma vez no meio (6 para 3 Mbps; `--encode-bitrate` e `--encode-bitrate-after`). Imprime: quadros entregues e recebidos, atraso médio e maior, quadros inteiros e tamanho médio, se cada pedido de quadro inteiro foi atendido (e quantos ms depois), se a mudança de taxa foi aceita e o tamanho médio antes e depois. Sem codificador por hardware, imprime "não achou codificador de H.264 por hardware" e sai com código 3 (4 se achou e nenhum iniciou). `ENCODE_PROBE_SEM_PLACA=1` finge que não achou nenhum, para testar esse caminho.

## O que foi achado na API da Media Foundation e da `libwebrtc` (T010 e T011)

### Nomes conferidos na crate `windows` 0.62.2

Existem e foram usados: `MFTEnumEx`, `MFT_ENUM_FLAG_HARDWARE`, `MFT_ENUM_FLAG_SORTANDFILTER`, `MFT_FRIENDLY_NAME_Attribute`, `MFT_ENUM_ADAPTER_LUID`, `MFCreateDXGIDeviceManager`, `IMFDXGIDeviceManager::ResetDevice`, `MF_SA_D3D11_AWARE`, `MFCreateDXGISurfaceBuffer`, `MF_TRANSFORM_ASYNC`, `MF_TRANSFORM_ASYNC_UNLOCK`, `METransformNeedInput` (601), `METransformHaveOutput` (602), `METransformDrainComplete`, `MEError`, `IMFMediaEventGenerator::GetEvent`, `MFT_MESSAGE_SET_D3D_MANAGER`, `MFT_MESSAGE_NOTIFY_BEGIN_STREAMING`, `MFT_MESSAGE_NOTIFY_START_OF_STREAM`, `ICodecAPI`, `CODECAPI_AVLowLatencyMode`, `CODECAPI_AVEncCommonRateControlMode`, `CODECAPI_AVEncCommonMeanBitRate`, `CODECAPI_AVEncMPVGOPSize`, `CODECAPI_AVEncVideoForceKeyFrame`, `CODECAPI_AVEncMPVDefaultBPictureCount`, `eAVEncH264VProfile_ConstrainedBase` (256), `eAVEncH264VProfile_Base` (66), `MF_MT_MPEG2_PROFILE`, `MF_MT_MPEG_SEQUENCE_HEADER`, `MFSampleExtension_CleanPoint`, `MF_VERSION`, `MFSTARTUP_FULL`.

Diferenças em relação ao que a tarefa supunha:

- **Features extras da crate `windows`**: `Win32_System_Ole` e `Win32_System_Variant`. Sem elas `ICodecAPI::SetValue` não existe (exige `Win32_System_Com` + `Win32_System_Ole` + `Win32_System_Variant`) e o tipo `VARIANT` nem pode ser nomeado. As sete da lista do T009 bastaram para o resto.
- `MFSetAttributeSize` e `MFSetAttributeRatio` (inline no cabeçalho C) **não existem** na crate: empacotar à mão, `(alto << 32) | baixo` num `SetUINT64` (`MF_MT_FRAME_SIZE`, `MF_MT_FRAME_RATE`, `MF_MT_PIXEL_ASPECT_RATIO`).
- `MF_LOW_LATENCY` e `CODECAPI_AVLowLatencyMode` são o mesmo GUID. `VARIANT::from(bool | u32)` existe; `VT_BOOL` para baixa latência, `VT_UI4` para o resto.
- Objetos COM da `windows` não são `Send`: o codificador vive numa thread só, e a fila de eventos (`IMFMediaEventGenerator`) vai para uma segunda thread embrulhada num tipo `Send` explícito. Uma thread espera `GetEvent` bloqueando e repassa os eventos pelo mesmo canal dos comandos, sem sono de 15 ms do Windows.

### Quem iniciou

`MFTEnumEx` (hardware, NV12 para H.264) devolveu 3: `AMDh264Encoder` (duas vezes) e `NVIDIA H.264 Encoder MFT`. Os dois da AMD **não iniciam**: `MFT_MESSAGE_SET_D3D_MANAGER` volta `0x80004005` quando o dispositivo Direct3D está na NVIDIA (a máquina tem a Radeon integrada e a GTX 1660 SUPER). O atributo `MFT_ENUM_ADAPTER_LUID` **não existe** no `IMFActivate` deste computador (nada para escolher a placa por LUID), então a sonda usa a placa padrão e cai para o próximo candidato. Não confirmado: que o motivo da AMD é mesmo a placa errada (não testado com um dispositivo na integrada). Se a placa padrão de outro computador for a integrada, o candidato da NVIDIA provavelmente falharia do mesmo jeito: a escolha do dispositivo por codificador fica em aberto para o app.

O da NVIDIA: assíncrono, `MF_SA_D3D11_AWARE` = 1, fornece as amostras de saída, aceitou o quadro como buffer `MFCreateDXGISurfaceBuffer` sobre textura NV12 (`BIND_RENDER_TARGET`, `USAGE_DEFAULT`) do mesmo dispositivo. 600 de 600 quadros aceitos e devolvidos, sem erro.

### Ordem importa: propriedades ANTES dos tipos de mídia

Com `ICodecAPI` ajustado **depois** de `SetOutputType`/`SetInputType`, o `CODECAPI_AVEncMPVGOPSize` é "aceito" (sem erro), mas lido de volta vale **60**, e o codificador manda quadro inteiro sozinho a cada 1 s, mesmo com 0xFFFFFFFF ou 0x7FFFFFFF. Ajustado **antes** dos tipos, o valor fica e só sai quadro inteiro no primeiro quadro e nos pedidos. A sonda ajusta antes (padrão). `ENCODE_PROBE_PROPS_DEPOIS=1` reproduz o comportamento antigo; `ENCODE_PROBE_GOP=<n>` força um valor de GOP.

### O que cada propriedade respondeu (NVIDIA H.264 Encoder MFT)

| Propriedade | Resposta |
|---|---|
| `CODECAPI_AVLowLatencyMode` (VT_BOOL true) | aceita |
| `CODECAPI_AVEncCommonRateControlMode` (CBR = 0) | aceita; lido de volta 0 |
| `CODECAPI_AVEncCommonMeanBitRate` (6 000 000) | aceita; lido de volta 6 000 000 |
| `CODECAPI_AVEncMPVDefaultBPictureCount` (0) | **recusada** `0x80070057`. Sem efeito prático: o perfil é Baseline, que não tem quadro B |
| `CODECAPI_AVEncMPVGOPSize` (0xFFFFFFFF) | aceita; só vale se ajustada antes dos tipos |
| `CODECAPI_AVEncVideoForceKeyFrame` (VT_UI4 1, antes do `ProcessInput` pedido) | aceita 4 de 4; o quadro inteiro chegou 3 a 7 ms depois do pedido |
| `CODECAPI_AVEncCommonMeanBitRate` ao vivo (6 para 3 Mbps) | **aceita ao vivo**, sem reabrir: 6,63 Mbps antes, 2,63 Mbps depois (sem o primeiro segundo de cada janela). Também 500 para 300 kbps a 360p15: 0,52 e 0,29 Mbps |
| `MF_MT_MPEG2_PROFILE` = `eAVEncH264VProfile_ConstrainedBase` | aceito |

### Formato do quadro comprimido

Primeiro quadro inteiro: **tem códigos de início (Annex B)**, **traz SPS e PPS junto**, NALs `[9, 7, 8, 5]` (delimitador de acesso, SPS, PPS, IDR), `MFSampleExtension_CleanPoint` = 1, SPS `profile_idc=66`, `constraint_flags=0xC0`, `level_idc=42` a 1080p60 (22 a 360p15). O plano B da R4 (acrescentar SPS/PPS do `MF_MT_MPEG_SEQUENCE_HEADER`) está escrito e **nunca foi exercitado** (aqui o quadro já traz os dois). O NAL 9 não atrapalhou o decodificador FFmpeg da sonda que assistia, com e sem `--sem-aud` (60 qps, zero congelamentos); o Chrome ainda não viu.

### Achados da `libwebrtc` 0.4.0 com vídeo já codificado (T011)

Todos vistos entre duas ou três sondas na mesma sala (a que assiste é uma sonda, decodificador FFmpeg do motor), nunca no Chrome:

- Os nomes da tarefa existem como ela diz: `NativeVideoSource::new_encoded(VideoResolution)`, `capture_encoded_frame(&EncodedVideoFrame)` (devolve `bool`), `take_keyframe_request()`, `take_rate_control_request()` (`EncodedRateControl { target_bitrate_bps, framerate_fps }`), `EncodedVideoFrame { codec, payload, timestamp_us, frame_type, resolution, frame_metadata }`, `EncodedVideoCodec::H264`, `EncodedFrameType::{Key, Delta}`, `VideoEncoderBackend::PreEncoded` (`set_video_encoder_backend`), `PeerConnectionFactory::video_encoder_codecs(PreEncoded)` (devolve VP8, VP9, AV1, H264 e H265). `create_video_track` exige importar `PeerConnectionFactoryExt`. O quadro de vídeo só entra com `PreEncoded` setado antes da primeira negociação; o canal criado com `add_transceiver` e `send_encodings`.
- **O vídeo da placa chega e decodifica**: a sonda que assiste mostrou 1920x1080 a 60 qps, `decoderImplementation='FFmpeg'`, 0 pacotes perdidos, 0 congelamentos, atraso do buffer de 25 a 30 ms. No envio, `encoderImplementation='LiveKit pre-encoded passthrough'` e `powerEfficientEncoder=false` (a biblioteca não informa isso para o repassador). **Isso responde ao "não confirmado" do contrato (o que o diagnóstico mostra como encoder com `PreEncoded`).**
- `capture_encoded_frame` **não recusou nenhum quadro** (contador `recusados pela fonte` sempre 0).
- **Pedidos de quadro inteiro**: a biblioteca pede quadro inteiro quando um espectador entra; pedidos juntos viram um só quadro inteiro pela janela de 300 ms (visto na reduzida: `pedidos da biblioteca 3, emitidos 1`).
- **Alvo de taxa** (`take_rate_control_request`): sobe devagar a partir de ~0,3 Mbps (estimativa de banda). Com as barras paradas a imagem comprime a ~0,3 Mbps, e o alvo e a banda estimada ficam baixos (0,3 a 0,65 Mbps em 20 s). Com `--ruido` o alvo chegou a 5,6 Mbps em 10 s e a sonda enviou 6 Mbps. Por isso o G4 e o G5 devem rodar com `--ruido`.
- **Trocar a faixa de um canal ativo funciona sem renegociar** (R7): `set_track(Some(outra faixa))` entre a cheia e a reduzida levou ~0 ms, e a sonda que assistia passou de 1920x1080 a 60 qps para 640x360 a 15 qps na hora, sem erro e sem congelar; abrir a reduzida pela primeira vez custa ~0,4 s (inicialização do codificador).
- **Trocar da placa para o processador sem renegociar funciona** (R11): `set_video_encoder_backend(Software)` mais `set_track(fonte crua)` no canal ativo; depois disso o `encoderImplementation` passa para `OpenH264` e `origem=biblioteca`. Pausa vista por quem assistia: maior intervalo entre quadros de 31 ms (uma sonda assistindo).
- **`fail --reabrir`**: o servidor só aceita o `unlive` repetindo o `requestId` e o número de ordem do `live` em vigor (`shared/media.ts`, `nextBroadcastOrder`); o `live` seguinte usa o número + 1 e um `requestId` novo. Senão volta `bad-message` "Transmissão encerrada ou substituída". Quem assiste (sonda) volta sozinho ao receber o `peer-live`.
- **Achado no caminho pelo processador (vale também para o G1)**: duas ligações dividindo a **mesma** fonte crua, uma de miniatura (`maxFramerate` 15) e uma cheia, fazem a cheia cair para **15 qps**: o difusor da fonte junta os pedidos dos receptores e fica com o menor. Com as duas em `full` voltou a 60 qps; ao pôr uma de volta em `thumb`, caiu de novo. No app, o caminho pelo processador precisa de uma fonte crua por qualidade (ou aceitar essa perda). Não corrigido na sonda.
- Depois de `active=false`, o `get_stats` do envio mostra `0x0 a 0 qps`; ao voltar, o vídeo volta na primeira entrega.
- `[envio vídeo]` logo depois da troca de faixa ainda mostra o `encoderImplementation` antigo uma vez (a estatística é do intervalo anterior).

## O que a sonda não prova sozinha

- **A imagem no Chrome**: o que foi visto foi uma sonda (decodificador FFmpeg do motor, estatísticas dela), não o Chrome. Decodificação, `frameWidth/Height`, `framesPerSecond`, `freezeCount`, o `profile-level-id` aceito pelo Chrome e se o NAL 9 passa são do G4 e do G5.
- **A captura**: a sonda gera o quadro no processador e o sobe para uma textura. Não prova captura e codificador no mesmo dispositivo gráfico, nem o custo no FPS do jogo (G6; plano B, entrada pela memória).
- **O processador do app**: o número da sonda inclui gerar, subir para a textura e (com `--ruido`) criar ruído; o app não gasta isso.
- **Outras marcas** (AMD, Intel): só a NVIDIA iniciou aqui. Conferência com o grupo.
- **Mudança de tamanho ao vivo**: a mudança de taxa dispensou reabrir o codificador; a de tamanho (R18) não foi testada.
- **A regra de banda** (descer para a reduzida por banda fraca) não está na sonda: só o comando `tier` troca de qualidade à mão.
- **Safari/iPhone**: não testado.
- **A cheia sempre no ar**: a sonda não para a cheia quando ninguém a recebe (só a reduzida para).
