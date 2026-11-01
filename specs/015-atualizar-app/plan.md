# Implementation Plan: Atualizar o app sozinho

**Branch**: `015-atualizar-app` | **Date**: 2026-11-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/015-atualizar-app/spec.md`

## Summary

O app passa a usar o plugin oficial de atualização do Tauri, só pelo Rust (R1). O plugin lê `/download/latest.json` no próprio site, confere a assinatura com uma chave do projeto (R3) e instala em silêncio. O instalador reabre o app sozinho (R2).

Quando instala:
- **ao abrir o processo**: instala na hora, se ainda não entrou numa sala (R6). Isso cobre o "atualizar quando reiniciar" e o "Depois";
- **app aberto**: avisa no canto da página, com "Atualizar agora" e "Depois" (R8). Ao vivo, pede confirmação antes;
- **escondido na bandeja e fora de sala há 30 min**: instala em silêncio e continua escondido (R9).

Depois de atualizar, volta para a mesma sala por um arquivo de retomada (R7). A versão mínima vem num campo extra do JSON: abaixo dela, o aviso perde o "Depois" mas não bloqueia nada (R5). Isso entra no app 0.1.9 e pede a emenda 2.1.0 da constituição.

## Technical Context

**Language/Version**: Rust 1.90 (Tauri 2.12), TypeScript + Svelte 5 (site), Node (script de lançamento)

**Primary Dependencies**: `tauri-plugin-updater` 2 (novo, só Rust). O resto já existe (single-instance, window-state, notification, opener).

**Storage**: `app_config_dir/update-resume.json` (apagado ao ler). Nada novo no site nem no `localStorage`.

**Testing**: `cargo test` (regra pura `update::decide`, leitura do `minimum` e da retomada), `vitest` (view-model do aviso, script de lançamento), mocks, ponta a ponta com app de teste instalado e site local (quickstart).

**Target Platform**: app Windows (NSIS, instalação do usuário). O site no navegador não muda.

**Project Type**: app desktop (Tauri) + site (SPA Svelte) + Worker

**Performance Goals**: "Atualizar agora" → app reaberto em ≤ 30 s (SC-001). Abertura com versão nova ≤ 15 s a mais (SC-006). Checagem de hora em hora com custo desprezível (um JSON de ~400 bytes).

**Constraints**:
- sem GitHub nem serviço pago (FR-013);
- não manda nada da pessoa (FR-014);
- nunca instala ao vivo sem confirmação (FR-005);
- nunca instala em sala sozinho (US4-2);
- só instala o que tem assinatura válida (FR-008).

**Scale/Scope**: ~10 amigos, 1 instalador de ~1,7 MB por versão.

## Constitution Check

| Princípio | Situação |
|---|---|
| I. Simplicidade | ✅ Menos passos: hoje é entrar no site, baixar e instalar; passa a ser zero ou um clique. O site continua sem instalar nada. |
| II. Sem servidor de mídia | ✅ Não toca na mídia. |
| III. Sem contas / nada sobre as pessoas | ✅ A checagem pede um arquivo estático sem mandar nada da pessoa. O `update-resume.json` guarda só a sala e uma hora, e é apagado ao ler. |
| IV. Camada visual separada | ✅ O Rust decide. `src/lib/app-update.svelte.ts` monta o view-model e `UpdateNotice.svelte` só mostra. |
| V. Telas mockáveis | ✅ 6 cenas (quickstart). |
| VI. Segredos fora do navegador | ✅ A chave privada fica só no PC de quem publica. O app e o site têm só a pública. |
| Restrições técnicas: API nativa só a listada | ⚠️ **Emenda 2.1.0**: novo comando `app_update(now)` página → app e gancho `window.espiaUpdate` app → página (contracts/update.md). O plugin não é exposto à página. |

**Gate**: passa com a emenda 2.1.0 (MINOR, acrescenta um item à lista de APIs nativas). Ela é feita na primeira tarefa da implementação, antes de qualquer código.

**Re-check pós-design**: igual. O desenho não acrescentou nada além do comando e do gancho.

## Project Structure

### Documentation (this feature)

```text
specs/015-atualizar-app/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/update.md
├── checklists/requirements.md
└── tasks.md              # /speckit-tasks
```

### Source Code (repository root)

```text
.specify/memory/constitution.md      # emenda 2.1.0
package.json                         # "espia": { "appMinimum": "0.1.9" }
scripts/copy-installer.mjs           # + latest.json, .exe por versão, apaga antigos, chave da assinatura
src-tauri/
├── Cargo.toml                       # + tauri-plugin-updater; versão 0.1.9
├── tauri.conf.json                  # plugins.updater (endpoint, pubkey, quiet), createUpdaterArtifacts
├── tauri.test.conf.json             # novo: http local para o teste ponta a ponta
├── capabilities/capture.json        # + allow-app-update
├── build.rs                         # + "app_update" na lista de comandos (gera allow-app-update)
├── src/update.rs                    # novo: decide() pura, checagem de hora em hora, instalar, retomada, app_update
├── src/engine.rs                    # + hidden_out_since
├── src/lib.rs                       # plugin, comando, abertura lendo update-resume.json, reenvio no PageLoad
└── www/                             # loader: estado "Atualizando o Espia…"
src/
├── lib/app-update.svelte.ts         # novo: gancho espiaUpdate + view-model + appUpdate(now)
├── lib/desktop.ts                   # + invoke('app_update')
├── ui/components/UpdateNotice.svelte # novo: aviso no canto (normal, mínimo, baixando, instalando, falhou)
├── App.svelte                       # monta o aviso uma vez, vale para início, entrada e sala
└── mocks/                           # cenas update* (home.ts, room…)
src/worker/index.ts                  # só se o R4 mostrar cache longo no latest.json
```

**Structure Decision**: o mesmo projeto de sempre. A lógica nativa nova fica isolada em `src-tauri/src/update.rs`, e a da página em `src/lib/app-update.svelte.ts`. O aviso é montado na raiz, porque aparece em qualquer tela do app.

## Ordem de entrega

1. **Emenda 2.1.0** + chave de assinatura (o usuário roda o `signer generate`) + pubkey no config.
2. **Mock primeiro**: `UpdateNotice` e as 6 cenas (princípio V).
3. **US1 + US2 (MVP)**: plugin, `update::decide`, checagem na abertura e de hora em hora, `app_update`, retomada, aviso real. Teste ponta a ponta 1–6.
4. **US3**: `minimum` no script e no Rust (o aviso já sabe mostrar). Teste 10.
5. **US4**: `hidden_out_since` + instalação silenciosa + abrir só na bandeja. Testes 7–8.
6. **Assinatura** (teste 9) e acabamento: gate, relatório.
7. **Lançamento 0.1.9**: `desktop:release` com a chave, `deploy` (o usuário). O grupo baixa uma última vez pelo site (M1).

## Riscos

- **Perder a chave privada**: ninguém atualiza mais sozinho. → Cópia de segurança no quickstart.
- **O instalador silencioso não reabrir** em algum PC (antivírus): o app some até a pessoa abrir de novo. Links `espia://` continuam abrindo. → Medir no ponta a ponta. Se acontecer, trocar para `passive` só no "Atualizar agora".
- **Cache do `latest.json`** no Cloudflare: o aviso atrasa. → Conferir cabeçalhos (R4).
- **Primeira vez**: quem não baixar o 0.1.9 fica para trás como hoje (R11).

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Novo comando nativo `app_update` (emenda 2.1.0) | O aviso precisa do visual do Espia e de mock (FR-016), e a página é quem sabe se está ao vivo | Caixa nativa do Windows: sem visual nem mock, e precisaria de outro canal para saber se está ao vivo |
