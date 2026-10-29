# Contract delta: ponte de captura nativa (010)

Muda **dentro** de [`specs/005-seletor-nativo-desktop/contracts/native-capture.md`](../../005-seletor-nativo-desktop/contracts/native-capture.md). Nenhum comando novo, nenhuma capability nova, sem emenda à constituição.

## Transporte das mensagens do app para a página

| | Antes (005) | Agora (010, app 0.1.7+) |
|---|---|---|
| Som | canal `sound` do Tauri, bytes crus (10 ms, 3.840 bytes) → `eval` + `fetch` por pedaço | `chrome.webview` "message": `{"espia":"pcm","d":"<base64 de 10 ms f32 2 canais 48 kHz>"}` |
| Avisos (`frame`, `audio`, `warn`, `ended`) | canal `frames` do Tauri → `eval` por aviso | `chrome.webview` "message": `{"espia":"f","m":<FrameMsg>}` |
| Buffers de vídeo | `sharedbufferreceived` | igual |

Motivo: cada mensagem de canal do Tauri executa um script na página, e 100 + 60 por segundo faziam a página crescer sem parar (`relatorio.md`).

`capture_start` continua recebendo `frames` e `sound` (canais do Tauri), que o app 0.1.7 ignora.

## Página (`src/lib/native-capture.ts`)

- `Bridge.onHostMessage(cb)` ouve `chrome.webview` "message" e entrega só `espia: 'pcm' | 'f'`;
- os canais do Tauri continuam ligados aos mesmos tratadores (app 0.1.6);
- som: descarta o pedaço se houver mais de **200 ms** na fila do gerador (`shouldDropSound`), mas o tempo continua andando; conta `somDescartado` no log de 10 s.

## Compatibilidade

- Site novo + app 0.1.6: funciona (canais do Tauri; o descarte também vale).
- App 0.1.7 + site antigo: **sem som e sem imagem** (o site antigo só ouve os canais do Tauri). Por isso o site sobe antes do app 0.1.7.
