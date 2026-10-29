# Data model — 002 Som da transmissão

## AudioKind (novo)

`'tab' | 'window' | 'none'` — o som que realmente está saindo na transmissão.

## Regra de decisão (função pura, `src/lib/audio-source.ts`)

Entrada:
- `wanted: boolean`: "Compartilhar áudio" ligado no modal.
- `surface: 'browser' | 'window' | 'monitor' | undefined`: `displaySurface` da faixa de vídeo.
- `audioLabel: string | null`: rótulo da faixa de áudio recebida, ou `null` se não veio faixa.

Saída: `{ kind: AudioKind; drop: boolean; warn: boolean }`
- `drop`: a faixa de áudio recebida MUST ser parada e removida antes de enviar.
- `warn`: mostrar o toast do FR-208/FR-203.

| wanted | surface | faixa de áudio | kind | drop | warn |
|---|---|---|---|---|---|
| não | qualquer | não veio | none | — | não |
| não | qualquer | veio | none | sim | não |
| sim | browser | veio | tab | não | não |
| sim | window | veio, rótulo não diz "sistema" | window | não | não |
| sim | window | veio, rótulo diz "sistema" (R3) | none | sim | sim |
| sim | monitor | veio | none | sim | sim |
| sim | monitor | não veio | none | — | sim |
| sim | browser/window | não veio | none | — | não |
| sim | indefinida | veio | none | sim | não |

O toast não aparece quando a pessoa simplesmente desmarcou o som no seletor de aba ou janela: foi escolha dela (edge case da spec).

## Mudanças em tipos existentes

- `shared/protocol.ts` → `LiveInfo.audio: AudioKind` (era `boolean`). Mensagem `live` do cliente: `audio: AudioKind` (o Worker ainda aceita `boolean`, ver contrato).
- `lib/types.ts` → `MyBroadcastView` ganha `audioKind: AudioKind` e `audioReason?: 'monitor'`. A escolha `audio: boolean` do modal (GoLivePrefs) continua igual.
- `StreamView.quality` (subtítulo de quem assiste) passa a incluir "· com som" / "· sem som".

## Estados ao longo da transmissão

```
captura → decide (regra acima) → [drop?] → anuncia live{audio: kind} → envia
   ↑                                                        |
   └── configurationchange / addtrack / removetrack ────────┘  (reavalia; só reanuncia se o kind mudou)
```
