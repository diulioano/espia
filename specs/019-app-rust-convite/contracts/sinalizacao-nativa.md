# Contrato: sinalização e ligação do app nativo (R5, R6, R20)

O app nativo fala com o mesmo servidor, pelo mesmo protocolo do site. As referências são `src/shared/protocol.ts`, `src/shared/media.ts`, `src/worker/index.ts` e `src/worker/validate.ts`. Os tipos em `espia-core::protocol` e `espia-core::media_policy` espelham esses arquivos.

**Nenhuma mensagem nova, nenhum campo novo, nenhuma mudança no Worker.**

Para o servidor, o app nativo é mais um cliente. Exemplos JSON de cada tipo de mensagem ficam em `app/fixtures/protocol/` (T015) e são conferidos dos dois lados: `cargo test` e `test/shared/protocol-fixtures.test.ts`, com os validadores de `src/worker/validate.ts` (T016).

## Rotas HTTP que o app chama

Base: `https://espia.diuliano.com`. A variável de ambiente `ESPIA_SERVER` (exemplo `http://127.0.0.1:8787`) troca o endereço base, só em build de depuração (debug). Build de release usa sempre `https://espia.diuliano.com` e ignora a variável. O app manda `Origin` igual à origem do endereço base em uso (o Worker exige `Origin` com o mesmo host do pedido, em `sameOrigin`, `src/worker/index.ts`). Com o endereço oficial, `Origin: https://espia.diuliano.com`. Com o servidor local, `Origin: http://127.0.0.1:8787`. Em release o app só fala com o endereço oficial: com `Origin` oficial e host antigo o Worker responderia 403 `origin`.

| Chamada | Corpo | Resposta de sucesso | Falhas |
|---|---|---|---|
| `POST /api/rooms` | vazio = sala rápida; `{ "kind": "fixed", "name": "…" }` = sala fixa | `201 { "code" }` ou, em sala fixa, `201 { "code", "ownerKey" }` | `400 name`, `403 origin`, `429` (com `Retry-After: 60`), `503 busy` |
| `POST /api/rooms/status` | `{ "codes": [...] }`: de 1 a 5 códigos válidos, sem repetir | `200 { "rooms": [RoomStatus] }` (só números, nunca apelidos) | `400 codes`, `403 origin`, `429` |
| `GET /api/rooms/:code` | | `200 RoomPreview` | `404 not-found`, `410 { "reason": "deleted" \| "expired" }`, `400 code`, `429` |
| `PATCH /api/rooms/:code` | `{ "name": "…" }` | `200 { "name" }` | `403 owner`, `404`, `410`, `400 name`, `429` |
| `DELETE /api/rooms/:code` | | `204` | `403 owner`, `404`, `410`, `429` |
| `POST /api/rooms/:code/remove` | `{ "peerId": "<16 hex>" }` | `204` | `400 peer`, `403 owner`, `404`, `410`, `429` |

- Corpos de até 1024 caracteres (`MAX_BODY`). Nome de sala: 1 a 40 caracteres, normalizado como `normalizeRoomName` (`src/shared/room-name.ts`).
- Ações de dono (`PATCH`, `DELETE`, `remove`) levam `Authorization: Bearer <chave de dono, 64 hex>`. A chave vem de `owners` em `device.json`.
- A chave de dono vem só na resposta de criação da sala fixa e vai direto para `device.json`. Nunca é registrada nem enviada em outra chamada além das ações de dono.
- Mapa de erros do app: o do `OwnerFailure` de `src/lib/api.ts` (`403` = `forbidden`, `404` e `410` = `gone`, `400` = `name`, `429` = `rate-limited`, o resto = `network`), com os mesmos textos de `ownerErrorText`.

## WebSocket

```text
wss://espia.diuliano.com/ws/<código>
Origin: https://espia.diuliano.com
```

Com `ESPIA_SERVER` (só em debug), o esquema vira `ws://` ou `wss://` conforme o endereço base, e `Origin` é a origem desse endereço.

O código tem que casar `ROOM_ID_RE`. Sem `Origin` válido o servidor responde `403 origin`; fora do formato, `400 code`; acima do limite, `429`.

### Entrar e retomar

Ao abrir o socket, o primeiro envio é um destes:

| Situação | Mensagem |
|---|---|
| Sem sessão guardada para a sala | `{ "t": "join", "nickname": "…", "mediaProtocol": 2 }` |
| Com sessão em `sessions` (menos de 24 h) | `{ "t": "resume", "peerId": "…", "resumeToken": "…", "mediaProtocol": 2 }` |

- `mediaProtocol` é sempre `2` (`MEDIA_PROTOCOL`). O servidor fecha com 4426 quem não manda `2`.
- Apelido: 1 a 24 caracteres depois da limpeza do servidor (`cleanNickname`).
- Resposta normal: `welcome`. O app confere `mediaProtocol === 2` e `mediaPolicy.kind === 'direct-only'`; se não bater, encerra como `update` (mesmo efeito do 4426). Guarda `{ peerId: you.id, resumeToken }` em `sessions` e passa a `open`.
- `resume-failed` (passou da tolerância de 60 s): apaga a sessão e manda `join` na mesma conexão, como pessoa nova.
- `room-not-found`, `room-full`, `room-deleted` e `removed` encerram a sala sem reconectar, como os fechamentos correspondentes.

### Manter viva

- A cada 20 s o app manda exatamente `{"t":"ping"}` (`PING`, sem espaços; o objeto Durable responde sem acordar).
- Resposta esperada: `{"t":"pong"}`. Sem ela em 8 s, o app abandona o socket e reconecta na hora.

### Queda e volta

- Queda de rede ou fechamento fora da tabela abaixo: o app mostra a queda (`link: reconnecting`) e tenta de novo com espera crescente: 1 s, 2 s, 4 s, 8 s e depois 10 s a cada tentativa.
- Reconectando, manda `resume` com a sessão. Voltou: o `welcome` zera as tentativas e o app mostra "De volta!".
- Sem voltar em 60 s (`RECONNECT_FAIL_MS`): `link: failed`, a tela "Não deu pra voltar pra sala" com "Tentar de novo". O app continua tentando em segundo plano.
- "Tentar de novo" e a volta da internet tentam na hora, sem esperar a espera crescente.
- Mensagens perdidas durante a queda não fazem falta: o estado volta no `welcome`.
- Sair da sala: o app fecha com o código 4001 (os outros recebem `peer-left` na hora, sem esperar a tolerância de 60 s) e apaga a sessão.

### Códigos de fechamento

Fonte: `CLOSE` em `src/shared/protocol.ts` e `Signaling.dropped` em `src/lib/signaling.ts`. O que a tela mostra segue `src/routes/RoomRoute.svelte`.

| Código | Nome | O app reconecta? | O que o app mostra |
|---|---|---|---|
| 4426 | `update` | não | "Atualize o Espia" com o botão de atualizar (`contracts/atualizacao.md`) |
| 1008 | `policy` | não | volta à entrada da sala (consulta a prévia da sala, como o site) |
| 4404 | `notFound` | não | "Sala não encontrada" (a prévia diz se era rápida, apagada ou expirada) |
| 4409 | `full` | não | entrada com "sala cheia" |
| 4000 | `replaced` | não | entrada com o aviso de sala aberta em outra aba (a sessão fica guardada) |
| 4001 | `leave` | não (o app que fecha) | início |
| 4410 | `deleted` | não | "Sala não encontrada" por sala apagada com você dentro; se você mesmo apagou, volta ao início sem aviso |
| 4403 | `removed` | não | início com o aviso "O dono te tirou da sala"; o link continua valendo |
| qualquer outro (1006, rede, servidor) | | sim | `link: reconnecting`, depois `failed` em 60 s |

Em todo caso sem reconexão: a transmissão própria para, a mídia fecha e a sessão é apagada (menos no 4000).

## Ligação entre pessoas

Uma ligação WebRTC por par, em malha (`src/lib/mesh.ts`, `src/lib/peer.ts`), sem servidor de mídia. O servidor só repassa `signal`.

### Negociação perfeita

- Os dois lados podem renegociar. Quem é educado (polite) é quem tem o `id` menor: `myId < id` (`mesh.ts:148`).
- Chegou uma oferta junto de outra em andamento (`makingOffer` ou estado diferente de `stable`): o lado que não é educado ignora a oferta; o educado aceita e desiste da sua. Candidatos ICE da oferta ignorada também são ignorados.
- Mensagens `signal` de um par são processadas em ordem (um candidato nunca passa na frente da oferta).
- Candidatos ICE vão aos poucos (trickle) pelo `signal`.
- Se a conexão cai para `failed`, o app tenta um `restartIce` automático; falhando de novo, o tile mostra "Tentar de novo".

### ICE: só STUN

- O app usa só os servidores `stun:` e `stuns:` entregues pelo servidor no `welcome` e em `ice-servers` (`directIceServers`): descarta qualquer outra URL e qualquer servidor que fique sem URL.
- Candidato recebido do tipo `relay` (`/\btyp relay\b/i`) é descartado. Não há retransmissão (ADR 0001, constituição II).
- O app nunca manda `turn-refresh` (que o site manda antes de `iceExpiresAt`), porque usa só STUN. Entradas de `ice-servers` que não são STUN são ignoradas.

### Canal `ctl` e `ViewMessage` versão 2

- Quem chega abre o canal de dados `ctl` (`createDataChannel('ctl')`); o outro lado recebe pelo `ondatachannel`, só se o rótulo for `ctl`.
- Ao abrir, cada lado manda sua `view` atual e, se está transmitindo, pede a do outro com `view-sync`.
- Formato e validação de `ViewMessage` e de `view-sync`: ver `data-model.md` (seção `ViewMessage`).
- **Regra do Assistir (FR-021)**: quem transmite só envia imagem a quem mandou `full` ou `thumb`, e só envia som a quem não mandou `off`. A mensagem tem que ser válida, ser da transmissão atual e ter `revision` maior que a última aceita. Sem mensagem válida o modo é `off` e nenhuma faixa vai. Mensagem inválida nunca autoriza mídia. Quando o canal fecha, o modo volta a `off`.
- Nada é recebido antes de a pessoa clicar em Assistir: quem assiste manda `off` até lá.

| `mode` | Vídeo vai? | Som vai? |
|---|---|---|
| `full` | sim | sim |
| `thumb` | sim | sim |
| `hidden` | não | sim |
| `off` | não | não |

(`wantsVideo` e `wantsSound`, `src/lib/quality.ts`.)

### Parâmetros de envio por modo

Mesmos números de `sendParams` (`src/lib/quality.ts`), aplicados à codificação de cada ligação. `q` é a qualidade efetiva (abaixo).

| `mode` | `active` | `scaleResolutionDownBy` | `maxFramerate` | `maxBitrate` |
|---|---|---|---|---|
| `full` | `true` | `1` | `q.fps` | `q.maxBitrate` |
| `thumb` | `true` | `q.resolution / 360` | `15` | `500_000` (miniatura: cerca de 360p, 15 fps, 0,5 Mbps) |
| `hidden` | `false` | `1` | `q.fps` | `q.maxBitrate` |
| `off` | `false` | `1` | `q.fps` | `q.maxBitrate` |

- Som: `active` = som permitido pelo modo (`mode !== 'off'`), com `maxBitrate` de 128_000 (`AUDIO_BITRATE`, `peer.ts`).
- Os dois canais de envio (vídeo e som) são criados uma vez por ligação e reaproveitados entre uma transmissão e outra. Sem faixa, o canal fica `inactive`.
- O teto entra junto com a faixa, na criação do canal, e é reaplicado quando a negociação assenta (`signalingState === 'stable'`). Mudar parâmetros não pode se cruzar com outra mudança em andamento na mesma ligação.
- Preferência de degradação (`contentPreference`): dica `motion` = manter fps; `detail` ou `text` = manter resolução; sem dica, tela mantém resolução e câmera mantém fps.

### Qualidade por número de espectadores

`effectiveQuality(chosen, fullViewers, cpuSteps)` de `src/lib/quality.ts`. Só quem está em `full` conta (miniatura e escondido não contam).

| Espectadores em `full` | Qualidade |
|---|---|
| 1 a 4 | a escolhida |
| 5 a 6 | teto de 4 Mbps; com 60 fps escolhido, 720p; com 30 fps, mantém a resolução escolhida |
| 7 ou mais | 720p30 |

Nunca sobe acima do escolhido. Depois dessa regra, o ajuste por processador (`cpuSteps`, `src/lib/cpu-governor.ts`) tira degraus: primeiro o fps (60 para 30), depois a resolução (para 720p30, mínimo). Vale o mais baixo.

Bitrate máximo por espectador, antes do teto de 4 Mbps (`BITRATE`):

| Resolução e fps | Bitrate |
|---|---|
| 720p30 | 2.500.000 |
| 720p60 | 4.000.000 |
| 1080p30 | 4.500.000 |
| 1080p60 | 6.000.000 |

### Codec

H.264 primeiro, como no site (`orderCodecs`, `peer.ts`): primeiro H.264 com `packetization-mode=1` (o que decodificadores de celular e Safari esperam), depois os outros H.264, depois o resto, mantendo a ordem original dentro de cada grupo. Sem H.264 disponível no motor, fica a ordem padrão. Quem recebe segue a ordem da oferta. Se a crate `libwebrtc` expõe essa preferência: não confirmado, item do gate G1.

### Transmitir (`live` e `unlive`)

O app manda `live` e `unlive` com `requestId` (UUID v4) e `broadcastSeq`, e espera `live-ack`. O servidor aceita a ordem por `nextBroadcastOrder` (`src/shared/media.ts`): a primeira transmissão da sala usa `broadcastSeq` 0, e cada nova usa o anterior mais 1; o `unlive` repete o `requestId` e o `broadcastSeq` da transmissão que encerra. O `welcome` traz `broadcastOrder` para a retomada. O texto do que está sendo transmitido (`label`) tem até 60 caracteres (`LIVE_LABEL_MAX`) e nunca é o título de uma janela.
