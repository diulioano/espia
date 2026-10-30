# Research: Salas fixas e chamada que sobrevive a quedas

## R1 — Por que a chamada caiu de vez em 2026-10-29

**Achado**: hoje quem está na sala fica guardado **só no attachment do WebSocket** (`serializeAttachment`). Quem caiu vira "fantasma" no storage, mas isso só acontece dentro de `webSocketClose`. Segundo a documentação da Cloudflare (Durable Object Lifecycle, "Shutdown behavior"; WebSockets, "WebSocket disconnection on deploy"): *"Code updates disconnect all WebSockets. Deploying a new version restarts every Durable Object"* e *"WebSocket requests are terminated automatically during shutdown"*. A instância antiga morre sem rodar `webSocketClose`, e a nova não tem nenhum socket. Resultado: `members()` vazio, `ghosts` vazio, `reservedUntil` vencido → `exists()` = false. O `resume` de todo mundo cai em `resume-failed`, o `join` seguinte recebe `room-not-found` e a tela mostra "Essa sala não existe mais".

**Decision**: guardar a **lista de quem está na sala no storage do DO** (`roster`), escrita a cada entrada, retomada, live/unlive e saída. Ao acordar (construtor, dentro de `blockConcurrencyWhile`), quem está no `roster` sem socket vivo vira fantasma com prazo `agora + tolerância`, e o alarme é marcado. Com isso, depois de uma publicação, todo mundo retoma a **mesma identidade** (`resume`), as ligações P2P continuam (o `syncMesh` já mantém as conexões quando o id é o mesmo) e ninguém vê saída/entrada.

**Rationale**: corrige a causa, não o sintoma. Vale para sala rápida e fixa. O `roster` é apagado quando a sala esvazia de vez, como os fantasmas hoje (princípio III).

**Teste**: `evictDurableObject(stub, { webSockets: 'close' })` de `cloudflare:test` (presente no `@cloudflare/vitest-plugin` 1.4.10) simula a publicação: fecha os sockets sem `webSocketClose` na instância antiga.

**Alternatives considered**:
- Só aumentar a tolerância de 20 s: não resolve, porque o fantasma nunca é criado no reinício.
- `ctx.getWebSockets()` depois do reinício: vem vazio (os sockets foram terminados).

## R2 — Tolerância individual e espera da sala rápida

**Decision**:
- `GRACE_MS` 20 s → **60 s** (lugar guardado por pessoa; igual ao tempo até a tela "Não deu pra voltar").
- Quando a **última pessoa conectada cai** (não sai), a sala ganha `holdUntil = agora + 5 min` (`HOLD_MS`, var do wrangler, encurtada nos testes). `exists()` passa a considerar `holdUntil`. Sair com "Sair da sala" (4001) não cria `holdUntil`.
- No reinício (R1), se ninguém tem socket, também se cria `holdUntil`.

**Rationale**: separa "a pessoa" (60 s) de "a sala" (5 min), como pede o FR-005. A volta depois de 60 s e antes de 5 min entra como pessoa nova, com o mesmo apelido, sem tela de entrada (o `Signaling` já faz `join` depois de `resume-failed`).

**Alternatives considered**: tolerância individual de 5 min. Rejeitada: quem fechou a aba ficaria 5 min "na sala" pros outros e ocupando lugar.

## R3 — Onde guardar a sala fixa

**Decision**: no **storage do próprio DO da sala** (SQLite-backed, já em uso), na chave `fixed`: `{ name, ownerHash, createdAt, lastUsed }`. `exists()` = true enquanto `fixed` existir. Expiração por **alarme** em `lastUsed + 30 dias`, remarcado a cada `join` (não `resume`). O alarme continua sendo um só: `scheduleAlarm()` passa a considerar também a expiração quando a sala está sem ninguém.

**Rationale**: o código da sala já é o nome do DO (`idFromName(code)`). Nada de banco novo, nada de lista global de salas (nada pra varrer, princípio VI). Uma sala fixa vazia e hibernada não custa duração, só os poucos bytes guardados.

**Alternatives considered**: D1 ou KV com todas as salas fixas. Rejeitado: cria um índice global sem necessidade, e a consulta de Suas salas já sabe os códigos.

## R4 — Prova de dono

**Decision**: ao criar a sala fixa, o **Worker** sorteia `ownerKey` (32 bytes, CSPRNG, hex) e devolve **uma vez** na resposta. O DO guarda só `ownerHash = SHA-256(ownerKey)`. Renomear/apagar mandam `Authorization: Bearer <ownerKey>`. O DO compara os hashes em tempo constante (`timingSafeEqual`, como os tokens de hoje). O navegador guarda a chave em `localStorage` (`espia:owner:<code>`), **separada** da lista Suas salas, então tirar da lista (ou passar das 5) não perde a posse.

**Rationale**: sem conta e sem login (princípio I). Um vazamento do storage do servidor não dá a chave. O app tem a própria lista porque o WebView2 usa outro perfil de armazenamento: não precisa de API nativa nova.

**Alternatives considered**: chave gerada no navegador e mandada na criação. Equivalente, mas o servidor sortear mantém "segredos nascem no servidor" e a mesma função `randomHex`.

## R5 — Sala apagada ou expirada: N1/N3 sem guardar nada sobre as pessoas

**Decision**: ao apagar ou expirar, o DO apaga tudo e deixa só uma **lápide** `{ reason: 'deleted' | 'expired', until: agora + 30 dias }`, **sem o nome**. A lápide faz o código responder `410 { reason }` (em vez de 404) e impede que o código seja sorteado de novo enquanto valer. Depois de 30 dias, o alarme apaga a lápide e o código vira 404 comum. O nome que aparece em N1–N3 vem de **Suas salas do aparelho**, quando ele conhece a sala.

**Rationale**: dá pra distinguir N1/N3 de N4 (FR-029) sem guardar nada além do motivo. Reaproveitar o código de uma sala apagada mandaria quem tem o link antigo pra uma sala de outra pessoa.

## R6 — Apagar com gente dentro (N2) e renomear ao vivo

**Decision**:
- **Apagar**: o DO manda `{ t: 'room-deleted' }` a todos, fecha com `CLOSE.deleted` (4410), revoga o TURN de todos e grava a lápide. O `Signaling` trata 4410 / `room-deleted` como fim (`SignalingEnd = 'deleted'`) e a rota mostra N2.
- **Renomear**: o DO grava o nome e manda `{ t: 'room-renamed', name }` a todos. O store atualiza o cabeçalho e o item de Suas salas.
- Os dois passam pelo Worker como HTTP (`PATCH`/`DELETE /api/rooms/:code`), pra funcionar também pelo "⋯" de Suas salas, sem estar na sala.

## R7 — Suas salas: uma consulta só, só números

**Decision**: `POST /api/rooms/status` com `{ codes: string[] }` (até 5, códigos válidos, sem repetir). O Worker consulta os DOs em paralelo (RPC `summary()`) e devolve, para cada código: `ok` (nome atual se fixa, `people`, `streaming`), `gone` (`deleted`/`expired`) ou `notfound`. **Nunca apelidos.** Conta **uma vez** no `RL_LOOKUP` (20/min por IP), então recarregar a tela inicial não esgota o limite.

**Rationale**: FR-019, FR-032, SC-005. Com 5 salas, 5 RPCs em paralelo cabem fácil em 2 s.

**Alternatives considered**: um `GET /api/rooms/:code` por sala. Rejeitado: devolve apelidos (é a prévia da tela de entrada) e consome 5 consultas do limite por abertura.

## R8 — A queda na tela

**Decision**: o `RoomStore` ganha `link: 'ok' | 'reconnecting' | 'failed'` e `downSince`.
- `reconnecting`: aparece assim que o `Signaling` avisa (sem esperar). Substitui o toast de erro de hoje pelo aviso do design (pílula "Reconectando… Você continua na sala"), esmaece a lista, troca os textos do cartão e da pílula de ao vivo. "Transmitir" fica desabilitado.
- `failed`: depois de 60 s em `reconnecting`. O `Signaling` **continua tentando** em segundo plano (backoff até 10 s). Se voltar, a tela de falha some sozinha e aparece "De volta!". "Tentar de novo" chama `signaling.retryNow()` (zera o backoff, como o `onOnline` de hoje).
- Volta (`welcome` depois de queda): toast `De volta!` (tipo `copied`, como no design).
- A mídia P2P **não é fechada** durante a queda (já é assim). Se o id mudou na volta, o `syncMesh` refaz tudo (já é assim).

## R9 — Limite de Suas salas

**Decision**: no máximo **5** (pedido da pessoa usuária em 2026-10-30). Ao entrar numa 6ª, sai a de `lastUsed` mais antigo. Agrupamento "Com gente agora" / "Usadas recentemente" só com 4+ salas e os dois grupos não vazios. O mock `many` passa a ter 5 salas (3 com gente, 2 vazias).

## R10 — Nome da sala

**Decision**: normalização igual no navegador e no Worker (função compartilhada em `src/shared/room-name.ts`): `NFC`, troca qualquer espaço em branco por espaço, junta repetidos, `trim`, recusa caracteres de controle (`\p{Cc}`) e de formatação invisíveis (`\p{Cf}`, ex.: U+200B, U+202E), 1–40 caracteres (por code point). O servidor recusa com 400 o que não passa. A tela só usa como texto (Svelte já escapa).

## R11 — Publicação

**Decision**: tudo sai num `npm run deploy` (site + Worker juntos). App 0.1.7 carrega o site publicado, então **não precisa de versão nova do app**. A mudança é compatível com abas abertas na versão antiga: o cliente velho recebe as mensagens novas (`room-renamed`, `room-deleted`) e as ignora, e o `welcome` só ganha um campo. Esta publicação ainda derruba as salas abertas na hora (o `roster` não existe na versão velha). As próximas não derrubam mais.
