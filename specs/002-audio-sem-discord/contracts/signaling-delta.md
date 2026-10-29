# Contrato — mudança na sinalização (delta sobre 001/contracts/signaling.md)

## `LiveInfo`

```ts
type AudioKind = 'tab' | 'window' | 'none';
interface LiveInfo { resolution: 720 | 1080; fps: 30 | 60; audio: AudioKind; startedAt: number }
```

## Cliente → Durable Object: `live`

| Campo | Antes | Agora |
|---|---|---|
| `audio` | `boolean` | `'tab' \| 'window' \| 'none'` |

**Compatibilidade (temporária)**: o Worker aceita `audio: boolean` de clientes antigos e converte antes de guardar e repassar:
- `true` → `'tab'`
- `false` → `'none'`

Qualquer outro valor → `error bad-message` (regra de validação existente).

## Durable Object → Cliente

`peer-live.live.audio` e `welcome.peers[].live.audio` passam a ser sempre `AudioKind`. O Worker nunca repassa o booleano.

## Testes de contrato (Worker)

- `live` com `audio: 'window'` → os outros recebem `peer-live` com `audio: 'window'`.
- `live` com `audio: true` (cliente antigo) → os outros recebem `audio: 'tab'`; com `false`, recebem `'none'`.
- `live` com `audio: 'system'` → `error bad-message`.
