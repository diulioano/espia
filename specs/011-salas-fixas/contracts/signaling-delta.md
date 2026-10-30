# Contrato — Sinalização (mudanças da 011)

Base: [001 signaling.md](../../001-espia-screen-share/contracts/signaling.md), [006 signaling-live.md](../../006-transmitir-camera/contracts/signaling-live.md).

## Servidor → navegador

| Mensagem | Mudança |
|---|---|
| `welcome` | ★ campo `room: { kind: 'quick' } \| { kind: 'fixed'; name: string }` |
| ★ `{ t: 'room-renamed', name }` | A todos na sala quando o dono renomeia |
| ★ `{ t: 'room-deleted' }` | A todos antes de fechar com `4410` |

Clientes antigos ignoram mensagens desconhecidas e o campo novo (compatível, R11).

## Códigos de fechamento

| Código | Nome | Cliente faz |
|---|---|---|
| ★ `4410` | `deleted` | Para de reconectar. `SignalingEnd = 'deleted'` → tela N2 |
| `4404` | `notFound` | Igual a hoje. Se for lápide, a rota pergunta a prévia (`410`) pra escolher N1/N3 |

## Tempos

| Valor | Antes | Agora | Onde |
|---|---|---|---|
| Tolerância por pessoa (`GRACE_MS`) | 20 s | **60 s** | wrangler vars |
| Sala rápida segurada depois que todos caíram (`HOLD_MS`) | — | **5 min** | wrangler vars ★ |
| Tela "Não deu pra voltar" | — | 60 s em `reconnecting` | cliente |
| Expiração da sala fixa | — | 30 dias sem `join` | DO |
| Lápide | — | 30 dias | DO |

## Reinício do DO (publicação)

Quem está no `roster` e não tem socket vira fantasma com `until = agora + GRACE_MS`. O `resume` dessa pessoa funciona como numa queda comum: mesma identidade, sem `peer-joined`/`peer-left` pros outros.
