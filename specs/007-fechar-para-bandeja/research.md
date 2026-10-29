# Research: Fechar o app sem sair (bandeja)

## R1. Ícone na bandeja

- **Decision**: `tauri::tray::TrayIconBuilder` (feature `tray-icon` do crate `tauri`, que já traz `tray-icon` no `Cargo.lock`). Menu com `tauri::menu::{Menu, MenuItem, CheckMenuItem, PredefinedMenuItem::separator}`. Clique esquerdo (`TrayIconEvent::Click { button: Left, button_state: Up }`) mostra a janela; o menu abre no clique direito (`show_menu_on_left_click(false)`).
- **Rationale**: é a API oficial do Tauri 2, sem dependência nova e sem código Win32 próprio.
- **Alternatives**: `Shell_NotifyIconW` direto (mais código, e o Tauri já faz); `tauri-plugin-positioner` (desnecessário).

## R2. WebView2 escondido continua transmitindo?

- **Decision**: esconder com `window.hide()` e **testar** (quickstart B2/B3). O WebView2 trata a janela escondida como o Chromium trata uma aba em segundo plano ou ocluída: reduz timers de JS para ~1 por segundo, mas **não congela** páginas com `RTCPeerConnection` ativo nem com áudio tocando. A malha P2P, o envio da captura do navegador e da câmera seguem pelo processo de rede/GPU, não por timers.
- **Ponto de atenção**: a captura nativa da 005 entrega quadros por `Channel` (mensagem da ponte, não timer) e o `MediaStreamTrackGenerator` escreve na hora. Deve seguir igual; medir os quadros descartados no log `[espia] captura nativa:` com a janela escondida.
- **Plano B** (só se o teste mostrar queda): passar `--disable-background-timer-throttling --disable-renderer-backgrounding --disable-backgrounding-occluded-windows` em `additional_browser_args` também no release; ou manter o controller do WebView2 com `IsVisible = true` enquanto houver transmissão.
- **Alternatives**: minimizar em vez de esconder (fica na barra de tarefas, contraria Q1: A).

## R3. O X

- **Decision**: `.on_window_event` no builder: `WindowEvent::CloseRequested { api, .. }` → se `closeOnX` desligado, `api.prevent_close()` + `window.hide()` (+ aviso da primeira vez). Se ligado, segue o mesmo caminho do "Sair" (R5).
- **Rationale**: janela escondida some da barra de tarefas sozinha (`SW_HIDE`); não precisa de `set_skip_taskbar`.
- **Desligamento do Windows**: o Windows manda `WM_QUERYENDSESSION`/`WM_ENDSESSION`, não `WM_CLOSE`. Então o `CloseRequested` não dispara e nada pergunta (FR-705). O processo é encerrado pelo sistema.

## R4. Saber se está ao vivo

- **Decision**: comando novo `app_set_live(live: bool)`, chamado pela página sempre que `myBroadcast` passa de nulo para preenchido e vice-versa (inclui câmera e captura do navegador). Estado em `tauri::State<TrayState>` (`AtomicBool`). Recarregar a página (`PageLoadEvent::Started`) zera para `false`, como a 005 já faz com a captura.
- **Rationale**: só a página sabe de todas as formas de transmitir. Booleano sem retorno, mínimo de superfície.
- **Alternatives**: olhar só a sessão da captura nativa (perde câmera e modo navegador); `eval` com retorno (o Tauri não devolve valor do `eval`); título da página (frágil).
- **Exige**: emenda 1.4.0, permissão `allow-app-set-live` na capability `capture.json` (mesma origem e janela), `build.rs` com o comando novo.

## R5. Sair e confirmação

- **Decision**: "Sair" (e o X com `closeOnX`) → se ao vivo, `MessageBoxW(hwnd, "Você está transmitindo. Sair mesmo assim?", "Espia", MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2)` numa thread fora do loop de eventos (para não travar a bandeja); "Sim" → `app.exit(0)`. Não ao vivo → `app.exit(0)` direto. Antes de sair, a janela é mostrada se estiver escondida, para a caixa ter dono visível.
- **Rationale**: o `windows` crate já tem `Win32_UI_WindowsAndMessaging`; `tauri-plugin-dialog` seria dependência a mais para uma caixa só.
- **Saída da sala**: ao fechar, o WebSocket cai e o Durable Object avisa a sala (comportamento atual de fechar o app). Nada novo.
- **Posição da janela**: o `tauri-plugin-window-state` salva no `CloseRequested`; como agora ele é impedido, chamar `app.save_window_state(StateFlags::all())` antes do `exit`.

## R6. Aviso da primeira vez

- **Decision**: `tauri-plugin-notification`, chamado só do Rust: título "O Espia continua aberto", corpo "Ele fica aqui perto do relógio (às vezes na setinha ^). Pra sair de vez, clique com o botão direito no ícone e escolha Sair." Marca `trayNoticeShown = true` em seguida.
- **Rationale**: o `tray-icon` do Tauri não faz balão; o plugin usa a notificação do Windows. Nenhuma permissão é dada à página (a capability não inclui `notification:*`).
- **Alternatives**: balão `NIF_INFO` do `Shell_NotifyIconW` (precisa do HWND interno do tray-icon, não exposto); mostrar o aviso na página antes de esconder (atrasaria o X).

## R7. Preferências

- **Decision**: `app.path().app_config_dir()/tray.json` com `{ "closeOnX": false, "trayNoticeShown": false }`, lido no `setup` e gravado a cada mudança. Arquivo ausente ou inválido → padrão. Parse/serialização em funções puras com teste.
- **Alternatives**: `tauri-plugin-store` (dependência a mais para dois booleanos); `localStorage` da página (o Rust não lê, e o X precisa decidir sem a página).

## R8. Segunda abertura e links

- **Decision**: o `focus()` atual do `single_instance` já faz `unminimize` + `show` + `set_focus`. Funciona com a janela escondida sem mudança. O link de sala segue o fluxo da 003 (`espiaOpen` pergunta se está transmitindo).
- **Clique no ícone**: o mesmo `focus()`.
