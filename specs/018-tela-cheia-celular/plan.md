# Implementation Plan: Tela cheia e sala responsiva no celular

**Branch**: `018-tela-cheia-celular` (trabalho direto na `main`, como nas features anteriores) | **Date**: 2026-11-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/018-tela-cheia-celular/spec.md`

## Summary

1. **Visualizador próprio (R2)**: `Viewer.svelte` é uma camada fixa sobre a sala, com imagem inteira, controles de toque, zoom e saída.
   - Onde o aparelho deixa (Android, iPad), a camada também vira tela cheia do sistema.
   - No iPhone ela ocupa a página toda, porque o Safari do iPhone não põe em tela cheia nada além do player nativo, e o player nativo não controla o nosso som.
   - O estado `fullscreenId` mora na store e a regra `viewerStream()` em `room-view.ts`. Abrir não mexe em assistir nem em `viewModes()`, então a recepção não muda.
2. **Voltar fecha (R3)**: uma entrada no histórico, no mesmo endereço, enquanto o visualizador está aberto. O router deixa de reatribuir `route` quando o caminho não muda.
3. **Zoom (R4)**: Pointer Events com contas puras em `src/lib/zoom.ts`, de 1× a 4×, com toque duplo para voltar e sem biblioteca.
4. **Sala de celular deitada (R1, R6)**: a consulta passa a ser `(max-width: 639px), (pointer: coarse) and (hover: none) and (max-height: 540px)`. A `MobileRoomPage` ganha um layout deitado de duas colunas.
5. **Acabamento de toque (R7, R8)**:
   - `viewport-fit=cover` e áreas seguras em todas as páginas;
   - `dvh` e diálogos que convivem com o teclado;
   - botões de pelo menos 44 px no toque;
   - sem barra de volume no iPhone, onde ela não funciona.

## Technical Context

**Language/Version**: TypeScript 6, Svelte 5 (runes). Sem mudanças no Worker nem no app Tauri.

**Primary Dependencies**: as de hoje (`@lucide/svelte`). Nenhuma nova.

**Storage**: nada novo (ver [data-model.md](./data-model.md)).

**Testing**:
- Vitest para `zoom.ts`, `layout.ts`, `viewerStream`, `fullscreenId` na store e o router;
- Playwright no scratchpad com emulação de Pixel 7 e iPhone 14;
- roteiro em aparelhos reais em [quickstart.md](./quickstart.md).

**Target Platform**: o site, assistido em Chrome Android e Safari iPhone (duas versões principais mais recentes). O computador e o app desktop ficam como estão.

**Project Type**: SPA + Worker (+ app Tauri), no mesmo repositório. Esta feature mexe só na SPA.

**Performance Goals**:
- girar ou entrar e sair da tela cheia reorganiza em ≤ 1 s, sem corte de som (SC-003);
- o gesto de zoom acompanha o dedo sem refazer layout, só com `transform`.

**Constraints**:
- nenhuma nova recepção ou decodificação ao abrir o visualizador; o `<video>` do tile some enquanto ele está aberto;
- sem travar a orientação;
- sem user agent para decidir o layout.

**Scale/Scope**:
- 1 componente novo (`Viewer`) e 3 módulos puros (`zoom`, `layout`, `media-caps`);
- ajustes em `StreamTile`, `MobileRoomPage`, `RoomRoute`, `router`, store, tipos, páginas de entrada e `index.html`;
- 8 cenas de mock novas.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Situação |
|-----------|----------|
| I. Simplicidade | ✅ Nada de instalação, conta ou passo novo para entrar. Tela cheia, giro e zoom não contam como Assistir e não iniciam recepção (R5). Textos em português informal. |
| II. Sem servidor de mídia | ✅ Nada muda no transporte. O visualizador reaproveita o `MediaStream` já recebido. |
| III. Nada sobre as pessoas | ✅ Nada é guardado. `fullscreenId`, zoom e capacidade de volume vivem só na memória. |
| IV. Camada visual separada | ✅ `fullscreenId` e `viewerStream()` ficam em `src/lib`. O `Viewer` recebe estado pronto e emite eventos. As contas de zoom e layout são funções puras. Cores e raios vêm de `tokens.css`. |
| V. Telas mockáveis | ✅ 8 cenas novas, inclusive um quadro de celular deitado (contrato). `RoomView.mockViewer` só existe nos mocks. |
| VI. Segredos fora do navegador | ✅ Não se aplica. |
| Restrições técnicas | ✅ Sem dependência nova nem API nativa nova. O app desktop não muda. |

**Re-check pós-design**: ✅ sem violações, e a Complexity Tracking fica vazia.

O único ajuste é na spec, não na constituição: o FR-006 deixou de pedir a tela cheia nativa do vídeo (R2).

## Project Structure

### Documentation (this feature)

```text
specs/018-tela-cheia-celular/
├── plan.md              # este arquivo
├── research.md          # R1–R9
├── data-model.md        # fullscreenId, ZoomState, layout, volume
├── quickstart.md        # gates, mocks, emulação, aparelhos reais
├── contracts/
│   └── viewer.md        # Viewer, StreamTile, MobileRoomPage, cenas de mock
├── checklists/
│   └── requirements.md
└── tasks.md             # /speckit-tasks
```

### Source Code (repository root)

```text
index.html                         # viewport-fit=cover, interactive-widget
src/
├── lib/
│   ├── zoom.ts                    # novo: pinch, pan, clampZoom, isDoubleTap
│   ├── layout.ts                  # novo: PHONE_LAYOUT_QUERY, isPhoneLayout
│   ├── media-caps.ts              # novo: canSetVolume
│   ├── room-view.ts               # + viewerStream
│   ├── router.svelte.ts           # popstate sem reatribuir route no mesmo caminho
│   ├── types.ts                   # RoomView.fullscreenId, mockViewer; RoomActions.fullscreen
│   └── stores/room.svelte.ts      # fullscreenId: ação, fechamentos (some, watch false, pip)
├── routes/RoomRoute.svelte        # usa PHONE_LAYOUT_QUERY
├── ui/
│   ├── components/
│   │   ├── Viewer.svelte          # novo
│   │   ├── StreamTile.svelte      # onfullscreen, videoHidden, volumeAdjustable, toque 44 px
│   │   ├── NameDialog.svelte      # teclado no celular
│   │   └── UpdateRequired.svelte  # 100dvh
│   └── pages/
│       ├── MobileRoomPage.svelte  # Viewer, layout deitado, áreas seguras
│       ├── HomePage.svelte        # áreas seguras, dvh
│       ├── JoinPage.svelte        # áreas seguras, teclado
│       └── NotFoundPage.svelte    # áreas seguras
└── mocks/
    ├── mobile.ts                  # 8 cenas novas
    ├── MockRoom.svelte            # actions.fullscreen no mock
    └── MockApp.svelte             # quadro 844×390 para device 'mobile-landscape'
test/lib/
├── zoom.test.ts                   # novo
├── layout.test.ts                 # novo
├── room-view.test.ts              # + viewerStream
├── view-modes.test.ts             # abrir o visualizador não muda viewModes
└── router.test.ts                 # novo (mesmo caminho mantém route)
```

**Structure Decision**: o repositório já é SPA + Worker. Tudo desta feature fica em `src/lib` (regras puras e store) e `src/ui` (telas), seguindo a constituição IV.

**Ordem de entrega** (constituição: mock → real):
1. mocks e layout deitado;
2. `Viewer` com mock;
3. ligar à store e ao histórico;
4. zoom;
5. acabamento de toque e áreas seguras;
6. aparelhos reais.

## Complexity Tracking

Sem violações.
