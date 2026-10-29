# Tasks: Instalar o app pela tela inicial (e instalador com a cara do Espia)

**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/download.md](./contracts/download.md), [quickstart.md](./quickstart.md)

**Testes**: Vitest para a regra pura do convite (o plano pede). Instalador e ícone: roteiro manual I1–I6 e as conferências do quickstart.

## Phase 1: Setup

- [x] T801 [P] Gerar os ícones do site a partir de `src-tauri/icons/icon.png` (Pillow): `public/favicon.ico` (16, 32, 48), `public/icon-192.png` e `public/apple-touch-icon.png` (180×180, fundo cheio na cor de fundo de `src/ui/tokens.css`, sem transparência) (R5)
- [x] T802 [P] Gerar as imagens do instalador a partir de `src-tauri/icons/icon.png` e das cores de `src/ui/tokens.css`: `src-tauri/nsis/sidebar.bmp` (164×314, fundo escuro do Espia, castor grande, "Espia") e `src-tauri/nsis/header.bmp` (150×57, fundo claro, castor pequeno à direita), BMP 24 bits sem alfa (R4)

## Phase 2: Foundational

- [x] T803 Criar `src/lib/app-invite.ts`: `DOWNLOAD_URL = 'https://github.com/DiulianoVargas/espia/releases/latest/download/Espia-setup.exe'`, `RELEASES_URL = 'https://github.com/DiulianoVargas/espia/releases/latest'`, `appInviteVisible({ platform, mobile, ua, inApp, dismissed })` pura conforme R2 (`inApp` ou `dismissed` → false; `platform === 'Windows' && mobile !== true` → true; sem `platform`: `/Windows NT/` e sem `/Mobile|Android/` no UA → true; senão false) e `readInviteEnv()` que lê `navigator`/`window`
- [x] T804 [P] Em `src/lib/prefs.ts`: `loadAppInviteDismissed()` e `saveAppInviteDismissed()` com a chave `espia:appinvite` = `"dismissed"` (no padrão `espia:` das outras prefs) (data-model: "gravada no 'Agora não'; nunca apagada pelo site"), no mesmo padrão try/catch das outras prefs
- [x] T805 [P] Em `src/lib/types.ts`: `HomeView.appInvite?: 'offer' | 'hint'` e, em `HomeActions`, `downloadApp(): void` e `dismissAppInvite(): void`
- [x] T806 [P] Testes em `test/lib/app-invite.test.ts`: Windows Chrome (`userAgentData`), Windows Firefox (só UA), Mac, Linux, Android, iPhone, Windows com `mobile: true`, dentro do app, dispensado

**Checkpoint**: `npm test` passa com a regra do convite.

## Phase 3: User Story 4 — Ícone da aba (P1)

**Goal**: o castor na aba, nos favoritos e no atalho do celular.

**Independent Test**: `/favicon.ico` responde `image/x-icon`; a aba do Edge mostra o castor.

- [x] T807 [US4] Em `index.html`: `<link rel="icon" href="/favicon.ico" sizes="any">`, `<link rel="icon" type="image/png" sizes="192x192" href="/icon-192.png">`, `<link rel="apple-touch-icon" href="/apple-touch-icon.png">` e `<meta name="theme-color">` com o fundo do Espia; conferir com `npm run dev:worker` que `/favicon.ico` sai como imagem

## Phase 4: User Story 1 — Baixar o app pela tela inicial (P1) 🎯 MVP

**Goal**: convite com "Baixar o app" que baixa o instalador mais novo em 1 clique.

**Independent Test**: mock `?mock=home&scene=invite` e, no Windows, o clique baixa o `Espia-setup.exe` (quickstart I1, I2).

- [x] T808 [US1] Criar `src/ui/components/AppInvite.svelte`: recebe `state: 'offer' | 'hint'` e emite `ondownload`/`ondismiss`; textos do `data-model.md` ("Tem Windows? Baixe o app do Espia", a linha do que traz, "Baixar o app", "Agora não", dica do SmartScreen, "Não baixou? Abra a página de versões" com `RELEASES_URL` em `target="_blank" rel="noopener"`); visual secundário (borda sutil, botão `btn-secondary`), só com os tokens de `src/ui/tokens.css`, ícone `Download` do `@lucide/svelte`; o link de download é um `<a href={DOWNLOAD_URL}>` (download direto) que também chama `ondownload`
- [x] T809 [US1] Em `src/ui/pages/HomePage.svelte`: mostrar `<AppInvite>` abaixo do formulário "Tem um link?" quando `view.appInvite` existir; a frase "Sem conta, sem instalar nada" continua (FR-803); conferir que cabe sem rolagem em 1366×768 (SC-804)
- [x] T810 [P] [US1] Mocks em `src/mocks/home.ts` e `src/mocks/MockHome.svelte`: cenas `invite` ("Convite do app") e `invitehint` ("Convite: baixando"), com `downloadApp` → `hint` e `dismissAppInvite` → some (FR-809)
- [x] T811 [US1] Em `src/routes/HomeRoute.svelte`: `appInvite` inicial = `appInviteVisible({ ...readInviteEnv(), dismissed: loadAppInviteDismissed() }) ? 'offer' : undefined`; `downloadApp` → `'hint'`; `dismissAppInvite` → `saveAppInviteDismissed()` + `undefined`
- [x] T812 [US1] Em `.github/workflows/release.yml`, depois do `tauri-action`: passo `shell: bash` que copia `src-tauri/target/release/bundle/nsis/Espia_*_x64-setup.exe` para `Espia-setup.exe` e roda `gh release upload "${{ github.ref_name }}" Espia-setup.exe --clobber` (só quando `github.ref_type == 'tag'`; `GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}`); registrar o asset em `specs/003-espia-desktop/contracts/release.md` (contracts/download.md)
- [x] T813 [US1] Subir `Espia-setup.exe` na Release `v0.1.4` já publicada (`gh release download v0.1.4 -p "Espia_*_x64-setup.exe"` + `gh release upload v0.1.4 Espia-setup.exe`) — **pedir OK da pessoa usuária antes** — e conferir o contrato com `curl -sIL` terminando em `200`

## Phase 5: User Story 3 — Não incomodar (P2)

**Goal**: sem convite no celular, Mac, Linux e no app; "Agora não" vale para sempre nesse navegador.

**Independent Test**: os casos do T806 e o quickstart I3, I4.

- [x] T814 [US3] Conferir no Edge com a emulação de celular e dentro do app (`target/debug/espia.exe` com `ESPIA_START`) que o convite não aparece; "Agora não" + recarregar não traz de volta (a lógica já vem de T803/T811)

## Phase 6: User Story 2 — Instalador com a cara do Espia (P1)

**Goal**: assistente do Windows com a arte e os textos do Espia, "Abrir o Espia" marcado no fim.

**Independent Test**: `npm run tauri build`, rodar o instalador (quickstart I5, I6).

- [x] T815 [US2] Criar `src-tauri/nsis/hooks.nsh` com `!define MUI_WELCOMEPAGE_TITLE`, `MUI_WELCOMEPAGE_TEXT`, `MUI_FINISHPAGE_TITLE`, `MUI_FINISHPAGE_TEXT` e `MUI_FINISHPAGE_RUN_TEXT "Abrir o Espia"` com os textos do `data-model.md` (R3; `$\r$\n` para quebrar linha)
- [x] T816 [P] [US2] Criar `src-tauri/nsis/PortugueseBR.nsh` a partir do arquivo oficial do Tauri 2.12.4 (`crates/tauri-bundler/src/bundle/windows/nsis/languages/PortugueseBR.nsh`, mesmas chaves `LangString`), reescrito em português informal e com os erros do original corrigidos ("primeirl", "executare")
- [x] T817 [US2] Em `src-tauri/tauri.conf.json` (`bundle.windows.nsis`): `"sidebarImage": "nsis/sidebar.bmp"`, `"headerImage": "nsis/header.bmp"`, `"installerHooks": "nsis/hooks.nsh"`, `"customLanguageFiles": { "PortugueseBR": "nsis/PortugueseBR.nsh" }`
- [x] T818 [US2] `npm run tauri build`; abrir o instalador gerado e conferir as telas de boas-vindas, do meio e do fim (sem instalar por cima do app da pessoa sem avisar); anotar o tamanho (meta: + < 150 KB). Se os `!define` não pegarem, aplicar o plano B do R3 (`nsis.template`)

## Phase 7: Polish

- [x] T819 `npm run check`, `npm test` e os mocks `default`, `invite` e `invitehint` conferidos em 1440×900 e 1366×768
- [x] T820 Versão `0.1.5` em `package.json`, `src-tauri/tauri.conf.json` e `src-tauri/Cargo.toml`
- [x] T821 Deploy do site (`npm run deploy`, ou pedir para a pessoa rodar se o modo automático bloquear) e conferir `curl -sI …/favicon.ico` → `image/x-icon`
- [x] T822 Roteiro I1–I6 de `quickstart.md` — **precisa da pessoa usuária** (I5/I6: instalar de verdade)
- [x] T823 Commit, tag `v0.1.5` (o workflow gera o instalador com a arte e o `Espia-setup.exe`) e `specs/008-instalar-app/relatorio.md` no formato do da 007

## Dependencies

- T801 → T807 (US4). T802 → T817 (US2).
- T803–T805 → T808–T811 (US1) → T814 (US3).
- T812 independe do site; T813 depende do OK da pessoa.
- T815–T817 → T818. Polish por último; a tag (T823) depois do deploy (T821).

## Parallel

- T801 e T802; T804, T805 e T806; T810 com T808; T816 com T815.

## Implementation Strategy

1. **MVP**: US4 (ícone) + US1 (convite + link fixo). Já resolve "não tem ícone" e "instalar pela tela inicial" com o instalador atual.
2. US3 (conferência) e US2 (instalador com arte).
3. Polish, deploy e v0.1.5.
