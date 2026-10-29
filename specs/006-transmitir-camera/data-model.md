# Data model — 006

## VideoDevice (página, só no PC/celular de quem transmite)

| Campo | Tipo | Regra |
|---|---|---|
| `id` | string | `deviceId` do navegador. |
| `groupId` | string | Mesmo dispositivo físico = mesmo grupo (pareia com o som). |
| `label` | string | Nome do dispositivo. Vazio antes da permissão → a aba mostra "Permitir câmeras". |
| `kind` | `'camera' \| 'capture'` | `capture` se o nome bater com placa (R4); só muda o ícone e a sugestão de som. |
| `preview` | MediaStream \| null | Prévia de 320×180 a 15 fps. No máximo 4 não nulas ao mesmo tempo. |
| `busy` | boolean | `NotReadableError` ao abrir a prévia ("Em uso por outro programa"). |

## AudioInput

| Campo | Tipo | Regra |
|---|---|---|
| `id`, `groupId`, `label` | string | Da `enumerateDevices()`. |
| `suggestedFor` | string \| null | O `VideoDevice.id` com o mesmo `groupId` ou o nome mais parecido. |
| `role` | `'voice' \| 'game'` | Heurística R4; padrão `voice`. |

## CameraChoice (estado do seletor, aba Câmeras)

| Campo | Tipo | Notas |
|---|---|---|
| `permission` | `'unknown' \| 'prompt' \| 'granted' \| 'denied'` | Define o que a aba mostra. |
| `videoId` | string \| null | Obrigatório para liberar Transmitir. |
| `audioId` | string \| null | `null` = sem som. Só vale com "Compartilhar áudio" ligado. |
| `audioRole` | `'voice' \| 'game'` | Muda o processamento (FR-606). |
| `facing` | `'user' \| 'environment'` | Só no celular. |

**Transições de `permission`**:
- `unknown` → `prompt`, se a permissão ainda não foi pedida, ou → `granted`, se já foi;
- `prompt` → `granted` ou `denied`, no clique em "Permitir câmeras";
- `denied` → `granted` só se a pessoa liberar no navegador/app e reabrir o seletor.

## Constraints por tipo de som

| `audioRole` | echoCancellation | noiseSuppression | autoGainControl | channelCount |
|---|---|---|---|---|
| `game` | false | false | false | 2 |
| `voice` | true | true | true | 1 |

## GoLivePrefs (localStorage, estende 004/005)

- `surface` passa a aceitar `'camera'`.
- Campos novos: `lastVideoLabel?`, `lastAudioLabel?` e `lastAudioRole?`. Guarda o **nome**, porque o `deviceId` pode mudar entre sessões em alguns navegadores. Ao reabrir, remarca o dispositivo cujo nome bate.

## Protocolo

- `live`: `{ t, resolution, fps, audio, source? }`, com `source: 'screen' | 'camera'` (ausente = `'screen'`).
- `AudioKind`: `'tab' | 'window' | 'pc' | 'device' | 'none'`.
- `LiveInfo` ganha `source`. O tile mostra "câmera de <apelido>" quando `source === 'camera'`.
