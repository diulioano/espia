# Research: Tela cheia e sala responsiva no celular

**Feature**: [spec.md](./spec.md) · **Plan**: [plan.md](./plan.md) · **Date**: 2026-11-07

Estado de hoje, lido no código:

- `StreamTile.svelte` já tem um botão de tela cheia que chama `requestFullscreen()` no próprio tile.
  - No iPhone ele falha calado: o Safari do iPhone não põe em tela cheia um elemento que não seja vídeo.
  - No Android funciona, mas os controles têm 36 px no tamanho `grid`, que é o usado no celular.
- A sala do celular usa `MobileRoomPage` quando `matchMedia('(max-width: 639px)')` bate (`RoomRoute.svelte`).
  - Um celular deitado tem 844–932 px de largura e cai na `RoomPage` de computador.
  - O mesmo `mobile` liga a regra "uma transmissão por vez" na store (`viewModes`, `chooseWatch`).
- O `index.html` não tem `viewport-fit=cover`. Sem isso, `env(safe-area-inset-*)` vale 0 no iPhone, e o `max(54px, env(...))` da sala do celular nunca entra em ação.
- O vídeo é sempre mudo e o som toca num `<audio>` à parte (specs/014). Isso pesa na escolha da tela cheia (R2).

## R1 — Quando mostrar a sala de celular

**Decision**: a versão de celular vale quando bate `(max-width: 639px), (pointer: coarse) and (hover: none) and (max-height: 540px)`.

- A consulta fica numa constante em `src/lib/layout.ts` (`PHONE_LAYOUT_QUERY`).
- A mesma regra existe como função pura, `isPhoneLayout({ width, height, coarse, hover })`, para os testes.
- `RoomRoute` troca a consulta de hoje por essa e continua passando `store.mobile`.

**Rationale**:

- Celular deitado tem altura útil de 320–430 px. Tablet deitado tem 700 px ou mais, e computador tem mouse.
- O corte por altura com toque separa o celular sem olhar o user agent.
- Manter `store.mobile` ligado à mesma regra mantém "uma transmissão por vez" no celular deitado, como a 017 pede.

**Alternatives considered**:

- User agent: frágil, e o iPad se identifica como Mac.
- Só `orientation`: pegaria tablet deitado.
- Só `pointer: coarse`: pegaria notebook com tela de toque.

## R2 — Como fazer a tela cheia

**Decision**: um componente novo, `Viewer.svelte`, que é sempre uma camada `position: fixed; inset: 0` sobre a sala, com seus próprios controles e zoom. Depois de aberta, a camada pede a tela cheia do aparelho quando ele deixa.

- Com `element.requestFullscreen` (ou `webkitRequestFullscreen`), a camada vira tela cheia do sistema. Vale para Android Chrome e iPad.
- Sem isso (iPhone), a camada ocupa toda a área da página (`100dvw × 100dvh`). No iPhone deitado o Safari recolhe as barras sozinho.
- O pedido de tela cheia sai no mesmo toque do botão, porque os navegadores exigem gesto do usuário.

**Rationale**:

- Um caminho só para controles, zoom, saída e avisos, testável com mock.
- A tela cheia do sistema vira só um "esconder as barras" por cima.

**Alternatives considered**:

- **Tela cheia nativa do `<video>` no iPhone** (`webkitEnterFullscreen`), como a spec previa no FR-006. Rejeitada:
  - o player nativo controla o vídeo, que é mudo; o volume dele não mexe no `<audio>` à parte, então o som ficaria sem controle e o botão de mudo mentiria;
  - perde o zoom da US3 e os nossos controles;
  - o comportamento com `srcObject` de `MediaStream` varia entre versões do iOS.
- **Continuar pondo o `StreamTile` em tela cheia**: não funciona no iPhone, e o tile está preso no layout da sala.

**Spec atualizada**: o FR-006 passou a pedir que, sem tela cheia do aparelho, o Espia ocupe a página toda com imagem, som, controles e zoom.

## R3 — Gesto e botão de voltar

**Decision**:

- **Ao abrir**: `history.pushState({ ...history.state, viewer: true }, '')`, mesmo endereço.
- **`popstate` com o visualizador aberto e sem `viewer` no estado**: fecha o visualizador. Isso cobre o gesto de voltar e o botão do Android.
- **Fechar pelo botão ou pelo store**: se o topo do histórico ainda tem `viewer`, chama `history.back()`.
- **Saída da tela cheia do sistema** (o Android consome o primeiro "voltar" para isso): `fullscreenchange` fecha o visualizador e faz `history.back()`. Um "voltar" só sai da tela cheia, sem sair da sala.
- **Router** (`router.svelte.ts`): no `popstate`, se o caminho não mudou, mantém o mesmo objeto `route` e só atualiza `state`. Hoje ele reatribui `route` a cada `popstate`. O `{#key router.route.code}` do `App.svelte` não remontaria a sala, mas evitamos efeitos à toa.

**Rationale**: o "voltar" do celular é o jeito esperado de fechar algo que cobre a tela (FR-004). Sem a entrada no histórico, o voltar tiraria a pessoa da sala.

**Alternatives considered**:

- Só ouvir `fullscreenchange`: não funciona no iPhone, onde não há tela cheia do sistema.
- `navigation.intercept`: não existe no Safari.

## R4 — Zoom com dois dedos

**Decision**:

- **Gestos**: Pointer Events no visualizador com `touch-action: none`. No iOS também `gesturestart`/`gesturechange` com `preventDefault`, para o Safari não aproximar a página.
- **Desenho**: a imagem fica num invólucro com `transform: translate(x, y) scale(s)` e `transform-origin: 0 0`.
- **Contas**: funções puras em `src/lib/zoom.ts`:
  - `pinch(state, before, after, box)` aproxima em volta do ponto médio entre os dedos;
  - `pan(state, dx, dy, box)` arrasta;
  - `clampZoom(state, box)` limita a escala a 1–4 e não deixa a imagem descolar das bordas;
  - `isDoubleTap(prev, now)` reconhece dois toques em até 300 ms e 24 px um do outro.
- **Volta à imagem inteira**: toque duplo, saída, `resize` ou giro (`ZOOM_NONE`).

**Rationale**:

- Sem biblioteca, como pede a constituição.
- As contas ficam testáveis no Vitest, e o componente só liga os eventos.
- `transform` não força novo layout do vídeo nem pede mais qualidade à origem.

**Alternatives considered**:

- Zoom nativo do navegador: aproximaria a página toda, inclusive os controles.
- Ajustar `object-position`/`width` do vídeo: refaz o layout a cada quadro do gesto.

## R5 — Quando o visualizador fecha sozinho

**Decision**:

- **Estado**: `fullscreenId: string | null` na store, exposto no `RoomView`. A ação nova é `actions.fullscreen(id | null)`.
- **Regra**: `viewerStream(view)` em `room-view.ts` devolve a transmissão aberta só se ela:
  - existe,
  - está sendo assistida (`watching === true`),
  - e a ligação não está em `failed`.
- **Fechamento**: a store zera `fullscreenId` onde já zera o `pipId` quando a transmissão some, e também em `watch(id, false)`. A página só abre o visualizador quando `viewerStream` devolve algo.
- **Ser tirado da sala ou ver a sala apagada**: sai da sala e o visualizador some junto.
- **Avisos informativos**: o visualizador não mostra toasts. Eles continuam na sala por baixo.
- **Queda e volta automática** (`reconnecting`): mantém o visualizador e mostra o `ReconnectPill` compacto dentro dele.
- **Picture-in-picture** aberto pelo visualizador: fecha o visualizador e o PiP continua, como hoje.

**Rationale**:

- Regra pura e testável.
- O visualizador nunca fica aberto sobre uma tela que não chega mais.
- Abrir o visualizador não muda `watching` nem `viewModes`: a transmissão já era a `full` do celular. Assim nada no envio ou na recepção muda (FR-005, constituição I).

**Alternatives considered**: estado só local no componente. Rejeitado porque não dá para mockar (constituição V) nem fechar a partir da store.

## R6 — Giro e layout deitado

**Decision**:

- Nada de `screen.orientation.lock`: a tela segue o aparelho e a trava de giro, como diz a spec.
- `MobileRoomPage` ganha `@media (orientation: landscape)`:
  - à esquerda, o palco com a altura toda (fora as áreas seguras);
  - à direita, uma coluna de 280–340 px que rola, com cabeçalho, chips, pessoas e os botões Transmitir/Parar câmera/Sair.
- Em pé, o layout fica como hoje. O palco continua em 16:9, com `max-height: 60dvh`.
- Girar só troca CSS. O `StreamTile` e o `<video>` continuam montados, então o vídeo não reinicia (FR-005, SC-003).

**Rationale**: é a menor mudança que dá à imagem a maior parte da tela deitada sem esconder ações (FR-011).

## R7 — Áreas seguras, teclado e unidades

**Decision**:

- **`index.html`**: `<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover, interactive-widget=resizes-content">`.
- **Áreas seguras**: com `viewport-fit=cover`, toda página precisa respeitar `env(safe-area-inset-*)`. As margens da raiz entram nos blocos de celular de `HomePage`, `JoinPage`, `NotFoundPage` e `MobileRoomPage` (esquerda e direita no deitado, topo e base em pé). O visualizador mantém a imagem na tela toda e põe só os controles dentro das áreas seguras.
- **Altura**: `100dvh` no lugar de `100vh`, inclusive em `UpdateRequired`. Diálogos com `max-height: calc(100dvh - 32px)` e rolagem interna.
- **Teclado**: no Android, `interactive-widget=resizes-content` encolhe a página com o teclado aberto. No iPhone, os diálogos ficam presos no topo da área visível (`align-items: start` com margem) no celular, para o teclado não cobrir o campo. Isso vale para `NameDialog` e a entrada da `JoinPage`.

**Rationale**: sem `viewport-fit=cover`, o FR-014 não tem como ser atendido no iPhone deitado. O conteúdo fica com faixas pretas, mas os cantos arredondados e a ilha ainda cortam a imagem da tela cheia.

## R8 — Áreas de toque e volume no iPhone

**Decision**:

- **Áreas de toque**: `@media (pointer: coarse)` sobe `.ctl` para 44 px. Chips, botões da sala e o botão "⋯" sobem para pelo menos 44 px. No visualizador, a barra de controles tem botões de 48 px.
- **Controles no toque**: aparecem a cada toque e somem em 3 s. Hoje são 2 s, pensados para o mouse.
- **Volume no iPhone**: o Safari do iPhone ignora `audio.volume`, que fica sempre em 1. O Espia testa uma vez (`canSetVolume()` em `src/lib/media-caps.ts`: escreve 0,5 num `<audio>` solto e lê de volta).
  - Sem volume ajustável, o visualizador e o tile mostram só o botão de mudo. O volume fica nos botões do aparelho.
  - O volume guardado por pessoa continua valendo no computador.

**Rationale**: mostrar um controle de volume que não faz nada é pior que não mostrar. O FR-003 continua atendido, porque o som se controla pelo mudo e pelo volume do aparelho.

## R9 — Como testar

**Decision**:

- **Vitest**:
  - `zoom.ts`;
  - `isPhoneLayout`;
  - `viewerStream` e o fechamento do `fullscreenId` na store, nos testes existentes de `room-view` e `view-modes`;
  - o router sem reatribuir `route` quando o caminho não muda.
- **Mocks**: cenas novas na tela `mobile` (ver [contracts/viewer.md](./contracts/viewer.md)) e um quadro de celular deitado (844×390) no `MockApp`.
- **Playwright** no scratchpad, em emulação de celular (Pixel 7 e iPhone 14, em pé e deitado):
  - sem rolagem lateral nas larguras 320, 360, 390, 430, 844 e 932;
  - áreas de toque de pelo menos 44 px;
  - abrir e fechar o visualizador pelo botão e pelo voltar.
- **Aparelhos reais**: roteiro em [quickstart.md](./quickstart.md), com `npm run dev:lan`, num Android e num iPhone. Isso cobre tela cheia do sistema, gesto de voltar, zoom e giro com som, que a emulação não prova.
