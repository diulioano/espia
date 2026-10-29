# Data model — 005

Tudo vive **só no PC de quem transmite**. Nada vai para o Worker, exceto o novo valor `'pc'` de `AudioKind`.

## CaptureSource (Rust → página, a cada 2 s com o seletor aberto)

| Campo | Tipo | Regra |
|---|---|---|
| `id` | string | `"w:<hwnd>"` para janela, `"m:<hmonitor>"` para monitor. Vale enquanto a fonte existir. |
| `kind` | `'window' \| 'monitor'` | |
| `title` | string | Título da janela ou "Tela N". Não vazio. |
| `app` | string | Nome do programa (FileDescription ou nome do .exe). Em monitor: "Principal" ou "". |
| `icon` | string \| null | `data:image/png` 32×32, cacheado por processo. |
| `thumb` | string \| null | `data:image/jpeg` ≤ 320×180. `null` = minimizada ou captura preta → a página mostra o ícone. |
| `playing` | boolean | Selo "tocando som" (R8). Sempre `false` em monitor. |
| `primary` | boolean | Só monitor. |

Filtro de janelas (R2): visível, com título, sem `WS_EX_TOOLWINDOW`, não cloaked, sem dono e com PID ≠ Espia.

## PickerState (página, `src/lib`, view-model do seletor)

| Campo | Tipo | Notas |
|---|---|---|
| `mode` | `'native' \| 'browser'` | `'native'` se `capture_supported()` respondeu `true`. |
| `tab` | `'window' \| 'monitor'` | O `surface` atual das prefs. |
| `sources` | `CaptureSource[]` | Filtradas pela aba. As contagens das abas saem da lista completa. |
| `selectedId` | string \| null | Some da seleção quando a fonte some da lista (US1 cenário 5). |
| `audio`, `resolution`, `fps` | prefs | Como na 004. |
| `audioText` | string | Ver a tabela de textos abaixo. |
| `canStart` | boolean | `mode === 'browser' \|\| selectedId !== null`. |

## GoLivePrefs (localStorage, estende a 004)

Os campos novos são `lastSourceKey?: string`, que identifica a fonte para remarcar (US5): `"<app>|<title>"` para janela e `"m:<title>"` ("m:Tela 2") para monitor. O HWND muda quando o programa reabre, por isso a chave não usa ele.

## NativeCaptureSession (Rust)

Estados: `idle → starting → live → (quality-change → live) → stopped`.

- `starting`:
  - cria o item WGC, os slots (3× NV12) e o cliente de áudio conforme a `AudioPlan`;
  - qualquer falha de vídeo leva a `stopped` e a página recebe um erro;
  - uma falha de áudio **não** derruba o vídeo: a sessão segue com `audio = 'none'` e `audioWarn`.
- `live`: quadros pelo anel, áudio pelo canal e, quando aplicável, o vigia do Discord (1 s).
- `stopped`: libera WGC, áudio e slots, e manda `ended` pelo canal. Acontece quando a fonte fecha, quando a página chama `capture_stop` ou quando a janela recarrega.

Há uma sessão por vez. `capture_start` com uma sessão ativa encerra a anterior.

## AudioPlan (decisão pura, testável)

| Fonte | Som ligado? | Discord | Plano | `AudioKind` enviado |
|---|---|---|---|---|
| janela | não | — | nenhum | `none` |
| janela | sim | — | `include(raiz da janela)` | `window` |
| janela | sim, mas ativação falhou | — | nenhum + aviso | `none` |
| monitor | não | — | nenhum | `none` |
| monitor | sim | 0 raízes | `exclude(pid do Espia)` | `pc` |
| monitor | sim | 1 raiz | `exclude(raiz)` | `pc` |
| monitor | sim | 2+ raízes | nenhum + aviso "Mais de um Discord aberto" | `none` |
| monitor | sim, mas ativação falhou | — | nenhum + aviso | `none` |

Se a janela escolhida **for** do Discord, ela vai com o som dela, porque a pessoa escolheu isso de propósito.

## Textos do "Compartilhar áudio" (modo nativo)

| Situação | Texto |
|---|---|
| Janela + som | "Só o som de *<title>* — nada de notificação vazando." |
| Janela sem escolha + som | "Só o som da janela que você escolher — Discord e notificações ficam de fora." |
| Tela + som | "Todo o som do computador vai junto — menos o Discord." |
| Som desligado | "Só a imagem vai pra sala." |

No modo `'browser'`, os textos da 004 continuam iguais.

## AudioKind (protocolo, `src/shared/protocol.ts`)

`'tab' | 'window' | 'pc' | 'none'`. `'pc'` aparece no card como "com som do PC (sem Discord)" e, para quem assiste, como "· com som".
