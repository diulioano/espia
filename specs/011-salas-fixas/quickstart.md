# Quickstart: validar a 011

## Pré-requisitos

- `npm install` já feito. Para os testes locais com o Worker: `npm run dev:worker` (http://127.0.0.1:8787).
- Gates de sempre: `npm run check`, `npm run build`, `npm test`.

## 1. Testes automáticos (Vitest + Worker)

`test/worker/room.test.ts` (novos blocos) — tempos encurtados no `vitest.config.ts` (`GRACE_MS`, `RESERVE_MS`, ★ `HOLD_MS`):

| Caso | Esperado |
|---|---|
| **Publicação**: 3 pessoas, `evictDurableObject(stub, { webSockets: 'close' })`, todas mandam `resume` | `welcome` com o mesmo id pra todas, nenhum `peer-left`/`peer-joined` (R1) |
| Publicação e ninguém volta | sala rápida existe até `HOLD_MS`, depois 404 |
| Todos caem (sem 4001), um volta antes de `HOLD_MS` | mesma sala |
| Todos saem com 4001 | 404 na hora (sem `HOLD_MS`) |
| Criar fixa: nome válido / vazio / 41 caracteres / U+202E | `201 {code, ownerKey}` / `400` / `400` / `400` |
| Fixa vazia depois de passar `GRACE`+`HOLD` | prévia `200` com `fixed.name` |
| Fixa: avançar o relógio 30 dias sem `join` (alarme) | `410 expired`; código não é sorteado de novo |
| `status` com 3 códigos (fixa com gente, rápida, inexistente) | `ok`/`ok`/`notfound`, **nenhum `nickname` no JSON** |
| `status` com 6 códigos ou código repetido | `400` |
| `PATCH` sem chave / chave errada / chave certa | `403` / `403` / `200` + `room-renamed` pra quem está dentro |
| `DELETE` com 2 pessoas dentro | `room-deleted` + fechamento 4410 pras duas, TURN revogado, depois prévia `410 deleted` |
| `welcome` de sala fixa | `room: { kind: 'fixed', name }` |

`test/lib/` (puros): `room-name.test.ts` (normalização), `saved-rooms.test.ts` (limite 5, ordem, grupos, `recentLabel`), `signaling` (fim em 4410, `retryNow`).

## 2. Mocks (sem servidor)

Abrir cada uma e comparar com o design (Claude Design, projeto `dc89098d`):

- `?mock=home&scene=` first · few · menu · many · loading · gone · create
- `?mock=join&scene=fixed`
- `?mock=room&scene=` fixed · created · ownermenu · rename · delete · guest · reconnecting · back · reconnectfail
- `?mock=notfound&scene=` quick · deleted · kicked · expired (cada um também com `&saved=1`)
- `?mock=mobile&scene=` home · reconnecting

## 3. Ponta a ponta local (`npm run dev:worker`, 2 janelas do Edge)

1. Janela A: "Criar sala fixa" → "CS da galera" → cai na sala vazia, convite copiado, toast "Sala criada — fixe o link no grupo", "⋯" ao lado do nome.
2. Janela B (perfil InPrivate): abre o link → selo "Sala fixa", nome, "Ao entrar, ela fica em Suas salas". Entra. **Sem "⋯"**.
3. A transmite uma aba. **Reinicia o `dev:worker`** (simula publicação). Nas duas: pílula "Reconectando…", lista esmaecida, o vídeo continua. Ao subir de novo: "De volta!", mesma lista, a transmissão segue. (SC-001, SC-002)
4. Para o `dev:worker` por mais de 60 s → "Não deu pra voltar pra sala." Sobe de novo → a tela some sozinha e aparece "De volta!".
5. A renomeia pra "CS da galera 2" → muda na tela de B sem recarregar.
6. Volta pra tela inicial nas duas → "Suas salas" com "2 pessoas agora · 1 transmitindo" (A com "Sua"). Inspecionar a resposta de `/api/rooms/status`: só números. (SC-006)
7. A apaga a sala com B dentro → B vai para "Esta sala foi apagada… todo mundo saiu junto", com "Tirar de Suas salas". Na tela inicial de A, o item some, e em B aparece "Esta sala não existe mais".
8. Entrar em 6 salas fixas diferentes → a lista mostra só as 5 mais recentes.

## 4. Produção (depois do `npm run deploy`)

- Com 3 pessoas na sala, fazer outra publicação qualquer (`npm run deploy` de novo) → todo mundo volta sozinho em até 30 s (SC-001). **A primeira publicação, a que leva a 011, ainda derruba as salas abertas** (R11): avisar a galera antes.
- App desktop 0.1.7: abrir uma sala fixa pelo app → aparece em Suas salas do app, não na do navegador.
