# Contrato — remover pessoa (014 US4)

Base: [011 http-api-delta.md](../../011-salas-fixas/contracts/http-api-delta.md) e [011 signaling-delta.md](../../011-salas-fixas/contracts/signaling-delta.md).

## HTTP

| Método | Rota | Limite por IP | Corpo | Resposta |
|---|---|---|---|---|
| `POST` | `/api/rooms/:code/remove` | 10/min (`RL_CREATE`) | `{ "peerId": "<16 hex>" }` + `Authorization: Bearer <ownerKey>` | `204` (também se a pessoa já tinha saído) · `400 { error: 'peer' }` · `403` chave errada ou sala rápida · `404` / `410 { reason }` |

- Exige `Origin` do próprio site, como `PATCH`/`DELETE`.
- `peerId`: exatamente o formato que o servidor gera (`randomHex(8)` = 16 caracteres hex).
- RPC do DO: `removePerson(ownerHash: string, peerId: string): Promise<OwnerResult>`.

## WebSocket

```ts
// ServerMessage, só para quem foi removido, logo antes do fechamento:
| { t: 'removed' }

// CLOSE
removed: 4403  // o dono tirou você da sala: não reconecta
```

- Os outros recebem o `peer-left` de sempre (igual a "Sair da sala").
- O cliente trata `removed` ou 4403 como fim `'removed'` (`SignalingEnd`): não reconecta, apaga a sessão guardada e vai pro início com o aviso.
- Cliente antigo: não conhece 4403, tenta `resume`, recebe `resume-failed` e entra como pessoa nova (R7).

## Garantias

- O servidor não guarda quem foi removido. Abrir o link de novo entra normalmente (FR-017).
- Sem a chave do dono, nada acontece (FR-014). Os testes do Worker cobrem: chave errada (`403`), sala rápida (`403`), pessoa conectada, pessoa fantasma (em queda) e pessoa inexistente (`204`).
