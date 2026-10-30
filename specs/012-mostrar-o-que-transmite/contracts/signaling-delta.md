# Contrato — Sinalização (mudanças da 012)

Base: [001 signaling.md](../../001-espia-screen-share/contracts/signaling.md), [006 signaling-live.md](../../006-transmitir-camera/contracts/signaling-live.md), [011 signaling-delta.md](../../011-salas-fixas/contracts/signaling-delta.md).

| Mensagem | Mudança |
|---|---|
| `live` (cliente → servidor) | ★ campo opcional `label: string` (≤ 60, texto puro) |
| `LiveInfo` em `welcome.you/peers[].live` e `peer-live.live` | ★ campo opcional `label` |

- O servidor normaliza o `label` como o apelido. Inválido → o `live` vale sem `label` (nunca derruba a mensagem).
- `label` **não** aparece em `GET /api/rooms/:code` (prévia) nem em `POST /api/rooms/status`.
- Cliente antigo: não manda e ignora o campo. Compatível nos dois sentidos.
- `label` nunca contém o título de uma janela: só o nome do programa, o tipo ou a câmera (spec FR-002).
