# Quickstart: validar a 012

## Testes automáticos

- `test/lib/live-label.test.ts`: todas as linhas da tabela de `data-model.md`.
- `test/worker/room.test.ts` (bloco "Descrição da transmissão (specs/012)"):
  - `live` com `label: 'Hollow Knight'` → os outros recebem `peer-live.live.label`; quem entra depois recebe no `welcome`;
  - `label` com U+202E/controle → normalizado; com 61+ caracteres ou não-string → `live` aceito **sem** `label`;
  - `live` sem `label` (cliente antigo) continua funcionando;
  - prévia (`GET /api/rooms/:code`) e `status` não contêm o `label`.

## Mocks

- `?mock=room&scene=grid` · `focus` · `single` · `live` · `livecam` · `crowd` (um `label` comprido, cortado) — chip do tile, miniaturas, lista "Na sala" e cartão "Você está ao vivo".
- `?mock=mobile&scene=watching` — chips "Ao vivo agora" e lista.
- Uma transmissão sem `label` (versão antiga) em `grid`.

## Ponta a ponta (Worker local, 2 janelas do Edge)

Captura real não é automatizada (regra do projeto: nunca capturar a tela real nos testes). Roteiro manual:
1. Site: transmitir uma janela → quem assiste vê "Janela"; trocar pra "compartilhar esta aba" no meio → muda pra "Aba do navegador".
2. App 0.1.7: transmitir a janela de um jogo → "nome do jogo"; tela inteira com 1 monitor → "Tela inteira".
3. Câmera → "Câmera · <nome>".
4. Conferir que a prévia da entrada e Suas salas não mostram nada disso.
