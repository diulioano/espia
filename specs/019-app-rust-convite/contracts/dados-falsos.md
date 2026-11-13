# Contrato: dados falsos no app nativo (R17)

Toda tela e todo estado do app nativo abre com dados falsos, sem servidor, câmera, captura ou rede (FR-018, constituição V). É o equivalente de `?mock=<tela>&scene=<cena>` do site (`src/mocks/registry.ts`).

## Opções

```text
Espia.exe --mock                       # índice das telas e cenas
Espia.exe --mock <tela>                # a tela, na cena padrão
Espia.exe --mock <tela> --scene <cena> # a tela, na cena pedida
```

- `--mock` sozinho abre o índice (porte de `MockIndex.svelte` e `MockSwitcher.svelte`, T027).
- Cena padrão por tela, como no `defaultScene` do site: `home` = `default`, `join` = `default`, `room` = `single` (`DEFAULT_ROOM_SCENE`), `notfound` = `quick`.
- Tela ou cena que não existe: o app abre o índice, com uma linha no alto dizendo qual nome não existe.
- Combinar `--mock` com link de sala: o mock vale e o link é ignorado.

## Telas

| Tela (`--mock`) | Arquivo do site | Cenas no site |
|---|---|---|
| `home` | `src/mocks/home.ts` | 13 |
| `join` | `src/mocks/join.ts` | 5 |
| `room` | `src/mocks/room.ts` | 47 |
| `notfound` | `src/mocks/notfound.ts` | 4 |

Total: 69 cenas portadas (13 + 5 + 47 + 4). O app é só para desktop. Por isso a tela `mobile` (`src/mocks/mobile.ts`, 15 cenas: `viewer*`, `landscape*` e as de sala no celular) e toda cena marcada `device: 'mobile'` ficam só no site: `home`/`mobile`, `join`/`mobile`, `join`/`fixedmobile` e `notfound`/`mobile`.

### Como contei

- `home.ts`: 14 itens em `HOME_SCENES`, 13 portados (`default`, `empty`, `error`, `invite`, `invitehint`, `first`, `create`, `few`, `menu`, `many`, `loading`, `gone`, `removed`). Fica de fora `mobile`.
- `join.ts`: 7 itens em `JOIN_SCENES`, 5 portados (`default`, `nopreview`, `full`, `fixed`, `replaced`). Ficam de fora `mobile` e `fixedmobile`.
- `notfound.ts`: 5 itens em `NOTFOUND_SCENES`, 4 portados (`quick`, `deleted`, `kicked`, `expired`). Fica de fora `mobile`.
- `room.ts`: 47 em `ROOM_SCENES`. São `update-required` (1), seis gerados por `.map` (`watch-available`, `watch-stopped`, `watch-opening`, `watch-playing`, `watch-unavailable`, `watch-reconnecting`) e 40 escritos um a um. O teste de paridade não pode só procurar `id:` no texto: precisa contar também os seis gerados (T033 define como).

## Regra de paridade

**Todo `id` de cena de `src/mocks/{home,join,room,notfound}.ts` que não é marcado `device: 'mobile'` existe no app**, na tela de mesmo nome: home 13, join 5, room 47, notfound 4, total 69. Os seis ids `watch-*` gerados de `room.ts` contam.

- O app pode ter cenas a mais (as novas abaixo). Nunca a menos.
- Quatro ids dessas quatro telas são marcados `device: 'mobile'` no site: `home`/`mobile`, `join`/`mobile`, `join`/`fixedmobile` e `notfound`/`mobile`. Eles ficam só no site e o teste os pula.
- O teste do app lê os arquivos do site (`src/mocks/*.ts`), extrai os ids e confere um a um (T033). Quando alguém acrescenta uma cena no site, o teste do app falha até a cena existir no app.

## O que uma cena pode conter

Uma cena é um view-model pronto (`HomeView`, `JoinView`, `RoomView`, `NotFoundView`, de `espia-core::view_model`) mais, se preciso, o estado de apresentação.

| Pode | Não pode |
|---|---|
| Textos, apelidos, nomes de sala e códigos inventados (`k7f-q2x`) | Rede: nenhuma chamada HTTP nem WebSocket |
| Imagens fixas embutidas: miniaturas de janelas, ícones, fundos | Câmera: nenhuma leitura de dispositivo |
| Transmissões como placeholders (cor e legenda, campo `caption` do `StreamView`; sem `stream`) | Captura de tela, janela ou som |
| Listas de fontes de captura e de câmeras inventadas (`CaptureSource`, `CameraView`) | Ligação de vídeo: nenhum par, nenhum quadro real |
| Avisos (`toasts`), diálogos abertos, menus abertos, estados de carregando e de erro | Ler ou gravar `device.json`, `tray.json`, `window.json` ou `resume.json` |

A camada visual (`espia-ui`) depende só de `espia-core` e não enxerga rede, captura nem mídia. Isso garante o limite pelo grafo de dependências (R1, constituição IV): uma cena não tem como fazer nenhum desses acessos.

## Cenas novas desta feature

O site não tem estas cenas. O app ganha:

| Tela | Cena | Estado | Tarefa |
|---|---|---|---|
| `home` | `update-available` | aviso "Tem versão nova", com "Atualizar agora" e "Depois" | T096 |
| `home` | `update-minimum` | versão abaixo de `minimum`: aviso sem "Depois", só "Atualizar agora" | T096 |
| `offline` | `default` | tela de sem internet com "Tentar de novo", porte de `src-tauri/www/index.html` | T030 |

Essas três cenas são além das 69 portadas. `offline` é uma tela nova, com a cena `default`. A tela de entrada em sala com a versão velha demais ("Atualize o Espia") já tem cena no site, `update-required` em `room`, e entra na paridade.

Os textos dessas cenas estão em `contracts/atualizacao.md`.

## Teste de paridade (T033)

- Onde: no próprio app, rodado com `cargo test` em `app/`.
- O que faz: lê `src/mocks/home.ts`, `join.ts`, `room.ts` e `notfound.ts`, monta a lista de ids (inclusive os seis gerados de `room.ts`) e confere que cada id existe no registro de cenas do app (`app/crates/espia-ui/src/mocks/`), na mesma tela.
- Falha quando falta um id. Não falha por id a mais.
- Conta esperada hoje: 13, 5, 47 e 4 (69). Se o número do site mudar, o teste mostra qual id falta.
- Vale também como conferência manual: `Espia.exe --mock <tela> --scene <cena>` abre cada cena sem rede.
