---

description: "Task list for 018 — Tela cheia e sala responsiva no celular"
---

# Tasks: Tela cheia e sala responsiva no celular

**Input**: Design documents from `/specs/018-tela-cheia-celular/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/viewer.md](./contracts/viewer.md), [quickstart.md](./quickstart.md)

**Tests**: o plano pede Vitest para as regras puras (zoom, layout, `viewerStream`, store, router). Os testes vêm antes da implementação de cada parte.

**Ordem da constituição**: em cada história, primeiro o mock, depois a lógica real. **Gate de cada tarefa**: `npm run check` e `npm run build` sem erros, e o mock da tela abrindo todos os estados.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo, porque mexe em arquivo diferente e não depende de tarefa pendente.
- **[Story]**: US1 (tela cheia), US2 (sala responsiva), US3 (zoom).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: base de viewport e mocks que as três histórias usam.

- [X] T001 Em `index.html`, trocar a meta viewport por `width=device-width, initial-scale=1, viewport-fit=cover, interactive-widget=resizes-content` (research R7).
- [X] T002 [P] Em `src/mocks/registry.ts` e `src/mocks/mobile.ts`, aceitar `device: 'mobile-landscape'` nas cenas. Em `src/mocks/MockApp.svelte`, desenhar o iframe do quadro em 844×390 quando `scene.device === 'mobile-landscape'` (390×844 continua para `'mobile'`) e tratar `'mobile-landscape'` como `mobile` ao montar `MockRoom`. Usar `title="Celular deitado 844×390"`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: tipos e estado que US1 e US3 usam. US2 não depende desta fase.

- [X] T003 Em `src/lib/types.ts`:
  - adicionar a `RoomView` o campo `fullscreenId: string | null` e o campo opcional `mockViewer?: { controls: boolean; zoom: ZoomState }`, comentado como "só mocks; a store real nunca preenche";
  - adicionar a `RoomActions` a ação `fullscreen(id: string | null): void`;
  - importar `ZoomState` de `src/lib/zoom.ts`, criando nesse arquivo, por enquanto, só `export interface ZoomState { scale: number; x: number; y: number }`, `export interface Box { width: number; height: number }`, `export const ZOOM_NONE: ZoomState = { scale: 1, x: 0, y: 0 }` e `export const MAX_ZOOM = 4`.
- [X] T004 Em `src/mocks/room.ts`, `src/mocks/mobile.ts` e em qualquer outro construtor de `RoomView` que o `npm run check` apontar, preencher `fullscreenId: null`. Em `src/mocks/MockRoom.svelte`, implementar `actions.fullscreen(id)`: aceita `id` só se a transmissão existe com `watching === true`, e `null` fecha. Também zerar `fullscreenId` em `watch(id, false)` e em `pip(id)` não nulo.

**Checkpoint**: `npm run check` verde, com `fullscreenId` presente em todo `RoomView`.

---

## Phase 3: User Story 1 - Ver a transmissão em tela cheia no celular (Priority: P1) 🎯 MVP

**Goal**: tocar em tela cheia abre um visualizador que cobre a tela toda, com a imagem inteira. Funciona em pé e deitado, e sai pelo botão ou pelo voltar sem parar de assistir.

**Independent Test**: [quickstart.md](./quickstart.md) §2 (cenas `viewer*`) e §4, passos 1–3 e 6–10, num Android e num iPhone.

### Tests for User Story 1

- [X] T005 [P] [US1] Em `test/lib/room-view.test.ts`, testar `viewerStream(view)`. Ela devolve a transmissão de `fullscreenId` só quando ela existe, tem `watching === true` e `view.link !== 'failed'`. Devolve `undefined` com `fullscreenId: null`, com id inexistente, com `watching === false` e com `link: 'failed'`. Com `link: 'reconnecting'`, a transmissão continua sendo devolvida.
- [X] T006 [P] [US1] Em `test/lib/view-modes.test.ts`, testar que `viewModes()` dá o mesmo resultado com e sem visualizador aberto. A entrada não tem `fullscreenId`, e o teste documenta, com um comentário citando FR-005, que abrir o visualizador não pode mudar a recepção.
- [X] T007 [P] [US1] Criar `test/lib/router.test.ts`, com `history`/`location` do jsdom ou um stub mínimo. Com um `popstate` de mesmo `pathname`, `router.route` deve ser o **mesmo objeto** e `router.state` deve ser atualizado. Com um `popstate` para outro caminho, `route` deve mudar.
- [X] T008 [P] [US1] Em `test/lib/peer-lifecycle.test.ts`, ou num novo `test/lib/fullscreen-store.test.ts` se a store for instanciável lá, testar o `RoomStore`:
  - `actions.fullscreen(id)` recusa uma transmissão não assistida;
  - `fullscreenId` volta a `null` quando a transmissão some ou reinicia com `startedAt` novo;
  - `fullscreenId` volta a `null` em `watch(id, false)` e ao abrir o PiP (`pip(id)`);
  - `fullscreenId` volta a `null` quando o `link` sai de `failed` para `ok`.

### Implementation for User Story 1

- [X] T009 [US1] Em `src/lib/room-view.ts`, implementar `export function viewerStream(view: RoomView): StreamView | undefined` conforme o [data-model.md](./data-model.md). Faz o T005 passar.
- [X] T010 [US1] Em `src/lib/router.svelte.ts`, no `popstate`, só reatribuir `this.route` quando `parse(location.pathname)` difere da rota atual (mesmo `name` e mesmo `code`). `this.state` é sempre atualizado. Faz o T007 passar.
- [X] T011 [US1] Em `src/lib/stores/room.svelte.ts`:
  - criar `fullscreenId = $state<string | null>(null)`;
  - ação `fullscreen: (id) => { if (id === null || this.peers.some(p => p.id === id && p.live) && this.watching.has(id)) this.fullscreenId = id; }`;
  - zerar `fullscreenId` no mesmo ponto em que o `pipId` é zerado quando a transmissão some (perto da linha 313 de hoje), em `watch(id, false)` e em `pip(id)` não nulo;
  - zerar `fullscreenId` também quando `link` passa de `failed` para `ok`;
  - expor `fullscreenId` no `RoomView` montado perto da linha 1190.

  Faz o T008 passar.
- [X] T012 [P] [US1] Criar `src/lib/media-caps.ts` com `export function canSetVolume(): boolean`. Ele cria um `<audio>` solto, escreve `volume = 0.5`, devolve `audio.volume === 0.5` e guarda o resultado em memória (um teste só por página). Fora do navegador (sem `document`), devolve `true`.
- [X] T013 [US1] Criar `src/ui/components/Viewer.svelte` conforme [contracts/viewer.md](./contracts/viewer.md), com as props `view`, `link`, `volumeAdjustable`, `initial?`, `onclose`, `ontogglemute`, `onvolume` e `onpip`. Nesta tarefa, **sem zoom**: o zoom entra no T030.
  - camada `position: fixed; inset: 0` com fundo `var(--color-stage)` e `<video autoplay playsinline muted>` com `object-fit: contain` recebendo `view.stream`;
  - controles na base, com mudo, volume (só se `volumeAdjustable`), PiP (se `document.pictureInPictureEnabled`) e "Sair da tela cheia" (`Minimize` do Lucide), com botões de 48 px dentro de `env(safe-area-inset-*)`;
  - os controles aparecem a cada toque e somem 3 s depois, ou ficam visíveis se `initial.controls`;
  - nome da pessoa e descrição da transmissão no topo, junto com os controles;
  - `ReconnectPill compact` quando `link === 'reconnecting'`;
  - `Esc` chama `onclose`;
  - cores só de `src/ui/tokens.css`.
- [X] T014 [US1] Em `Viewer.svelte`, cuidar da tela cheia do sistema e do voltar (research R3), garantindo que `onclose` seja chamado **uma vez só**:
  - ao montar, fazer `history.pushState({ ...history.state, viewer: true }, '')`;
  - ouvir `popstate` e, se `history.state?.viewer` não existir mais, chamar `onclose`;
  - ouvir `fullscreenchange`/`webkitfullscreenchange` e, quando `document.fullscreenElement` deixar de ser a camada depois de ter sido, chamar `onclose`;
  - ao desmontar, se `history.state?.viewer` ainda existir, chamar `history.back()` e, se a camada ainda estiver em tela cheia, `document.exitFullscreen()`;
  - expor `export function requestSystemFullscreen()`, que chama `el.requestFullscreen?.() ?? el.webkitRequestFullscreen?.()` com `.catch(() => {})` e fica no modo `page` se falhar.
- [X] T015 [US1] Em `src/ui/components/StreamTile.svelte`:
  - nova prop `onfullscreen?: () => void`; quando presente, o botão "Tela cheia" chama a prop em vez de `container.requestFullscreen()`, e sem ela o computador fica como hoje;
  - nova prop `videoHidden?: boolean`, que não monta o `<video>` enquanto for `true` e mantém o `<audio>`;
  - nova prop `volumeAdjustable?: boolean = true`; com `false`, esconde o `.slider`.
- [X] T016 [US1] Em `src/ui/pages/MobileRoomPage.svelte`:
  - `const viewer = $derived(viewerStream(view))`;
  - passar ao `StreamTile` do palco `onfullscreen={openViewer}`, `videoHidden={!!viewer}` e `volumeAdjustable={canSetVolume()}`;
  - `openViewer` chama `actions.fullscreen(selected.id)` e, no mesmo gesto, depois do `await tick()`, chama `requestSystemFullscreen()` da instância do `Viewer` (via `bind:this`);
  - renderizar `{#if viewer}<Viewer view={viewer} link={view.link} volumeAdjustable={canSetVolume()} initial={view.mockViewer} onclose={() => actions.fullscreen(null)} ontogglemute={() => actions.toggleMute(viewer.id)} onvolume={(v) => actions.setVolume(viewer.id, v)} onpip={...} />{/if}`;
  - `onpip` abre o PiP no `<video>` do visualizador, chama `actions.pip(viewer.id)` e fecha o visualizador.
- [X] T017 [US1] Em `src/mocks/mobile.ts`, adicionar as cenas `viewer` (M·F1), `viewer-controls` (M·F2), `viewer-reconnecting` (M·F4), `viewer-nosound` (M·F5) e `viewer-landscape` (M·L2, `device: 'mobile-landscape'`).
  - todas partem da cena `watching`, com `fullscreenId = 'lu'`;
  - `mockViewer = { controls, zoom: ZOOM_NONE }`, com `controls` em `true` nas cenas `-controls` e `-nosound`;
  - `link = 'reconnecting'` em M·F4.

  Para M·F5 ser mockável, acrescentar em `mockViewer` o campo opcional `volume?: boolean` (também em `src/lib/types.ts`). `MobileRoomPage` passa `volumeAdjustable={view.mockViewer?.volume ?? canSetVolume()}`, e a cena M·F5 usa `volume: false`. A store real não muda.
- [X] T018 [US1] Conferir a [quickstart.md](./quickstart.md) §2, linhas `viewer`, `viewer-controls`, `viewer-reconnecting`, `viewer-nosound` e `viewer-landscape`, e rodar `npm run check`, `npm test` e `npm run build`.

**Checkpoint**: US1 funciona sozinha. A tela cheia abre e fecha, inclusive pelo voltar, e fecha quando a transmissão acaba ou a pessoa para de assistir.

---

## Phase 4: User Story 2 - Sala que se adapta ao celular em pé e deitado (Priority: P1)

**Goal**: a versão de celular aparece em pé e deitado. Deitado, o palco fica com a altura toda. Nada de rolagem lateral, nada escondido sob a ilha ou o entalhe, botões de pelo menos 44 px e campos visíveis com o teclado aberto.

**Independent Test**: [quickstart.md](./quickstart.md) §2 (cenas `landscape*`), §3 (emulação) e §4, passos 9 e 11–13.

### Tests for User Story 2

- [X] T019 [P] [US2] Criar `test/lib/layout.test.ts` para `isPhoneLayout({ width, height, coarse, hover })`:
  - 390×844 com toque dá `true`;
  - 844×390 com toque e sem hover dá `true`;
  - 932×430 com toque dá `true`;
  - 1024×768 com toque dá `false` (tablet deitado);
  - 1280×500 com mouse dá `false` (notebook baixo);
  - 600×900 com mouse dá `true` (janela estreita).

### Implementation for User Story 2

- [X] T020 [P] [US2] Criar `src/lib/layout.ts` com `export const PHONE_LAYOUT_QUERY = '(max-width: 639px), (pointer: coarse) and (hover: none) and (max-height: 540px)'` e `export function isPhoneLayout(i)`, com a mesma regra em código. Faz o T019 passar.
- [X] T021 [US2] Em `src/routes/RoomRoute.svelte`, trocar `matchMedia('(max-width: 639px)')` por `matchMedia(PHONE_LAYOUT_QUERY)`. O `store.mobile` continua seguindo `mobile`.
- [X] T022 [US2] Em `src/ui/pages/MobileRoomPage.svelte`, layout deitado com `@media (orientation: landscape)` (research R6):
  - `.mobile-room` vira uma grade de duas colunas, `minmax(0, 1fr)` e `clamp(280px, 34vw, 340px)`;
  - o palco fica à esquerda com `height: 100%` e `aspect-ratio: auto`, com margens `max(12px, env(safe-area-inset-left))`, topo e base;
  - a coluna da direita rola, com cabeçalho, chips, pessoas e a `.bar` (Transmitir/Parar câmera/Sair) no fim, que não pode ficar fora da tela;
  - `padding-right: env(safe-area-inset-right)`.

  Em pé, o layout fica como hoje, só com `max-height: 60dvh` no palco. Nenhuma mudança de marcação pode remontar o `StreamTile` no giro: só CSS.
- [X] T023 [US2] Em `src/ui/pages/MobileRoomPage.svelte`, áreas de toque e seguras:
  - `.owner-btn` com 44×44;
  - `.scroll` com `padding-left/right: env(safe-area-inset-left/right)`;
  - `.bar` com `padding-bottom: max(16px, env(safe-area-inset-bottom))`, sem o mínimo fixo de 30 px;
  - toasts acima da barra também no deitado (`.toasts` dentro da coluna direita, ou `bottom` calculado).
- [X] T024 [P] [US2] Em `src/ui/components/StreamTile.svelte`, `@media (pointer: coarse)`: `.ctl` com `--c: 44px` em todos os tamanhos e `.slider` com área de toque de 44 px de altura. Nos dispositivos de toque, `wake()` deixa os controles visíveis por 3 s em vez de 2 s: guardar `e.pointerType === 'touch'` no `onpointerdown`.
- [X] T025 [P] [US2] Em `src/ui/pages/HomePage.svelte`, `src/ui/pages/JoinPage.svelte` e `src/ui/pages/NotFoundPage.svelte`, no bloco `@media (max-width: 639px)` e num novo bloco `(pointer: coarse) and (max-height: 540px)`:
  - padding com `env(safe-area-inset-*)`;
  - `100vh` trocado por `100dvh`;
  - nada de largura fixa maior que `100%`.

  Na `JoinPage`, o campo de apelido e o botão Entrar ficam no topo da área visível no celular, com `align-items: start`, para o teclado não cobrir.
- [X] T026 [P] [US2] Em `src/ui/components/NameDialog.svelte`, `src/ui/components/DeleteRoomDialog.svelte`, `src/ui/components/RemoveDialog.svelte` e `src/ui/components/GoLiveModal.svelte`:
  - `max-height: calc(100dvh - 32px)` com rolagem interna;
  - no celular (`max-width: 639px` ou `(pointer: coarse) and (max-height: 540px)`), diálogo alinhado ao topo com margem `max(16px, env(safe-area-inset-top))`;
  - botões de pelo menos 44 px.

  Em `src/ui/components/UpdateRequired.svelte`, trocar `100vh` por `100dvh`.
- [X] T027 [US2] Em `src/mocks/mobile.ts`, adicionar `landscape` (M·L1: base `watching`, 2 ao vivo) e `landscape-camera` (M·L3: base `camera` com `goLiveOpen = false` e `myBroadcast` preenchido, como uma câmera no ar), as duas com `device: 'mobile-landscape'`.
- [X] T028 [US2] Emulação com o Playwright no scratchpad (não no repositório), conforme a [quickstart.md](./quickstart.md) §3:
  - Pixel 7 e iPhone 14, em pé e deitado, nas larguras 320, 360, 390, 430, 844 e 932;
  - mocks `home`, `join`, `mobile` (todas as cenas) e `notfound`;
  - verificar `scrollWidth <= innerWidth` e que todo controle visível tem pelo menos 44×44;
  - corrigir o que falhar nos arquivos das T022–T026.

**Checkpoint**: US2 funciona sozinha. A sala de celular aparece em pé e deitado, sem rolagem lateral, com tudo alcançável.

---

## Phase 5: User Story 3 - Aproximar detalhes da tela transmitida (Priority: P2)

**Goal**: na tela cheia, aproximar com dois dedos até 4×, arrastar dentro das bordas e voltar à imagem inteira com toque duplo. A página não se move e a imagem continua ao vivo.

**Independent Test**: [quickstart.md](./quickstart.md) §2 (`viewer-zoom`) e §4, passos 4–5. Depende da US1, que traz o `Viewer`.

### Tests for User Story 3

- [X] T029 [P] [US3] Criar `test/lib/zoom.test.ts`, com `box = { width: 400, height: 800 }`:
  - `clampZoom` mantém `1 ≤ scale ≤ MAX_ZOOM`, com `MAX_ZOOM = 4`;
  - `clampZoom` mantém `box.width·(1 − scale) ≤ x ≤ 0` e o mesmo para `y` com `height`;
  - `scale === 1` dá `x = y = 0`;
  - `pinch` com os dedos se afastando de 100 para 200 px dobra a escala e mantém o ponto médio fixo;
  - `pinch` não passa de 4×;
  - `pan` com `scale === 1` não move;
  - `pan` não passa das bordas;
  - `isDoubleTap` vale com 250 ms e 10 px, e não vale com 350 ms ou 30 px.

### Implementation for User Story 3

- [X] T030 [US3] Em `src/lib/zoom.ts`, implementar `clampZoom(state, box)`, `pinch(state, a0, b0, a1, b1, box)`, `pan(state, dx, dy, box)` e `isDoubleTap(prev: { t: number; x: number; y: number }, now)`, seguindo as regras citadas no T029 e no [data-model.md](./data-model.md). Faz o T029 passar.
- [X] T031 [US3] Em `src/ui/components/Viewer.svelte`, ligar o zoom:
  - o vídeo fica num invólucro com `transform: translate(x px, y px) scale(s)` e `transform-origin: 0 0`;
  - `touch-action: none` vale só na camada;
  - Pointer Events seguem até dois ponteiros: com dois, `pinch`; com um e `scale > 1`, `pan`; um toque sem arrastar mostra os controles;
  - `gesturestart`/`gesturechange` com `preventDefault`, para o Safari do iOS não aproximar a página;
  - toque duplo (`isDoubleTap`), `resize` da janela e troca de `view.id` voltam a `ZOOM_NONE`;
  - estado inicial vindo de `initial.zoom`;
  - `will-change: transform` só durante o gesto.
- [X] T032 [US3] Em `src/mocks/mobile.ts`, adicionar a cena `viewer-zoom` (M·F3): base `viewer` com `mockViewer.zoom = { scale: 2.5, x: -300, y: -600 }`, já passando por `clampZoom` para a caixa 390×844.

**Checkpoint**: zoom funciona na tela cheia em Android e iPhone sem mover a página.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T033 Rodar `npm run check`, `npm test` e `npm run build` e conferir todas as cenas novas da [quickstart.md](./quickstart.md) §2.
- [ ] T034 ⏳ (pendente: aparelhos reais; ver relatorio.md) Rodar a [quickstart.md](./quickstart.md) §4 (passos 1–13) com `npm run dev:lan` num Android (Chrome) e num iPhone (Safari) reais. Registrar modelo, sistema, navegador e resultado de cada passo em `specs/018-tela-cheia-celular/relatorio.md`. Passo que não puder ser feito fica pendente, nunca aprovado por emulação.
- [X] T035 [P] Em `DEFINICOES.md`, registrar:
  - a sala de celular também deitada (nova regra de layout);
  - a tela cheia própria no celular, com o motivo de não usar o player nativo do iPhone;
  - a falta de volume ajustável no iPhone.

  No `README.md`, citar a tela cheia e o zoom no celular, se o README lista recursos.
- [X] T036 [P] Conferir no computador (Edge) que a sala de computador continua igual: tela cheia do tile pelo `requestFullscreen`, controles e grade. No app desktop, que nada mudou com a meta viewport nova.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Fase 1)**: sem dependências.
- **Foundational (Fase 2)**: depende do T001–T002 só para os mocks. Bloqueia US1 e US3.
- **US1 (Fase 3)**: depende da Fase 2.
- **US2 (Fase 4)**: depende só da Fase 1. Pode correr em paralelo com a Fase 2 e com a US1.
- **US3 (Fase 5)**: depende da US1 (T013, T014, T016).
- **Polish (Fase 6)**: depois das histórias desejadas.

### User Story Dependencies

- **US1**: independente de US2 e US3.
- **US2**: independente. O layout deitado vale com ou sem o visualizador.
- **US3**: precisa do `Viewer` da US1.

### Within Each User Story

- Os testes vêm antes da implementação e devem falhar antes dela.
- Primeiro o mock, depois a lógica real (constituição).
- US1: T009 → T011 → T015 → T016 → T017.
- T013 e T014 mexem no mesmo arquivo e são sequenciais.
- T022 e T023 mexem no mesmo arquivo e são sequenciais.

### Parallel Opportunities

- T002 em paralelo com T001.
- US1: T005, T006, T007, T008 e T012 em paralelo.
- US2: T019, T020, T024, T025 e T026 em paralelo, cada um num arquivo.
- US2 inteira em paralelo com a Fase 2 e a US1, com cuidado: T016 e T022/T023 tocam o mesmo `MobileRoomPage.svelte`, então devem ser feitas em sequência.
- T035 e T036 em paralelo.

---

## Parallel Example: User Story 1

```text
Task: "T005 viewerStream em test/lib/room-view.test.ts"
Task: "T006 viewModes inalterado em test/lib/view-modes.test.ts"
Task: "T007 router mesmo caminho em test/lib/router.test.ts"
Task: "T012 canSetVolume em src/lib/media-caps.ts"
```

## Parallel Example: User Story 2

```text
Task: "T019 + T020 isPhoneLayout em test/lib/layout.test.ts e src/lib/layout.ts"
Task: "T024 toque 44 px em src/ui/components/StreamTile.svelte"
Task: "T025 áreas seguras em HomePage/JoinPage/NotFoundPage"
Task: "T026 diálogos e teclado em NameDialog/DeleteRoomDialog/RemoveDialog/GoLiveModal"
```

---

## Implementation Strategy

### MVP First (User Story 1 + T020/T021)

1. Fases 1 e 2.
2. US1 inteira.
3. T020 e T021 da US2, para o celular deitado não cair na sala de computador ao sair da tela cheia.
4. **Parar e validar**: §4, passos 1–3 e 6–10, num Android e num iPhone.

### Incremental Delivery

1. Setup e Foundational.
2. US1, que é o MVP: dá para assistir em tela cheia.
3. US2: sala deitada e acabamento de toque.
4. US3: zoom.
5. Polish, com o relatório dos aparelhos reais.

---

## Notes

- Nenhuma tarefa mexe no Worker, no transporte nem no app Tauri.
- Abrir, fechar ou aproximar nunca altera `watching`, `stopped`, `focusedId` nem `viewModes()` (FR-005).
- Textos novos em português informal ("Sair da tela cheia", "Sem som"/"Com som").
- Fazer commit depois de cada tarefa ou grupo lógico, só quando pedido.
