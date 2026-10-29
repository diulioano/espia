# Telas — Espia

Inventário das telas do handoff (`design_handoff_espia/`), com rota real, parâmetro de mock e
estados. Todo estado listado MUST abrir via `?mock=<tela>&scene=<cena>` (Constituição, princípio V).

> Abrir `/?mock` sem valor (ou com valor desconhecido) mostra o índice de todas as telas e cenas.

## Desktop

| # | Tela | Rota real | `mock` | `scene` | Ref. design |
|---|---|---|---|---|---|
| D1 | Página inicial | `/` | `home` | `default`, `empty` (sem apelido salvo), `error` (link inválido) | README §D1 |
| D2 | Entrar por link | `/s/:código` | `join` | `default`, `nopreview` (sem pilha de avatares) | README §D2 |
| E2 | Sala não encontrada | `/s/:código` | `join` | `notfound` | README §E2 |
| — | Sala cheia (não desenhada) | `/s/:código` | `join` | `full` | layout do E2 |
| M1 | Entrar no celular | `/s/:código` | `join` | `mobile` (abre num quadro 390×844) | README §M1 |
| D3–D7 | Sala | `/s/:id` | `room` | ver abaixo | `Room.dc.html` |

### Cenas da Sala (`?mock=room&scene=…`)

| scene | Código | O que mostra |
|---|---|---|
| `connecting` | E1 | Spinner "Entrando na sala…", skeleton na sidebar, Go Live desabilitado |
| `empty` | D3 | "Ninguém ao vivo ainda." + Go Live grande |
| `copied` | D3 | Igual ao vazio, botão "Copiado!" + toast `copied` |
| `modal` | D6 | Modal Go Live aberto sobre a sala vazia |
| `denied` | E3 | Alerta "O navegador não deixou compartilhar" |
| `single` *(padrão)* | D4 | Lu ao vivo no palco cheio, controles visíveis, toast `join` |
| `grid` | D5 | 3 transmissões em grade 2×2 (uma carregando) + célula de dica, toast `live` |
| `focus` | D5b | Bia em foco + faixa de miniaturas, toast `leave` |
| `streamfail` | E4 | Bia em foco com `status=failed` |
| `live` | D7 | Você ao vivo (card na sidebar, Parar transmissão, contador), Lu no palco, toast `live` |
| `crowd` | D5+ | Limite: 10 pessoas, 6 ao vivo em grade 3×2, qualidade "ajustado" (não desenhado; extrapola D5) |

## Celular (390×844) — só assistir

| # | Tela | `mock` | `scene` | Ref. |
|---|---|---|---|---|
| M1 | Entrar | `join` | `mobile` | README §M1 (mesma JoinPage, responsiva) |
| M2 | Assistindo | `mobile` | `watching` | README §M2 (mesmo `RoomView` da sala) |
| — | Conectando / falha | `mobile` | `connecting`, `streamfail` | E1/E4 adaptados |
| M3 | Ninguém ao vivo | `mobile` | `empty` | README §M3 |

## Componentes por tela

| Componente | Usado em |
|---|---|
| `Logo` | D1, D2, E2, Sala |
| `LiveBadge` (sm/md) | D2, Participant, StreamTile, M1/M2 |
| `Avatar` / `AvatarStack` | D2, M1, Participant |
| `Participant` | Sala (sidebar), M2 |
| `StreamTile` (stage/grid/thumb · ok/loading/failed) | D1 (ilustração), Sala, M2 |
| `Stage` (vazio/único/grade/foco/conectando) | Sala, M2/M3 |
| `Toast` + `ToastStack` | Sala, M3 |
| `GoLiveModal` | D6 |
| `PermissionAlert` | E3 |
| `Button`, `TextField`, `Segmented`, `Switch` | todas |

## Ordem de construção

1. **Sala mockada** (`?mock=room`, todas as cenas) — concentra quase todos os componentes. ← primeira entrega
2. Página inicial e Entrar/Não encontrada/Cheia mockadas.
3. Celular mockado.
4. Lógica real atrás das mesmas telas (roteador, Worker, DO, WebRTC).
