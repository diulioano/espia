# Implementation Plan: Memória do app

**Branch**: `009-memoria-do-app` (trabalho direto na `main`, como na 004–008) | **Date**: 2026-10-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/009-memoria-do-app/spec.md`

## Summary

Os prints do Gerenciador (spec, Contexto) mostram que **parar de transmitir já devolve a memória da transmissão** (243 → 228 MB). O que pesa é o **motor de página carregado** (~228 MB), mesmo minimizado e sem ninguém transmitindo. O plano:

1. **Medir com a régua do Gerenciador** (R1): `scripts/memoria.ps1` soma a memória privada da árvore do `Espia.exe`, a mesma coluna "Memória" dos prints.
2. **Escondido fora de sala → o motor sai** (US1, Q1: A, 🎯 MVP):
   - depois de 30 s escondido **e** fora de sala, o app destrói a janela e o motor. Fica só o `Espia.exe` com o ícone da bandeja;
   - abrir de novo (ícone, atalho, `espia://`) cria a janela outra vez;
   - para saber "numa sala", a página avisa com um comando novo, `app_set_room(inRoom)`, no mesmo formato do `app_set_live`. **Exige a emenda 1.5.0.**
3. **Minimizado ou escondido numa sala → o motor solta o que não usa** (US2): `MemoryUsageTargetLevel` baixo; volta ao normal ao mostrar a janela. Se cortar o som, fica sem efeito (plano B decidido, R3).
4. **Parar solta tudo** (US3): só revisão dos caminhos além do botão (sair da sala e sala acabar transmitindo) e checagem de vazamento. Sem "memória baixa depois de parar" (removida, analyze I2).
5. **Nada sobra** (US4): conferência no roteiro M9. **Sem Job Object** (analyze C1: levaria junto o navegador aberto por links). O agrupamento "WebView2 Manager" com o castor já existe e foi aceito.

Os "Node.js" são do Zed. Ficam fora.

## Technical Context

**Language/Version**: Rust 1.90+ (edição 2024) no app; TypeScript 6 + Svelte 5 no site

**Primary Dependencies**:
- **App**: Tauri 2.12 (já tem `tray-icon`), `webview2-com` 0.39 (`ICoreWebView2_19::SetMemoryUsageTargetLevel`, já presente), `windows` 0.62 (sem features novas).
- **Site**: `@tauri-apps/api/core` (já usado).

**Storage**: nada novo.

**Testing**:
- `cargo test`: a regra pura "o que fazer com o motor" (`engine_action(window, in_room, live)`, com `window` = visível, minimizada ou escondida) e o tempo de espera;
- Vitest: `reportRoom` no `desktop.ts` e quando o `room.svelte.ts` avisa entrar/sair;
- `scripts/memoria.ps1` + roteiro manual M1–M9 ([quickstart.md](./quickstart.md)).

**Target Platform**: Windows 10/11 x64 (o mesmo do app).

**Project Type**: SPA + Worker (sem mudança) + casca desktop `src-tauri/`

**Performance Goals**: SC-901 a SC-906 da spec. As principais:
- escondido fora de sala, até 40 MB e 0 motor (hoje ~228 MB);
- reabrir em até 2 s;
- depois de parar, até +50 MB do valor de antes (já atende pelo botão).

**Constraints**:
- não pode derrubar quem assiste (esconder transmitindo = nada muda);
- quem está minimizado ou escondido numa sala continua **ouvindo** a galera (007, US1), então o modo de memória baixa não pode cortar o som (R3);
- nenhuma regressão no site;
- só uma API nova (`app_set_room`).

**Scale/Scope**:
- **Rust**: 1 módulo novo `engine.rs` (~150 linhas: regra pura, timer, memória baixa, destruir/recriar), extrair `create_main()` do `lib.rs`, `RunEvent::ExitRequested` (não sair sem janela), `WindowEvent::Resized` (minimizar), ajustes no `tray.rs` (abrir = criar se não existe).
- **Site**: `desktop.ts` (+ `reportRoom`), `room.svelte.ts` (avisa entrar/sair, revisão do `stopBroadcast`).
- **Novo**: `scripts/memoria.ps1`.
- **Mocks**: nenhuma tela nova.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio / restrição | Situação |
|---|---|
| I. Simplicidade (site sem instalar nada) | ✅ Só o app opcional muda. |
| II. Sem servidor de mídia | ✅ Nada muda na mídia. |
| III. Efêmero e sem contas | ✅ Nada guardado. |
| IV. Camada visual separada | ✅ O aviso de sala sai de `src/lib/` (`desktop.ts`, store), não de componentes. |
| V. Telas mockáveis | ✅ Sem tela nova. |
| VI. Segredos fora do navegador | ✅ Sem segredos. |
| Restrição técnica: "expõe à página **apenas** a ponte de captura e `app_set_live`" | ❌ **Violada** pelo `app_set_room` → emenda 1.5.0 abaixo. |

**Emenda proposta (MINOR → 1.5.0)**: a restrição do app desktop passa a citar também "o aviso de sala (`specs/009-memoria-do-app/contracts/engine.md`: um comando, `app_set_room(inRoom)`, sem retorno)". Motivo: pedido de 2026-10-29 (soltar o motor escondido fora de sala, Q1: A).

Alternativa sem emenda considerada (R4): o Rust olhar o endereço da página (`/s/…`). Foi descartada porque a tela "a sala acabou" e o formulário de apelido também ficam em `/s/…`. Isso violaria o FR-904b (sala acaba com a janela escondida → soltar na hora).

**Re-check pós-design**: ✅ com a emenda. O comando recebe só um booleano e não lê nada do PC. O pior que uma página maliciosa conseguiria seria o app soltar o motor quando escondido, o que só recarrega a página ao reabrir.

## Project Structure

### Documentation (this feature)

```text
specs/009-memoria-do-app/
├── spec.md
├── plan.md                    # este arquivo
├── research.md                # R1–R8
├── data-model.md              # EngineState, regra, tempos
├── quickstart.md              # memoria.ps1 + roteiro M1–M9
├── contracts/
│   └── engine.md              # app_set_room, ciclo da janela, processos
├── checklists/requirements.md
└── tasks.md                   # /speckit-tasks
```

### Source Code (repository root)

```text
scripts/memoria.ps1                   # NOVO: soma a memória da árvore do Espia.exe (por tipo de processo)

src/lib/
├── desktop.ts                        # + reportRoom(inRoom)
└── stores/room.svelte.ts             # avisa entrou (welcome) / saiu (leave, ended, destroy); revisão do stopBroadcast

src-tauri/
├── build.rs                          # AppManifest::commands([... + "app_set_room"])
├── capabilities/capture.json         # + allow-app-set-room (mesma origem, janela main)
└── src/
    ├── lib.rs                        # create_main() extraído; .build().run() com ExitRequested; Resized → minimizada
    ├── engine.rs                     # NOVO: regra pura + testes, timer de 30 s, memória baixa/normal, soltar/recriar
    ├── tray.rs                       # show() cria a janela se não existe; hide() avisa o engine
    └── capture/                      # revisão do que sobra ao parar (R2)

test/lib/desktop.test.ts              # + reportRoom
.specify/memory/constitution.md       # emenda 1.5.0
```

**Structure Decision**: a decisão "manter, baixar memória ou soltar o motor" é uma função pura em `engine.rs`, com testes. O resto do módulo só executa essa decisão. No site, só o `desktop.ts` fala com o app.

## Ordem de entrega

1. `scripts/memoria.ps1` e a base com a régua do Gerenciador (R1). Os prints já dão ~228 MB.
2. **Spike** (R5, analyze U1): no debug, destruir a janela `main` e ver se o motor sai em ≤ 10 s. Se não sair, resolver isso antes de seguir.
3. Emenda 1.5.0.
4. **US1**:
   - `create_main()` extraído e não sair sem janela;
   - `app_set_room` e o site (deploy do site **antes** do app);
   - `engine.rs` com a regra, o timer de 30 s, soltar e recriar (bandeja, atalho, `espia://`).
5. **US2**: minimizado e escondido numa sala → memória baixa. Conferir o som (M6).
6. **US3**: revisar os caminhos de parar (sair da sala ou a sala acabar transmitindo) e checar 5 ciclos.
7. **US4**: conferir M9 (nada sobra; o navegador aberto por link continua).
8. Opcional: flags do motor (R6). Build, roteiro M1–M9, versão 0.1.6.
