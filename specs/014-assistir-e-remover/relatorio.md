# Relatório — 014

## T1401 · Spike: o app minimizado/na bandeja deixa a página "hidden"? (2026-10-31)

App de debug (`cargo build`), site local (`npm run dev:worker`). A janela é mexida por Win32 (`ShowWindow`, `WM_CLOSE` = o X). A página é lida por CDP **puro** (scratchpad `spike-raw.mjs`).

| Estado da janela | Sem correção (0.1.7) | Com `SetIsVisible` (0.1.8) |
|---|---|---|
| visível | `visible` | `visible` |
| minimizada | `visible` ❌ | `hidden` ✅ |
| restaurada | `visible` | `visible` |
| X → bandeja | `visible` ❌ | `hidden` ✅ |

**Conclusão**: o WebView2 nunca avisava a página. Minimizado ou na bandeja, o app continuava recebendo e desenhando as telas dos outros, o que provavelmente responde por parte do FPS perdido. → **T1416 feita** (app 0.1.8).

**Armadilha de teste**: com o Playwright conectado ao app por `connectOverCDP`, a página **sempre** diz `visible` (mesmo com `Emulation.setFocusEmulationEnabled(false)`). Para medir a visibilidade é preciso desconectar o Playwright e ler por CDP puro. Outra armadilha: ao desconectar, o Playwright desfaz as permissões que deu (`grantPermissions`), e a câmera morre. Então a permissão é dada por uma conexão CDP à parte (`Browser.grantPermissions`).

## T1416 · `SetIsVisible` no `engine.rs`

- `page_visible(window)` = só `Visible`. Vale também ao vivo: o envio não depende do desenho.
- `focus()` (toda volta da janela: bandeja, atalho/link pela instância única, confirmação de sair) passa a marcar `Visible`. Antes, só o "Abrir" da bandeja e a instância única marcavam.

## E2E (scratchpad, site local + app de debug, nada da tela capturado)

**Assistir** (`assistir.mjs`, site × site): ver tasks.md, nota da US1.

**App assistindo** (`app-assiste.mjs`): Ana (Edge, câmera falsa) transmite e o app assiste.

| | Vídeo de Ana pro app | Som | Ana vê |
|---|---|---|---|
| app visível | 41 quadros/2 s | 100 pacotes/2 s | 1 assistindo |
| minimizado | 0 | 101 | 0 assistindo |
| na bandeja | 0 | 101 | 0 assistindo |
| de volta | em ~230 ms | | |

**App transmitindo escondido** (`app-transmite.mjs`): este era o risco do R2. O WebView2 com `IsVisible=false` podia frear o envio.

| Modo | Visível | Minimizado (página `hidden`) | Bandeja (página `hidden`) |
|---|---|---|---|
| câmera falsa | 60 quadros/3 s · 151 pacotes de som | 60 · 150 | 60 · 150 |
| captura nativa (janela de teste, ~10 fps) | 37 · 151 | 32 · 150 | 30 · 150 |

O envio não cai com a página escondida.

## Falta (manual, com o grupo)

- **M2**: FPS no jogo com o app 0.1.8 minimizado, outra pessoa transmitindo (antes × depois).
- **M3**: som pela bandeja por 5 min sem cortes, com um som real (não o tom de teste).
- **M4**: picture-in-picture com o Espia minimizado (headless não abre PiP).
- **M5**: segundo monitor: o foco no jogo não esconde nada.
