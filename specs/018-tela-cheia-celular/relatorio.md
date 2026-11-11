# Relatório: 018 — Tela cheia e sala responsiva no celular

**Data**: 2026-11-07 · implementação local, sem commit nem publicação.

## Gates automáticos

| Gate | Resultado |
|------|-----------|
| `npm run check` | ✅ 0 erros, 0 avisos |
| `npm test` | ✅ 30 arquivos, 273 testes. Novos: `zoom`, `layout`, `router`, `viewerStream`/`keepFullscreen`, `viewModes` inalterado |
| `npm run build` | ✅ |

## Emulação (Playwright + Edge, no scratchpad)

**Ambiente**: emulação de celular (`isMobile`, `hasTouch`) nas larguras 320×640, 360×780, 390×844, 430×932, 844×390 e 932×430. Mocks `home`, `join`, `notfound` e `mobile`, todas as cenas.

**Rolagem lateral**: nenhuma tela teve rolagem lateral.

**Layout**: toda cena `mobile` mostrou a sala de celular, inclusive deitada.

**Áreas de toque**: todo botão, link, chip e barra de volume visível tem pelo menos 44×44. A única exceção é o link de texto "Não baixou? Tente de novo" no meio de uma frase (cena `home/invitehint`). Links dentro de texto são exceção aceita (WCAG 2.5.8).

**Fluxo** (cena `watching`):

| Verificação | Resultado |
|-------------|-----------|
| O botão "Tela cheia" abre o visualizador | ✅ |
| O `<video>` do tile some e o vídeo não fica duplicado | ✅ |
| Voltar fecha e a pessoa continua na sala | ✅ |
| O botão "Sair da tela cheia" fecha | ✅ |
| O histórico fica sem a entrada do visualizador | ✅ |
| Esc fecha | ✅ |

**Zoom** (dois dedos via CDP):

| Verificação | Resultado |
|-------------|-----------|
| Pinça de 50 para 170 px | ✅ 3,4×, com o ponto entre os dedos parado |
| Página não aproxima | ✅ `visualViewport.scale` continua 1 |
| Arrastar | ✅ move a imagem |
| Toque duplo | ✅ volta a 1× |

**Computador** (1024, 1280 e 1920, com mouse):

| Verificação | Resultado |
|-------------|-----------|
| Continua a sala de computador | ✅ |
| Tela cheia | ✅ ainda é a do sistema, no tile (`document.fullscreenElement` = tile), sem visualizador |
| Rolagem lateral | ✅ nenhuma |

**Bug encontrado e corrigido**: no toque, soltar o dedo dispara `pointerleave`, e o tile escondia os controles antes do clique chegar. Com isso, o toque em "Tela cheia" (e nos outros botões do tile) caía no vídeo. Agora o `pointerleave` de toque não esconde os controles, que somem sozinhos em 3 s.

## Desvios do tasks.md

- **T008** (testes da store): o Vitest daqui roda no runtime do Worker, sem compilar runas do Svelte, então o `RoomStore` não é instanciável nos testes. A regra de fechamento virou a função pura `keepFullscreen()` em `room-view.ts`, testada, e a store só a aplica: quando a transmissão some ou reinicia, ao parar de assistir, quando o celular troca de tela e no PiP. A volta de `failed` para `ok` zera o `fullscreenId` direto na store.
- **T007** (router): a mesma limitação. `parseRoute`/`sameRoute` foram para `src/lib/route.ts` (puro, testado), e o router usa essas funções.
- **Visualizador sem botão de PiP**: o vídeo do visualizador some quando ele fecha, e com ele o PiP. O PiP continua no tile da sala, como antes.
- **T025** (`JoinPage`): o cartão de entrada continua ancorado embaixo, como no design. Com `interactive-widget=resizes-content` (Android) e a rolagem automática do iOS até o campo focado, ele fica visível. Os diálogos (`NameDialog` e outros) passaram a ficar no topo no celular. **A conferir no iPhone (passo 12)**.

## Aparelhos reais (T034) — pendente

É preciso rodar a [quickstart.md](./quickstart.md) §4 (passos 1–13) com `npm run dev:lan` num Android (Chrome) e num iPhone (Safari). A emulação não prova estes pontos:
- a tela cheia do sistema no Android;
- a camada de página inteira no iPhone;
- o gesto de voltar de verdade;
- o som sem corte ao girar;
- a legibilidade com zoom (SC-006);
- as áreas seguras reais (entalhe e ilha);
- o teclado do iPhone.

| Aparelho | Sistema / navegador | Passos aprovados | Observações |
|----------|---------------------|------------------|-------------|
| _(pendente)_ | | | |
| _(pendente)_ | | | |
