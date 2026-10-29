# Implementation Plan: Fechar o app sem sair (bandeja)

**Branch**: `007-fechar-para-bandeja` (trabalho direto na `main`, como na 004–006) | **Date**: 2026-10-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/007-fechar-para-bandeja/spec.md`

## Summary

No app desktop, o X passa a **esconder** a janela em vez de fechar. O Espia fica num ícone na bandeja do Windows (perto do relógio), continua na sala e não para a transmissão. O menu do ícone tem "Abrir o Espia", "Parar transmissão" (só ao vivo), "Fechar no X" (liga/desliga) e "Sair". Sair com transmissão ativa pede confirmação.

Como funciona:

- **Bandeja**: `TrayIconBuilder` do próprio Tauri (feature `tray-icon`). Clique esquerdo traz a janela de volta. O menu é montado em Rust.
- **X**: `WindowEvent::CloseRequested` → `prevent_close()` + `hide()` (janela escondida não tem botão na barra de tarefas).
- **Ao vivo**: a página avisa o app com um comando novo, `app_set_live(live)`. O app troca o ícone e a dica ("Espia · ao vivo") e decide se "Sair" pergunta. **Exige a emenda 1.4.0** (a constituição só permite a ponte de captura).
- **Parar transmissão** pelo menu: o app chama `window.espiaStopLive()` na página (mesmo padrão do `espiaOpen` da 003).
- **Confirmação ao sair**: `MessageBoxW` do Windows, chamado no Rust (nada novo na página).
- **Aviso da primeira vez**: notificação do Windows (`tauri-plugin-notification`, usado só pelo Rust).
- **Preferências** ("Fechar no X", aviso já mostrado): um JSON pequeno na pasta de configuração do app.

No site, nada muda (FR-708).

## Technical Context

**Language/Version**: Rust 1.90+ (edição 2024) no app; TypeScript 6 + Svelte 5 no site

**Primary Dependencies**:
- **App**: Tauri 2.12 com a feature `tray-icon` (+ `image-png` para os ícones), `tauri-plugin-notification` 2 (só lado Rust), `windows` 0.62 (`Win32_UI_WindowsAndMessaging` já presente, para `MessageBoxW`).
- **Site**: `@tauri-apps/api/core` (já usado na 005).

**Storage**: `%APPDATA%\<identifier>\tray.json` (`closeOnX`, `trayNoticeShown`). Nada no servidor, nada no `localStorage`.

**Testing**: `cargo test` (funções puras: preferências, rótulos do menu, decisão do X/Sair), Vitest (gancho `espiaStopLive` e o aviso de ao vivo com uma ponte falsa), roteiro manual B1–B8 ([quickstart.md](./quickstart.md)).

**Target Platform**: Windows 10/11 x64 (o mesmo do app). Linux fora (Q2).

**Project Type**: SPA + Worker (sem mudança) + casca desktop `src-tauri/` (cresce: bandeja)

**Performance Goals**: trazer a janela de volta < 1 s (SC-702); nenhuma queda na transmissão ao esconder (SC-701).

**Constraints**:
- o WebView2 escondido não pode congelar a malha P2P nem a captura nativa (research R2);
- instalador: + < 0,3 MB;
- nenhuma regressão no site;
- nenhuma API nativa nova além de `app_set_live`.

**Scale/Scope**:
- **Rust**: 1 módulo novo (`tray.rs`, ~200 linhas) + ajustes no `lib.rs` e no `build.rs`. 2 ícones novos (normal e ao vivo).
- **Site**: o gancho do `desktop.ts` cresce (`espiaStopLive` e o aviso de ao vivo). `room.svelte.ts` avisa quando `myBroadcast` muda.
- **Mocks**: nenhuma tela nova no site (a bandeja é nativa).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio / restrição | Situação |
|---|---|
| I. Simplicidade (site sem instalar nada) | ✅ Só o app opcional muda. O site continua igual. |
| II. Sem servidor de mídia | ✅ Nada muda na mídia. |
| III. Efêmero e sem contas | ✅ Nada no servidor. As duas preferências ficam no PC, no arquivo do app. |
| IV. Camada visual separada | ✅ O aviso de ao vivo sai de `src/lib/desktop.ts`, não de componentes. |
| V. Telas mockáveis | ✅ Sem tela nova no site. O menu e o aviso são nativos (roteiro manual). |
| VI. Segredos fora do navegador | ✅ Sem segredos novos. |
| Restrição técnica: "expõe à página **apenas** a ponte de captura" | ❌ **Violada** pelo `app_set_live` → emenda 1.4.0 abaixo. |

**Emenda proposta (MINOR → 1.4.0)**: a restrição do app desktop passa a dizer "expõe à página **apenas** a ponte de captura (`specs/005…/contracts/native-capture.md`) e o aviso de ao vivo para a bandeja (`specs/007-fechar-para-bandeja/contracts/tray.md`: um comando, `app_set_live(live: bool)`, sem retorno), **apenas** para a origem do site, na janela principal. Qualquer outra API nativa exige emenda." Motivo: o pedido de 2026-10-29 (fechar para a bandeja sem perder a transmissão).

**Re-check pós-design**: ✅ com a emenda. O comando novo só recebe um booleano e não lê nada do PC. O caminho contrário (app → página) usa `eval` de uma função fixa, como o `espiaOpen` da 003.

## Project Structure

### Documentation (this feature)

```text
specs/007-fechar-para-bandeja/
├── spec.md
├── plan.md                    # este arquivo
├── research.md                # R1–R8
├── data-model.md              # TrayPrefs, LiveState, menu, textos
├── quickstart.md              # automático + roteiro B1–B8
├── contracts/
│   └── tray.md                # app_set_live, espiaStopLive, menu, tray.json
├── checklists/requirements.md
└── tasks.md                   # /speckit-tasks
```

### Source Code (repository root)

```text
src/
├── lib/
│   ├── desktop.ts                    # + espiaStopLive, + reportLive(live) (invoke só no app)
│   └── stores/room.svelte.ts         # efeito: myBroadcast mudou → reportLive; expõe stopGoLive ao gancho
└── routes/ ou main.ts                # instala o gancho (já instala o espiaOpen)

src-tauri/
├── Cargo.toml                        # tauri features tray-icon + image-png; tauri-plugin-notification
├── build.rs                          # AppManifest::commands([... + "app_set_live"])
├── capabilities/capture.json         # + allow-app-set-live (mesma origem, janela main)
├── icons/tray.png, tray-live.png     # NOVOS: 32×32 normal e com o ponto vermelho
└── src/
    ├── lib.rs                        # registra tray::setup, o comando e o CloseRequested
    └── tray.rs                       # NOVO: ícone, menu, prefs (puro + testes), confirmação, aviso

test/lib/desktop.test.ts              # + espiaStopLive e reportLive
.specify/memory/constitution.md       # emenda 1.4.0
```

**Structure Decision**: tudo o que é nativo fica em `src-tauri/src/tray.rs`. As regras puras (o que o X faz, se Sair pergunta, os itens do menu, ler/gravar `tray.json`) são funções sem Tauri e têm testes. No site, só o `desktop.ts` sabe que o app existe.

## Ordem de entrega

1. Emenda 1.4.0 da constituição.
2. **Rust, sem ao vivo**: ícone na bandeja, X esconde, clique traz de volta, menu Abrir/Sair, segunda abertura (atalho/link) mostra a janela escondida. Preferência "Fechar no X" + `tray.json`.
3. Aviso da primeira vez (notificação).
4. **Ao vivo**: `app_set_live` (comando + capability + `build.rs`), ícone/dica ao vivo, "Parar transmissão" → `espiaStopLive`, confirmação no Sair e no X com "Fechar no X" ligado.
5. Site: `desktop.ts` + `room.svelte.ts` + testes. Deploy do site.
6. Conferir R2 (WebView2 escondido não congela): transmitir, esconder 5 min, outra pessoa assiste.
7. Build, medição do instalador, roteiro B1–B8, versão 0.1.4.

## Riscos

| Risco | Mitigação |
|---|---|
| WebView2 escondido reduzir timers e travar a transmissão ou o som | R2: o Chromium não congela abas com WebRTC/áudio ativos. Testar no passo 6. Plano B: manter o controller visível (`SetIsVisible(true)`) ou `--disable-renderer-backgrounding`/`--disable-background-timer-throttling` nos argumentos do WebView2. |
| A página antiga (site não publicado) não chama `app_set_live` | O app trata "nunca avisou" como fora do ar: o X esconde normalmente, Sair não pergunta. A confirmação só passa a valer após o deploy. Por isso o deploy vem antes da tag. |
| A notificação sem AppUserModelID no modo dev | Em dev o Windows mostra como PowerShell. No app instalado (NSIS) aparece como Espia. Só estético. |
| Ícone escondido na "seta" da bandeja | O texto do aviso explica onde achar (edge case da spec). |
| Instalador novo com o app escondido | O NSIS do Tauri já fecha o app em execução. Conferir no roteiro (B8). |

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Comando `app_set_live` exposto à página | O app precisa saber se há transmissão (câmera, captura nativa ou do navegador) para o ícone, a dica, o menu e a confirmação ao sair | Só olhar a captura nativa da 005: não vê a câmera (006) nem o modo do navegador. Ler o título da página: gambiarra frágil. `eval` não devolve valor no Tauri. |
