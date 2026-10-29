# Implementation Plan: Espia Desktop — app leve para Windows

**Branch**: `003-espia-desktop` | **Date**: 2026-10-28 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-espia-desktop/spec.md`

## Summary

Um app Tauri 2 para Windows que abre o Espia publicado numa janela própria usando o WebView2 que já
vem no Windows (sem Chromium embutido → instalador de poucos MB). Uma página local mínima
(`src-tauri/www/`) confere a conexão e segue para `https://espia.espia.workers.dev`, ou mostra "Sem
conexão". Links `espia://s/<código>` abrem a sala; uma segunda abertura foca a janela existente e
chama `window.espiaOpen()` no site, que pergunta antes de sair se você está transmitindo. A barra
flutuante "… está compartilhando …" que o WebView2 mostra ao capturar é escondida pelo app (pedido da
pessoa usuária). O repositório vai para o GitHub (`espia`) com CI, release do instalador por tag e
deploy do Worker. Detalhes em [research.md](./research.md).

## Technical Context

**Language/Version**: Rust 1.90+ (edição 2024) no app; TypeScript 6 + Svelte 5 no site (pequena mudança)

**Primary Dependencies**: Tauri 2.12; plugins `single-instance` (com `deep-link`), `deep-link`, `window-state`, `opener`; `webview2-com` 0.39 e `windows` 0.62 (só Windows, para esconder a barra)

**Storage**: estado da janela (plugin window-state, arquivo local); perfil do WebView2 em `%LOCALAPPDATA%\dev.espia.app`

**Testing**: `cargo test` (links e regra da barra); Vitest para `src/lib/desktop.ts`; teste manual guiado no [quickstart.md](./quickstart.md) (captura real precisa de clique — não automatizável sem capturar a tela real, ver memória do projeto)

**Target Platform**: Windows 10/11 x64 com WebView2 (instalado sob demanda pelo instalador)

**Project Type**: SPA + Worker (existente) + casca desktop `src-tauri/`

**Performance Goals**: instalador ≤ 10 MB; app ocioso numa sala ≤ ~150 MB de RAM total; abrir até a tela inicial ≤ 3 s

**Constraints**: nenhuma API nativa exposta ao site (sem capabilities para a origem remota); nenhum segredo no repositório; sem assinatura de código

**Scale/Scope**: ~4 arquivos Rust, 3 arquivos da página local, 1 módulo novo no site, 3 workflows

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Situação |
|---|---|
| I. Simplicidade (sem instalação obrigatória; app desktop fora do escopo sem emenda) | ⚠️ Exige emenda: o app é **opcional**; o site continua sem instalação. Emenda 1.1.0 abaixo. |
| II. Sem servidor de mídia | ✅ Mesma malha P2P; o app é só outro navegador. |
| III. Efêmero e sem contas | ✅ Nada novo no servidor. O app guarda só tamanho/posição da janela. |
| IV. Camada visual separada | ✅ `src/lib/desktop.ts` é lógica; nenhuma tela muda. A página local do app é isolada em `src-tauri/www`. |
| V. Telas mockáveis | ✅ Nenhuma tela nova no site. A página local do app abre direto no navegador (`src-tauri/www/index.html`) e o estado "sem conexão" aparece desligando a rede. |
| VI. Segredos fora do navegador | ✅ O app não tem chaves; o token da Cloudflare do deploy vive só como segredo do GitHub. |

**Emenda proposta (MINOR → 1.1.0)**: Princípio I passa a dizer que *nenhuma funcionalidade pode exigir*
instalação, mas um app desktop **opcional** que abre o mesmo site é permitido. A restrição técnica
ganha "App desktop: Tauri 2 + WebView2, Windows, carrega o site publicado".

**Re-check pós-design**: ✅ (com a emenda aplicada).

## Project Structure

### Documentation (this feature)

```text
specs/003-espia-desktop/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── deep-link.md       # espia://, args, window.espiaOpen
│   └── release.md         # tags, workflows, segredos
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── lib/desktop.ts         # NOVO: window.espiaOpen + openAction (puro)
├── lib/router.svelte.ts   # + leaveGuard
├── routes/RoomRoute.svelte# registra leaveGuard (transmitindo → pergunta)
└── main.ts                # instala o gancho

src-tauri/                 # NOVO: app desktop
├── Cargo.toml / build.rs / tauri.conf.json
├── icons/ (gerados) · icons-src/icon.svg (castor)
├── www/                   # página local: carregando / sem conexão
│   ├── index.html · loader.css · loader.js
└── src/
    ├── main.rs · lib.rs   # janela, plugins, navegação, links
    ├── links.rs           # parse de links + o que é "dentro do app" (testado)
    └── share_bar.rs       # esconde a barra de compartilhamento do WebView2

test/lib/desktop.test.ts   # NOVO
.github/workflows/
├── ci.yml                 # check + test + build (web) e cargo test (Windows)
├── release.yml            # tag v* → instalador NSIS na Release
└── deploy.yml             # main → wrangler deploy (se houver CLOUDFLARE_API_TOKEN)
```

**Structure Decision**: Layout padrão do Tauri (`src-tauri/` na raiz) ao lado do site existente, um
único `package.json` com scripts `desktop:dev`/`desktop:build`.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Princípio I (app desktop) | Pedido explícito da pessoa usuária; o grupo quer um ícone leve | Atalho de "app" do Edge (PWA) não dá `espia://`, instância única nem esconde a barra de compartilhamento |
| `share_bar.rs` mexe em janelas de outro processo | Pedido explícito: não mostrar a barra "está compartilhando" | Não há opção no WebView2; o evento ScreenCaptureStarting só permite cancelar a captura |
