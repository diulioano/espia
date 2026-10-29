# Tasks: Memória do app

**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/engine.md](./contracts/engine.md), [quickstart.md](./quickstart.md)

**Testes**: `cargo test` para a regra pura de `engine.rs` e Vitest para o `reportRoom` (o plano pede). O resto é medição com `scripts/memoria.ps1` e o roteiro manual M1–M9.

**Ordem de publicação**: o site (T912) vai para o ar **antes** do app 0.1.6 (research R4).

**Revisado em 2026-10-29** depois do analyze e dos prints do Gerenciador: a prioridade passou a ser descarregar o motor (US1); saíram o Job Object (C1) e a memória baixa depois de parar (I2); sair da sala escondido usa os mesmos 30 s (I1); minimizado entrou (US2).

## Phase 1: Setup

- [x] T901 Criar `scripts/memoria.ps1`:
  - acha o(s) `Espia.exe` e percorre os filhos recursivamente via `Get-CimInstance Win32_Process` (`ParentProcessId`), sem contar `msedgewebview2.exe` de outras árvores (WhatsApp, SearchHost);
  - para cada processo, pega a **memória privada do conjunto de trabalho** (a coluna "Memória" do Gerenciador): `Get-CimInstance Win32_PerfFormattedData_PerfProc_Process` casando `IDProcess`, campo `WorkingSetPrivate`;
  - imprime o total e o detalhe por tipo (`Espia.exe`, motor principal = `--embedded-browser-webview`, `renderer`, `gpu-process`, `utility`, `crashpad-handler`, pelo `--type=` da linha de comando) e o número de processos do motor;
  - `-Watch` imprime uma linha `hh:mm:ss total MB (n processos do motor)` a cada 5 s até Ctrl+C, e continua rodando (mostrando 0) se o motor sair;
  - sem Espia rodando, avisa e sai com código 1.
  - Conferir com o Gerenciador aberto lado a lado: o total bate com o do grupo (±5 %).
- [x] T902 Spike (research R5, analyze U1), num build de debug (`npm run tauri dev`) com um `#[cfg(debug_assertions)]` temporário em `src-tauri/src/lib.rs` (variável `ESPIA_SPIKE_DESTROY=1`: 10 s depois de abrir, `window.destroy()` na `main`, com `prevent_exit` no `ExitRequested`):
  - `scripts/memoria.ps1 -Watch` deve mostrar 0 processos do motor em ≤ 10 s, com o `Espia.exe` ainda vivo;
  - registrar o resultado em `research.md` R5 e remover o código temporário;
  - **se o motor não sair**, parar aqui e investigar quem segura o ambiente do WebView2 (Tauri/wry) antes da Phase 3.

## Phase 2: Foundational

- [x] T903 Emenda 1.5.0 em `.specify/memory/constitution.md`:
  - a restrição do app desktop cita também "o aviso de sala (`specs/009-memoria-do-app/contracts/engine.md`: um comando, `app_set_room(inRoom)`, sem retorno)";
  - subir a versão e a data;
  - linha nova no Sync Impact Report: "1.4.0 → 1.5.0 … feature 009, pedido de 2026-10-29 (Q1: A)".
- [x] T904 Criar `src-tauri/src/engine.rs` com a parte pura, sem Tauri:
  - `enum WindowMode { Visible, Minimized, Hidden }`, `enum EngineAction { Normal, Low, ReleaseAfter(Duration) }`;
  - `pub fn engine_action(window: WindowMode, in_room: bool, live: bool) -> EngineAction`, exatamente como `data-model.md`: "live → Normal; Visible → Normal; Minimized → Low; Hidden && in_room → Low; Hidden && !in_room → ReleaseAfter(30 s)";
  - constante `RELEASE_AFTER = Duration::from_secs(30)`;
  - comentário de cabeçalho apontando para `specs/009-memoria-do-app/`;
  - `mod engine;` em `src-tauri/src/lib.rs`.
- [x] T905 [P] Testes `#[cfg(test)]` em `src-tauri/src/engine.rs`:
  - as 5 linhas da regra;
  - ao vivo dá `Normal` em todas as combinações de `WindowMode` × `in_room`;
  - minimizado nunca dá `ReleaseAfter`.
- [x] T906 Em `src-tauri/src/lib.rs`, extrair o builder da janela `main` do `setup` para `pub(crate) fn create_main<R: Runtime>(app: &AppHandle<R>, room: Option<&str>) -> tauri::Result<WebviewWindow<R>>`:
  - mesmo título, tema, tamanhos, `on_navigation`, `on_new_window`, `on_page_load`, args de debug, `share_bar::install`;
  - o `setup` passa a chamar `create_main(app.handle(), room.as_deref())`;
  - sem mudar o comportamento atual.
- [x] T907 Em `src-tauri/src/lib.rs`, trocar `.run(tauri::generate_context!()).expect(…)` por `.build(tauri::generate_context!()).expect("erro ao iniciar o Espia").run(|app, event| …)`:
  - em `RunEvent::ExitRequested { code: None, api, .. }`, chamar `api.prevent_exit()`, para que a última janela destruída não feche o app;
  - "Sair" (`app.exit(0)`, `code: Some(0)`) continua saindo.

**Checkpoint**: `cargo test` passa; o app abre e se comporta igual à 0.1.5.

## Phase 3: User Story 1 — Escondido fora de sala, o motor sai (P1) 🎯 MVP

**Goal**: escondido na bandeja e fora de sala por 30 s → janela e motor destruídos (≤ 40 MB, 0 motor); reabrir em ≤ 2 s.

**Independent Test**: quickstart M5 e M7.

- [x] T908 [P] [US1] Em `src/lib/desktop.ts`, `export function reportRoom(inRoom: boolean): void`, no mesmo formato do `reportLive`: só dentro do app, `invoke('app_set_room', { inRoom }).catch(() => {})`. Atualizar o comentário do topo citando specs/009.
- [x] T909 [P] [US1] Testes em `test/lib/desktop.test.ts`:
  - `reportRoom` não chama nada fora do app;
  - dentro do app (com `__TAURI_INTERNALS__` definido e `@tauri-apps/api/core` mockado), chama `invoke('app_set_room', { inRoom: true })` e `{ inRoom: false }`;
  - um erro do `invoke` não escapa.
- [x] T910 [US1] Comando `app_set_room(in_room: bool)` (`#[tauri::command]`, `#[cfg(windows)]`) em `src-tauri/src/engine.rs`:
  - confere a origem como o `app_set_live` em `src-tauri/src/tray.rs`;
  - guarda em `EngineState.in_room` e, se mudou, chama `apply(app)`;
  - registrar em `generate_handler!` de `src-tauri/src/lib.rs`.
- [x] T911 [P] [US1] Adicionar `"app_set_room"` em `AppManifest::commands` de `src-tauri/build.rs` (e atualizar o comentário para 1.5.0) e `"allow-app-set-room"` nas `permissions` de `src-tauri/capabilities/capture.json`. Conferir também `src-tauri/capabilities-dev/capture-dev.json`.
- [x] T912 [US1] Em `src/routes/RoomRoute.svelte`, ao lado do `reportLive`:
  - `const inRoom = $derived(!!store?.connected && !store?.ended)` e `$effect(() => reportRoom(inRoom))`;
  - no `onDestroy`, `reportRoom(false)`;
  - `npm run check`, `npm run build`, `npm test`. **Deploy do site** (`! npm run deploy`, pela pessoa usuária) antes de publicar o app.
- [x] T913 [US1] Em `src-tauri/src/engine.rs` (Windows), `EngineState` gerenciado com `app.manage` no `setup` de `src-tauri/src/lib.rs`:
  - `window: Mutex<WindowMode>` (padrão `Visible`), `in_room: AtomicBool` (padrão `false`), `timer_gen: AtomicU64`;
  - `pub fn set_window<R: Runtime>(app, mode)` grava e chama `apply`.
  - `pub fn apply<R: Runtime>(app: &AppHandle<R>)`:
    - incrementa `timer_gen`;
    - lê `window`, `in_room` e `TrayState.live` e chama `engine_action`;
    - com a janela `main` existindo: `ReleaseAfter(d)` → guarda a geração e, depois de `d` (`tauri::async_runtime::spawn` + `tokio::time::sleep` ou `std::thread` + `sleep`), se a geração for a mesma e a regra ainda der `ReleaseAfter`, chama `release(app)`;
    - o nível de memória (`Normal`/`Low`) fica para a US2 (T918).
- [x] T914 [US1] Em `src-tauri/src/engine.rs`, `fn release<R: Runtime>(app: &AppHandle<R>)`:
  - com a janela `main`: `capture::stop_all(window.as_ref(), &app.state::<capture::Capture>())`, `app.save_window_state(StateFlags::all())` (`tauri_plugin_window_state::AppHandleExt`) e `window.destroy()`;
  - log `[espia] motor solto (escondido fora de sala)`.
- [x] T915 [US1] Em `src-tauri/src/tray.rs` e `src-tauri/src/lib.rs`, ligar o ciclo:
  - o `hide()` da bandeja chama `engine::set_window(app, Hidden)`;
  - o `show(app)` da bandeja ("Abrir o Espia" e clique): se não existir janela `main`, `create_main(app, None)`; depois `focus` e `engine::set_window(app, Visible)`;
  - `tray::set_live` chama `engine::apply` quando o valor muda;
  - no `on_page_load` `Started` (`lib.rs`), zerar `in_room` como já se faz com `live`.
- [x] T916 [US1] No callback do `tauri_plugin_single_instance` em `src-tauri/src/lib.rs`:
  - sem janela `main`, chamar `create_main(app, links::room_path_from_args(&args).as_deref())` (a sala vai pelo `start_url`, como na primeira abertura) e `focus`;
  - com janela, manter o comportamento atual (`focus` + `open_room`);
  - nos dois casos, `engine::set_window(app, Visible)`.
- [ ] T917 [US1] Conferir no build de debug com `scripts/memoria.ps1 -Watch`: M5 (tela inicial + X → 0 motor em ~30 s; numa sala + X + sair da sala → 0 motor 30 s depois de sair) e M7 (reabrir por ícone, `espia://` e atalho em ≤ 2 s; reabrir antes de 30 s não recarrega). Anotar em `specs/009-memoria-do-app/relatorio.md`.

**Checkpoint**: SC-903 e SC-903b atendidos. Já resolve o pedido ("fechar continua rodando o WebView2").

## Phase 4: User Story 2 — Minimizado ou escondido numa sala, o motor solta o que não usa (P2)

**Goal**: minimizado (qualquer situação) ou escondido numa sala, sem transmitir → memória baixa, sem perder a sala nem o som.

**Independent Test**: quickstart M6.

- [x] T918 [US2] Em `src-tauri/src/engine.rs` (Windows), `fn set_memory_level<R: Runtime>(window: &WebviewWindow<R>, low: bool)`:
  - via `window.with_webview(|wv| …)`: `wv.controller().CoreWebView2()?.cast::<ICoreWebView2_19>()?.SetMemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW | _NORMAL)` (`webview2-com` 0.39);
  - erro (runtime antigo sem a interface) só no log `[espia]`;
  - no `apply` (T913): `Normal` → `set_memory_level(false)`; `Low` e `ReleaseAfter` → `set_memory_level(true)`. Só chamar quando o nível muda (guardar o último).
- [x] T919 [US2] Em `src-tauri/src/lib.rs`, no `on_window_event` da janela `main`, tratar `WindowEvent::Resized(_)`:
  - `window.is_minimized()` → `engine::set_window(app, Minimized)`;
  - senão, se estava `Minimized` e a janela está visível → `engine::set_window(app, Visible)`;
  - não mexer quando o modo é `Hidden`.
- [ ] T920 [US2] **precisa da pessoa usuária** (alguém transmitindo com som). M6: minimizado e escondido numa sala, 1 min cada.
  - O som continua?
  - A memória fica ≤ a da janela aberta?
  - Se o som cortar, aplicar o plano B do research R3: na regra (T904/T905), `Minimized` e `Hidden && in_room` passam a dar `Normal`, e o FR-904a/904c ficam registrados como sem efeito.
  - Anotar em `relatorio.md` e `research.md` R3.

**Checkpoint**: SC-903a.

## Phase 5: User Story 3 — Parar de transmitir solta tudo da transmissão (P3)

**Goal**: todos os caminhos de parar soltam captura, som, câmera e envio; sem vazamento em ciclos.

**Independent Test**: quickstart M1–M3.

- [x] T921 [P] [US3] *(conferido sem mudança: o `Channel` do `@tauri-apps/api` 2.12 chama `cleanupCallback()` quando o Rust solta o canal no fim da sessão)* Em `src/lib/native-capture.ts`, `stop()`/`finish()`:
  - conferir se os `Channel` `frames` e `sound` continuam registrados depois do `capture_stop` (research R2, suspeita);
  - se continuarem, soltar os callbacks (trocar `onmessage` por um no-op e soltar as referências a `slots`, `videoWriter`, `audioWriter`);
  - ajustar `test/lib/native-capture.test.ts` se o comportamento mudar.
- [x] T922 [P] [US3] *(conferido sem mudança: `stopCapture` para todas as faixas; `destroy()` e `onEnd` já chamam `stopBroadcast(false)`)* Em `src/lib/stores/room.svelte.ts`:
  - `stopBroadcast`: todas as faixas do `this.stream` (vídeo **e** som, inclusive faixas de som trocadas no meio) recebem `.stop()`, e `this.mesh?.setLocal(null, null)` chega a `replaceTrack(null)` em todos os peers (`src/lib/peer.ts`);
  - `destroy()` e o fim da sala (`ended`) **com transmissão ativa** passam por `stopBroadcast` (ou param a captura nativa e as faixas do mesmo jeito), para o FR-901 valer em "sair da sala" e "sala acabou";
  - corrigir o que faltar, sem mudança visível.
- [ ] T923 [US3] **precisa da pessoa usuária**. M1–M3 com o build de debug e `scripts/memoria.ps1`. Se algum passar do limite, achar o processo culpado pelo detalhe do script e corrigir. Anotar em `relatorio.md` e `research.md` R2.

**Checkpoint**: SC-901 e SC-902 por todos os caminhos.

## Phase 6: User Story 4 — Tudo junto e nada sobra (P3)

**Goal**: 0 processos do Espia depois de sair ou de ser finalizado; nada de fora é afetado.

**Independent Test**: quickstart M8 e M9.

- [ ] T924 [US4] **precisa da pessoa usuária**. M8 e M9 com o app instalado:
  - "Sair" e "Finalizar tarefa" → `scripts/memoria.ps1` não acha nada em 5 s;
  - o navegador aberto por um link externo continua aberto;
  - o grupo "WebView2 Manager" com o castor contém só os processos do Espia.
  - Se sobrar processo do motor: voltar ao research R5 (job com *breakaway* para não levar o navegador junto) antes do release.

**Checkpoint**: SC-905.

## Phase 7: Polish

- [ ] T925 [P] (Opcional, research R6) Testar `--disable-features=AudioServiceOutOfProcess` somado aos padrões do wry (`--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,AudioServiceOutOfProcess`) em `additional_browser_args` também no release, em `create_main` de `src-tauri/src/lib.rs`. Só manter se: a medição cair; o som da captura nativa, o da câmera e o de quem assiste continuarem ok; e o `ESPIA_BROWSER_ARGS` de debug continuar acrescentando por cima. Senão, descartar e registrar em R6. Registrar o resultado no SC-904.
- [x] T926 Subir a versão para 0.1.6 em `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` e `package.json`; `cargo test`; `npm run tauri build`; medir o tamanho do instalador. *(Feito: `Espia_0.1.6_x64-setup.exe`, 1,62 MiB.)*
- [ ] T927 **precisa da pessoa usuária**. Roteiro M1–M9 do `quickstart.md` com o app instalado (site já publicado). Números em `specs/009-memoria-do-app/relatorio.md`, comparando com a base (~228 MB).
- [x] T928 [P] Atualizar `README.md` (seção do app desktop: o Espia solta o motor escondido fora de sala; os "Node.js" do Gerenciador não são do Espia) e `DEFINICOES.md`, se houver decisão nova (Q1: A).

## Dependencies & Execution Order

- **Phase 1**: T901 → T902 (o spike usa o script). Se o T902 falhar, nada da Phase 3 segue.
- **Phase 2**: T903 e T904/T905 independentes. T906 → T907 (o mesmo `lib.rs`).
- **US1** (Phase 3) depende das Phases 1 e 2:
  - site: T908 → T912 → deploy; T909 em paralelo;
  - app: T910, T911 → T913 → T914 → T915 → T916 → T917.
- **US2** depende de T913 (o `apply`). T918 → T919 → T920.
- **US3** só depende da Phase 2 e pode andar em paralelo com a US1 (só mexe no site).
- **US4** depois da US1 (o ciclo de criar e destruir janelas é o que pode deixar sobras).
- **Polish** por último. T925 é opcional.

### Parallel examples

- Phase 2: T903 (constituição) ‖ T904+T905 (`engine.rs`).
- US1: T908+T909 (site) ‖ T910+T911 (comando e permissões).
- US3: T921 (`native-capture.ts`) ‖ T922 (`room.svelte.ts`), enquanto a US1 anda no Rust.

## Implementation Strategy

1. **MVP = Phase 1 + 2 + US1**: escondido fora de sala, o motor sai (~228 MB → ≤ 40 MB). É o pedido.
2. **US2**: o caso do print (minimizado numa sala), com o plano B se cortar o som.
3. **US3 e US4**: confirmar e cobrir o que a medição já mostra quase ok.
4. Polish e release 0.1.6.
