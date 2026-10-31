# Quickstart — validar a 014

## Automático

```bash
npm run check && npm test && npm run build
```

- `room-view.test.ts`: `viewModes()` com todas as combinações do R5 (não assistida, página escondida, PiP, celular, foco/grade).
- Store: `unwatched` apagado quando o `startedAt` muda e no `peer-left`, e mantido numa queda.
- Worker (`room.test.ts`): `removePerson` com chave certa/errada, sala rápida, conectado, fantasma e inexistente. O removido recebe `removed` + 4403, os outros recebem `peer-left` e o `resume` dele falha.

## Mocks (`npm run dev`)

| URL | Deve mostrar |
|---|---|
| `?mock=room&scene=unwatched` | grade com uma tela em cartão "Assistir" |
| `?mock=room&scene=unwatched-all` | palco só com cartões |
| `?mock=room&scene=self-stage` | própria tela grande ("Você"), prévia da barra marcada |
| `?mock=room&scene=live` | prévia "Você está ao vivo" como imagem parada |
| `?mock=room&scene=owner-remove` | lista com "Remover da sala" ao lado das pessoas |
| `?mock=room&scene=remove-confirm` | confirmação "Tirar Lu da sala?" |
| `?mock=home&scene=removed` | tela inicial com o aviso "O dono te tirou da sala" |
| `?mock=mobile&scene=unwatched` | celular com o cartão "Assistir" |

## E2E local (scratchpad, Playwright + Edge, sem `--use-fake-ui-for-media-stream`)

1. **US1**: A transmite (aba de teste via `--auto-select-tab-capture-source-by-title`) e B assiste. B clica em "Parar de assistir". Em até 3 s, o `framesSent` de A para B para de subir, o `framesDecoded` de B para, e o áudio de B continua recebendo pacotes. O `viewers` de A cai para 0. B clica em "Assistir" e o `framesDecoded` volta a subir em até 3 s.
2. **US2 (site)**: B abre outra aba por cima (a aba da sala fica `hidden`). A vê o envio para B pausar. B volta, e o envio retoma.
3. **US3**: com A transmitindo, a prévia é um `<canvas>` (nenhum `<video>` na barra) e troca de conteúdo no máximo a cada ~5 s. Clicando nela, aparece um `<video>` "Você" no palco. Esc remove o vídeo.
4. **US4**: A cria uma sala fixa (é dono), B entra, A remove B. B cai no início com o aviso e não reconecta. A lista de A não mostra mais B. B abre o link e entra de novo.

## Manual (precisa de captura real e do app)

- **M1**: spike do R2. No app instalado, com a depuração remota ligada (`ESPIA_BROWSER_ARGS`), minimizar e mandar pra bandeja, e ler o `document.visibilityState` pelo CDP. Anotar o resultado em `relatorio.md`.
- **M2**: jogo + transmissão. Medir o FPS do jogo com o Espia **minimizado** (antes: versão de hoje; depois: 014), com outra pessoa transmitindo ao mesmo tempo (SC-001).
- **M3**: app na bandeja por 5 min, assistindo um amigo. O som continua sem cortes e, ao abrir, a tela volta em até 3 s.
- **M4**: picture-in-picture de uma tela e Espia minimizado. O PiP continua com imagem.
- **M5**: segundo monitor. O Espia à vista e o foco no jogo: as telas continuam aparecendo (FR-007).
