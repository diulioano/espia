# Tasks: Atualizar o app sozinho

**Input**: [spec.md](./spec.md), [plan.md](./plan.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/update.md](./contracts/update.md), [quickstart.md](./quickstart.md)

**Tests**: como nas features anteriores, entram testes da lógica pura: `update::decide` e as leituras em `cargo test`, o view-model e o script de lançamento em `vitest`. As telas são conferidas pelos mocks. O ponta a ponta instala de verdade um app de teste (quickstart).

Gate de toda fase: `npm run check`, `npm test`, `npm run build`, `cargo test` (em `src-tauri`) e os mocks da fase abrindo.

## Phase 1: Setup

- [ ] T1501 Emenda **2.1.0** em `.specify/memory/constitution.md`:
  - na lista de APIs nativas ("Restrições técnicas"), acrescentar: o app pode se atualizar a partir do site (`/download/latest.json`), só com instalador assinado pela chave do projeto, e expõe à página **um** comando, `app_update(now)`, sem retorno, e recebe `window.espiaUpdate(state)` (`specs/015-atualizar-app/contracts/update.md`). O plugin de atualização não é exposto à página;
  - subir para 2.1.0, `Last Amended: 2026-11-01`, Sync Impact Report com o motivo (MINOR: acrescenta item à lista).
- [ ] T1502 **Usuário** (o auto mode e a senha interativa impedem o agente): rodar `! npx tauri signer generate -w %USERPROFILE%\.tauri\espia.key` e guardar uma cópia de `espia.key` fora do PC. O agente então lê **só** `espia.key.pub` e anota o caminho em `specs/015-atualizar-app/relatorio.md` (sem a chave privada).
- [ ] T1503 `src-tauri/Cargo.toml`: + `tauri-plugin-updater = "2"`; versão `0.1.9` (também em `src-tauri/tauri.conf.json` e `package.json`). `cargo check` passa.
- [ ] T1504 `src-tauri/tauri.conf.json`:
  - `bundle.createUpdaterArtifacts: true`;
  - `plugins.updater`: `endpoints: ["https://espia.espia.workers.dev/download/latest.json"]`, `pubkey: <conteúdo de espia.key.pub>`, `windows.installMode: "quiet"` (research R2).
- [ ] T1505 [P] `src-tauri/tauri.test.conf.json` (novo, só para o ponta a ponta): `plugins.updater.dangerousInsecureTransportProtocol: true`, `endpoints: ["http://127.0.0.1:8787/download/latest.json"]` (research R10). Mesclado com `tauri build --config src-tauri/tauri.test.conf.json`.
- [ ] T1506 [P] `package.json`: `"espia": { "appMinimum": "0.1.9" }` (research R5).

## Phase 2: Foundational (regra, contrato e aviso mockado)

- [ ] T1507 `src-tauri/src/update.rs` (novo), parte pura, sem Tauri:
  - `enum Phase { None, Available, Downloading, Installing, Failed }`;
  - `fn is_newer(published: &str, installed: &str) -> bool` (semver; inválido = `false`, FR-009);
  - `fn required(installed: &str, minimum: Option<&str>) -> bool` (`minimum` ausente ou inválido = `false`);
  - `enum Decision { Nothing, Notify, InstallNow, InstallHidden }` e `fn decide(input: Input) -> Decision`, com `Input { newer: bool, required: bool, dismissed: bool, since_start: Duration, resumed: bool, in_room: bool, live: bool, hidden_out_for: Option<Duration> }`. Regra da tabela de `contracts/update.md`:
    1. `!newer` → `Nothing`;
    2. `live` → `Notify` (nunca instala ao vivo sem clique, FR-005);
    3. `!resumed && since_start <= 10 s && !in_room` → `InstallNow` (R6);
    4. `hidden_out_for >= 30 min && !in_room` → `InstallHidden` (R9);
    5. `dismissed && !required` → `Nothing`;
    6. senão → `Notify`.
  - Constantes `START_WINDOW = 10 s`, `HIDDEN_AFTER = 30 min`, `CHECK_EVERY = 1 h`. Em debug, `ESPIA_UPDATE_EVERY` e `ESPIA_HIDDEN_UPDATE_AFTER` (segundos) trocam as duas últimas.
- [ ] T1508 `src-tauri/src/update.rs`, retomada (pura):
  - `struct Resume { room: Option<String>, hidden: bool, at: u64 }`;
  - `fn parse_resume(text: &str, now_ms: u64) -> Option<Resume>`: inválido, ou `at` com mais de 5 min (`"> 5 min → ignorado"`), → `None`; `room` passa por `links::room_path`, e se for inválida vira `None` (o resto vale);
  - `fn resume_json(r: &Resume) -> String`.
- [ ] T1509 [P] Testes em `src-tauri/src/update.rs` (`#[cfg(test)]`):
  - `is_newer` (maior, igual, menor, inválido);
  - `required` (ausente, inválido, maior, menor);
  - `decide`, uma linha por regra, e também: ao vivo nunca instala em nenhuma combinação, `resumed` nunca instala na abertura, 10 s + 1 ms vira `Notify`, bandeja 29 min não instala, `required` ignora `dismissed`;
  - `parse_resume` (velho, inválido, sala inválida, ida e volta).
- [ ] T1510 `src/lib/app-update.svelte.ts` (novo):
  - tipo `UpdateState` exatamente como em `contracts/update.md`;
  - store com `state = $state<UpdateState>({ phase: 'none' })`;
  - `install()` registra `window.espiaUpdate = (s) => state = s` (só no app, `inApp()` de `desktop.ts`);
  - função pura `updateView(state, live: boolean): UpdateView | null` → `{ title, body, primary?: 'Atualizar agora', canLater, busy, failed, confirmLive }`. Textos:
    - normal: "Tem versão nova do app" / "Leva uns segundos e volta onde você está.";
    - `required`: "Essa versão do app não funciona mais com o Espia" / "Atualize pra continuar.";
    - `downloading`: "Baixando a versão nova…";
    - `installing`: "Atualizando, o Espia volta em instantes";
    - `failed`: "Não deu pra atualizar" / link "Baixar pelo site".
  - `canLater = !required && phase === 'available'`.
- [ ] T1511 [P] `src/lib/desktop.ts`: `export function appUpdate(now: boolean): void` → `invoke('app_update', { now })`, só no app, erros ignorados (como `reportLive`).
- [ ] T1512 [P] `test/lib/app-update.test.ts`: `updateView` para cada fase, `required` sem "Depois", `confirmLive` só ao vivo e só em `available`/`failed`, `none` → `null`.
- [ ] T1513 `src/ui/components/UpdateNotice.svelte` (novo): recebe `view: UpdateView | null` e os eventos `onupdate`, `onlater`, `onconfirm`, `oncancel`. Mostra:
  - um cartão no canto inferior direito, acima da barra da sala, sem cobrir as telas nem pegar o foco (FR-002), com o castor pequeno e os tokens de `tokens.css`;
  - com `confirmLive`, depois de "Atualizar agora", uma confirmação no estilo do `RemoveDialog`: "Sua transmissão vai parar por alguns segundos." com "Atualizar agora" / "Cancelar";
  - o link "Baixar pelo site" com `target="_blank"` para `/download/Espia-setup.exe`. No app, `on_new_window` abre no navegador; um link normal navegaria a janela para o `.exe`.
- [ ] T1514 Mocks (princípio V): `src/mocks/home.ts` + cenas `update` e `update-failed`; `src/mocks/room.ts` + `update`, `update-required`, `update-confirm-live`, `update-installing`. Passar `update` ao `MockHome.svelte` e ao `MockRoom.svelte`, que montam o `UpdateNotice`. Labels no `registry.ts`. Prints em 1440 (scratchpad `shots.mjs`).

**Checkpoint**: as 6 cenas do quickstart abrem nos mocks. No app real, nada muda ainda.

## Phase 3: User Story 1 — Aviso e "Atualizar agora" (P1) 🎯 MVP

**Goal**: com o app aberto, a versão nova aparece no aviso, e um clique reabre o app atualizado na mesma sala.

**Independent Test**: quickstart, ponta a ponta 4 e 5.

- [ ] T1515 [US1] `src-tauri/src/update.rs`, parte Tauri (`#[cfg(windows)]`):
  - `UpdateState` gerenciado (`Mutex`): `found: Option<Update>`, `required`, `dismissed`, `phase`, `started_at: Instant`, `resumed: bool` (data-model.md);
  - `fn check(app)`: `app.updater_builder()`, em debug com `ESPIA_UPDATE_URL` em `.endpoints(...)`, depois `.build()?.check().await`. Erro de rede → silêncio (FR-011). Achou: `required` vem de `update.raw_json["minimum"]`;
  - `fn announce(app)`: `window.eval("window.espiaUpdate?.(" + json + ")")` com o estado atual. Sem janela → nada.
- [ ] T1516 [US1] `src-tauri/src/update.rs`: `fn install(app, hidden: bool)`:
  1. sala atual = `links::room_path(window.url().path())` (sem janela = `None`);
  2. grava `app_config_dir/update-resume.json` com `resume_json`;
  3. `phase = Downloading` → announce;
  4. `update.download_and_install(...)`, com `phase = Installing` → announce no fim do download;
  5. erro (rede, assinatura) → apaga o `update-resume.json`, `phase = Failed` → announce.

  O plugin sai do processo sozinho (research R1).
- [ ] T1517 [US1] `src-tauri/src/update.rs`: comando `#[tauri::command] app_update(webview, now: bool)`, com `capture::origin_ok(&webview)` como os outros:
  - `now` → `install(app, false)`;
  - `!now && !required` → `dismissed = true`, `phase = None` → announce;
  - `!now && required` → ignora.
- [ ] T1518 [US1] `src-tauri/build.rs`: + `"app_update"` na lista de comandos (gera `allow-app-update`); `src-tauri/capabilities/capture.json`: + `"allow-app-update"` e descrição citando a specs/015.
- [ ] T1519 [US1] `src-tauri/src/lib.rs`:
  - `.plugin(tauri_plugin_updater::Builder::new().build())`;
  - `update::app_update` no `generate_handler!`;
  - `app.manage(update::UpdateState::new(resumed))`;
  - em `on_page_load` com `PageLoadEvent::Finished` → `update::announce` (o aviso sobrevive a recarregar e a recriar o motor, R8);
  - thread de checagem: na abertura e a cada `CHECK_EVERY` → `check` → `decide` → `Notify` = `phase = Available` + announce.
- [ ] T1520 [US1] `src-tauri/src/lib.rs` `setup`: ler `update-resume.json` (`parse_resume`) e apagá-lo. Se veio, `resumed = true` e `room` vira o endereço inicial (`create_main(app, room)`), por cima do argumento de linha de comando só se este não tiver sala.
- [ ] T1521 [US1] `src/App.svelte`: montar `UpdateNotice` uma vez (início, entrada e sala, só no app) com `updateView(store.state, live)`. `live` vem da store da sala quando há uma. "Atualizar agora": com `confirmLive`, mostra a confirmação; senão, `appUpdate(true)`. "Depois" → `appUpdate(false)`. Chamar `install()` do `app-update.svelte.ts` no início.
- [ ] T1522 [US1] Ponta a ponta 4 e 5 (quickstart): app 0.1.9 de teste instalado, numa sala com outra pessoa (Edge headless, câmera falsa, scratchpad), "0.1.10" publicado no site local. O aviso aparece. "Atualizar agora" → reabre como 0.1.10 na mesma sala, e a outra pessoa não vê "saiu/entrou". Ao vivo, pede a confirmação, e "Cancelar" não faz nada. Registrar em `relatorio.md`. Se o app **não** reabrir sozinho, anotar e trocar para `passive` no "Atualizar agora" (risco do plan).

**Checkpoint**: MVP. Com o app aberto, a versão nova chega com um clique.

## Phase 4: User Story 2 — Abrir o app já atualiza ("Depois" e reiniciar) (P1)

**Goal**: toda abertura nova do processo, fora de sala, instala na hora; o "Depois" só esconde o aviso.

**Independent Test**: quickstart, ponta a ponta 3 e 6.

- [ ] T1523 [US2] `src-tauri/src/update.rs`: na checagem da abertura, se `decide` der `InstallNow` (≤ 10 s desde `started_at`, `!resumed`, `!in_room`, `!live`), chama `install(app, false)`. `in_room` e `live` vêm de `EngineState` e `TrayState`. Teto: check + download com timeout de 15 s no total (SC-006). Estourou → `Notify`.
- [ ] T1524 [P] [US2] `src-tauri/www/loader.js` + `index.html` + `loader.css`: gancho `window.espiaUpdate = (s) => { if (s.phase === 'installing' || s.phase === 'downloading') main.dataset.state = 'updating' }`, com o texto "Atualizando o Espia…" e o castor, no mesmo visual de `loading`. Enquanto `updating`, o loader **não** segue para o site.
- [ ] T1525 [US2] Ponta a ponta 3 e 6: abrir o 0.1.9 → reabre 0.1.10 em ≤ 15 s sem clique, mostrando "Atualizando o Espia…". "Depois" → o aviso some e não volta (recarregar a página não traz de volta). Sair pela bandeja e abrir → 0.1.10. Registrar em `relatorio.md`.

## Phase 5: User Story 3 — App abaixo da versão mínima (P2)

**Goal**: abaixo do `minimum`, o aviso perde o "Depois" e não some, sem bloquear nada.

**Independent Test**: quickstart, ponta a ponta 10.

- [ ] T1526 [US3] `scripts/copy-installer.mjs` (ver T1531) escreve `minimum` a partir de `package.json` → `espia.appMinimum`. Conferir que o Rust (T1515) lê `raw_json["minimum"]` e que `app_update(false)` com `required` não esconde o aviso.
- [ ] T1527 [US3] Ponta a ponta 10: `minimum = 0.1.10` → aviso de versão mínima sem "Depois". Entrar numa sala e transmitir a câmera falsa funciona normalmente. Registrar.

## Phase 6: User Story 4 — Atualizar sozinho na bandeja (P3)

**Goal**: escondido e fora de sala há 30 min, atualiza em silêncio e continua escondido.

**Independent Test**: quickstart, ponta a ponta 7 e 8.

- [ ] T1528 [US4] `src-tauri/src/engine.rs`: `EngineState` + `hidden_out_since: Mutex<Option<Instant>>`. Em `apply`: janela `Hidden` (ou sem janela, motor solto) e `!in_room` → começa a contar, se ainda não contava; qualquer outro estado → `None`. `pub fn hidden_out_for(&self) -> Option<Duration>`. Teste puro do contador, se der para separar (como `engine_action`).
- [ ] T1529 [US4] `src-tauri/src/update.rs` + `lib.rs`:
  - a checagem passa `hidden_out_for` ao `decide`. `InstallHidden` → `install(app, true)`;
  - na abertura, `update-resume.json` com `hidden: true` → **não** criar janela: só a bandeja, com `engine::set_window(Hidden)`, como depois de soltar o motor (009). O ícone da bandeja e um link `espia://` criam a janela normalmente (`create_main`);
  - falha silenciosa: `Failed` sem janela não aparece, e tenta na próxima hora.
- [ ] T1530 [US4] Ponta a ponta 7 e 8 com `ESPIA_HIDDEN_UPDATE_AFTER=60` e `ESPIA_UPDATE_EVERY=20`: no X, fora de sala, atualiza e continua só na bandeja (nenhuma janela aparece; conferir com `win.ps1`). Clicar no ícone abre 0.1.10. No X, mas numa sala, não atualiza. Registrar.

## Phase 7: Polish & Cross-Cutting

- [ ] T1531 `scripts/copy-installer.mjs`:
  - além de `public/download/Espia-setup.exe`, copia `Espia_<versão>_x64-setup.exe` e escreve `public/download/latest.json`, exatamente como em `contracts/update.md`: `version`, `minimum`, `pub_date` ISO, `platforms.windows-x86_64.url` com a URL **por versão** e `signature` = conteúdo do `.sig` gerado pelo build;
  - apaga os `Espia_*_x64-setup.exe` de outras versões em `public/download`;
  - aceita `ESPIA_SITE` para gerar a URL do site local (ponta a ponta).

  `package.json` `desktop:release`: antes do `tauri build`, um passo Node que define `TAURI_SIGNING_PRIVATE_KEY` a partir de `%USERPROFILE%\.tauri\espia.key` (ou da variável já definida) e falha com uma mensagem clara se a chave não existir. Separar a parte pura (montar o JSON, escolher o que apagar) para testar.
- [ ] T1532 [P] `test/scripts/copy-installer.test.ts`: o JSON gerado (URL por versão, `minimum`, assinatura) e a lista de arquivos a apagar.
- [ ] T1533 Cache (R4): com o `wrangler dev` e depois em produção, ver o `Cache-Control` de `/download/latest.json`. Se for longo, `src/worker/index.ts` responde esse caminho com `Cache-Control: no-cache` (a partir do asset). Registrar.
- [ ] T1534 Assinatura (SC-005, ponta a ponta 9): trocar 1 byte do instalador servido pelo site local → o app recusa, mostra `failed` e continua no 0.1.9. Registrar.
- [ ] T1535 Gate final: `npm run check`, `npm test`, `npm run build`, `cargo test`; prints dos mocks. `specs/015-atualizar-app/relatorio.md` completo. Commit.
- [ ] T1536 Lançar o 0.1.9: `npm run desktop:release` (com a chave), depois `! npm run deploy` (usuário). Conferir que `https://espia.espia.workers.dev/download/latest.json` responde 0.1.9 e que a assinatura bate. Avisar o grupo para baixar pelo site uma última vez (M1).

## Dependencies & Execution Order

- **Setup (T1501–T1506)**: a T1502 (a chave, pelo usuário) bloqueia a T1504 e todo build assinado. T1505 e T1506 são paralelas.
- **Foundational (T1507–T1514)**: a regra em Rust (T1507–T1509) e a página (T1510–T1514) são independentes entre si.
- **US1 (T1515–T1522)**: depende de tudo acima. A T1531 (script com `latest.json`) precisa vir **antes** do ponta a ponta T1522, porque é ela que gera o "0.1.10" local. Puxar a T1531 para o começo da US1.
- **US2**: depende da US1 (usa `install` e `check`).
- **US3**: depende da US1 e da T1531.
- **US4**: depende da US1. É independente da US2 e da US3.
- **Polish**: T1533/T1534 depois da US1; T1535/T1536 no fim.

## Parallel Opportunities

- T1505 ∥ T1506.
- Rust puro (T1507–T1509) ∥ página (T1510–T1514).
- T1511 ∥ T1512 ∥ T1513 (arquivos diferentes, depois da T1510).
- T1524 (loader) ∥ T1523.
- T1532 ∥ T1533.

## Implementation Strategy

1. **MVP = Setup + Foundational + US1**: o aviso com "Atualizar agora" já resolve o problema principal ("ninguém sabe que saiu versão").
2. **+ US2**: abrir o app atualiza sozinho. É o "atualizar quando reiniciar" do pedido.
3. **+ US3, US4**: versão mínima e bandeja.
4. Tudo entra **num só lançamento, o 0.1.9**: cada lançamento pede que o grupo reinstale até existir a atualização sozinha, então não vale lançar pela metade.
