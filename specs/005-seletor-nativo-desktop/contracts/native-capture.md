# Contrato — ponte de captura nativa (app desktop ↔ site)

Esta é a **única** API nativa que o app expõe à página (emenda 1.2.0 da constituição). Ela só existe na janela `main`, só para a origem `https://espia.diuliano.com` (e `http://127.0.0.1:*` em builds de debug), e é liberada por `src-tauri/capabilities/capture.json` (`remote.urls`). Os tipos estão em [data-model.md](../data-model.md).

## Comandos (`invoke`)

| Comando | Args | Retorno | Erros |
|---|---|---|---|
| `capture_supported` | — | `{ ok: boolean, version: 1, win11: boolean }` | nunca lança |
| `capture_list` | `{ thumbs: boolean }` | `CaptureSource[]` (janelas + monitores) | `"list_failed"` |
| `capture_start` | `{ id, resolution: 720\|1080, fps: 30\|60, audio: boolean, frames: Channel<FrameMsg>, sound: Channel<ArrayBuffer> }` | `{ width, height, audio: AudioKind, audioWarn?: 'unsupported'\|'many_discords' }` | `"source_gone"`, `"capture_failed"` |
| `capture_set_quality` | `{ resolution, fps }` | `{ width, height }` | `"not_live"` |
| `capture_stop` | — | `null` | nunca lança |

- **Detecção pela página**: `capture_supported` só é chamado se `'__TAURI_INTERNALS__' in window`. Qualquer erro, `ok: false` ou `version` diferente de 1 coloca o seletor no modo `'browser'` (fluxo v0.1.2, FR-511).
- **Uso de `capture_list`**: a página chama ao abrir o seletor e a cada 2 s enquanto ele estiver aberto (com `thumbs: true`), e para quando ele fecha. Nada roda em segundo plano. O app só gera miniaturas com a janela do Espia em foco; fora disso, a lista vem com `thumb: null` (os cartões mostram o ícone).
- **Ativação do usuário**: `capture_start` não exige, mas a página só chama depois do clique em Transmitir.

## Buffers compartilhados (vídeo)

1. Antes de responder ao `capture_start` (e depois de cada `capture_set_quality`), o app posta **3 buffers** com `PostSharedBufferToScript`, com acesso somente leitura e `additionalData`:
   ```json
   { "espia": "frame-slot", "gen": 1, "slot": 0, "width": 1920, "height": 1080, "format": "NV12" }
   ```
2. A página guarda os buffers por `(gen, slot)` em `chrome.webview` → `sharedbufferreceived` e descarta os de `gen` anterior (chamando `releaseBuffer`).
3. A cada quadro chega um `FrameMsg` pelo canal `frames`:
   ```ts
   type FrameMsg =
     | { t: 'frame'; gen: number; slot: 0 | 1 | 2; seq: number; ts: number /* µs */ }
     | { t: 'ended'; reason: 'source_closed' | 'stopped' | 'error' }
     | { t: 'audio'; kind: AudioKind; warn?: 'many_discords' | 'unsupported' } // som mudou no ar (vigia do Discord)
     | { t: 'warn'; reason: 'black' }; // os 30 primeiros quadros saíram pretos (jogo em tela cheia exclusiva)
   ```
4. **Regra de concorrência**: o app só reescreve um slot depois de anunciar os outros dois. A página copia o quadro (`new VideoFrame(...)`) no mesmo tick em que recebe o `frame`. Se um `seq` atrasar mais de 2 em relação ao último anunciado, a página o descarta.

## Áudio

> **Atualizado na 010** ([delta](../../010-transmissao-leve/contracts/native-capture-delta.md)): no app 0.1.7+, o som e os avisos de `frames` chegam pelo canal de mensagens do WebView2 (`chrome.webview` "message", `{espia:'pcm', d: base64}` e `{espia:'f', m: FrameMsg}`), não pelos canais do Tauri. Cada mensagem de canal do Tauri executava um script na página, e a página crescia sem parar. A página descarta som com mais de 200 ms na fila.

O canal `sound` recebe `ArrayBuffer`s de PCM **float32 intercalado, 48 kHz, 2 canais, 480 quadros (10 ms)**. A página monta um `AudioData` (`format: 'f32'`) com um timestamp contínuo e o escreve no gerador de áudio. Se o som não vier, o stream sai sem track de áudio.

## O que a página garante

- O `MediaStream` montado (vídeo + áudio opcional) segue o mesmo caminho do stream do `getDisplayMedia`: `mesh.setLocal`, `sendLive` com o `AudioKind` devolvido e `ended` → `stopBroadcast`.
- No modo nativo, a rede de segurança de áudio da 002 (`applyAudioRule`) **não** é aplicada, porque quem decide o som é a `AudioPlan` no app. No modo `'browser'`, ela continua igual.

## Mudança no protocolo do Worker

`AUDIO_KINDS` passa a ser `['tab', 'window', 'pc', 'none']`. O Worker aceita `'pc'` no `live` e repassa. Os outros valores continuam como na 004.
