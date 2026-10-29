# Tasks: Fechar o app sem sair (bandeja)

**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/tray.md](./contracts/tray.md), [quickstart.md](./quickstart.md)

**Testes**: `cargo test` para as funções puras de `tray.rs` e Vitest para o gancho do site (o plano pede). A bandeja em si é nativa: roteiro manual B1–B8.

## Phase 1: Setup

- [x] T701 Emenda 1.4.0 da constituição (`app_set_live` na restrição do app desktop) em `.specify/memory/constitution.md`
- [ ] T702 Ativar as features `tray-icon` e `image-png` do `tauri` e adicionar `tauri-plugin-notification = "2"` em `src-tauri/Cargo.toml`
- [ ] T703 [P] Criar os ícones da bandeja `src-tauri/icons/tray.png` (32×32, o mesmo do app) e `src-tauri/icons/tray-live.png` (32×32, com um ponto vermelho no canto inferior direito, cor do "ao vivo" de `src/ui/tokens.css`)

## Phase 2: Foundational

- [ ] T704 Criar `src-tauri/src/tray.rs` com as partes puras, sem Tauri: `TrayPrefs { close_on_x: bool, tray_notice_shown: bool }` (padrão `false`/`false`; "arquivo ausente, ilegível ou com campos estranhos → padrão (campos desconhecidos são ignorados)"), `parse_prefs(&str) -> TrayPrefs`, `prefs_json(&TrayPrefs) -> String` (chaves `closeOnX`, `trayNoticeShown`), `enum CloseAction { Hide, Exit, ConfirmExit }`, `on_close(prefs, live)` e `on_quit(live)` conforme a tabela de `data-model.md`, e as constantes de texto (dica, aviso, confirmação) de `data-model.md`
- [ ] T705 [P] Testes `#[cfg(test)]` em `src-tauri/src/tray.rs`: `parse_prefs` (vazio, JSON inválido, campos extras, valores corretos), ida e volta `prefs_json`→`parse_prefs`, as 3 linhas de `on_close` e as 2 de `on_quit`
- [ ] T706 Em `src-tauri/src/tray.rs`, `TrayState` (gerenciado com `app.manage`): `live: AtomicBool`, `prefs: Mutex<TrayPrefs>`, o caminho `app.path().app_config_dir()/tray.json`; `load(app)` no `setup` e `save()` (cria a pasta, grava `prefs_json`; erro só no log `[espia]`)

**Checkpoint**: `cargo test` passa com as regras puras.

## Phase 3: User Story 1 — O X não fecha o app (P1) 🎯 MVP

**Goal**: o X esconde a janela; o app segue na sala e transmitindo, com ícone na bandeja.

**Independent Test**: transmitir uma janela, clicar no X; outra pessoa continua vendo; clicar no ícone traz a janela de volta (quickstart B1, B2, B4).

- [ ] T707 [US1] Em `src-tauri/src/tray.rs`, `setup(app)`: `TrayIconBuilder::with_id("main")` com `tray.png` (`include_bytes!` + `Image::from_bytes`), dica `Espia`, `show_menu_on_left_click(false)`; no `on_tray_icon_event`, `TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }` → mostra a janela `main` (reusar a lógica de `focus` de `src-tauri/src/lib.rs`: mover `focus` para `pub(crate)` ou para `tray.rs`)
- [ ] T708 [US1] Em `src-tauri/src/lib.rs`, chamar `app.manage(TrayState::load(app))` e `tray::setup(app)?` no `setup`, registrar `.plugin(tauri_plugin_notification::init())`, e tratar `WindowEvent::CloseRequested { api, .. }` da janela `main` (via `.on_window_event` no `tauri::Builder`): `on_close(prefs, live)` → `Hide` = `api.prevent_close()` + `window.hide()`; `Exit`/`ConfirmExit` = `api.prevent_close()` + `tray::quit(app)` (T715)
- [ ] T709 [US1] Aviso da primeira vez em `src-tauri/src/tray.rs`: ao esconder com `tray_notice_shown == false`, mostrar a notificação (`NotificationExt`, título "O Espia continua aberto", corpo do `data-model.md`), marcar `tray_notice_shown = true` e `save()` (FR-703)
- [ ] T710 [US1] Conferir que o `single_instance` (`src-tauri/src/lib.rs`) mostra a janela escondida e segue o `open_room` para links `espia://s/...` (R8); ajustar só se `focus` não bastar
- [ ] T711 [US1] Verificar R2 com `npm run tauri dev`: transmitir (captura nativa e câmera), esconder 5 min, outra janela/navegador assiste; olhar o log `[espia] captura nativa:`. Se houver queda, aplicar o plano B do `research.md` R2 em `src-tauri/src/lib.rs` (`additional_browser_args` também no release) e registrar o resultado em `specs/007-fechar-para-bandeja/research.md` R2

**Checkpoint**: X esconde, ícone traz de volta, transmissão não cai.

## Phase 4: User Story 2 — Sair de verdade (P1)

**Goal**: menu do ícone com Abrir, Parar transmissão (só ao vivo) e Sair; Sair pergunta se estiver transmitindo.

**Independent Test**: com o app escondido e transmitindo, Sair → confirmação → app fecha e a sala vê a saída (quickstart B5, B6, B8).

- [ ] T712 [US2] Comando `app_set_live(live: bool)` em `src-tauri/src/tray.rs` (`#[tauri::command]`, só `#[cfg(windows)]` como a captura): confere a origem da janela como `src-tauri/src/capture/mod.rs` faz; se mudou, grava em `TrayState.live` e chama `refresh(app)`; registrar em `generate_handler!` de `src-tauri/src/lib.rs`
- [ ] T713 [P] [US2] Adicionar `"app_set_live"` em `AppManifest::commands` de `src-tauri/build.rs` e `"allow-app-set-live"` nas `permissions` de `src-tauri/capabilities/capture.json` (mesma `remote.urls` e `windows: ["main"]`); conferir também `src-tauri/capabilities-dev/capture-dev.json`
- [ ] T714 [US2] Em `src-tauri/src/tray.rs`: menu (`open` "Abrir o Espia", `stop` "Parar transmissão" só quando `live`, separador, `close-on-x` reservado para a US3, separador, `quit` "Sair"); `refresh(app)` refaz o menu e troca ícone (`tray.png` ↔ `tray-live.png`) e dica (`Espia` ↔ `Espia · ao vivo`) (FR-706); `on_menu_event`: `open` → focus, `stop` → `window.eval("window.espiaStopLive?.()")`, `quit` → `quit(app)`
- [ ] T715 [US2] `quit(app)` em `src-tauri/src/tray.rs`: `on_quit(live)`; `ConfirmExit` → mostrar a janela e, numa thread (`std::thread::spawn`), `MessageBoxW(hwnd, "Você está transmitindo. Sair mesmo assim?", "Espia", MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2)`; "Sim" ou `Exit` → `app.save_window_state(StateFlags::all())` (`tauri_plugin_window_state::AppHandleExt`) + `app.exit(0)` (FR-705, R5)
- [ ] T716 [US2] No `on_page_load` de `src-tauri/src/lib.rs` (`PageLoadEvent::Started`), zerar `TrayState.live` e chamar `tray::refresh` (data-model: "página recarrega / navega → false")
- [ ] T717 [P] [US2] Em `src/lib/desktop.ts`: `reportLive(live: boolean)` (só dentro do app, `'__TAURI_INTERNALS__' in window`; `invoke('app_set_live', { live })` de `@tauri-apps/api/core`, ignora erro) e `installStopHook(stop: () => void)` que define `window.espiaStopLive` (contrato `contracts/tray.md`)
- [ ] T718 [US2] Em `src/lib/stores/room.svelte.ts`: chamar `reportLive(true)` quando `myBroadcast` passa a existir (tela nativa, navegador e câmera) e `reportLive(false)` em `stopBroadcast` e ao sair da sala; em `src/routes/RoomRoute.svelte`, instalar `installStopHook(() => room.stopGoLive())` enquanto a sala está montada (remover ao desmontar)
- [ ] T719 [P] [US2] Testes em `test/lib/desktop.test.ts`: `reportLive` não chama nada fora do app; chama `invoke('app_set_live', { live })` com `__TAURI_INTERNALS__` falso definido (mock de `@tauri-apps/api/core`); `espiaStopLive` chama a função instalada e some ao remover

**Checkpoint**: ícone ao vivo, Parar transmissão pelo menu e Sair com confirmação.

## Phase 5: User Story 3 — Escolher o comportamento do X (P3)

**Goal**: opção "Fechar no X" no menu, lembrada entre aberturas.

**Independent Test**: ligar a opção, reabrir o app, clicar no X: fecha (com confirmação se transmitindo) — quickstart B7.

- [ ] T720 [US3] Em `src-tauri/src/tray.rs`, o item `close-on-x` como `CheckMenuItem` "Fechar no X", marcado conforme `prefs.close_on_x`; no clique, inverter, `save()` e `refresh` (FR-707: "Padrão: desligada (o X esconde)")

## Phase 6: Polish

- [ ] T721 `npm run check`, `npm test`, `cargo test`, `cargo clippy --all-targets` sem erros nem avisos
- [ ] T722 Versão `0.1.4` em `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` e `package.json`; `npm run tauri build` e anotar o tamanho do instalador (meta: + < 0,3 MB)
- [ ] T723 `npm run deploy` do site com o `app_set_live` (antes da tag: o app só sabe que está ao vivo com o site novo)
- [ ] T724 Roteiro manual B1–B8 de `specs/007-fechar-para-bandeja/quickstart.md` — **precisa da pessoa usuária**
- [ ] T725 Commit, tag `v0.1.4` e `specs/007-fechar-para-bandeja/relatorio.md` no formato do da 006

## Dependencies

- Setup (T701–T703) → Foundational (T704–T706) → US1 → US2 → US3 → Polish.
- US2 depende de US1 (o `quit` é chamado pelo X com `Exit/ConfirmExit`; o menu precisa do ícone).
- US3 só acrescenta um item ao menu da US2.
- Site (T717–T719) não depende do Rust e pode correr em paralelo à US1.

## Parallel

- T703 com T702; T705 com T706.
- US2: T713, T717 e T719 em paralelo (arquivos diferentes); T718 depois de T717.

## Implementation Strategy

1. **MVP**: Phases 1–3. O X já esconde e a transmissão não cai; Sair ainda não pergunta (sem `app_set_live`, `live` fica `false`).
2. US2: ao vivo, menu completo e confirmação. Deploy do site.
3. US3, Polish, roteiro com a galera e v0.1.4.
