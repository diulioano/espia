# Contrato — HTTP do Worker (mudanças da 011)

Base: [001 http-api.md](../../001-espia-screen-share/contracts/http-api.md). Regras comuns de lá continuam valendo (código válido senão `400`, `Origin` do próprio host senão `403`, `429` com `Retry-After`, cabeçalhos de segurança).

| Método | Rota | Limite por IP | Corpo | Resposta |
|---|---|---|---|---|
| `POST` | `/api/rooms` | 10/min (`RL_CREATE`) | vazio **ou** `{ "kind": "quick" }` | `201 { code }`, igual a hoje (sala rápida) |
| `POST` | `/api/rooms` | 10/min (`RL_CREATE`) | `{ "kind": "fixed", "name": "CS da galera" }` | ★ `201 { code, ownerKey }` · `400 { error: 'name' }` se o nome não passa (R10) |
| `GET` | `/api/rooms/:code` | 20/min (`RL_LOOKUP`) | — | `200 { code, people, capacity, fixed?: { name } }` ★ campo `fixed` · `404` não existe · ★ `410 { reason: 'deleted' \| 'expired' }` |
| `POST` | `/api/rooms/status` | 20/min (`RL_LOOKUP`, **1 por chamada**) | `{ "codes": ["k7f-q2x", …] }` (1–5, válidos, sem repetir) | ★ `200 { rooms: RoomStatus[] }` na mesma ordem · `400` lista inválida |
| `PATCH` | `/api/rooms/:code` | 10/min (`RL_CREATE`) | `{ "name": "…" }` + `Authorization: Bearer <ownerKey>` | ★ `200 { name }` · `400 name` · `403` chave errada ou sala rápida · `404`/`410` |
| `DELETE` | `/api/rooms/:code` | 10/min (`RL_CREATE`) | `Authorization: Bearer <ownerKey>` | ★ `204` · `403` · `404`/`410` |

```ts
type RoomStatus =
  | { code: string; state: 'ok'; name: string | null; people: number; streaming: number }  // name null = sala rápida
  | { code: string; state: 'gone'; reason: 'deleted' | 'expired' }
  | { code: string; state: 'notfound' };
```

- `RoomStatus` **nunca** traz apelidos, tons ou ids (FR-019, SC-006). `people` conta conectados + na tolerância.
- `status`, `PATCH` e `DELETE` exigem `Origin` do próprio site (são chamados só pela SPA).
- `status` **não** conta como uso para os 30 dias. Só `join` conta.
- `ownerKey`: 64 caracteres hex. Aparece só na resposta de criação. O servidor guarda só o SHA-256.
- Chave errada ou ausente responde `403`. Não esconde se a sala existe: a prévia já mostra isso pra quem tem o código.

Durable Object (RPC interno): `reserve()` → `reserve(kind, fixed?)`, `preview()` (+`fixed`, `gone`), ★ `summary()`, ★ `rename(hash, name)`, ★ `remove(hash)`.
