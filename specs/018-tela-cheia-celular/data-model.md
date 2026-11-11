# Data Model: Tela cheia e sala responsiva no celular

Nada é guardado no servidor nem no `localStorage`. Tudo vive na memória da página enquanto a pessoa está na sala.

## Estado da sala (store → `RoomView`)

| Campo | Tipo | Onde | Regra |
|-------|------|------|-------|
| `fullscreenId` | `string \| null` | `RoomStore` → `RoomView.fullscreenId` | id da transmissão aberta no visualizador. Começa `null`. |
| `mockViewer` | `{ controls: boolean; zoom: ZoomState; volume?: boolean } \| undefined` | só `RoomView` dos mocks | estado inicial do visualizador nas cenas mockadas (constituição V). A store real nunca preenche. |

**Ação nova** em `RoomActions`: `fullscreen(id: string | null): void`.
- `id` só é aceito se a transmissão existe e está sendo assistida (`watching === true`). Caso contrário, nada muda.
- `null` fecha.

**Regra derivada** em `src/lib/room-view.ts`: `viewerStream(view): StreamView | undefined`.
- Devolve `view.streams.find(s => s.id === view.fullscreenId)` quando `watching === true` e `view.link !== 'failed'`.
- Senão devolve `undefined`.

**Transições de `fullscreenId`**:

```text
null ──fullscreen(id)──▶ id
id ──fullscreen(null) | voltar | saída da tela cheia do sistema──▶ null
id ──transmissão some / reinicia (startedAt novo) | watch(id,false) | pip abre──▶ null
id ──link: reconnecting──▶ id         (continua; ReconnectPill no visualizador)
id ──link: failed──▶ viewerStream = undefined (visualizador some; o id é zerado ao voltar a 'ok')
id ──sair da sala / removido / sala apagada──▶ (store destruída)
```

Abrir ou fechar não mexe em `watching`, `stopped`, `focusedId` nem `viewModes()`. A recepção não muda (FR-005).

## Zoom (`src/lib/zoom.ts`, estado local do `Viewer`)

```ts
interface ZoomState { scale: number; x: number; y: number }   // x,y em px, translate antes do scale
interface Box { width: number; height: number }               // tamanho do visualizador
const ZOOM_NONE: ZoomState = { scale: 1, x: 0, y: 0 };
const MAX_ZOOM = 4;
```

**Regras** (`clampZoom`):
- `1 ≤ scale ≤ MAX_ZOOM`;
- `box.width·(1 − scale) ≤ x ≤ 0`, e o mesmo para `y` com `height`, para a imagem nunca descolar das bordas;
- `scale === 1` faz `x = y = 0`.

**Gestos**:

| Função | O que faz |
|--------|-----------|
| `pinch(state, a0, b0, a1, b1, box)` | aproxima ou afasta mantendo o ponto médio dos dedos fixo; depois aplica `clampZoom`. |
| `pan(state, dx, dy, box)` | arrasta com um dedo, só quando `scale > 1`; depois aplica `clampZoom`. |
| `isDoubleTap(prev, now)` | `true` se `now.t − prev.t ≤ 300 ms` e a distância é de até 24 px; aí o zoom volta a `ZOOM_NONE`. |

**Volta a `ZOOM_NONE`**: toque duplo, fechar o visualizador, `resize` (giro) ou troca da transmissão aberta.

## Modo do visualizador (local do `Viewer`)

`'system' | 'page'`:
- `system`: a camada está em tela cheia do aparelho (`document.fullscreenElement === camada`);
- `page`: a camada só cobre a página, como no iPhone ou quando o pedido de tela cheia foi negado.

Só muda a aparência: em `page`, o botão de sair fica visível e os controles respeitam as áreas seguras.

## Layout de celular (`src/lib/layout.ts`)

```ts
const PHONE_LAYOUT_QUERY = '(max-width: 639px), (pointer: coarse) and (hover: none) and (max-height: 540px)';
function isPhoneLayout(i: { width: number; height: number; coarse: boolean; hover: boolean }): boolean;
```

Vale `true` quando a largura é de até 639 px, ou quando há toque sem hover e a altura é de até 540 px. Isso alimenta `RoomRoute.mobile` e `store.mobile`, como hoje.

## Capacidade de volume (`src/lib/media-caps.ts`)

`canSetVolume(): boolean` faz um teste só, guardado em memória: escreve `volume = 0.5` num `<audio>` solto e confere se leu 0,5. Com `false`, o tile e o visualizador escondem o controle de volume e deixam só o mudo.
