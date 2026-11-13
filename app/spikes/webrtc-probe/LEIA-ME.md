# Sonda G1 (019, T009): libwebrtc da LiveKit no app nativo do Espia

Crate isolada (fora do workspace de `app/`) que entra numa sala do site local como se fosse mais um cliente, usando `libwebrtc` `=0.4.0`. Serve para o gate G1 (tarefa T010). Nunca captura tela, câmera ou microfone: o vídeo é gerado (barras coloridas, contador de quadros e um clarão por segundo) e o som é um tom de 440 Hz com um bip de 1 kHz por segundo.

## Compilar (item a)

```powershell
cd app\spikes\webrtc-probe
cargo build --release -j 6
.\target\release\webrtc-probe.exe --help
```

- O `.cargo\config.toml` liga `+crt-static`. Sem isso o `link.exe` falha com LNK2038 (`RuntimeLibrary`), porque o `webrtc.lib` pré-compilado usa a biblioteca C estática (/MT).
- Na primeira compilação o `webrtc-sys` baixa o pacote pronto (`webrtc-a545de4`, ~90 MB de `webrtc.lib`) de `github.com/livekit/rust-sdks/releases` e guarda em `target\release\build\scratch-*\out\livekit_webrtc\livekit\win-x64-release-webrtc-a545de4\win-x64-release\`. Precisa de internet só nessa vez.
- Use `-j 6`: o computador já reiniciou com builds pesados em paralelo. Não rode dois builds juntos.
- Tamanho do executável (item i, parte 1): `(Get-Item .\target\release\webrtc-probe.exe).Length` (22.457.344 bytes, ~21,4 MB, medido em 2026-11-11 neste build).

## Preparação comum

1. Em outro terminal, na raiz do repositório: `npm run dev:worker` (site e servidor locais, padrão `http://127.0.0.1:8787`).
2. No Chrome, abrir `http://127.0.0.1:8787`, criar uma sala e anotar o código (ex. `abcd-efgh`).
3. A sonda manda `Origin` igual ao endereço de `--base` e `mediaProtocol: 2`. Se o servidor responder erro 403 ou fechar com 4426, é porque a base não é a do site ou o protocolo não bateu.
4. Para parar a sonda: digite `quit` + Enter, ou Ctrl+C.
5. Comandos ao vivo (digite uma linha e Enter): `bitrate N`, `fps N`, `scale X`, `active on|off`, `reset`, `view full|thumb|hidden|off`, `stats`, `quit`.

Variável usada abaixo: `$S = ".\target\release\webrtc-probe.exe"` e `$SALA = "<código>"`.

## Itens (a) a (j) da T010

### (a) Compila no Windows

```powershell
cargo build --release -j 6
```

Observar: termina sem erro e o `--help` roda. Anotar versão da crate (`libwebrtc 0.4.0`, `webrtc-sys 0.3.53`).

### (b) O Chrome recebe vídeo 1080p60 H.264 e som

```powershell
& $S --room $SALA
```

No Chrome (na sala), clicar em Assistir na transmissão "Sonda G1". Observar:
- No log da sonda: `codificadores disponíveis`, `ordem depois de orderCodecs` (H264 primeiro), `canal de vídeo criado (H.264 primeiro=true ...)` e as linhas `[envio vídeo] ... encoderImplementation='...'`.
- No Chrome: `chrome://webrtc-internals` (aba da ligação, `inbound-rtp` de vídeo): `frameWidth=1920`, `frameHeight=1080`, `framesPerSecond` perto de 60, `codecId` apontando para `video/H264`, `decoderImplementation`. Ouvir o bip a cada segundo.
- Anotar sim/não e os números reais (a sonda pode ficar limitada pelo processador: ver `qualityLimitationReason`).

### (c) A sonda recebe vídeo e som do Chrome

No Chrome, a pessoa começa a transmitir (captura real escolhida por ela, janela de teste) e deixa a sonda como espectadora:

```powershell
& $S --room $SALA --no-live --view full
```

Observar: `chegou faixa de VÍDEO do outro lado`, `chegou faixa de SOM do outro lado` e, a cada 5 s, `[recebe vídeo] ... decoderImplementation` e `[recepção na sonda] vídeo N qps entregues ... som N amostras/s`. Passa se os quadros e as amostras sobem.

### (d) Troca de `ViewMessage` pelo canal `ctl`

Qualquer um dos comandos acima. Observar no log: `canal ctl open`, `ctl → view v2 modo=...`, `ctl ← view v2 modo=... (decodeView ok)` e `ctl ← view-sync`. Mudar o modo com `view thumb`, `view hidden`, `view off` e ver o Chrome responder. Uma mensagem inválida aparece como `ctl ← mensagem inválida, ignorada`.

### (e) Parâmetros por ligação e `qualityLimitationReason`

Com a sonda transmitindo e o Chrome assistindo (como em (b)):

```powershell
& $S --room $SALA --cycle 20      # troca sozinha a cada 20 s: cheio, miniatura, fps 30, 1 Mbps, escala 2, inativo
```

ou, à mão, pelo teclado: `bitrate 1000000`, `fps 30`, `scale 2`, `active off`, `active on`, `reset`. Cada troca imprime `set_parameters(vídeo) ok -> pedi ... | o motor guardou ...`. Observar:
- Efeito no Chrome (`webrtc-internals`): resolução cai com `scale 2`, fps cai com `fps 30`, bitrate cai com `bitrate`, o vídeo para com `active off`.
- Nas linhas `[envio vídeo]`: `qualityLimitationReason=None|Cpu|Bandwidth|Other` e `tempo por motivo de limitação`.
- Também vale `--bitrate`, `--fps`, `--scale`, `--active` na linha de comando para começar já com o ajuste.

### (f) Placa de vídeo ou processador

```powershell
& $S --room $SALA --stats-secs 5
```

Observar `encoderImplementation='...'` e `powerEfficientEncoder=` nas linhas `[envio vídeo]` e `[proc] cpu X% de um núcleo (Y% da máquina)`. Comparar com o G0 (T008): passa se a placa codifica, ou se o `[proc]` fica em até metade do número do G0 (mesmo jogo, mesma máquina). Esperado neste build: `Hardware = []` no Windows (ver tabela da API abaixo), então o caminho é o processador. Comparar também `--backend software` e `--backend auto`.

### (g) Som e imagem a até 80 ms em 10 minutos

Com o Chrome transmitindo e a sonda só vendo (como em (c)):

```powershell
& $S --room $SALA --no-live --minutes 10 --exit-after
```

A cada minuto a sonda imprime `[sync] ... estimatedPlayoutTimestamp som - imagem = +X ms` e, no 10º, `RESULTADO ... maior |som - imagem|`. Passa se ficar até 80 ms. Limite do método: `estimatedPlayoutTimestamp` é uma estimativa do próprio motor, não uma medida ao ouvido. Para uma medida independente, rodar duas sondas (uma transmite, outra assiste e detecta o clarão e o bip):

```powershell
& $S --room $SALA --nick Emissora                      # terminal 1
& $S --room $SALA --no-live --detect-pulse --minutes 10 --exit-after --nick Receptora   # terminal 2
```

Isso mede som x imagem entre sondas (mesmo motor nas duas pontas); o Chrome como emissor só entra pela estatística. Se a estatística vier vazia (`sem estimatedPlayoutTimestamp dos dois lados`), anotar "não deu para medir" e medir ao ouvido/gravando a tela do Chrome com o tom conhecido.

### (h) Safari no iPhone recebe

O celular precisa alcançar o site: `npm run dev:lan` (como na 018) e usar o endereço da rede como base, o mesmo para a sonda e para o iPhone:

```powershell
& $S --base http://192.168.X.Y:8787 --room $SALA
```

No iPhone (Safari), abrir `http://192.168.X.Y:8787`, entrar na sala e assistir "Sonda G1". Observar imagem e som. Safari só decodifica H.264: se a sonda não oferecer H.264 não aparece nada (`--no-h264` serve de contraprova).

### (i) Memória do processo e tamanho do executável

- Memória: linha `[proc] ... memória privada N MB | working set N MB`, a cada 5 s, com a sonda transmitindo e vendo (anotar o valor estável depois de 2 minutos e o pico).
- Tamanho: `(Get-Item .\target\release\webrtc-probe.exe).Length`.
- Alternativa de fora: `npm run memoria -- -Nativo` (modo criado pelo T006, se já existir).

### (j) Aviso de firewall do Windows

Fechar a sonda, apagar a regra se existir (Firewall do Windows, Regras de entrada, `webrtc-probe.exe`) e rodar de novo como em (b). Anotar se o Windows abriu a janela "Permitir que este aplicativo se comunique", e em que momento (ao abrir o primeiro canal UDP). Rodar também com `--base` na rede local (como em (h)), porque só com 127.0.0.1 o aviso pode não aparecer.

## O que a sonda imprime

Todas as linhas começam com o tempo desde o início. Marcadores: `[envio vídeo]`, `[envio som]`, `[recebe vídeo]`, `[recebe som]`, `[rota]` (confere que não é `Relay`), `[recepção na sonda]`, `[proc]`, `[sync]`. Candidatos `typ relay` são descartados nos dois sentidos, e só servidores STUN do `iceServers` são usados.

## O que a API da `libwebrtc` 0.4.0 expõe

Conferido no código-fonte em `~/.cargo/registry/src/*/libwebrtc-0.4.0` e `webrtc-sys-0.3.53`; o build compila usando tudo o que está marcado "sim".

| Item | Expõe? | Tipo ou função |
|------|--------|----------------|
| Bitrate máximo por ligação | sim | `RtpSender::parameters()` / `set_parameters()`, campo `RtpEncodingParameters::max_bitrate` |
| FPS máximo por ligação | sim | `RtpEncodingParameters::max_framerate` |
| Redução de resolução | sim | `RtpEncodingParameters::scale_resolution_down_by` |
| Ativo/inativo | sim | `RtpEncodingParameters::active`; também `RtpSender::set_track(None)` |
| Preferência de degradação | sim | `RtpParameters::set_degradation_preference(DegradationPreference::MaintainResolution)` |
| Preferência de codec (H.264 primeiro) | sim | `PeerConnectionFactory::get_rtp_sender_capabilities` + `RtpTransceiver::set_codec_preferences` |
| Canal de dados | sim | `PeerConnection::create_data_channel`, `on_data_channel`, `DataChannel::send`, `on_message`, `on_state_change` |
| Negociação perfeita (rollback) | parcial | `SessionDescription::parse("", SdpType::Rollback)` + `set_local_description`; não há `setLocalDescription()` sem argumento, a sonda monta o rollback à mão |
| Estatísticas de envio | sim | `PeerConnection::get_stats()` -> `RtcStats::OutboundRtp`, campos `quality_limitation_reason`, `quality_limitation_durations`, `encoder_implementation`, `power_efficient_encoder`, `total_encode_time` |
| Estatísticas de recepção | sim | `RtcStats::InboundRtp`: `decoder_implementation`, `jitter_buffer_delay`, `estimated_playout_timestamp`, `freeze_count` |
| Fonte de vídeo própria | sim | `video_source::native::NativeVideoSource::new` + `capture_frame` com `I420Buffer`; `PeerConnectionFactory::create_video_track` |
| Fonte de som própria | sim | `audio_source::native::NativeAudioSource::new` + `capture_frame`; `create_audio_track` |
| Receber quadros e som | sim | `video_stream::native::NativeVideoStream`, `audio_stream::native::NativeAudioStream` |
| Processamento de voz (eco, ruído, ganho) | sim, em dois lugares | `AudioSourceOptions { echo_cancellation, noise_suppression, auto_gain_control }` na fonte e `native::apm::AudioProcessingModule::new(eco, ganho, passa-altas, ruído)` (`process_stream`, `process_reverse_stream`, `set_stream_delay_ms`). A sonda gera tom, então deixa tudo desligado |
| Codificação pela placa de vídeo no Windows | NÃO neste build | `VideoEncoderBackend::list_available()` devolve `[Auto, Software, PreEncoded]` e `video_encoder_codecs(Hardware)` devolve `[]`. No `webrtc-sys` 0.3.53 o NVENC só é compilado no Linux (`build.rs`, ramo com `CUDA_HOME`); o ramo Windows tem o VAAPI comentado e nenhuma fonte de Media Foundation. `VideoEncoderBackend::Nvenc` e `Hardware` existem na enum mas não servem no Windows. Resta o H.264 do OpenH264 em software (e VP8/VP9/AV1). Isso é o que o item (f) mede; se reprovar, é o plano B de R2 |
| Codificador "pré-codificado" | sim | `VideoEncoderBackend::PreEncoded`: passa quadros já codificados. Caminho possível para codificar com a Media Foundation por fora (não testado) |
| Atraso de reprodução baixo | sim | `PeerConnectionFactory::with_zero_playout_delay()` (opção `--low-latency`) |
| Link no Windows | exige `+crt-static` | `.cargo/config.toml`: `rustflags = ["-C", "target-feature=+crt-static"]` para `x86_64-pc-windows-msvc`. Sem isso: LNK2038. O app de verdade (`app/`) vai precisar da mesma flag, o que afeta qualquer outra crate que não aceite CRT estático |
| Saída de áudio (alto-falante) | não testado | a libwebrtc tem o dispositivo de áudio (`audio_device_controller` no `webrtc-sys`); a sonda lê as amostras com `NativeAudioStream` e não toca nada. Tocar no alto-falante é assunto da fase de som do app |

Pontos que a sonda não consegue provar sozinha: o item (c) com Chrome real, o (g) por estatística (depende de `estimated_playout_timestamp` vir preenchido), (h) e (j).

## Licenças do pacote baixado (R2, "não confirmado")

O pacote tem `LICENSE.md` (~197 KB) na raiz, em `target\release\build\scratch-*\out\livekit_webrtc\livekit\win-x64-release-webrtc-a545de4\win-x64-release\LICENSE.md`. Ele lista, em ordem: webrtc, abseil-cpp, boringssl, compiler-rt, dav1d, ffmpeg, fft, fiat, g711, g722, libaom, libc++, libjpeg_turbo, libsrtp, libvpx, libyuv, nasm, ooura, openh264, opus, perfetto, pffft, protobuf, rnnoise, spl_sqrt_floor, zlib.

Atenção para o T093 (arquivo de licenças do instalador):
- A seção `ffmpeg` traz os textos da GPL v2 e da GPL v3 (e menções à LGPL). O `args.gn` do pacote tem `ffmpeg_branding = "Chrome"` e `rtc_use_h264 = true`. Não foi possível confirmar, sem ler o binário, se o código do ffmpeg está de fato dentro do `webrtc.lib` nem sob qual licença (o `webrtc.ninja` cita ffmpeg em 2 linhas). Isto precisa de uma conferência antes de distribuir: se o ffmpeg com GPL estiver linkado, a obrigação muda.
- `openh264` é BSD de 2 cláusulas da Cisco. A Cisco só cobre a patente de H.264 se o binário vier da Cisco; compilado aqui dentro do pacote, a obrigação de patente de H.264 (MPEG LA) continua sem confirmação.
- O pacote inteiro deve ser copiado para o arquivo de licenças do instalador (T093).
