# Research: Espia Desktop

## R1. Tauri vs Electron vs PWA

- **Decision**: Tauri 2 (Rust + WebView2 do sistema).
- **Rationale**: Prioridade é ser leve. Tauri não embute Chromium: instalador de poucos MB e RAM de um
  WebView2 (que o Windows 11 já tem rodando para outros apps). WebView2 é Chromium (Edge), então
  `getDisplayMedia`, WebRTC e áudio de aba/janela se comportam como no Edge — mesmo alvo da 001.
- **Alternatives**: Electron (~100 MB de instalador, Chromium próprio); PWA instalada pelo Edge (zero
  código, mas sem `espia://`, sem instância única, sem controle da barra de compartilhamento).

## R2. Carregar o site publicado vs embutir o `dist/`

- **Decision**: Carregar `https://espia.espia.workers.dev` (página local só para "carregando/sem conexão").
- **Rationale**: Mesma origem do site → WebSocket, `/api`, CSP e `_headers` funcionam sem mudança no
  Worker; melhorias do site chegam ao app sem reinstalar (dispensa auto-update). Sem internet o Espia não
  serve para nada mesmo.
- **Alternatives**: Embutir `dist/` (origem `http://tauri.localhost`): exigiria URL absoluta da API, CORS e
  checagem de Origin no WebSocket, e app desatualizado a cada mudança do site.

## R3. Segurança da janela

- **Decision**: Sem capabilities (nenhum comando do Tauri acessível à página). `on_navigation` só deixa
  navegar na origem do Espia e na página local; o resto abre no navegador padrão; `window.open` vira
  navegador padrão. CSP própria na página local (`connect-src` só para o Espia).
- **Rationale**: FR-016. A origem remota nunca recebe IPC (o Tauri só libera IPC remoto com capability
  `remote`, que não existe).

## R4. Links `espia://` e instância única

- **Decision**: `tauri-plugin-deep-link` (registra `espia` no instalador NSIS; em debug,
  `register_all()`), `tauri-plugin-single-instance` com a feature `deep-link`. No Windows o link chega
  como argumento de linha de comando; `links::room_path_from_args` valida (`xxx-xxx`, mesmo alfabeto do
  site) e o app chama `window.espiaOpen(path)` no site. Ver [contracts/deep-link.md](./contracts/deep-link.md).
- **Rationale**: Navegação SPA (sem recarregar) e a pergunta "você está transmitindo" ficam no site, que
  sabe o estado. Se a página aberta for antiga (sem o gancho), o app cai em `location.assign`.

## R5. Barra "… está compartilhando …" do WebView2 (pedido da pessoa usuária)

- **Decision**: Esconder a barra pelo app (`src-tauri/src/share_bar.rs`): ao receber
  `ICoreWebView2_27::ScreenCaptureStarting`, por até 120 s (o seletor fica aberto nesse meio-tempo)
  procurar a cada 150 ms uma janela visível do processo do navegador do WebView2, com até 240 px de
  altura e o host do Espia no título, e escondê-la com `ShowWindowAsync(SW_HIDE)`.
- **Rationale**: O WebView2 mostra a mesma barra flutuante do Chrome e não tem opção para desligá-la;
  `ScreenCaptureStarting` só permite cancelar a captura; `--use-fake-ui-for-media-stream` quebra o áudio
  (e captura sem seletor — proibido pela memória do projeto). A mesma técnica é usada pelo OwnCord
  ([PR #2426](https://github.com/J3vb/OwnCord/pull/2426)). Parar continua possível pelo botão do Espia.
- **Riscos**: depende do título da barra conter o host; se o WebView2 mudar, a barra volta a aparecer
  (falha segura). Builds de debug listam as janelas vistas quando não acham a barra.
- **Spike (2026-10-28)**: WebView2 156 neste PC: `getDisplayMedia` existe e capturou uma janela com
  `--auto-select-desktop-capture-source`. Sem clique real (janela sem foco/sem gesto) a chamada fica
  pendente sem mostrar seletor, então a verificação final da barra é manual (quickstart Q3).
  Capturar a própria janela do app pode travar a janela — não usar isso em testes.

## R6. Instalador

- **Decision**: NSIS, `installMode: currentUser` (sem admin), `webviewInstallMode: downloadBootstrapper`
  (baixa o WebView2 só se faltar), idioma PortugueseBR, perfil `release` com LTO/`opt-level = "s"`/strip.
- **Alternatives**: MSI (exige admin, maior); embutir o WebView2 fixo (+150 MB).

## R7. GitHub

- **Decision**: Repositório público `DiulianoVargas/espia` (a galera baixa o instalador sem conta).
  `ci.yml` (push/PR: `npm ci`, `check`, `test`, `build`; `cargo test` no Windows), `release.yml`
  (`tauri-apps/tauri-action` em tag `v*`, publica a Release), `deploy.yml` (push na `main`:
  `wrangler deploy` só se o segredo `CLOUDFLARE_API_TOKEN` existir). Ver [contracts/release.md](./contracts/release.md).
- **Rationale**: Nada secreto no código (TURN_KEY_ID não é segredo; o token do TURN fica no Worker).

## R8. Renomear a pasta

- **Decision**: `Documents\screen-share` → `Documents\espia`, por último (a sessão roda dentro dela). A
  memória do Claude Code fica numa pasta indexada pelo caminho; é copiada para a nova.
