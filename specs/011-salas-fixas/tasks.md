# Tasks: Salas fixas e chamada que sobrevive a quedas

**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/](./contracts/), [quickstart.md](./quickstart.md)

**Testes**: o plano pede testes do Worker (Vitest + `@cloudflare/vitest-plugin`, inclusive `evictDurableObject` para a publicação) e testes puros de `src/lib`/`src/shared`. As telas são conferidas pelos mocks (princípio V) e pelo roteiro do quickstart.

**Design**: Claude Design, projeto `dc89098d-c77d-4d62-9ecf-8d4335dffa7d`: `Home.dc.html`, `SavedRoom.dc.html`, `Room.dc.html`, `NotFound.dc.html` e `Espia.dc.html` (F8, M4, M5). Textos, tamanhos e cores vêm de lá. "Go Live" continua "Transmitir".

**Publicação**: um `npm run deploy` só (site + Worker). A primeira publicação ainda derruba as salas abertas (research R11). A pessoa usuária aceitou em 2026-10-30.

## Phase 1: Setup

- [X] T1101 Emenda **2.0.0** em `.specify/memory/constitution.md`:
  - o princípio III passa a ser "Sem contas e nada sobre as pessoas", com o texto exato do `plan.md` ("Emenda proposta — 2.0.0");
  - subir a versão e a data (2026-10-30);
  - linha nova no Sync Impact Report: "1.5.0 → 2.0.0 (MAJOR: III redefinido — sala fixa guarda nome/último uso/hash da chave no servidor; Suas salas e chaves de dono no navegador) — feature 011, pedido de 2026-10-29/30".
- [X] T1102 [P] Atualizar `DEFINICOES.md`:
  - linha "Vida da sala": rápida = some quando todos saem, ou 5 min depois de todos caírem; fixa = some depois de 30 dias sem uso ou quando o dono apaga;
  - linha nova "Sala fixa / Suas salas" (máx. 5 por aparelho, dono = chave no aparelho);
  - incluir `specs/011-salas-fixas/` na lista do Spec Kit.
- [X] T1103 [P] Atualizar `specs/001-espia-screen-share/security.md`:
  - dados guardados: o storage do DO agora tem `roster` (some quando a sala esvazia), `fixed` (nome, `lastUsed`, `ownerHash`) e `tombstone` (só motivo e prazo);
  - ameaça nova "roubo da chave do dono": quem pega o `localStorage` pode renomear ou apagar. Impacto baixo, porque a sala expira e não há dados de pessoas. A mitigação é o servidor guardar só o SHA-256;
  - `status` nunca devolve apelidos.
- [X] T1104 `wrangler.jsonc`:
  - `vars.GRACE_MS` "20000" → "60000";
  - var nova `HOLD_MS` "300000", com comentário "sala rápida segurada depois que todos caíram (specs/011 R2)";
  - var nova `FIXED_TTL_MS` "2592000000" (30 dias), usada também para a lápide;
  - depois rodar `npm run types` e conferir `worker-configuration.d.ts`.
- [X] T1105 `vitest.config.ts`: nos bindings do miniflare, `HOLD_MS: '800'` e `FIXED_TTL_MS: '1500'`. `GRACE_MS` e `RESERVE_MS` continuam curtos.

## Phase 2: Foundational

- [X] T1106 `src/shared/protocol.ts`:
  - `export const ROOM_NAME_MAX = 40` e `export const SAVED_ROOMS_MAX = 5`;
  - `export type RoomInfo = { kind: 'quick' } | { kind: 'fixed'; name: string }`;
  - `welcome` ganha `room: RoomInfo`;
  - `ServerMessage` ganha `{ t: 'room-renamed'; name: string }` e `{ t: 'room-deleted' }`;
  - `CLOSE.deleted: 4410` (comentário: "o dono apagou a sala");
  - `RoomPreview` ganha `fixed?: { name: string }`;
  - tipos `RoomStatus` (exatamente como em `contracts/http-api-delta.md`) e `GoneReason = 'deleted' | 'expired'`.
- [X] T1107 [P] Criar `src/shared/room-name.ts`, com `normalizeRoomName(raw: string): string | null` (research R10), na ordem:
  - `raw.normalize('NFC')`;
  - qualquer `\s` vira espaço, espaços repetidos viram um, `trim`;
  - recusa (null) se tiver `\p{Cc}` ou `\p{Cf}`;
  - recusa se `[...s].length` for 0 ou maior que `ROOM_NAME_MAX` (40).
- [X] T1108 [P] `test/lib/room-name.test.ts`:
  - "  CS   da galera " → "CS da galera";
  - "" e "   " → null;
  - 40 emojis passam, 41 não;
  - "a‮b", "a​b" e "a\u0007b" → null;
  - tab e quebra de linha viram espaço.

**Checkpoint**: `npm run check` passa (o cliente ainda não usa os campos novos; `room` no welcome pode ser tratado como opcional no store até a US2).

## Phase 3: User Story 1 — Queda não é sair (P1) 🎯 MVP

**Goal**: publicação ou queda não tiram ninguém da sala. A tela mostra "Reconectando…", "De volta!" e a falha depois de 60 s.

**Independent Test**: quickstart §1 (casos de publicação/hold) e §3 passos 3–4.

### Servidor

- [X] T1109 [US1] `src/worker/room.ts`, guardar a lista no storage (research R1):
  - chave `roster: Record<string, Member>`;
  - helper `saveMember(member)`, que grava `roster[id] = member`, e `dropMember(id)`;
  - chamar `saveMember` no `join` (depois de `serializeAttachment`), no `resume` (os dois caminhos), no `live`/`unlive` e no `grantTurn` (quando muda `turn`);
  - chamar `dropMember` na saída 4001 e quando o fantasma vence no `alarm`;
  - `disposeIfEmpty` apaga também `roster` e `holdUntil`;
  - atualizar o comentário de cabeçalho ("onde fica cada coisa").
- [X] T1110 [US1] `src/worker/room.ts`, recuperar depois de reiniciar:
  - no construtor, `ctx.blockConcurrencyWhile(() => this.recover())`;
  - `recover()` lê `roster` e `ghosts`. Para cada `id` do roster sem socket com `joined` e sem fantasma, cria `ghosts[id] = { member, until: Date.now() + graceMs }`;
  - se criou algum e não há nenhum socket com `joined`, chama `holdIfQuick()` (T1111);
  - grava e chama `scheduleAlarm()`. Se o roster estiver vazio, não faz nada (custo zero ao acordar de hibernação comum).
- [X] T1111 [US1] `src/worker/room.ts`, segurar a sala rápida (research R2):
  - `private get holdMs()` (`Number(this.env.HOLD_MS) || 300_000`);
  - `holdIfQuick()`: se não existe `fixed`, grava `holdUntil = Date.now() + holdMs`;
  - em `gone()`, no caminho de queda (não 4001): depois de criar o fantasma, se `members().length === 0`, chama `holdIfQuick()`;
  - `exists()` considera `holdUntil > agora`. `scheduleAlarm()` inclui `holdUntil`;
  - `alarm()` apaga `holdUntil` vencido antes do `disposeIfEmpty(true)`;
  - um `join` ou `resume` com sucesso apaga `holdUntil`.
- [X] T1112 [US1] Testes em `test/worker/room.test.ts`, bloco `describe('Publicação e queda (specs/011)')`:
  - (a) 3 pessoas → `evictDurableObject(stub, { webSockets: 'close' })` → cada uma reconecta com `resume` → `welcome` com o mesmo `you.id`, e nenhuma recebe `peer-left`/`peer-joined`;
  - (b) publicação e ninguém volta → prévia 200 até `HOLD_MS`, depois 404;
  - (c) duas caem (fechar sem 4001) e uma volta com `join` depois de `GRACE_MS` e antes de `HOLD_MS` → entra na mesma sala;
  - (d) as duas saem com 4001 → 404 na hora;
  - (e) os testes antigos de tolerância continuam passando.
  - Ajustar o helper que pega o stub, se precisar (`env.ROOM.get(env.ROOM.idFromName(code))`).

### Cliente

- [X] T1113 [US1] `src/lib/signaling.ts`:
  - `retryNow()` público: zera `attempt`, limpa `retryTimer` e chama `abandon()` (mesma lógica do `onOnline`), ignorado se `done` ou já aberto;
  - o `onStatus('open')` depois de uma queda passa a ser distinguível. Novo callback opcional `onBack?()`, chamado no `welcome` quando o status anterior era `reconnecting`.
- [X] T1114 [US1] `src/lib/types.ts`: `RoomView` ganha `link: 'ok' | 'reconnecting' | 'failed'` (data-model.md).
- [X] T1115 [US1] `src/lib/stores/room.svelte.ts`:
  - `link = $state<'ok' | 'reconnecting' | 'failed'>('ok')` e `private downTimer`;
  - `onStatus('reconnecting')` com `connected` → `link = 'reconnecting'` e `downTimer = setTimeout(() => link = 'failed', 60_000)`. Constante `RECONNECT_FAIL_MS = 60_000`;
  - `onBack` → limpa o timer, `link = 'ok'` e toast `{ kind: 'copied', text: 'De volta!' }`;
  - **remover** o toast de erro "A conexão com a sala caiu. Reconectando…" e o `reconnectToast`;
  - ação nova `retryConnection()` → `signaling.retryNow()` e `link = 'reconnecting'`, recomeçando o timer de 60 s;
  - `buildView()` passa `link`;
  - abrir o Transmitir com `link !== 'ok'` não faz nada (FR-007);
  - limpar o timer no `destroy()`.
- [X] T1116 [P] [US1] Criar `src/ui/components/ReconnectPill.svelte` (Room.dc.html, `isReconnecting`):
  - pílula centrada no topo do palco (`top:20px`), `pointer-events:none`;
  - `Mascot mood="loading" size=32`, "Reconectando…" (fonte de título, 17px) e "Você continua na sala" (13px, 60%);
  - prop `compact` para o celular (M5): mascote 26, só "Reconectando…", embaixo do palco (`bottom:12px`).
- [X] T1117 [P] [US1] Criar `src/ui/components/ReconnectFail.svelte` (Room.dc.html, `isReconnectFail`):
  - camada `rgba(18,17,15,0.78)` sobre o palco e cartão de 520px;
  - `Mascot mood="sad" size=84`, "Não deu pra voltar pra sala." e "A conexão caiu faz 1 minuto e não voltou. Confira sua internet e tente de novo — a sala continua lá.";
  - botões "Tentar de novo" (ícone `RefreshCw`, `btn-primary`) e "Sair da sala" (`btn-secondary`), que emitem `onretry`/`onleave`.
- [X] T1118 [US1] `src/ui/pages/RoomPage.svelte`, com `view.link`:
  - `reconnecting` ou `failed` → `ReconnectPill` no palco (só quando `reconnecting`) e lista "Na sala" com `opacity: 0.55`;
  - cartão "Você está ao vivo": `liveNote` = "Sua transmissão continua · volta sozinha";
  - pílula do rodapé: "Ao vivo · reconectando…";
  - "Transmitir" `disabled`;
  - `failed` → `ReconnectFail` (`onretry` = `actions.retryConnection`, `onleave` = `actions.leave`);
  - adicionar `retryConnection` em `RoomActions` (`src/lib/types.ts`).
- [X] T1119 [US1] `src/ui/pages/MobileRoomPage.svelte` (M5): `ReconnectPill compact` sobre o palco, lista com `opacity:0.55`, "Transmitir" desabilitado e `ReconnectFail` em tela cheia quando `failed`.
- [X] T1120 [US1] Mocks:
  - `src/mocks/room.ts`: cenas `reconnecting` (Q1: base `live`, `link:'reconnecting'`), `back` (Q2: base `live` + toast `{kind:'copied', text:'De volta!'}`) e `reconnectfail` (Q3: base `live`, `link:'failed'`). As cenas existentes ganham `link: 'ok'`;
  - `src/mocks/mobile.ts`: cena `reconnecting` (M5: base `watching`, `link:'reconnecting'`);
  - conferir as 4 em `?mock=room&scene=…` e `?mock=mobile&scene=reconnecting` contra o design.

**Checkpoint US1**: `npm test`, `npm run check` e `npm run build` passam. Quickstart §3 passos 3–4 com `npm run dev:worker`. **Pode ir pro ar sozinha** (resolve a queda de 29/10).

## Phase 4: User Story 2 — Criar uma sala fixa e entrar (P2)

**Goal**: "Criar sala fixa" com nome, link que continua valendo vazio, selo na entrada e nome no cabeçalho. Expira em 30 dias.

**Independent Test**: quickstart §1 (criar, vazia, expirada) e §3 passos 1–2.

### Servidor

- [X] T1121 [US2] `src/worker/room.ts`, sala fixa no storage (research R3), `interface FixedRoom { name; ownerHash; createdAt; lastUsed }`:
  - `reserve()` vira `reserve(fixed?: { name: string; ownerHash: string })`. Recusa (false) se `exists()` **ou** se houver `tombstone`. Com `fixed`, grava `fixed = { ...fixed, createdAt: agora, lastUsed: agora }` além do `reservedUntil`;
  - `exists()`: false se houver `tombstone`; true se houver `fixed`;
  - `join()` com sucesso: se houver `fixed`, `lastUsed = agora` (`resume` não mexe);
  - `welcome` manda `room: fixed ? { kind: 'fixed', name } : { kind: 'quick' }`;
  - `holdIfQuick()` não faz nada em sala fixa;
  - `disposeIfEmpty()` nunca apaga `fixed`/`tombstone`. Numa sala fixa vazia, ele só limpa `roster`, `ghosts`, `holdUntil`, `reservedUntil` e `joins`, e retorna false.
- [X] T1122 [US2] `src/worker/room.ts`, expiração e lápide (research R5):
  - `private get fixedTtlMs()` (`FIXED_TTL_MS`, padrão 30 dias);
  - `scheduleAlarm()` inclui `fixed.lastUsed + fixedTtlMs` quando não há ninguém (sem membros, fantasmas nem reserva) e `tombstone.until`;
  - `alarm()`: se for sala fixa, sem ninguém e com `lastUsed + ttl <= agora` → `bury('expired')`. Se `tombstone.until <= agora`, apaga a lápide;
  - `bury(reason)`: apaga `fixed`, `roster`, `ghosts`, `holdUntil`, `reservedUntil` e `joins`, grava `tombstone = { reason, until: agora + fixedTtlMs }` e marca o alarme.
- [X] T1123 [US2] `src/worker/room.ts` `preview()`: devolve `{ gone: reason }` se houver lápide (o Worker responde 410), `fixed: { name }` se for sala fixa, e o resto igual. Mudar o tipo de retorno para `RoomPreview | { gone: GoneReason } | null`.
- [X] T1124 [US2] `src/worker/validate.ts`: `parseCreateBody(text: string)` → `{ kind: 'quick' } | { kind: 'fixed'; name: string } | null`. Corpo vazio = `quick`. JSON inválido, `kind` desconhecido ou nome que não passa no `normalizeRoomName` → null. Corpo com mais de 1 KB → null.
- [X] T1125 [US2] `src/worker/index.ts` `POST /api/rooms`:
  - lê o corpo com `parseCreateBody` (null → `400 { error: 'name' }`);
  - `quick` → igual a hoje;
  - `fixed` → `ownerKey = randomHex(32)` (CSPRNG, função local igual à do `room.ts`) e `ownerHash = sha256Hex(ownerKey)` (`crypto.subtle.digest`);
  - `reserve({ name, ownerHash })` com as mesmas 5 tentativas → `201 { code, ownerKey }`;
  - `GET /api/rooms/:code`: `{ gone }` → `410 { reason }`.
- [X] T1126 [US2] Testes em `test/worker/room.test.ts`, bloco `describe('Sala fixa (specs/011)')`:
  - criar com nome válido → 201 com `ownerKey` de 64 hex;
  - nome vazio, de 41 caracteres ou com U+202E → 400;
  - corpo vazio continua criando sala rápida;
  - fixa: entrar, sair (4001) e esperar `GRACE_MS + HOLD_MS` → prévia 200 com `fixed.name`;
  - `welcome` traz `room.kind === 'fixed'`;
  - avançar até `FIXED_TTL_MS` sem `join` (`runDurableObjectAlarm`) → prévia `410 { reason: 'expired' }` e `ws` com `room-not-found`;
  - `reserve` num código com lápide → false;
  - depois de mais `FIXED_TTL_MS` → 404.

### Cliente

- [X] T1127 [US2] `src/lib/api.ts`:
  - `createFixedRoom(name)` → `{ ok: true; code; ownerKey } | { ok: false; reason: 'rate-limited' | 'name' | 'network' }` (POST com `Content-Type: application/json`);
  - `fetchPreview` trata `410` → `{ status: 'gone'; reason: GoneReason }` e devolve `fixedName` quando vier `fixed`.
- [X] T1128 [P] [US2] `src/lib/prefs.ts`:
  - `loadOwnerKey(code)`, `saveOwnerKey(code, key)` e `forgetOwnerKey(code)` com a chave `espia:owner:<code>` (`contracts/local-storage.md`), validando o valor como 64 hex;
  - `KEY.owner`.
- [X] T1129 [US2] `src/lib/types.ts`:
  - `NameDialogView { mode: 'create' | 'rename'; value: string; busy: boolean; error: string | null }`;
  - `HomeView += { fixedDialog?: NameDialogView }` e `HomeActions += { openFixedDialog(); closeFixedDialog(); createFixedRoom(nickname, name) }`;
  - `JoinView += { fixedName?: string }`;
  - `RoomView += { room: { kind: 'quick' } | { kind: 'fixed'; name: string; own: boolean } }`;
  - `NotFoundView { reason: 'quick' | 'deleted' | 'kicked' | 'expired'; name: string | null; saved: boolean }` e `NotFoundActions { createRoom(); pasteOtherLink(); forget() }`.
- [X] T1130 [US2] `src/lib/stores/room.svelte.ts`:
  - `roomInfo = $state<RoomInfo>({ kind: 'quick' })`, atualizado no `welcome` (`msg.room ?? { kind: 'quick' }`);
  - `own = !!loadOwnerKey(code)`;
  - `buildView()` monta `room`;
  - prop de construtor `opts.justCreated` → depois do primeiro `welcome`, copia o convite e mostra o toast `{ kind: 'copied', text: 'Sala criada — fixe o link no grupo' }` (S1), no lugar do "Link copiado".
- [X] T1131 [US2] `src/lib/room-view.ts` `header()`: se `view.room.kind === 'fixed'`, `title = view.room.name` e o subtítulo vem dos casos de hoje (Room.dc.html `fixedSub`):
  - vazio: "Ninguém transmitindo agora";
  - single: "Lu está ao vivo · 1080p · 60 fps";
  - eu e outro ao vivo: "Lu está ao vivo · você também está transmitindo";
  - grade: "3 telas ao vivo";
  - foco: "Vendo Bia · 3 telas ao vivo".
  - Teste novo em `test/lib/room-view.test.ts`.
- [X] T1132 [P] [US2] Criar `src/ui/components/NameDialog.svelte` (Home.dc.html `showCreate` e Room.dc.html `showRename`):
  - `mode='create'`: ícone `Pin` em círculo sage, "Criar sala fixa", "O link não muda. Fixe no grupo e use sempre o mesmo. Some se ficar 30 dias sem uso.", rótulo "Nome da sala", dica "Aparece só na tela, nunca no link." e o contador `n/40`. Botões "Cancelar" / "Criar e entrar";
  - `mode='rename'`: "Renomear sala", "O link continua o mesmo.", só o contador. "Cancelar" / "Salvar";
  - `maxlength` = 40 contando code points (cortar no `input`);
  - botão principal desabilitado se `normalizeRoomName(value)` for null ou se `busy`;
  - `error` aparece embaixo do campo;
  - Esc e o X fecham;
  - usar as classes `dialog-backdrop`/`dialog` de `base.css`, como o `GoLiveModal`.
- [X] T1133 [US2] `src/ui/pages/HomePage.svelte` (Home.dc.html):
  - embaixo da linha apelido + "Criar sala", botão sem fundo, cor `--color-accent-400`, ícone `Pin` 17px, "Criar sala fixa", mais o texto "Com nome e um link que não muda." (14px, 55%);
  - "Tem um link?" continua embaixo;
  - `view.fixedDialog` → `NameDialog mode="create"`;
  - no celular (M4), o botão vira "Sala fixa" ao lado de "Criar sala", com fundo `rgba(198,113,57,0.12)`.
- [X] T1134 [US2] `src/routes/HomeRoute.svelte`:
  - ações `openFixedDialog`, `closeFixedDialog` e `createFixedRoom(nickname, name)`;
  - `createFixedRoom` salva o apelido e chama `api.createFixedRoom`. Se der certo, `saveOwnerKey` e `router.go(roomPath(code), { nickname, created: true })`. Erros: `rate-limited` → "Muitas salas criadas daqui agora. Tenta de novo em um minuto.", `name` → "Esse nome não dá. Use letras, números e espaços.", `network` → a mensagem de hoje;
  - `NavState` (`src/lib/router.svelte.ts`) ganha `created?: boolean`.
- [X] T1135 [US2] `src/ui/pages/JoinPage.svelte` (Espia.dc.html F8), com `view.fixedName`:
  - selo "Sala fixa" (ícone `Pin` 14px, fundo `rgba(174,191,146,0.16)`, cor `--color-accent-2-300`, 13px/600) acima do título;
  - título = o nome (52px);
  - subtítulo "Te chamaram pra cá. Lu, Bia e Tomás já estão lá." (`whoIsThere`);
  - embaixo do Entrar: "Ao entrar, ela fica em Suas salas neste aparelho." (13px, 55%);
  - sala rápida: igual a hoje.
- [X] T1136 [US2] Criar `src/ui/pages/NotFoundPage.svelte` (NotFound.dc.html):
  - logo + cartão 560px com `Mascot` (`sleepy` se `expired`, senão `sad`, 96);
  - `name` em 15px/600/60% acima do título, quando `reason !== 'quick'` e houver nome;
  - título e texto por motivo, **literalmente** do design:
    - quick: "Essa sala não existe mais." / "O link pode estar incompleto, ou todo mundo saiu — salas rápidas somem quando ficam vazias.";
    - deleted: "Esta sala foi apagada." / "Quem criou a sala apagou. O link não funciona mais.";
    - kicked: "Esta sala foi apagada." / "Quem criou a sala apagou agora há pouco, e todo mundo saiu junto.";
    - expired: "Esta sala ficou 30 dias sem uso e deixou de existir." / "Salas fixas somem sozinhas depois de um mês vazias. Dá pra criar outra com o mesmo nome.";
  - botões "Criar uma sala nova" e, como segundo, `saved` ? "Tirar de Suas salas" : "Colar outro link";
  - celular: mesmo layout do "não existe" de hoje da `JoinPage`;
  - **remover** o ramo `notfound` da `JoinPage` (o ramo `full` fica).
- [X] T1137 [US2] `src/routes/RoomRoute.svelte`:
  - estado `gone: NotFoundView | null`. Prévia `notfound` → `{ reason: 'quick' }`, `gone` → `{ reason }`. `ended === 'not-found'` → pergunta a prévia de novo pra decidir entre quick/expired/deleted;
  - renderiza `NotFoundPage` quando `gone`;
  - `name`/`saved` vêm de Suas salas (T1146; até lá, `null`/`false`);
  - passa `fixedName` pra `JoinPage`;
  - passa `justCreated: router.state.created` pro `RoomStore`;
  - `forget()` (T1146).
- [X] T1138 [US2] Mocks:
  - `src/mocks/home.ts`: cena `create` (F7: `fixedDialog: { mode:'create', value:'CS da galera', … }`) e `first` (F1);
  - `src/mocks/join.ts`: cena `fixed` (F8: `fixedName:'CS da galera'`, Lu ao vivo, Bia e Tomás);
  - `src/mocks/room.ts`: cenas `fixed` (base `empty`, fixa, own), `created` (base `copied` + toast S1, own) e `guest` (base `single`, fixa, `own:false`);
  - criar `src/mocks/notfound.ts` (`NOTFOUND_SCENES`: quick, deleted, kicked, expired; `&saved=1` liga `saved`, ignorado em quick; nome "CS da galera") e `MockNotFound.svelte`;
  - registrar `{ id: 'notfound', label: 'Sala não encontrada' }` em `src/mocks/registry.ts` e no `MockApp.svelte`;
  - tirar a cena `notfound` de `join.ts`, que agora fica na tela própria.

**Checkpoint US2**: `npm test`, `check` e `build`. Mocks F1/F7/F8/S1/S5/N3/N4. Quickstart §3 passos 1–2.

## Phase 5: User Story 3 — Suas salas na tela inicial (P3)

**Goal**: painel com até 5 salas fixas deste aparelho, com números ao vivo, sem apelidos.

**Independent Test**: quickstart §1 (`status`) e §3 passos 6 e 8.

- [X] T1139 [US3] `src/worker/room.ts`: RPC `summary()` → `{ state: 'ok'; name: string | null; people; streaming } | { state: 'gone'; reason } | { state: 'notfound' }`:
  - `people` = `everyone().length`;
  - `streaming` = quantos têm `live !== null`;
  - **sem nenhum campo de pessoa**;
  - não mexe em `lastUsed`.
- [X] T1140 [US3] `src/worker/validate.ts` `parseStatusBody(text)`: JSON `{ codes: string[] }`, 1–`SAVED_ROOMS_MAX` (5) itens, todos em `ROOM_ID_RE`, sem repetidos, até 1 KB. Senão, null.
- [X] T1141 [US3] `src/worker/index.ts` rota `POST /api/rooms/status` (**antes** do match de `/api/rooms/:code`):
  - `sameOrigin` senão 403;
  - **um** `allowed(env.RL_LOOKUP)`;
  - `parseStatusBody` senão 400;
  - `Promise.all(codes.map(c => room(env, c).summary()))` → `200 { rooms: [{ code, ...s }] }` na mesma ordem.
- [X] T1142 [US3] Testes em `test/worker/room.test.ts`, bloco `describe('Suas salas: status (specs/011)')`:
  - 3 códigos (fixa com 2 pessoas e 1 ao vivo, rápida com 1, inexistente) → `ok{name,2,1}`, `ok{null,1,0}` e `notfound`;
  - `JSON.stringify(resposta)` não contém nenhum dos apelidos usados (SC-006);
  - 6 códigos, repetido ou código ruim → 400;
  - Origin de fora → 403;
  - fixa expirada → `gone{expired}`;
  - `status` não muda o prazo de 30 dias (consultar perto do fim e ver que expira mesmo assim).
- [X] T1143 [P] [US3] Criar `src/lib/saved-rooms.ts` (funções puras + storage, `contracts/local-storage.md`):
  - `loadSavedRooms(): SavedRoom[]` (descarta inválidos) e `saveSavedRooms(list)` na chave `espia:rooms`;
  - `rememberRoom(list, { code, name }, now)`: upsert com `lastUsed = now`, ordena por `lastUsed` desc e **corta em `SAVED_ROOMS_MAX` (5)**;
  - `forgetRoom(list, code)` e `renameSaved(list, code, name)`;
  - `roomTone(code): Tone` (hash simples do código → `TONES`, estável);
  - `recentLabel(lastUsed, now)`: "usada hoje", "usada ontem", "há N dias" (2–6), "há 1 semana", "há N semanas", "há mais de um mês";
  - `savedRoomSub(view)`: "N pessoa agora"/"N pessoas agora" + " · M transmitindo" se M>0; "Vazia · <recent>"; "Vazia";
  - `buildSavedRooms(list, statuses | null, owned: (code) => boolean, now): SavedRoomsView`. Ordem: `people > 0` primeiro (mais gente antes), depois `lastUsed` desc; `gone` fica no lugar do seu `lastUsed`. Grupos "Com gente agora" / "Usadas recentemente" só se `rooms.length >= 4` e os dois grupos não estiverem vazios, senão um grupo sem rótulo. `note` = "neste aparelho", ou "N salas neste aparelho" quando agrupado. `statuses === null` → todas `loading`.
- [X] T1144 [P] [US3] `test/lib/saved-rooms.test.ts`:
  - limite 5: a 6ª entrada tira a de `lastUsed` mais antigo;
  - upsert não duplica;
  - ordem com gente/sem gente;
  - agrupamento a partir de 4 com os dois tipos, e não com 5 vazias;
  - todos os `recentLabel`;
  - textos do `savedRoomSub`;
  - `loadSavedRooms` com JSON corrompido → [].
- [X] T1145 [US3] `src/lib/api.ts`: `fetchStatuses(codes)` → `RoomStatus[] | null` (null em erro/429; a tela fica em `loading` e "Entrar" continua funcionando).
- [X] T1146 [US3] Lembrar a sala:
  - `src/lib/stores/room.svelte.ts`: no `welcome` com `room.kind === 'fixed'`, `saveSavedRooms(rememberRoom(loadSavedRooms(), { code, name }, Date.now()))`;
  - `src/routes/RoomRoute.svelte`: `NotFoundView.name`/`saved` vêm de `loadSavedRooms()`. `forget()` faz `forgetRoom` + `forgetOwnerKey` e vai pra `/`;
  - prévia ou status `gone`/`notfound` de uma sala com chave de dono → `forgetOwnerKey`.
- [X] T1147 [US3] `src/lib/types.ts`:
  - `SavedRoomView` e `SavedRoomsView` exatamente como em `data-model.md`;
  - `HomeView += { rooms: SavedRoomsView | null }` (null = lista vazia → ilustração);
  - `HomeActions += { enterSaved(code); forgetSaved(code); toggleSavedMenu(code | null) }`.
- [X] T1148 [P] [US3] Criar `src/ui/components/SavedRoom.svelte` (SavedRoom.dc.html):
  - linha 26px de raio e avatar com a inicial (46px; 42 no compacto), cores pelo `tone`. Com alguém transmitindo, anel `0 0 0 2px #26231f, 0 0 0 4px var(--color-accent)`. Em `gone`, fundo `rgba(245,234,216,0.08)`;
  - nome 16px/600 com reticências e selo "Sua";
  - `loading` → barrinha 128×10; `ok` → `savedRoomSub`; `gone` → "Esta sala não existe mais" (`--color-accent-300`) e nome com opacidade 0.6;
  - selo "AO VIVO" quando houver transmissão;
  - "Entrar" (`btn-secondary`, 40px; escondido no `compact`) + "⋯" (44px); em `gone`, só "Tirar da lista";
  - no `compact`, a linha inteira é um botão que entra;
  - o menu (`RoomMenu`, T1158) abre a partir do "⋯".
- [X] T1149 [P] [US3] Criar `src/ui/components/SavedRoomsPanel.svelte` (Home.dc.html `hasRooms`):
  - cartão 600px, raio 36px, fundo `--color-surface`, sombra `0 30px 70px rgba(0,0,0,0.35)`;
  - `Mascot mood="peek" size=92` girado 4° em cima à direita;
  - título "Suas salas" (26px) + `note`;
  - grupos com rótulo em 12px/700 maiúsculas;
  - versão `compact` do celular (M4): sem cartão, título 22px, linhas `compact`.
- [X] T1150 [US3] `src/ui/pages/HomePage.svelte`: `view.rooms` não nulo → `SavedRoomsPanel` no lugar da ilustração (mascote + duas `StreamTile`), no computador e no celular (M4, entre o formulário e "Colar link da sala").
- [X] T1151 [US3] `src/routes/HomeRoute.svelte`:
  - ao montar, `list = loadSavedRooms()`. Se vazia, `view.rooms = null`. Senão, `buildSavedRooms(list, null, …)` e depois `fetchStatuses(codes)` → `buildSavedRooms(list, statuses, …)`;
  - nome mudou no status → `renameSaved` + salvar;
  - `enterSaved(code)` → `router.go(roomPath(code), { nickname })` se já houver apelido, senão vai pra tela de entrada;
  - `forgetSaved` → `forgetRoom` + salvar + reconstruir (a chave de dono fica);
  - `toggleSavedMenu`.
- [X] T1152 [US3] Mocks em `src/mocks/home.ts`, com os dados do `Home.dc.html`:
  - `few`: CS da galera (3 pessoas, 1 transmitindo, Sua, accent), Trampo do TCC (2, sand) e Filme de sexta (vazia, "usada ontem", sage);
  - `loading`: as 3 em `loading`;
  - `gone`: + "Minecraft do Caio" em `gone`, deep;
  - `many`: **5 salas** (research R9). Com gente: CS da galera (4/2, Sua), Lu joga Hollow Knight (3/1) e Trampo do TCC (2/0). Recentes: Filme de sexta (usada ontem) e Revisão de código (há 3 dias, Sua). Agrupadas, nota "5 salas neste aparelho";
  - `first` já existe (T1138);
  - em `src/mocks/mobile.ts` ou num `home` com `device:'mobile'`, a cena `home` (M4): as 3 do `few` em `compact`.

**Checkpoint US3**: `npm test`, `check` e `build`. Mocks F1, F2, F4, F5, F6 e M4. Quickstart §3 passos 6 e 8.

## Phase 6: User Story 4 — O dono renomeia e apaga (P4)

**Goal**: "⋯" só pro dono. Renomear avisa ao vivo. Apagar expulsa com aviso e deixa o link em N1.

**Independent Test**: quickstart §1 (PATCH/DELETE) e §3 passos 5 e 7.

- [ ] T1153 [US4] `src/worker/room.ts`, RPCs `rename(ownerHash, name)` e `remove(ownerHash)` → `'ok' | 'forbidden' | 'notfound' | 'gone'`:
  - compara `ownerHash` com `fixed.ownerHash` usando `tokensEqual`. Sala rápida → `forbidden`;
  - `rename`: grava `fixed.name` e faz `broadcast({ t: 'room-renamed', name })`;
  - `remove`: `broadcast({ t: 'room-deleted' })`, fecha cada socket com `CLOSE.deleted` ("sala apagada"), revoga o TURN de todos (membros + fantasmas) e chama `bury('deleted')`.
- [ ] T1154 [US4] `src/worker/index.ts`, `PATCH` e `DELETE /api/rooms/:code`:
  - `sameOrigin`, `allowed(env.RL_CREATE)` e código válido;
  - `Authorization: Bearer <64 hex>` senão 403, e `ownerHash = sha256Hex(key)`;
  - `PATCH` lê `{ name }` com `normalizeRoomName` (null → `400 { error: 'name' }`);
  - mapear `ok` → `200 { name }`/`204`, `forbidden` → 403, `notfound` → 404 e `gone` → `410 { reason }`;
  - o `GET` continua igual, e outros métodos → 405.
- [ ] T1155 [US4] Testes em `test/worker/room.test.ts`, bloco `describe('Dono (specs/011)')`:
  - `PATCH` sem cabeçalho, com chave errada e em sala rápida → 403;
  - chave certa → 200, quem está dentro recebe `room-renamed`, e a prévia e o `status` mostram o nome novo;
  - `DELETE` com 2 dentro → as duas recebem `room-deleted` e fechamento 4410, o TURN é revogado (API simulada registra), a prévia vira `410 { reason:'deleted' }` e um `ws` novo recebe `room-not-found`;
  - Origin de fora → 403.
- [ ] T1156 [US4] Cliente, sinalização e API:
  - `src/lib/signaling.ts`: `SignalingEnd += 'deleted'`; `room-deleted` ou fechamento `CLOSE.deleted` → `end('deleted')`;
  - `src/lib/api.ts`: `renameRoom(code, key, name)` → `{ ok: true; name } | { ok: false; reason: 'forbidden' | 'gone' | 'name' | 'network' }` e `deleteRoom(code, key)` → `{ ok: true } | { ok: false; reason }`.
- [ ] T1157 [US4] `src/lib/stores/room.svelte.ts`:
  - `room-renamed` → atualiza `roomInfo.name` e `renameSaved` na lista;
  - estado `ownerMenu`, `nameDialog` e `deleteDialog` (`DeleteDialogView { name; people; busy }`);
  - ações `toggleOwnerMenu()`, `openRename()`, `submitRename(name)` (`api.renameRoom`; o nome muda pelo `room-renamed`), `openDelete()` (`people` = `peers.length + 1` menos eu = `peers.length`) e `confirmDelete()` (`api.deleteRoom` → `forgetRoom` + `forgetOwnerKey` → `onLeave()`), mais `closeDialogs()`;
  - tudo desabilitado se `link !== 'ok'`;
  - adicionar em `RoomActions`/`RoomView` (`src/lib/types.ts`).
- [ ] T1158 [P] [US4] Criar `src/ui/components/RoomMenu.svelte` (popover de SavedRoom.dc.html e Room.dc.html):
  - largura 220/230px, raio 22px, fundo `#322e28`, sombra do design;
  - itens de 44px: "Renomear sala" (`Pencil`), "Apagar sala" (`Trash2`, `--color-accent-300`) e separador, só se `own`; depois "Tirar da lista" (`ListX`/`AlignLeft` do design) com a nota "Some só deste aparelho. A sala continua pros outros.";
  - variante `room`: só os itens do dono, com a nota "Só aparece pra você — a sala foi criada neste aparelho.";
  - fecha com Esc e com clique fora.
- [ ] T1159 [P] [US4] Criar `src/ui/components/DeleteRoomDialog.svelte` (Room.dc.html `showDelete`):
  - círculo 52px `--color-accent-300` com `Trash2`;
  - "Apagar "<nome>"?" e "O link para de funcionar para todo mundo. Não tem como desfazer.";
  - se `people > 0`, a caixa `rgba(198,113,57,0.12)` com `Users` e "Tem N pessoas aqui agora. Elas saem da sala e veem um aviso." ("1 pessoa"/"N pessoas");
  - "Cancelar" e "Apagar sala" (fundo `--color-accent-300`, texto `--color-accent-900`).
- [ ] T1160 [US4] `src/ui/pages/RoomPage.svelte` (S2–S5):
  - sala fixa e `own` → botão "⋯" (36px) ao lado do título, `RoomMenu variant="room"` abaixo, `NameDialog mode="rename"` e `DeleteRoomDialog`;
  - sem `own`, nada (S5);
  - `MobileRoomPage`: "⋯" ao lado do nome, com as mesmas ações.
- [ ] T1161 [US4] Home e rota:
  - `src/routes/HomeRoute.svelte` + `HomePage`: o menu do "⋯" de Suas salas tem "Renomear sala" e "Apagar sala" quando há chave de dono (`renameSaved`/`deleteSaved`, usando `NameDialog` e `DeleteRoomDialog`). O número de pessoas no apagar vem do último `status`. Depois de apagar, tira da lista e esquece a chave. Resposta `gone` → item vira `gone`;
  - `src/routes/RoomRoute.svelte`: `ended === 'deleted'` → `NotFoundView { reason: 'kicked', name, saved }` (N2).
- [ ] T1162 [US4] Mocks:
  - `src/mocks/room.ts`: `ownermenu` (S2), `rename` (S3: `nameDialog { mode:'rename', value:'CS da galera 2' }`) e `delete` (S4: `deleteDialog { name:'CS da galera', people:3 }`);
  - `src/mocks/home.ts`: `menu` (F3: CS da galera com `menuOpen`, own);
  - `notfound` deleted/kicked já existem (T1138).
  - Conferir com o design.

**Checkpoint US4**: `npm test`, `check` e `build`. Mocks F3, S2–S4, N1, N2. Quickstart §3 passos 5 e 7.

## Phase 7: Polish & Cross-Cutting

- [ ] T1163 [P] `specs/001-espia-screen-share/screens.md`: as cenas novas, com os códigos do design (F1–F8, S1–S5, Q1–Q3, N1–N4, M4–M5) e o mock de cada uma. Conferir que **todas as 23** do FR-034 abrem pelo `MockIndex`.
- [ ] T1164 [P] `specs/001-espia-screen-share/contracts/http-api.md` e `signaling.md`: links para os deltas da 011 e a tabela de tempos atualizada (20 s → 60 s).
- [ ] T1165 `npm run check`, `npm run build` e `npm test` sem erros nem avisos novos.
- [ ] T1166 Roteiro do quickstart §3 inteiro com `npm run dev:worker` e 2 janelas do Edge (uma InPrivate). Registrar o resultado em `specs/011-salas-fixas/relatorio.md`.
- [ ] T1167 Publicar (`npm run deploy`; se o auto mode bloquear, a pessoa usuária roda `! npm run deploy`), depois de avisar a galera que essa publicação ainda derruba as salas abertas. Fazer a checagem do quickstart §4 com o app 0.1.7 e anotar no `relatorio.md`.
- [ ] T1168 Commit "011: salas fixas e chamada que sobrevive a quedas" (sem push, sem GitHub).

## Dependencies

- **Setup (T1101–T1105)** → **Foundational (T1106–T1108)** → histórias.
- **US1** (T1109–T1120) não depende das outras e é o MVP. Pode ser publicada sozinha.
- **US2** depende do Foundational. Na prática vem depois da US1, porque mexe nos mesmos arquivos (`room.ts`, `room.svelte.ts`, `RoomPage`).
- **US3** depende da US2 (só existe lista de salas fixas depois de criar uma).
- **US4** depende da US2 (dono e chave) e usa o `RoomMenu` na Home da US3 (T1161).
- **Polish** no fim.

## Parallel Opportunities

- Setup: T1102, T1103 em paralelo com T1101; T1104/T1105 juntos.
- Foundational: T1107 + T1108 em paralelo com T1106.
- US1: T1116 e T1117 (componentes) em paralelo com T1109–T1112 (servidor).
- US2: T1128 e T1132 em paralelo com o servidor (T1121–T1126).
- US3: T1143/T1144 (puro) e T1148/T1149 (componentes) em paralelo com T1139–T1142.
- US4: T1158 e T1159 em paralelo com T1153–T1155.

## Implementation Strategy

1. **MVP = Setup + Foundational + US1**. Publicar e pedir pra galera testar uma publicação com a chamada aberta. É a correção do problema de 29/10.
2. US2 → publicar (link fixo já útil, mesmo sem a lista).
3. US3 → US4 → Polish → publicar e fazer o commit.
4. Em cada história: mock da tela primeiro, depois a lógica real (constituição, "Fluxo de desenvolvimento").
