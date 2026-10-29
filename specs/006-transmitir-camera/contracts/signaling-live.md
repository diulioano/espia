# Contrato — mudança no `live` da sinalização (006)

Complementa `specs/001-espia-screen-share/contracts/signaling.md`. Os tipos estão em [data-model.md](../data-model.md).

## Cliente → Worker

```ts
{ t: 'live'; resolution: 720 | 1080; fps: 30 | 60; audio: AudioKind; source?: 'screen' | 'camera' }
type AudioKind = 'tab' | 'window' | 'pc' | 'device' | 'none';
```

| Entrada | Resultado |
|---|---|
| sem `source` (cliente antigo) | aceito, repassado como `source: 'screen'` |
| `source: 'camera'` | aceito e repassado |
| `source` com outro valor | `error` `bad-message` |
| `audio: 'device'` | aceito e repassado |
| `audio: 'system'` (continua proibido) | `error` `bad-message` |
| `audio: boolean` (cliente antigo) | convertido como hoje (`true` → `'tab'`) |

## Worker → clientes

`peer-live` / `welcome.peers[].live` ganham `source: 'screen' | 'camera'`, sempre presente.

## Exibição (quem assiste)

- Subtítulo do tile: `"<res>p · <fps> fps · com som|sem som"`. Igual a hoje.
- Rótulo/nome: `source === 'camera'` → "câmera de <apelido>". Senão, como hoje.

## Exibição (quem transmite, card "Você está ao vivo")

`audioLabelText('device')` → "com som do dispositivo". O card mostra "câmera · <nome do dispositivo>".
