# Implementation Plan: Instalar o app pela tela inicial (e instalador com a cara do Espia)

**Branch**: `008-instalar-app` (trabalho direto na `main`, como nas anteriores) | **Date**: 2026-10-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/008-instalar-app/spec.md`

## Summary

Três entregas pequenas:

1. **Convite na tela inicial** (US1, US3). Um bloco discreto abaixo de "Tem um link?", só em navegador de PC com Windows e fora do app, com "Baixar o app" e "Agora não". O botão aponta para um endereço fixo da Release mais nova: `…/releases/latest/download/Espia-setup.exe`. Depois do clique, aparece a dica do SmartScreen.
2. **Instalador com arte** (US2, Q1: A). O mesmo NSIS do Tauri, com:
   - imagem lateral (castor) e faixa do topo em BMP;
   - textos das páginas trocados por um arquivo de ganchos (`installerHooks`), incluído antes das páginas;
   - frases do Tauri em português informal (`customLanguageFiles`);
   - "Abrir o Espia" no fim (já vem marcado no modelo do Tauri).
3. **Ícone da aba** (US4). `favicon.ico` + PNGs (32, 192, 180 para o celular) gerados do ícone do app, e `<link rel="icon">` no `index.html`.

E o workflow de Release passa a publicar também uma cópia com nome fixo, `Espia-setup.exe`, que é o que o link usa.

## Technical Context

**Language/Version**: TypeScript 6 + Svelte 5 (site); NSIS via Tauri CLI 2.12.4 (instalador); YAML (GitHub Actions)

**Primary Dependencies**: nenhuma nova. Imagens geradas uma vez com Pillow (já usado na 007) e versionadas.

**Storage**: `localStorage`: `espia:appinvite` = `"dismissed"` (FR-805).

**Testing**:
- Vitest para `appInviteVisible()` (Windows × Mac × Linux × celular × app × dispensado);
- mocks `?mock=home&scene=invite` / `invitehint` / `default`;
- `npm run tauri build` + abrir o instalador e conferir as telas (roteiro I1–I6).

**Target Platform**: site em qualquer navegador (o convite só no Windows desktop); instalador Windows x64.

**Project Type**: SPA + Worker (existente) + casca desktop `src-tauri/` (só configuração do bundle)

**Performance Goals**: a tela inicial sem rolagem em 1366×768 com o convite (SC-804).

**Constraints**:
- o convite é secundário (FR-803) e a frase "sem instalar nada" continua;
- o instalador cresce menos de 150 KB (2 BMPs);
- nenhuma mudança no Rust.

**Scale/Scope**:
- **Site**: 1 módulo de lógica (`src/lib/app-invite.ts`), 1 componente (`AppInvite.svelte`), ajustes em `HomePage`, `types.ts`, a rota inicial, `prefs.ts`, mocks e `index.html`, e ícones em `public/`.
- **Instalador**: `src-tauri/nsis/` com 2 BMPs, `hooks.nsh` e `PortugueseBR.nsh`, mais `tauri.conf.json`.
- **CI**: 1 passo no `release.yml`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio / restrição | Situação |
|---|---|
| I. Simplicidade (nada exige instalar) | ✅ O convite é opcional e secundário. Criar e entrar em sala seguem iguais, e o texto "sem instalar nada" fica. O app continua opcional (1.1.0). |
| II. Sem servidor de mídia | ✅ Nada muda. |
| III. Efêmero e sem contas | ✅ Só uma preferência local (`espia:appinvite`). O download vem do GitHub, sem nada no Worker. |
| IV. Camada visual separada | ✅ A decisão de mostrar fica em `src/lib/app-invite.ts`. O `AppInvite.svelte` recebe `HomeView.appInvite` pronto e emite `downloadApp`/`dismissAppInvite`. |
| V. Telas mockáveis | ✅ Cenas `invite` e `invitehint` na tela inicial (FR-809). |
| VI. Segredos fora do navegador | ✅ Sem segredos. O link é público. |
| Restrição técnica do app (APIs nativas) | ✅ Nada novo exposto. Só a aparência do instalador. |

**Re-check pós-design**: ✅ sem violações.

## Project Structure

### Documentation (this feature)

```text
specs/008-instalar-app/
├── spec.md
├── plan.md            # este arquivo
├── research.md        # R1–R6
├── data-model.md      # convite (estados), preferência, textos
├── quickstart.md      # automático, mocks, roteiro I1–I6
├── contracts/
│   └── download.md    # endereço fixo do instalador + asset da Release
└── checklists/requirements.md
```

### Source Code (repository root)

```text
index.html                          # <link rel="icon">, apple-touch-icon, theme-color
public/
├── favicon.ico                     # NOVO: 16/32/48
├── icon-192.png                    # NOVO
└── apple-touch-icon.png            # NOVO: 180×180, fundo cheio
src/
├── lib/
│   ├── app-invite.ts               # NOVO: appInviteVisible(env), DOWNLOAD_URL, RELEASES_URL
│   ├── prefs.ts                    # + loadAppInviteDismissed / dismissAppInvite
│   └── types.ts                    # HomeView.appInvite?: 'offer' | 'hint'; HomeActions + downloadApp, dismissAppInvite
├── routes/HomeRoute.svelte        # monta HomeView.appInvite e as ações
├── ui/
│   ├── components/AppInvite.svelte # NOVO
│   └── pages/HomePage.svelte       # mostra o AppInvite abaixo de "Tem um link?"
└── mocks/home.ts, MockHome.svelte  # cenas invite / invitehint
test/lib/app-invite.test.ts         # NOVO

src-tauri/
├── tauri.conf.json                 # nsis.sidebarImage, headerImage, installerHooks, customLanguageFiles
└── nsis/                           # NOVO
    ├── sidebar.bmp                 # 164×314, castor + cores do Espia
    ├── header.bmp                  # 150×57
    ├── hooks.nsh                   # títulos/textos das páginas de boas-vindas e fim
    └── PortugueseBR.nsh            # frases do Tauri em tom informal

.github/workflows/release.yml       # + cópia "Espia-setup.exe" na Release
specs/003-espia-desktop/contracts/release.md   # registra o asset de nome fixo
```

**Structure Decision**: a mesma estrutura de sempre. A lógica do convite é uma função pura com entrada injetável (UA, `userAgentData`, se está no app, preferência), testável sem navegador. O instalador muda só por configuração e arquivos em `src-tauri/nsis/`, sem modelo próprio de NSIS.

## Ordem de entrega (mock → real)

1. Ícones: gerar e ligar no `index.html` (US4). Conferir no Edge.
2. **Visual**: `AppInvite.svelte` + cenas de mock. Conferir em 1440×900 e 1366×768.
3. Lógica: `app-invite.ts` + prefs + testes. Ligar na tela inicial real.
4. Release: o passo do asset fixo + subir `Espia-setup.exe` na `v0.1.4` já publicada (com OK da pessoa), para o link funcionar já.
5. Instalador: BMPs, `hooks.nsh`, `PortugueseBR.nsh`, `tauri.conf.json`. `npm run tauri build` e o roteiro I1–I6.
6. Versão 0.1.5, deploy do site, tag (o workflow gera o instalador novo com a arte).

## Riscos

| Risco | Mitigação |
|---|---|
| O `latest/download/Espia-setup.exe` dar 404 numa Release sem o asset | O passo novo no `release.yml` e o upload manual na `v0.1.4`. Conferir com `curl -I` (quickstart). |
| Os `!define MUI_*` do `hooks.nsh` não pegarem (ordem do modelo do Tauri) | R3: o modelo inclui os ganchos na linha 36, antes de `MUI_PAGE_WELCOME` (linha 171). Se mudar numa versão futura do Tauri, cair para `nsis.template`. |
| Detectar Windows errado (UA congelado, `userAgentData` ausente no Firefox) | R2: `userAgentData.platform` quando existe, senão `/Windows NT/` no UA, e celular fora. Na dúvida, não mostra. |
| O BMP sair com cores erradas ou serrilhado no NSIS | BMP 24 bits sem alfa, já no tamanho exato (R4). Conferir num Windows com escala de 150%. |
| SmartScreen assustar | A dica da US1, com o texto que a Release já usa. |
