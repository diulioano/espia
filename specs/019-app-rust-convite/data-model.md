# Data Model: App nativo em Rust e convite no endereço oficial

Nada novo vai ao servidor. Os dados do aparelho ficam no `localStorage` do navegador (por endereço) e, no app nativo, em arquivos JSON em `%APPDATA%\dev.espia.app\`. Nenhuma chave de dono, apelido ou preferência viaja para o servidor (FR-005, constituição III).

O que ainda não foi confirmado fica marcado como "não confirmado" ou "a definir", com a tarefa que resolve.

## Mapa das entidades

| Entidade | Onde vive | Quem escreve | Quem lê |
|---|---|---|---|
| `device.json` | `%APPDATA%\dev.espia.app\device.json` | app nativo (`espia-shell`, regras em `espia-core`) | app nativo |
| Pacote da ida de endereço (`#mv=`) | fragmento do endereço, só em trânsito | site no endereço antigo (`src/lib/origin-move.ts`) | site no endereço oficial (`src/main.ts`) |
| `latest.json` | `GET /download/latest.json` | `scripts/release-app.mjs` (T094) | app nativo |
| `resume.json` | `%APPDATA%\dev.espia.app\resume.json` | app nativo, antes de instalar uma atualização | app nativo, na abertura seguinte |
| `tray.json` | `%APPDATA%\dev.espia.app\tray.json` | app de antes e app nativo | app de antes e app nativo |
| `window.json` | `%APPDATA%\dev.espia.app\window.json` | app nativo | app nativo |
| Estado da sala | memória do processo, `espia-core::room` | redutor, a partir de eventos | telas, por view-models |
| `ViewMessage` | canal de dados `ctl`, em trânsito | quem assiste | quem transmite |

## `device.json`

Dados do aparelho do app nativo. É o equivalente das chaves `espia:*` que o site guarda no `localStorage`.

```json
{
  "version": 1,
  "nickname": "texto de até 24, ou null",
  "golive": { "…": "mesmo conteúdo de espia:golive do site" },
  "volumes": { "<apelido>": 0.8 },
  "rooms": [ { "code": "k7f-q2x", "name": "até 40", "lastUsed": 1762860000000 } ],
  "owners": { "<código>": "<64 hex>" },
  "sessions": { "<código>": { "peerId": "<16 hex>", "resumeToken": "texto", "at": 1762860000000 } },
  "importedFromWebview": false
}
```

| Campo | Tipo | Regra |
|---|---|---|
| `version` | inteiro | `1`. Outro valor: tratado como arquivo ilegível (ver "Arquivo ausente ou ilegível"). |
| `nickname` | texto ou `null` | Até 24 caracteres (`NICKNAME_MAX`, `src/shared/protocol.ts`). `null` = nunca escolheu. O servidor limpa e recusa vazio ou acima de 24 (`cleanNickname`, `src/worker/validate.ts`). |
| `golive` | objeto | Mesmo conteúdo do `GoLivePrefs` (ver abaixo). Ausente ou inválido: `DEFAULT_GO_LIVE` = `{ resolution: 1080, fps: 30, audio: true, surface: 'window' }`. |
| `volumes` | mapa apelido → número | Chave: apelido em minúsculas (`name.toLowerCase()`, como `espia:volume:<apelido>` em `src/lib/prefs.ts`). Valor entre 0 e 1, inclusive. Fora disso, o item é ignorado. |
| `rooms` | lista | No máximo 5 (`SAVED_ROOMS_MAX`), ordenada por `lastUsed` do mais recente para o mais antigo. Detalhe abaixo. |
| `owners` | mapa código → chave | Chave de dono: exatamente 64 hex minúsculos (`/^[0-9a-f]{64}$/`). **Nunca é cortada**, nem quando a sala sai de `rooms`. Só some quando a sala deixou de existir (`forgetOwnerKey`). |
| `sessions` | mapa código → sessão | Vale 24 h (`SESSION_MAX_AGE_MS`). Detalhe abaixo. |
| `importedFromWebview` | booleano | `true` depois da importação do app de antes, ou quando a pasta do app de antes não existe. |

### `golive`

Campos de `GoLivePrefs` (`src/lib/types.ts`) e regras de leitura de `loadGoLivePrefs` (`src/lib/prefs.ts`):

| Campo | Valores | Observação |
|---|---|---|
| `resolution` | `720` ou `1080` | Obrigatório. Outro valor invalida o objeto inteiro. |
| `fps` | `30` ou `60` | Obrigatório. Outro valor invalida o objeto inteiro. |
| `audio` | booleano | Lido como `!!valor`. |
| `surface` | `window`, `monitor` ou `camera` | Qualquer outro valor vira `window`. |
| `lastSourceKey` | texto | Opcional. `"<app>|<title>"` ou `"m:<title>"`. |
| `lastVideoLabel` | texto | Opcional. |
| `lastAudioLabel` | texto | Opcional. |
| `lastAudioRole` | `voice` ou `game` | Opcional. |

### `rooms`

Cada item é uma `SavedRoom` (`src/lib/saved-rooms.ts`):

| Campo | Tipo | Regra |
|---|---|---|
| `code` | texto | Formato `xxx-xxx`: `ROOM_ID_RE = /^[a-hjkmnp-z2-9]{3}-[a-hjkmnp-z2-9]{3}$/` (`src/lib/room-id.ts`). |
| `name` | texto | Último nome conhecido. Até 40 caracteres (`ROOM_NAME_MAX`). |
| `lastUsed` | número | Milissegundos desde 1970 da última entrada neste aparelho. |

Operações (porte de `src/lib/saved-rooms.ts`): `rememberRoom` põe a sala no topo e, passando de 5, sai a usada há mais tempo; `forgetRoom` tira da lista (não tira a posse); `renameSaved` troca o nome. Tirar da lista nunca apaga `owners`.

### `sessions`

| Campo | Tipo | Regra |
|---|---|---|
| `peerId` | texto | 16 hex minúsculos (`/^[0-9a-f]{16}$/`, o mesmo que o servidor exige em `resume`). |
| `resumeToken` | texto | Entregue pelo `welcome`. O servidor exige 64 hex em `resume` (`TOKEN_RE`). |
| `at` | número | Momento da gravação. Com mais de 24 h, a sessão é descartada na leitura e na gravação seguinte (`pruneSessions`). |

Uma sessão serve para retomar o mesmo lugar na sala depois de uma queda ou de uma atualização (ver `contracts/sinalizacao-nativa.md`).

### Arquivo ausente ou ilegível

Arquivo ausente, vazio, com JSON inválido ou com `version` desconhecida: o app abre com os padrões (lista vazia, sem apelido) e grava um arquivo novo na primeira mudança. Item inválido dentro de um arquivo bom é ignorado sozinho, sem derrubar o resto. Mesmo critério do `tray.json` de hoje (`parse_prefs`).

### Quem escreve e quem lê

- Escrita: só `espia-shell`, por arquivo temporário e renomeação (escrita atômica, R9). As regras de validação, junção e corte ficam em `espia-core` (`device.rs`, T019), sem acesso a disco.
- Leitura: o app nativo, na abertura. A importação do app de antes e a junção do `#mv=` usam as mesmas regras de junção (ver abaixo).

### Estados da importação do app de antes

```text
importedFromWebview = false  (nunca importado)
  ──primeira abertura, leitura do perfil do app de antes concluída──▶  true  (importado)
  ──pasta do app de antes não existe──▶  true
  ──leitura falhou, com a pasta presente──▶  continua false; o app abre com o que tem e não diz nada (ver contracts/app-nativo.md)
true ──qualquer abertura seguinte──▶ true   (não importa de novo)
```

Com a falha, o campo fica em `false` e o app tenta de novo a cada abertura, enquanto `importedFromWebview` for `false` e a pasta existir.

### Mapeamento: chave do site → campo do `device.json`

| Chave do site (`localStorage`) | Valor no site | Campo do `device.json` | Validação |
|---|---|---|---|
| `espia:nickname` | texto | `nickname` | até 24 |
| `espia:golive` | JSON de `GoLivePrefs` | `golive` | regras de `loadGoLivePrefs` |
| `espia:volume:<apelido>` | número em texto | `volumes["<apelido>"]` | entre 0 e 1 |
| `espia:owner:<código>` | 64 hex | `owners["<código>"]` | 64 hex; código no formato `xxx-xxx` |
| `espia:rooms` | JSON, lista de `{code, name, lastUsed}` | `rooms` | até 5; `name` até 40; código `xxx-xxx` |
| `espia:session:<código>` | JSON `{peerId, resumeToken, at}` | `sessions["<código>"]` | vale 24 h |
| `espia:appinvite` | `"dismissed"` | sem campo | O app nativo não mostra o convite de baixar o app (não confirmado na spec). |

### Regras de junção

Valem para a importação do app de antes e, com o mesmo texto, para o `#mv=` no site (R12). Só acrescenta; nunca troca um valor que já existe.

| Dado | Regra |
|---|---|
| `nickname`, `golive`, `volumes`, `owners`, `sessions` | Se o destino já tem o valor, fica o do destino. Se não tem, entra o da origem. |
| `rooms` | União por código, ordenada por `lastUsed` do mais recente para o mais antigo, cortada em 5. Com o mesmo código dos dois lados, fica a entrada de maior `lastUsed`, com o nome dela. |
| `owners` | Nunca descartada, mesmo de sala que ficou fora de `rooms`. |
| Item inválido | Ignorado em silêncio. |

## Pacote da ida de endereço (`#mv=`)

Montado no site aberto pelo endereço antigo e entregue ao site no endereço oficial pelo fragmento do endereço. Contrato completo em `contracts/convite-e-endereco.md`.

```json
{ "v": 1, "items": { "<chave>": "<valor cru do localStorage>" } }
```

| Campo | Tipo | Regra |
|---|---|---|
| `v` | inteiro | `1`. Outro valor: o pacote inteiro é ignorado. |
| `items` | mapa texto → texto | Chave `espia:*`. Valor é o texto cru do `localStorage` (sem reinterpretar). |

Validação de cada item (item inválido é ignorado em silêncio):

| Item | Regra |
|---|---|
| código de sala (em `espia:rooms`, `espia:owner:*`, `espia:session:*`) | formato `xxx-xxx` de `src/lib/room-id.ts` |
| chave de dono | 64 hex |
| nome de sala | até 40 |
| apelido | até 24 |
| volume | entre 0 e 1 |

Codificação: base64url sem preenchimento `=`. Tamanho: teto de 16.384 caracteres do texto codificado. Passando, descarta nesta ordem: `espia:session:*`, `espia:volume:*`, `espia:golive`, `espia:appinvite`. `espia:rooms`, `espia:owner:*` e `espia:nickname` sempre vão. Se ainda passar, a ida acontece sem `#mv=` e nada é apagado no endereço antigo.

Escreve: `src/lib/origin-move.ts` (empacotar). Lê: o mesmo módulo (desempacotar e juntar), chamado por `src/main.ts` no endereço oficial. Vive só no endereço, durante a ida: o oficial limpa o fragmento com `history.replaceState` depois de importar.

## `latest.json`

Publicado em `GET /download/latest.json`. Contrato completo em `contracts/atualizacao.md`.

| Campo | Tipo | Regra |
|---|---|---|
| `version` | texto | Versão mais nova publicada do app nativo. Primeira versão: `0.2.0` (R15). |
| `notes` | texto | Notas da versão. Pode ser vazio. |
| `pub_date` | texto | Data e hora ISO 8601 em UTC, como no contrato da 015. |
| `minimum` | texto | Menor versão que ainda serve. Faltando ou inválido: "sem mínimo" (regra herdada da 015). |
| `url` | texto | Endereço do instalador. |
| `signature` | texto | Assinatura minisign do instalador. |

Escreve: `scripts/release-app.mjs` (T094). Lê: `espia-shell::update` (T095).

Estados do aviso de atualização:

```text
nenhuma ──latest.json com versão maior que a instalada──▶ disponível
disponível ──"Depois" (só se a versão instalada >= minimum)──▶ nenhuma
disponível ──"Atualizar agora" (e sem transmissão própria)──▶ baixando
baixando ──instalador baixado e assinatura conferida──▶ pronta
pronta ──grava resume.json e abre o instalador──▶ instalando
instalando ──o app fecha, instala, reabre──▶ nenhuma  (na versão nova)
baixando | pronta ──falha de rede, de arquivo ou de assinatura──▶ disponível
```

A volta de falha para `disponível` e o aviso mostrado vêm da 015 ("Não deu pra atualizar"); o texto do app nativo fica para a tarefa T096. Transmitindo, nada baixa nem instala (R14). "Depois" fecha o aviso até a próxima abertura do app; na abertura seguinte ele volta, porque não há memória de versão pulada. O app consulta `latest.json` na abertura e a cada 6 horas enquanto aberto.

## `resume.json`

```json
{ "room": "<código>", "hidden": false, "at": 1762860000000 }
```

| Campo | Tipo | Regra |
|---|---|---|
| `room` | texto | Código no formato `xxx-xxx`. Inválido: o arquivo é ignorado. |
| `hidden` | booleano | `true` = reabrir escondido na bandeja. |
| `at` | número | Milissegundos desde 1970 da gravação. |

Escreve: o app nativo, antes de instalar. Lê: o app nativo, na abertura seguinte. Como na 015 (`update-resume.json`), o arquivo vale 5 minutos e é apagado ao ser lido. Mais velho que isso, é ignorado. Não guarda apelido: o apelido está em `device.json`.

## `tray.json`

Formato de hoje, reaproveitado sem mudança (`src-tauri/src/tray.rs`, `TrayPrefs`):

```json
{ "closeOnX": false, "trayNoticeShown": false }
```

| Campo | Tipo | Regra |
|---|---|---|
| `closeOnX` | booleano | `false` (padrão): o X esconde a janela na bandeja. `true`: o X fecha o app. |
| `trayNoticeShown` | booleano | O aviso da primeira vez ("O Espia continua aberto") já apareceu. |

Leitura (`parse_prefs`): arquivo ausente, ilegível ou JSON que não é objeto vira o padrão (os dois campos `false`). Campo que não é booleano vira `false`. Campos desconhecidos são ignorados. Escreve: o app de antes e o app nativo, na mesma pasta, e o app nativo reaproveita o arquivo na primeira abertura (T085). Lê: os dois.

## `window.json`

Posição e tamanho da janela do app nativo (R9).

```json
{ "x": 100, "y": 80, "width": 1280, "height": 800, "maximized": false }
```

| Campo | Tipo | Regra |
|---|---|---|
| `x`, `y` | inteiro | Posição da janela. Se cair fora de todos os monitores, a janela abre centralizada. |
| `width`, `height` | inteiro | Tamanho. Mínimo 720×480. Padrão 1280×800, centralizada. |
| `maximized` | booleano | `true` = abrir maximizada. |

Arquivo ausente ou ilegível: padrão. O app de antes guarda a posição com `tauri-plugin-window-state` (`src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`), em `%APPDATA%\dev.espia.app\.window-state.json`. O formato desse arquivo não foi confirmado; se não der para ler, vale o padrão. A conversão na primeira abertura é a tarefa T085.

## Estado da sala (`espia-core::room`)

O módulo `espia-core::room` é um redutor puro (R4): recebe eventos (mensagens do servidor, eventos de ligação, ações da pessoa, relógio) e devolve estado e efeitos. É o porte de `src/lib/stores/room.svelte.ts` e `src/lib/room-view.ts`. O binário executa os efeitos. As telas só enxergam os view-models de `src/lib/types.ts`, portados para `espia-core::view_model` (T018).

Campos de `RoomView` agrupados por finalidade:

| Grupo | Campos | Para que serve |
|---|---|---|
| Identidade da sala | `roomId`, `inviteUrl`, `room` (`{kind:'quick'}` ou `{kind:'fixed', name, own}`), `capacity` | Código, convite (sempre no endereço oficial, ver `contracts/convite-e-endereco.md`), tipo da sala e lotação. |
| Ligação com o servidor | `status` (`connecting` ou `ready`), `link` (`ok`, `reconnecting`, `failed`), `requiresUpdate` | Queda não é sair. `failed` vem depois de 60 s sem voltar (`RECONNECT_FAIL_MS`). `requiresUpdate` é o fechamento 4426. |
| Pessoas | `participants` (`ParticipantView`: `id`, `name`, `tone`, `you`, `live`, `liveLabel`, `removable`) | Lista, você primeiro. |
| Transmissões dos outros | `streams` (`StreamView`: `id`, `name`, `tone`, `quality`, `status`, `muted`, `volume`, `label`, `source`, `watching`, `watchState`, `you`), `focusedId` | O palco. `status` é `ok`, `loading` ou `failed`. |
| Minha transmissão | `myBroadcast` (`MyBroadcastView`: prefs escolhidas, `startedAt`, `adjusted`, `viewers`, `audioKind`, `audioReason`, `sourceTitle`, `label`, `source`, `onStage`), `canShare`, `canShareScreen` | O que sai, com qual qualidade e para quantos. `stream` e `snapshot` são tipos do navegador; no app nativo o equivalente é uma referência da camada de mídia, a definir na tarefa T018. |
| Seletor de transmissão | `goLiveOpen`, `goLivePrefs`, `picker` (`mode`, `sources`, `loading`, `selectedId`), `camera` (`permission`, `devices`, `audios`, `videoId`, `audioId`, `audioRole`, `facing`, `error`), `permissionDenied` | Janela de Transmitir, lista de telas e janelas, câmeras e entradas de som. |
| Convite e avisos | `copied`, `toasts` (`ToastView`: `id`, `kind`, `text`, `action`) | "Convite copiado" e avisos da sala. |
| Sala fixa e dono | `ownerMenu`, `nameDialog`, `deleteDialog`, `removeDialog` | Renomear, apagar e tirar pessoa. |
| Visualizador | `fullscreenId`, `mockViewer` | Tela cheia. `mockViewer` é só dos dados falsos de celular e não existe no app nativo (as cenas de celular ficam só no site). |

### Estado de assistir por transmissão

`StreamView.watchState` e a regra do Assistir (FR-021): sem clique, nada é recebido.

```text
available ──Assistir──▶ opening ──primeiro quadro──▶ playing
opening | playing ──ligação falhou ou passou do tempo──▶ unavailable
opening | playing | unavailable ──Parar de assistir──▶ stopped
stopped ──Assistir──▶ opening
qualquer ──a transmissão termina ou recomeça (startedAt novo)──▶ available
```

Origem no código de hoje: `watchState` em `src/lib/stores/room.svelte.ts` (`watching`, `stopped`, falha por `conn === 'failed'` ou tempo) e `src/lib/watch-state.ts`. As cenas `watch-available`, `watch-stopped`, `watch-opening`, `watch-playing`, `watch-unavailable` e `watch-reconnecting` de `src/mocks/room.ts` cobrem o que a tela mostra.

Modo de vista que quem assiste informa a quem transmite (`ViewMode`, `src/lib/quality.ts`): `full`, `thumb`, `hidden`, `off`. Sem clique em Assistir é `off`.

## `ViewMessage`

Mensagem de controle pelo canal de dados `ctl` entre duas pessoas (`src/lib/peer.ts`, `encodeView` e `decodeView`). Quem assiste manda para quem transmite como está vendo a transmissão.

```ts
interface ViewMessage { t: 'view'; version: 2; transmission: { peerId: string; startedAt: number }; revision: number; mode: 'full' | 'thumb' | 'hidden' | 'off' }
```

| Campo | Tipo | Regra de `decodeView` |
|---|---|---|
| `t` | `'view'` | Exatamente `view`. |
| `version` | `2` | Exatamente `2`. |
| `transmission.peerId` | texto | 16 hex minúsculos (`/^[0-9a-f]{16}$/`). |
| `transmission.startedAt` | número | Inteiro seguro e maior ou igual a 0 (`isSafeSequence`). |
| `revision` | número | Inteiro seguro e maior ou igual a 0. |
| `mode` | texto | `full`, `thumb`, `hidden` ou `off`. |

Qualquer outra forma é recusada (`null`). Campos extras são descartados na saída.

Regras de quem transmite ao receber (`PeerLink.setupCtl`): a mensagem só vale se `transmission` é a transmissão atual (`sameTransmission`) e `revision` é maior que a última aceita. Sem mensagem válida, o modo é `off` e nenhuma faixa é enviada.

Pedido de reenvio: `{ "t": "view-sync", "version": 2, "transmission": { … } }`, mandado por quem transmite. Quem assiste responde com a `view` atual se `transmission` for a que ele está vendo.

Em quem assiste, a `revision` começa em 0 e sobe 1 a cada envio (`++viewRevision`). Quando a transmissão vista muda (`setRemoteKey`), o modo volta a `off` e a nova `view` sai com a `revision` seguinte.

Escreve: quem assiste. Lê: quem transmite. O Worker não vê esta mensagem (vai pela ligação direta).
