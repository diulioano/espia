# Contrato: bandeja do app desktop (007)

Vale só dentro do app desktop, na janela `main`, para a origem `https://espia.espia.workers.dev`. No navegador nada disso existe e o site funciona igual (FR-708).

## Página → app

### `app_set_live`

```ts
import { invoke } from '@tauri-apps/api/core';
await invoke('app_set_live', { live: boolean });
```

- Sem retorno. Chamado quando a transmissão própria começa (`live: true`) e quando para (`live: false`), qualquer que seja a fonte (janela/tela nativa, captura do navegador, câmera).
- Idempotente: repetir o mesmo valor não faz nada.
- Fora do app (`!('__TAURI_INTERNALS__' in window)`) a página não chama.
- Permissão: `allow-app-set-live`, adicionada a `src-tauri/capabilities/capture.json` (mesma `remote.urls` e `windows: ["main"]`). `build.rs` lista o comando em `AppManifest::commands`.
- O app confere a origem da janela como na 005 antes de aceitar.

## App → página

### `window.espiaStopLive()`

```ts
(window as Window & { espiaStopLive?: () => void }).espiaStopLive = () => room?.stopGoLive();
```

- Instalado por `src/lib/desktop.ts` ao lado do `espiaOpen` (003).
- Chamado pelo item "Parar transmissão" do menu: `window.eval("window.espiaStopLive?.()")`.
- Para a transmissão como o botão "Parar" (avisa a sala com `unlive`), e a página chama `app_set_live(false)` em seguida.
- Sem sala ou sem transmissão: não faz nada.

## Arquivo `tray.json`

Caminho: `app_config_dir()/tray.json` (no Windows, `%APPDATA%\<identifier>\`).

```json
{ "closeOnX": false, "trayNoticeShown": false }
```

Ver [data-model.md](../data-model.md) para as regras.

## Fora do contrato

- A página **não** ganha acesso a notificações, diálogos, menu ou ícone. Tudo isso é chamado só pelo Rust.
- Nenhum outro comando novo.
