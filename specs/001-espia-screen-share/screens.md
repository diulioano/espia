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

## Salas fixas e queda (specs/011, Espia.dc.html §7)

| # | Tela | `mock` | `scene` |
|---|---|---|---|
| F1 | Sem salas fixas · convite | `home` | `first` |
| F2 | Suas salas · 3 salas | `home` | `few` |
| F3 | Menu do item (dono) | `home` | `menu` |
| F4 | Suas salas agrupadas (até 5) | `home` | `many` |
| F5 | Carregando quem está em cada sala | `home` | `loading` |
| F6 | Sala que deixou de existir | `home` | `gone` |
| F7 | Modal Criar sala fixa | `home` | `create` |
| F8 | Entrar por link · sala fixa | `join` | `fixed` (`fixedmobile` no celular) |
| S1 | Recém-criada · link copiado | `room` | `created` (vazia do dono: `fixed`) |
| S2 | Menu do dono | `room` | `ownermenu` |
| S3 | Renomear | `room` | `rename` |
| S4 | Apagar · com gente na sala | `room` | `delete` |
| S5 | Quem não é dono · sem menu | `room` | `guest` |
| Q1 | Reconectando… (você ao vivo) | `room` | `reconnecting` |
| Q2 | De volta! | `room` | `back` |
| Q3 | Não deu pra voltar | `room` | `reconnectfail` |
| N1 | Apagada pelo dono | `notfound` | `deleted` (`&saved=1`) |
| N2 | Apagada com você dentro | `notfound` | `kicked` (`&saved=1`) |
| N3 | Expirada (30 dias) | `notfound` | `expired` (`&saved=1`) |
| N4 | Sala rápida que acabou | `notfound` | `quick` |
| M4 | Início com Suas salas (celular) | `home` | `mobile` |
| M5 | Assistindo · reconectando (celular) | `mobile` | `reconnecting` |

A antiga cena `join&scene=notfound` (E2) virou `notfound&scene=quick`.

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
| `SavedRoom`, `SavedRoomsPanel`, `RoomMenu` | F2–F6, M4, S2 (specs/011) |
| `NameDialog`, `DeleteRoomDialog` | F7, S3, S4 (specs/011) |
| `ReconnectPill`, `ReconnectFail` | Q1, Q3, M5 (specs/011) |
| `NotFoundPage` | N1–N4 (specs/011) |
| `Button`, `TextField`, `Segmented`, `Switch` | todas |

## Ordem de construção

1. **Sala mockada** (`?mock=room`, todas as cenas) — concentra quase todos os componentes. ← primeira entrega
2. Página inicial e Entrar/Não encontrada/Cheia mockadas.
3. Celular mockado.
4. Lógica real atrás das mesmas telas (roteador, Worker, DO, WebRTC).
