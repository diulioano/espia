# Contrato — HTTP do Worker

| Método | Rota | Limite por IP | Resposta |
|---|---|---|---|
| `POST` | `/api/rooms` | 10/min | `201 { code }` — código gerado no servidor e reservado por 60 s |
| `GET` | `/api/rooms/:code` | 20/min | `200 { code, people: [{ nickname, tone, live: boolean }], capacity: 10 }` · `404` sem sala · `200` com `people.length === 10` = cheia |
| `GET` | `/ws/:code` | 20/min | `101` upgrade para WebSocket, repassado ao DO da sala (mensagens em [signaling.md](./signaling.md)) |
| `GET` | `/*` | — | Assets da SPA; rotas desconhecidas → `index.html` |

Regras comuns:
- Código fora de `/^[a-hjkmnp-z2-9]{3}-[a-hjkmnp-z2-9]{3}$/` → `400`.
- `POST` e upgrade de WebSocket com `Origin` diferente do próprio host → `403`.
- Limite excedido → `429` com `Retry-After`.
- Não existe endpoint público de TURN: credenciais só chegam pelo `welcome` do WebSocket.
- Cabeçalhos de segurança em todas as respostas: [security.md §5](../security.md).

Durable Object (chamadas internas, não expostas): `reserve()`, `preview()`, `fetch` (upgrade WS).

> **specs/011**: `POST /api/rooms` com `{ kind: 'fixed', name }`, `GET` com `410` para sala fixa apagada/expirada, `POST /api/rooms/status`, `PATCH`/`DELETE /api/rooms/:code` (dono). Ver [http-api-delta.md](../../011-salas-fixas/contracts/http-api-delta.md).
