# Data Model: Fechar o app sem sair (bandeja)

## TrayPrefs (arquivo `tray.json`, no PC)

| Campo | Tipo | Padrão | Regra |
|---|---|---|---|
| `closeOnX` | bool | `false` | `false`: o X esconde. `true`: o X fecha (com confirmação se ao vivo). FR-707 |
| `trayNoticeShown` | bool | `false` | Vira `true` depois do primeiro aviso. Nunca volta. FR-703 |

Arquivo ausente, ilegível ou com campos estranhos → padrão (campos desconhecidos são ignorados).

## LiveState (memória do app)

`live: bool`, começa `false`.

| Evento | Novo valor |
|---|---|
| página chama `app_set_live(true)` | `true` |
| página chama `app_set_live(false)` | `false` |
| página recarrega / navega (`PageLoadEvent::Started`) | `false` |

Ao mudar: troca ícone (`tray.png` ↔ `tray-live.png`), dica (`Espia` ↔ `Espia · ao vivo`) e mostra/esconde o item "Parar transmissão".

## Decisões (funções puras)

`on_close(prefs, live) -> CloseAction`:

| `closeOnX` | `live` | Resultado |
|---|---|---|
| false | qualquer | `Hide` (+ aviso se `!trayNoticeShown`) |
| true | false | `Exit` |
| true | true | `ConfirmExit` |

`on_quit(live)`: `live` → `ConfirmExit`; senão `Exit`.

## Menu do ícone

| Id | Texto | Quando aparece | Ação |
|---|---|---|---|
| `open` | Abrir o Espia | sempre | `focus()` |
| `stop` | Parar transmissão | só `live` | `eval("window.espiaStopLive?.()")` |
| — | separador | sempre | — |
| `close-on-x` | Fechar no X | sempre (marcável) | inverte `closeOnX`, grava `tray.json` |
| — | separador | sempre | — |
| `quit` | Sair | sempre | `on_quit` |

## Textos

| Onde | Texto |
|---|---|
| Dica do ícone | `Espia` / `Espia · ao vivo` |
| Aviso (título) | O Espia continua aberto |
| Aviso (corpo) | Ele fica aqui perto do relógio (às vezes na setinha ^). Pra sair de vez, clique com o botão direito no ícone e escolha Sair. |
| Confirmação | Você está transmitindo. Sair mesmo assim? (Sim / Não, padrão Não) |
