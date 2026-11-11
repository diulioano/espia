---

description: "Task list for 019 — App nativo em Rust e convite no endereço oficial"
---

# Tasks: App nativo em Rust e convite no endereço oficial

**Input**: Design documents from `/specs/019-app-rust-convite/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/convite-e-endereco.md](./contracts/convite-e-endereco.md), [contracts/app-nativo.md](./contracts/app-nativo.md), [contracts/sinalizacao-nativa.md](./contracts/sinalizacao-nativa.md), [contracts/atualizacao.md](./contracts/atualizacao.md), [contracts/dados-falsos.md](./contracts/dados-falsos.md), [quickstart.md](./quickstart.md)

**Tests**: o plano pede testes das regras puras: portes dos testes de Vitest em `test/lib` para `cargo test` (nas crates `espia-core`, `espia-net` e `espia-shell`), e Vitest para `src/lib/site-origin.ts`, `src/lib/origin-move.ts` e para as fixtures do protocolo em `app/fixtures/protocol/`. Os testes vêm antes da implementação de cada parte e devem falhar antes dela. O que depende de captura real, rede real ou do grupo é roteiro manual da `quickstart.md`, feito pela pessoa.

**Ordem da constituição (v4.0.0)**: em cada interface, primeiro o mock, depois a lógica real. No site: mock em `src/mocks/`. No app nativo: tela com dados falsos em `espia-ui` (`Espia.exe --mock <tela> --scene <cena>`), depois a ligação real em `espia`.

**Gate de cada tarefa**:

- **Site** (`src/`, `test/`, `scripts/`): `npm run check` e `npm run build` sem erros, e `npm test` verde nos testes tocados.
- **App** (`app/`): `cargo check` e `cargo test` em `app/` sem erros, e, em tarefa de tela, a tela abrindo no mock (`npm run app:mock -- <tela> --scene <cena>`, criado no T003) com todas as cenas citadas.
- Tarefa da pessoa: o registro em `specs/019-app-rust-convite/relatorio.md` é o gate.

**Regras de segurança (valem para todas as tarefas)**:

- Teste automático nunca captura a tela de verdade nem usa `--use-fake-ui-for-media-stream`. Captura real só no roteiro manual da pessoa.
- `npm run deploy`, `git push` e escrever segredo (chave privada de atualização, tokens) são feitos pela pessoa. Nenhum agente roda esses comandos.
- `src-tauri/` está congelado: nenhuma tarefa antes do T110 muda arquivo dentro dele. Copiar para `app/` é permitido; apagar, renomear ou editar o original não.
- O Worker (`src/worker/`) e `src/shared/protocol.ts` não mudam. O app só usa o protocolo que já existe.
- Nenhum segredo entra no executável nem no repositório.

## Format: `- [ ] T001 [P] [US1] Descrição com caminho de arquivo (executor)`

- **[P]**: pode rodar em paralelo, porque mexe em arquivo diferente e não depende de tarefa pendente.
- **[Story]**: só nas fases de história (Fases 3 a 9). US1 convite no endereço oficial, US2 chegar pelo endereço antigo, US3 transmitir pelo app nativo, US4 assistir pelo app nativo, US5 paridade, US6 app leve, US7 trocar de versão.
- **(executor)**: último item da linha da tarefa. `haiku` vasculha, copia, renomeia e confere lista. `sonnet` escreve código ou documento com instrução fechada. `principal` decide, revisa gate, integra partes delicadas. `pessoa` mede, testa com captura real ou com o grupo, gera chave, publica. Combinações seguem o texto de cada tarefa.
- Cada tarefa traz, em sub-itens, `Arquivos`, `Origem` (o que portar, com função ou linha), `Regras` (com os limites) e `Conferir`.
- Caminhos relativos à raiz do repositório. "Porte de X" quer dizer: ler X, reproduzir a mesma regra em Rust, com os mesmos números e os mesmos textos.

## Path Conventions

- Site e Worker: `src/`, `test/`, `scripts/` (como hoje).
- App nativo: `app/` (workspace Rust, sete crates, ver R1 do `research.md`): `app/crates/espia-core`, `espia-net`, `espia-media`, `espia-capture`, `espia-shell`, `espia-ui`, `espia`; `app/fixtures/protocol/`; `app/packaging/`; `app/spikes/`.
- App de antes: `src-tauri/` (congelado, só leitura).

---

## Phase 1: Setup (Shared Infrastructure)

**Goal**: workspace do app criado, CI, scripts, ADR e a linha de base medida.

**Independent Test**: `cargo check` em `app/` passa com as sete crates vazias; `specs/019-app-rust-convite/relatorio.md` tem a linha de base G0.

- [ ] T001 Criar o workspace do app nativo (sonnet)
  - Arquivos: `app/Cargo.toml` (workspace, `resolver = "3"`, edição 2024, Rust 1.90), `app/crates/{espia-core,espia-net,espia-media,espia-capture,espia-shell,espia-ui,espia}/Cargo.toml` e `src/lib.rs` vazio (a crate `espia` tem `src/main.rs` com `fn main() {}`), `app/README.md` (o que é cada crate e como rodar), `.gitignore` (acrescentar `app/target`).
  - Regras: dependências entre crates só as de R1: `espia-ui` depende só de `espia-core`; `espia-net`, `espia-media`, `espia-capture` e `espia-shell` dependem de `espia-core`; `espia` depende de todas. Nenhuma dependência externa nesta tarefa. Não tocar em `src-tauri/`.
  - Conferir: `cargo check --workspace` em `app/` passa; `git check-ignore app/target` confirma o ignore.
- [ ] T002 [P] Acrescentar o job do app nativo no CI (sonnet)
  - Arquivos: `.github/workflows/ci.yml`.
  - Regras: novo job `app` em `windows-latest`, com `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` com `workspaces: app`, e os passos `cargo check --workspace` e `cargo test --workspace` com `working-directory: app`. Não mudar os jobs `web` e `desktop`.
  - Conferir: o YAML continua válido (ler o arquivo inteiro); os três jobs aparecem.
- [ ] T003 [P] Acrescentar os scripts do app em `package.json` (haiku)
  - Arquivos: `package.json`.
  - Regras: quatro scripts novos, sem mexer nos existentes: `app:dev` = `cargo run -p espia --manifest-path app/Cargo.toml`, `app:check` = `cargo check --workspace --manifest-path app/Cargo.toml`, `app:test` = `cargo test --workspace --manifest-path app/Cargo.toml`, `app:mock` = `cargo run -p espia --manifest-path app/Cargo.toml -- --mock` (aceita `-- <tela> --scene <cena>` depois, como `npm run app:mock -- home --scene default`).
  - Conferir: o JSON continua válido; `npm run app:check` roda depois do T001.
- [ ] T004 [P] Explicar o uso das fixtures do protocolo (haiku)
  - Arquivos: `app/fixtures/protocol/README.md`.
  - Regras: explicar em até 25 linhas que os JSON desta pasta são lidos pelo teste Vitest `test/shared/protocol-fixtures.test.ts` (T016) e por `cargo test` em `espia-core` (T015); nome do arquivo = `client-<t>.json` ou `server-<t>.json`, onde `<t>` é o campo `t` da mensagem; mudar o protocolo exige mudar os dois lados; o Worker e `src/shared/protocol.ts` não mudam nesta feature.
  - Conferir: o texto cita os dois testes e a regra dos nomes.
- [ ] T005 [P] Escrever o ADR do app nativo e a nota no ADR 0001 (sonnet)
  - Arquivos: criar `docs/adr/0002-app-nativo-em-rust.md`; editar `docs/adr/0001-conexao-direta-e-franquia-de-midia.md` (só uma nota).
  - Origem: o formato de `docs/adr/0001-conexao-direta-e-franquia-de-midia.md`; as decisões R1 a R3 e "Contexto que mudou" do `research.md`.
  - Regras: o ADR 0002 registra a decisão de 2026-11-11 (app inteiro nativo em Rust, sem motor de página; site e servidor ficam), o motivo, as alternativas (só o envio nativo; app, site e servidor em Rust) e que ele substitui a frase "A migração ampla do motor para Rust foi adiada" do ADR 0001. No ADR 0001, acrescentar no fim uma nota de uma linha "Atualizado em 2026-11-11: ver ADR 0002 sobre a migração para Rust"; o resto continua valendo (conexão direta, sem SFU, sem TURN pago, atraso de até 500 ms, duas transmissões com até sete espectadores). Não editar `docs/design/transmissao-motor-transporte.md` aqui.
  - Conferir: os dois arquivos se citam; datas em AAAA-MM-DD.
- [ ] T006 [P] Dar a `scripts/memoria.ps1` o modo de um processo só (sonnet)
  - Arquivos: `scripts/memoria.ps1`.
  - Origem: o próprio script (parâmetros `-Watch`, `-Interval`, `-Name`; função `Get-Tree`, que soma a árvore de processos de `Espia.exe`).
  - Regras: novo parâmetro `-Nativo` (switch). Com ele, medir só o processo `Espia.exe` (sem descer para filhos), pela mesma coluna "Memória" (conjunto de trabalho privado) e imprimir uma linha `Espia (nativo): <MB> MB`. Também aceitar `-Processo <nome>` (padrão `Espia`, sem `.exe`), de modo que `-Processo Espia -Nativo` funcione. Sem `-Nativo`, o comportamento de hoje não muda.
  - Conferir: `powershell -File scripts/memoria.ps1` sem parâmetros continua igual; com `-Processo Espia -Nativo` e sem o app aberto, avisa que não achou o processo, sem erro de script.
- [ ] T007 Conferir a licença da Slint e da crate `libwebrtc` e registrar (principal)
  - Arquivos: `specs/019-app-rust-convite/research.md` (seções R2 e R3, parágrafo "Licença").
  - Regras: ler a licença atual de `slint` 1.17 (gratuita com atribuição e GPLv3, termos não conferidos até hoje) e de `libwebrtc` 0.4.x e do código do WebRTC que ela embrulha; registrar o que o projeto precisa cumprir (atribuição na tela "Sobre"? arquivo de licenças no instalador?) e se algum termo impede distribuir `Espia.exe` ao grupo. Não inventar: o que não conseguir confirmar fica como "não confirmado". Se algum termo impedir, parar e replanejar R3 (plano B `iced`) ou R2.
  - Conferir: R2 e R3 têm o parágrafo de licença com data e fonte.
- [ ] T008 G0: linha de base do app de hoje (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (criar, seção "G0").
  - O que fazer: na mesma máquina e com o mesmo jogo, usando o app instalado hoje (0.1.10) com `npm run memoria` (`scripts/memoria.ps1`) e o Diagnóstico de mídia do site: (1) FPS do jogo sem transmitir; (2) FPS do jogo transmitindo 1080p60 para 3 pessoas; (3) uso de processador do app nessa transmissão; (4) memória do app numa sala e na bandeja fora de sala; (5) qual codificador o motor usa (visto em Diagnóstico de mídia).
  - O que registrar: os cinco números, jogo, máquina (processador, placa, Windows) e data, em `relatorio.md`. Prova a base de SC-004, SC-005 e SC-006.
  - Critério de passe: os cinco itens registrados, na mesma sessão. Se não passar: parar, registrar e replanejar com o plano B de R19.

**Checkpoint**: workspace compila, CI conhece o app, linha de base G0 e licenças registradas.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: gates G1, G2, G3 e o núcleo do app nativo (lógica pura, rede, arquivos do aparelho, base de interface, esqueleto que anda).

**⚠️ CRITICAL**: bloqueia as histórias 3 a 7 (parte de app). A história 1 e a parte de site da história 2 (T080–T083) não dependem desta fase. Gate vermelho (T010, T012, T014) para tudo o que vem depois na mesma trilha.

### Gates

- [ ] T009 Prova G1 em `app/spikes/webrtc-probe/` (sonnet, com revisão do principal)
  - Arquivos: criar `app/spikes/webrtc-probe/` (crate própria, fora do workspace de produção, com `Cargo.toml` e `src/main.rs`) e `app/spikes/webrtc-probe/LEIA-ME.md` com os comandos.
  - Origem: `src/lib/peer.ts` e `src/lib/mesh.ts` (negociação perfeita, canal `ctl`, `ViewMessage` versão 2 com `encodeView`/`decodeView`), `src/lib/quality.ts` (`sendParams`), `src/shared/protocol.ts` (mensagens), `src/lib/signaling.ts` (WebSocket, `PING`).
  - Regras: usar `libwebrtc` 0.4.x. Entrar numa sala do site local (`npm run dev:worker`) com `Origin` igual ao do site local e `mediaProtocol: 2`; enviar vídeo gerado (padrão de teste, nunca captura real) 1080p60 H.264 e som; receber vídeo e som do Chrome; abrir o canal `ctl` e trocar `ViewMessage`; mudar por ligação bitrate, fps, `scaleResolutionDownBy` e `active`; imprimir qual codificador foi usado, uso de processador e memória privada do processo; medir a diferença entre som e imagem a cada minuto por 10 minutos (log). Só STUN, descartar candidatos `typ relay`. Nada de `--use-fake-ui-for-media-stream`.
  - Conferir: `cargo build --release` na crate compila no Windows; o `LEIA-ME.md` lista os itens (a) a (j) do T010 com o comando de cada um. O principal revisa antes do T010.
- [ ] T010 Rodar o G1 e registrar cada item; decidir R2 (pessoa e principal)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "G1"), `specs/019-app-rust-convite/research.md` (R2, marcar "confirmado" ou "plano B").
  - O que fazer: a pessoa roda a sonda do T009 contra o site local com o Chrome e com um iPhone (Safari) e anota o resultado de cada item; o principal decide R2.
  - Critérios de passe (todos): (a) compila no Windows com `cargo build --release`; (b) entra numa sala do site local e o Chrome recebe vídeo 1080p60 H.264 e som; (c) recebe vídeo e som do Chrome; (d) troca `ViewMessage` com o site pelo canal `ctl`; (e) muda parâmetros por ligação (bitrate, fps, redução de resolução, ativo) e lê `qualityLimitationReason` nas estatísticas de envio; (f) usa a placa de vídeo para codificar, ou o processador do processo fica em até metade do G0 (T008); (g) som e imagem não se afastam mais de 80 ms em 10 minutos; (h) Safari no iPhone recebe; (i) memória do processo e tamanho do executável anotados; (j) anotado se o Windows mostra aviso de firewall.
  - O que registrar: um número ou "sim/não" por item (a) a (j), versão da crate, máquina e data. Prova SC-004, SC-005, SC-009 (antecipado) e FR-017.
  - Se não passar: parar, registrar e replanejar com o plano B de R2 (versão modificada com codificador do Windows; plano C `str0m`, que exige replanejar as fases de mídia).
- [ ] T011 [P] Prova G2 em `app/spikes/ui-probe/` (sonnet)
  - Arquivos: criar `app/spikes/ui-probe/` (crate própria com Slint 1.17) e `LEIA-ME.md`.
  - Origem: `src/ui/pages/RoomPage.svelte`, `src/ui/components/Stage.svelte`, `StreamTile.svelte`, `src/ui/tokens.css`, `src/ui/base.css`, cenas de `src/mocks/room.ts`.
  - Regras: tela da sala com dados falsos (sem rede e sem captura), fontes Caprasimo e Figtree, cores de `tokens.css`; um vídeo 1080p60 de teste (padrão gerado) no palco e duas miniaturas, com imagem RGBA por quadro (R3). Imprimir a memória privada do processo.
  - Conferir: compila com `cargo build --release`; abre a janela; `LEIA-ME.md` diz como comparar lado a lado com `npm run dev` na cena `room` do mock.
- [ ] T012 Rodar o G2 e registrar; decidir R3 (pessoa e principal)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "G2"), `research.md` (R3).
  - O que fazer: a pessoa abre a sonda do T011 ao lado do site (mock `room`) e, com o motor do G1 carregado no mesmo processo, mede a memória; o principal decide R3.
  - Critérios de passe (todos): a tela da sala com dados falsos fica igual ao site na comparação lado a lado; mostra um vídeo 1080p60 e duas miniaturas sem engasgar; a memória privada do processo, com o motor do G1 carregado, fica em até 150 MB.
  - O que registrar: capturas das duas telas, FPS percebido, memória em MB, data. Prova a base de SC-006 e SC-010.
  - Se não passar: parar, registrar e replanejar com o plano B de R3 (`iced`).
- [ ] T013 [P] Prova G3 em `app/spikes/webview-data/` (sonnet)
  - Arquivos: criar `app/spikes/webview-data/` (crate própria com `rusty-leveldb`) e `LEIA-ME.md`.
  - Origem: R10 do `research.md`; chaves `espia:*` de `src/lib/prefs.ts` e `src/lib/saved-rooms.ts` (`espia:nickname`, `espia:golive`, `espia:volume:<apelido>`, `espia:owner:<código>`, `espia:rooms`, `espia:session:<código>`, `espia:appinvite`).
  - Regras: receber por argumento o caminho de uma cópia da pasta `%LOCALAPPDATA%\dev.espia.app\EBWebView\Default\Local Storage\leveldb`; ler sempre da cópia, nunca da pasta viva; listar as chaves das origens `https://espia.espia.workers.dev` e `https://espia.diuliano.com` e imprimir `espia:rooms`, as chaves de dono (só o código e se tem 64 hex, nunca o valor) e `espia:nickname`. O formato das chaves do Chromium é "não confirmado": documentar o que descobrir no `LEIA-ME.md`.
  - Conferir: compila; roda numa cópia de teste e imprime as chaves ou diz claramente por que não leu.
- [ ] T014 Rodar o G3 com uma cópia do perfil real e registrar; decidir R10 (pessoa e principal)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "G3"), `research.md` (R10).
  - O que fazer: com o app de antes fechado, a pessoa copia a pasta `leveldb` do perfil real para um lugar de teste, roda a sonda do T013 nela e anota o resultado; o principal decide R10.
  - Critério de passe: lê salas fixas, chaves de dono e apelido de uma cópia do perfil real do app instalado.
  - O que registrar: quantas salas, quantas chaves de dono e se o apelido apareceu (sem copiar valores secretos para o relatório), data. Prova a base de SC-003 (app).
  - Se não passar: parar, registrar e replanejar com o plano B de R10 (exportação manual pelo site, que exige replanejar a história 2 do app).

### Núcleo puro (`espia-core`)

- [ ] T015 [P] Protocolo e política de mídia em `espia-core`, com as fixtures (sonnet)
  - Arquivos: `app/crates/espia-core/src/protocol.rs`, `app/crates/espia-core/src/media_policy.rs`, `app/crates/espia-core/src/lib.rs` (declarar módulos), `app/fixtures/protocol/client-<t>.json` (6 arquivos) e `server-<t>.json` (15 arquivos), um por tipo de mensagem.
  - Origem: `src/shared/protocol.ts` (mensagens do cliente `join`, `resume`, `signal`, `live`, `unlive`, `turn-refresh`; do servidor `welcome`, `live-ack`, `room-not-found`, `room-full`, `resume-failed`, `peer-joined`, `peer-left`, `peer-live`, `signal`, `ice-servers`, `error`, `room-renamed`, `room-deleted`, `removed`, `pong`; `CLOSE`, `PING`, `NICKNAME_MAX`, `LIVE_LABEL_MAX`, `SAVED_ROOMS_MAX`) e `src/shared/media.ts` (`MEDIA_PROTOCOL = 2`, `directIceServers`, `nextBroadcastOrder`, `isRequestId`, `isSafeSequence`, `sameTransmission`).
  - Regras: tipos `serde` que espelham o TypeScript (campo `t` como tag). `PING` é exatamente `{"t":"ping"}`. Código de fechamento `update` é 4426. `nickname` até 24, rótulo ao vivo até 60, no máximo 5 salas guardadas. `directIceServers` mantém só URLs que começam com `stun:` ou `stuns:`. `nextBroadcastOrder` com as mesmas regras de `src/shared/media.ts`. Escrever os testes primeiro: portar `test/lib/media-policy.test.ts` e `test/worker/media-protocol.test.ts` para `cargo test`, e um teste que lê cada fixture, decodifica e recodifica sem perder campo.
  - Conferir: `cargo test -p espia-core` passa; existem 21 fixtures com os nomes `client-<t>.json`/`server-<t>.json`.
- [ ] T016 [P] Teste Vitest das fixtures do protocolo (sonnet)
  - Arquivos: criar `test/shared/protocol-fixtures.test.ts` (a pasta `test/shared/` ainda não existe; `vitest.config.ts` já inclui `test/**/*.test.ts`).
  - Origem: `src/worker/validate.ts` (`parseClientMessage` na linha 54), `src/shared/protocol.ts`; fixtures do T015.
  - Regras: ler `app/fixtures/protocol/client-*.json` e passar cada uma, como texto, por `parseClientMessage`; todas devem ser aceitas (não nulas) e devolver o mesmo `t`. Para `server-*.json` não há validador no Worker: o teste confere que o `t` é um dos 15 tipos de `ServerMessage` e que os campos obrigatórios do tipo `ServerMessage` em `src/shared/protocol.ts` existem (a lista de campos de cada tipo vem desse tipo e fica escrita no próprio teste). Falhar se faltar fixture de algum tipo. Não alterar `src/worker/validate.ts` nem `src/shared/protocol.ts`.
  - Conferir: `npm test -- test/shared/protocol-fixtures.test.ts` verde depois do T015; `npm run check` verde.
- [ ] T017 [P] Código, nome da sala e rota em `espia-core`, com testes (sonnet)
  - Arquivos: `app/crates/espia-core/src/room_id.rs`, `room_name.rs`, `route.rs`.
  - Origem: `src/lib/room-id.ts` (`ROOM_ID_ALPHABET`, `ROOM_ID_RE`, `generateRoomId`, `parseRoomId`, `roomPath`), `src/shared/room-name.ts` (`normalizeRoomName`), `src/lib/route.ts` (`parseRoute`, `sameRoute`); testes de `test/lib/room-id.test.ts`, `test/lib/room-name.test.ts`.
  - Regras: alfabeto `abcdefghjkmnpqrstuvwxyz23456789`, código `xxx-xxx`; `parseRoomId` aceita código solto e links (como a TS); nome de sala até 40, mesma normalização. Portar os testes primeiro.
  - Conferir: `cargo test -p espia-core` passa com os mesmos casos dos testes TS.
- [ ] T018 [P] Tipos de view-model em `espia-core` (sonnet)
  - Arquivos: `app/crates/espia-core/src/view_model.rs`.
  - Origem: `src/lib/types.ts` (`RoomView` na linha 152, `RoomActions` na linha 338 e os tipos que eles usam: pessoas, transmissões, `link`, avisos, `fullscreenId`).
  - Regras: structs e enums puros, `Clone`, `PartialEq`, sem referência a rede, mídia ou tela. Campo que só existe para mock do site (`mockViewer`) fica de fora; cenas de celular não entram (R17). Campo que não tiver equivalente claro: registrar em comentário.
  - Conferir: `cargo check -p espia-core`; um teste simples constrói um `RoomView` de exemplo.
- [ ] T019 [P] Dados do aparelho: `device.rs` com junção e corte, e testes (sonnet)
  - Arquivos: `app/crates/espia-core/src/device.rs`.
  - Origem: `data-model.md` (modelo do `device.json`), R9 e R12 do `research.md`, `src/lib/prefs.ts` e `src/lib/saved-rooms.ts` (`loadSavedRooms`, `rememberRoom`, `SAVED_ROOMS_MAX`).
  - Regras: modelo `{ version: 1, nickname, golive, volumes, rooms, owners, sessions, importedFromWebview }`. `rooms` no máximo 5, ordenada por `lastUsed` do mais recente. `owners` nunca é cortada. `sessions` vale 24 h. `volumes` entre 0 e 1. Validação: código `xxx-xxx`, chave de dono com 64 hex, nome de sala até 40, apelido até 24. Junção só acrescenta, nunca troca valor existente; `rooms` por união por código, e no mesmo código fica a entrada de maior `lastUsed`, com o nome dela; item inválido é ignorado em silêncio; junção idempotente. Escrever os testes primeiro (junção, corte em 5 com dono fora da lista, idempotência, inválidos).
  - Conferir: `cargo test -p espia-core device` passa.
- [ ] T020 Regras de qualidade, processador, assistir e afins em `espia-core`, com testes portados (sonnet)
  - Arquivos: `app/crates/espia-core/src/{quality.rs,cpu_governor.rs,watch_state.rs,live_label.rs,audio_source.rs,devices.rs,join_view.rs}`.
  - Origem: `src/lib/quality.ts` (`effectiveQuality`, `sendParams`, `stepDown`, `stepUp`, `isAdjusted`, `countFullViewers`, `ViewMode`, `wantsVideo`, `wantsSound`), `src/lib/cpu-governor.ts` (`CpuGovernor`, `DOWN_AFTER = 10`, `UP_AFTER = 60`, `MAX_STEPS = 2`), `src/lib/watch-state.ts` (`chooseWatch`, `pruneWatching`), `src/lib/live-label.ts` (`liveLabel`), `src/lib/audio-source.ts` (`decideAudio`, `audioLabelText`, `nativeAudioText`), `src/lib/devices.ts` (`isCaptureCard`, `audioRole`, `suggestAudio`, `previewIds`, `PREVIEW_LIMIT = 4`; as funções `audioConstraints` e `videoConstraints` são de navegador e não portam), `src/lib/join-view.ts` (`NICKNAME_MAX = 24`, `cleanNickname`, `whoIsThere`, `whoIsThereLong`, `liveLabel`); testes `test/lib/{quality,quality-content,cpu-governor,watch-state,live-label,audio-source,devices}.test.ts`.
  - Regras: números exatos: 1 a 4 espectadores cheios = qualidade escolhida; 5 a 6 = teto de 4 Mbps e 720p60 (se escolheu 60) ou 1080p30; 7 ou mais = 720p30; miniatura 360p, 15 fps, 500 kbps; `hidden` e `off` não enviam (`active: false`); mínimo 720p30 ao descer degrau. Portar os testes primeiro, um arquivo de teste por módulo.
  - Conferir: `cargo test -p espia-core` passa, com a mesma quantidade de casos dos testes TS portados.
- [ ] T021 Redutor da sala em `espia-core::room` (principal desenha, sonnet escreve)
  - Arquivos: `app/crates/espia-core/src/room/{mod.rs,state.rs,event.rs,effect.rs,reduce.rs}`.
  - Origem: `src/lib/stores/room.svelte.ts` (a classe `RoomStore`, que mistura estado, rede e mídia; porta só o estado e as decisões), `src/lib/room-view.ts` (`viewModes`, `viewerStream`).
  - Regras: função pura `reduce(estado, evento) -> (estado, efeitos)`. Eventos: mensagens do servidor, eventos de ligação, ações da pessoa, relógio. Efeitos: enviar mensagem, abrir ou fechar ligação, copiar convite, guardar dado do aparelho, mostrar aviso. O principal escreve o desenho dos estados, eventos e efeitos num comentário no topo de `mod.rs` antes de o sonnet escrever; sem rede, mídia ou tela. Escrever os testes primeiro, portando os de `test/lib/room-view.test.ts` e `test/lib/view-modes.test.ts`; casos de `test/lib/peer-lifecycle.test.ts` que dependem de WebRTC ficam para o T037.
  - Conferir: `cargo test -p espia-core room` passa; a crate continua sem dependência de rede, mídia ou tela (`cargo tree -p espia-core`).

### Rede (`espia-net`)

- [ ] T022 [P] API HTTP em `espia-net` (sonnet)
  - Arquivos: `app/crates/espia-net/src/api.rs`, `lib.rs`.
  - Origem: `src/lib/api.ts` (`createRoom`, `createFixedRoom`, `fetchStatuses`, `renameRoom`, `deleteRoom`, `removePerson`, `ownerErrorText`, `fetchPreview`; tipos `CreateResult`, `OwnerFailure`).
  - Regras: `reqwest` com `rustls`; base `https://espia.diuliano.com`. A variável de ambiente `ESPIA_SERVER` (exemplo `http://127.0.0.1:8787`) troca a base, só em build de depuração; em release a base é sempre `https://espia.diuliano.com`. Todo `POST` manda o cabeçalho `Origin` igual à origem da base em uso (o Worker exige mesmo host, `sameOrigin` em `src/worker/index.ts`). A base é parâmetro, para o teste usar um servidor local. Mesmos motivos de falha e mesmos textos de erro do site. Escrever os testes primeiro, com servidor local falso, conferindo o cabeçalho `Origin`.
  - Conferir: `cargo test -p espia-net api` passa.
- [ ] T023 [P] Sinalização por WebSocket em `espia-net` (sonnet)
  - Arquivos: `app/crates/espia-net/src/signaling.rs`.
  - Origem: `src/lib/signaling.ts` (classe `Signaling`, `PING_EVERY_MS = 20_000`, reconexão, retomada com `peerId` e `resumeToken`, `SignalingEnd`), `src/shared/protocol.ts` (`CLOSE`).
  - Regras: `tokio` e `tokio-tungstenite`; `GET /ws/:code` com `Origin` igual à origem da base em uso (`https://espia.diuliano.com`, ou o valor de `ESPIA_SERVER` só em build de depuração; o esquema vira `ws://` ou `wss://` conforme a base); a primeira mensagem é `join` ou `resume` com `mediaProtocol: 2`; keep-alive com o `PING` exato `{"t":"ping"}` a cada 20 s; reconexão com retomada; cada código de fechamento vira o mesmo `SignalingEnd` do site (`not-found`, `full`, `replaced`, `kicked`, `deleted`, `removed`, `update` para 4426). Nunca manda `turn-refresh` (só STUN) e ignora entradas de `ice-servers` que não são STUN. Escrever os testes primeiro, com servidor WebSocket local falso.
  - Conferir: `cargo test -p espia-net signaling` passa; o teste vê o `Origin` e o `PING` exato.

### Arquivos do aparelho (`espia-shell`)

- [ ] T024 [P] Arquivos do aparelho com escrita atômica (sonnet)
  - Arquivos: `app/crates/espia-shell/src/storage.rs`.
  - Origem: `data-model.md` e R9; `src-tauri/src/tray.rs` (`parse_prefs`, `prefs_json`, campos `closeOnX` e `trayNoticeShown` de `tray.json`).
  - Regras: pasta `%APPDATA%\dev.espia.app\` (a base é parâmetro, para o teste usar pasta temporária). Ler e gravar `device.json`, `tray.json` (formato de hoje, reaproveitado), `window.json` (posição e tamanho) e `resume.json` (`{ "room": "<código>", "hidden": bool, "at": ms }`). Escrita atômica: gravar em arquivo temporário e renomear. Arquivo ausente ou inválido vira o padrão, sem erro. Nada vai ao servidor. Escrever os testes primeiro (ida e volta, arquivo corrompido, escrita interrompida não deixa arquivo pela metade).
  - Conferir: `cargo test -p espia-shell storage` passa.

### Interface base (`espia-ui`)

- [ ] T025 `build.rs` dos tokens, fontes e ícones (sonnet)
  - Arquivos: `app/crates/espia-ui/build.rs`, `app/crates/espia-ui/ui/tokens.slint` (gerado pelo `build.rs` e listado no `.gitignore`), `app/crates/espia-ui/assets/` (fontes e ícones).
  - Origem: `src/ui/tokens.css` (cores, raios, sombras), `src/ui/base.css`, pacotes `@fontsource/caprasimo` e `@fontsource/figtree` já instalados em `node_modules` (pesos usados em `src/main.ts`: Caprasimo 400, Figtree 400, 600 e 700), ícones Lucide.
  - Regras: o `build.rs` lê `src/ui/tokens.css` e gera os valores para a Slint; se um token não puder ser lido, o build falha dizendo qual. Embutir as fontes nos pesos acima e os ícones Lucide usados como SVG. Cores só de tokens, nunca valor solto nas telas. Escrever primeiro um teste do leitor de tokens (em `espia-ui`, sem abrir janela).
  - Conferir: `cargo check -p espia-ui` passa; o arquivo gerado tem o mesmo número de tokens de cor que `tokens.css`.
- [ ] T026 Componentes base em Slint (sonnet)
  - Arquivos: `app/crates/espia-ui/ui/components/{logo,mascot,avatar,avatar-stack,spinner,text-field,toast,toast-stack,buttons}.slint`.
  - Origem: `src/ui/components/{Logo,Mascot,Avatar,AvatarStack,Spinner,TextField,Toast,ToastStack}.svelte` e `src/ui/base.css` (classes `.btn`).
  - Regras: mesma aparência, mesmos tokens do T025, mesmos textos. Componentes só recebem dados por propriedades (view-models de `espia-core`), nunca chamam rede, captura ou mídia (regra IV). Cada componente tem uma cena na infra de dados falsos (T027).
  - Conferir: `cargo check -p espia-ui`; abrir o índice do mock (`npm run app:mock`) mostra os componentes sem erro.
- [ ] T027 Infra de dados falsos: `--mock`, `--scene`, registro de cenas e índice (sonnet)
  - Arquivos: `app/crates/espia-ui/src/mocks/{mod.rs,registry.rs}`, `app/crates/espia-ui/ui/mock-index.slint`, `app/crates/espia-ui/ui/mock-switcher.slint`, `app/crates/espia/src/cli.rs`.
  - Origem: `src/mocks/registry.ts` (telas e cenas registradas), `src/mocks/MockIndex.svelte`, `src/mocks/MockSwitcher.svelte`, `contracts/dados-falsos.md`.
  - Regras: `Espia.exe --mock <tela> --scene <cena>`; `--mock` sozinho abre o índice. Telas: `home`, `join`, `room`, `notfound`, `offline` (a tela `mobile` e toda cena marcada `device: 'mobile'` ficam só no site). Tela ou cena que não existe abre o índice com uma linha no alto dizendo qual nome não existe, sem travar. `--mock` junto de um link de sala: vale o mock e o link é ignorado. O modo mock nunca abre rede, câmera nem captura. Escrever primeiro o teste do leitor de opções (`--mock`, `--scene`, link `espia://s/<código>` como argumento).
  - Conferir: `cargo test -p espia cli` passa; `npm run app:mock` abre o índice.
- [ ] T028 [P] Tela inicial com dados falsos (sonnet)
  - Arquivos: `app/crates/espia-ui/ui/pages/home.slint`, `ui/components/{saved-rooms-panel,saved-room,delete-room-dialog}.slint`, `app/crates/espia-ui/src/mocks/home.rs`.
  - Origem: `src/ui/pages/HomePage.svelte`, `src/ui/components/{SavedRoomsPanel,SavedRoom,DeleteRoomDialog}.svelte`, `src/mocks/home.ts` (todas as cenas não marcadas `device: 'mobile'`, mesmos `id`: 13).
  - Regras: mesma disposição e mesmos textos ("Transmitir", sem "Go Live" nem "compartilhar"); Suas salas com no máximo 5 salas.
  - Conferir: `npm run app:mock -- home` abre cada cena de `src/mocks/home.ts`; `cargo check` passa.
- [ ] T029 [P] Entrada com dados falsos (sonnet)
  - Arquivos: `app/crates/espia-ui/ui/pages/join.slint`, `ui/components/name-dialog.slint`, `app/crates/espia-ui/src/mocks/join.rs`.
  - Origem: `src/ui/pages/JoinPage.svelte`, `src/ui/components/NameDialog.svelte`, `src/mocks/join.ts` (cenas não marcadas `device: 'mobile'`: 5).
  - Regras: apelido até 24; no máximo os dois passos (apelido e entrar); mesmos textos.
  - Conferir: `npm run app:mock -- join` abre cada cena de `src/mocks/join.ts`; `cargo check` passa.
- [ ] T030 [P] Sala não encontrada e sem conexão com dados falsos (sonnet)
  - Arquivos: `app/crates/espia-ui/ui/pages/notfound.slint`, `ui/pages/offline.slint`, `app/crates/espia-ui/src/mocks/notfound.rs`.
  - Origem: `src/ui/pages/NotFoundPage.svelte`, `src/mocks/notfound.ts` (cenas não marcadas `device: 'mobile'`: 4), `src-tauri/www/index.html` (tela de sem conexão, com "tentar de novo").
  - Regras: a tela de sem conexão entra como tela nova do mock, `offline`, com a cena `default` (registrada em `contracts/dados-falsos.md`).
  - Conferir: `npm run app:mock -- notfound` abre cada cena e `npm run app:mock -- offline` abre a tela de sem conexão.
- [ ] T031 Sala com dados falsos, em dois lotes: estrutura e cenas (sonnet)
  - Arquivos: lote 1 (estrutura) `app/crates/espia-ui/ui/pages/room.slint` e `ui/components/{stage,stream-tile,participant,room-menu,live-badge,reconnect-pill,reconnect-fail,remove-dialog,media-diagnostics,permission-alert,update-required}.slint`; lote 2 (cenas) `app/crates/espia-ui/src/mocks/room.rs`.
  - Origem: `src/ui/pages/RoomPage.svelte` e `src/ui/components/{Stage,StreamTile,Participant,RoomMenu,LiveBadge,ReconnectPill,ReconnectFail,RemoveDialog,MediaDiagnostics,PermissionAlert,UpdateRequired}.svelte`; cenas de `src/mocks/room.ts` (o convite mostrado vem de `inviteUrl`, linha 308, e passa a ser `https://espia.diuliano.com/s/<código>`).
  - Regras: o lote 2 só começa depois do lote 1. Todas as cenas de `src/mocks/room.ts` entram, com os mesmos `id` (o teste do T033 confere). Tela cheia, janela flutuante e vídeo real ficam para as fases 4 e 5; aqui só os estados visuais.
  - Conferir: `npm run app:mock -- room` abre cada cena; `cargo check` passa.
- [ ] T032 Seletor de transmissão com dados falsos (sonnet)
  - Arquivos: `app/crates/espia-ui/ui/components/{go-live-modal,source-card}.slint`; acrescentar as cenas do seletor em `app/crates/espia-ui/src/mocks/room.rs`.
  - Origem: `src/ui/components/{GoLiveModal,SourceCard}.svelte` e as cenas correspondentes de `src/mocks/room.ts`.
  - Regras: lista de telas e janelas com miniatura, selo "tocando", qualidade (720p ou 1080p; 30 ou 60 fps), opção de som e aviso de som indisponível, mesmos textos do site.
  - Conferir: `npm run app:mock -- room --scene <cena do seletor>` abre cada cena do seletor.
- [ ] T033 Teste de paridade de cenas (sonnet)
  - Arquivos: `app/crates/espia-ui/tests/scene_parity.rs`.
  - Origem: `src/mocks/{home,join,room,notfound}.ts` (lidos como texto, a partir da raiz do repositório); `src/mocks/registry.ts`.
  - Regras: extrair todo `id` de cena desses quatro arquivos e conferir que existe no registro do app (T027); falhar listando os que faltam. O teste expande os seis ids `watch-*` gerados por `.map` em `src/mocks/room.ts` e pula toda cena marcada `device: 'mobile'` (`home`/`mobile`, `join`/`mobile`, `join`/`fixedmobile`, `notfound`/`mobile`). Conta esperada: home 13, join 5, room 47, notfound 4, total 69. Cenas só do app são permitidas (`home`/`update-available`, `home`/`update-minimum` e `offline`/`default`, T096 e T030).
  - Conferir: `cargo test -p espia-ui scene_parity` passa depois do T031 e do T032.

### Esqueleto que anda (`espia`)

- [ ] T034 Laço principal: Slint, `tokio`, estado e rotas (principal desenha, sonnet escreve)
  - Arquivos: `app/crates/espia/src/{main.rs,app.rs,router.rs,bind.rs}`.
  - Origem: `src/App.svelte` e `src/lib/router.svelte.ts` (rotas `home` e `room`); redutor do T021.
  - Regras: o principal define, num comentário no topo de `app.rs`, quem é dono do estado e como os efeitos do redutor viram chamadas a `espia-net`, `espia-media`, `espia-capture` e `espia-shell`. O endereço base do servidor é `https://espia.diuliano.com`; só em build de depuração a variável `ESPIA_SERVER` (exemplo `http://127.0.0.1:8787`) o troca, e o `Origin` segue a base em uso. A Slint roda na thread principal e o `tokio` em outra, ligados por canais; a tela só vê view-models de `espia-core` (nunca tipos de rede). O modo `--mock` continua sem rede.
  - Conferir: `cargo check` e `cargo test` em `app/` passam; `npm run app:dev` abre a janela com a tela inicial.
- [ ] T035 Tela inicial real (sonnet)
  - Arquivos: `app/crates/espia/src/home.rs` (e o que a tela precisar em `espia-ui`, só view-models).
  - Origem: `src/routes/HomeRoute.svelte` (criação em torno da linha 185: sala fixa guarda a sala, copia o convite e usa `created: true`), `src/lib/saved-rooms.ts` (`rememberRoom`, `renameSaved`, `forgetRoom`, `buildSavedRooms`).
  - Regras: criar sala rápida e sala fixa (nome até 40); entrar por código ou link colado (`parseRoomId`); Suas salas com no máximo 5 salas e os números de pessoas e ao vivo de `fetchStatuses`; ao criar sala fixa, guardar a chave de dono em `device.json` e copiar o convite com o endereço oficial.
  - Conferir: `cargo test` passa; roteiro curto: criar sala rápida e fixa contra o site local e ver a sala no navegador.
- [ ] T036 Entrada e sala sem mídia (sonnet)
  - Arquivos: `app/crates/espia/src/room_screen.rs`.
  - Origem: `src/routes/RoomRoute.svelte` (usa `loadSavedRooms`, `forgetRoom`; sala vencida e não encontrada), `src/lib/api.ts` (`fetchPreview`), redutor do T021.
  - Regras: prévia de quem está na sala, apelido (até 24), entrar, lista de pessoas, sair, queda com volta sozinha (retomada pela sessão de `device.json`, válida por 24 h, `SESSION_MAX_AGE_MS`), sala cheia, sala não encontrada. Sem vídeo nem som ainda.
  - Conferir: `cargo test` passa; roteiro curto contra o site local: entrar, cair a rede, voltar sem refazer nada.
- [ ] T037 Ligação entre pessoas sem mídia em `espia-media` (principal desenha, sonnet escreve)
  - Arquivos: `app/crates/espia-media/src/{peer.rs,mesh.rs,view_channel.rs,lib.rs}`.
  - Origem: `src/lib/peer.ts` (`encodeView`, `decodeView`, negociação perfeita, descarte de candidatos `typ relay` na linha 204, canal de dados `ctl`), `src/lib/mesh.ts`, `src/shared/media.ts` (`directIceServers`).
  - Regras: uma ligação por par; negociação perfeita (educado = `myId < id`); canal `ctl` com `ViewMessage` versão 2 (`{t:'view', version:2, transmission, revision, mode}`, modos `full`, `thumb`, `hidden`, `off`); só STUN (entradas de `ice-servers` que não são STUN são ignoradas e o app nunca manda `turn-refresh`), candidatos de retransmissão descartados; nada de vídeo ou som aqui. O principal desenha antes quem possui a ligação e como o `libwebrtc` é escondido atrás de um trait testável. Escrever os testes primeiro, portando os casos de `test/lib/peer-lifecycle.test.ts`, `test/lib/peer-codecs.test.ts` e `test/lib/signaling-media.test.ts` que não dependem de navegador.
  - Conferir: `cargo test -p espia-media` passa.
- [ ] T038 Conferência: o app entra numa sala do site local e troca `ViewMessage` (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "Esqueleto").
  - O que fazer: subir o site local (`npm run dev:worker`), abrir uma sala no Chrome, abrir `npm run app:dev` e entrar na mesma sala.
  - O que registrar: se o app aparece na lista de pessoas do Chrome e o Chrome na do app; se o canal `ctl` abre e troca `ViewMessage` (log da ligação); data e versão. Prova FR-014 (início) e que a Phase 2 pode seguir para as histórias 3 e 4.

**Checkpoint**: núcleo puro com testes verdes, app anda sem mídia, G1, G2 e G3 passaram. As fases 4 e 5 podem começar.

---

## Phase 3: User Story 1 - Convite sempre com o endereço oficial (Priority: P1) 🎯 MVP

**Goal**: o convite mostrado e copiado usa sempre `espia.diuliano.com`, em qualquer endereço em que a página foi aberta; links antigos continuam entrando na mesma sala.

**Independent Test**: abrir a mesma sala pelo endereço oficial, pelo endereço antigo e pelo app. Copiar o convite nos três e conferir o texto copiado e o texto mostrado na sala. Abrir um link antigo e conferir que cai na mesma sala.

> Só site. Não depende da Phase 2. Pode ser publicada sozinha (MVP).

- [X] T039 [P] [US1] Teste da tabela de hosts do convite (sonnet)
  - Arquivos: criar `test/lib/site-origin.test.ts`.
  - Origem: contrato `contracts/convite-e-endereco.md`; R11 do `research.md`.
  - Regras: casos de `inviteOrigin(location)`: `espia.diuliano.com` devolve `https://espia.diuliano.com`; `espia.espia.workers.dev` devolve `https://espia.diuliano.com`; `localhost:5173` e `127.0.0.1:8787` devolvem o `location.origin` deles (FR-007); host qualquer de teste devolve o próprio `origin`; constantes `OFFICIAL_ORIGIN = 'https://espia.diuliano.com'` e `LEGACY_HOSTS = ['espia.espia.workers.dev']`. Entrada é um objeto `{ origin, host }` (sem depender de `window`).
  - Conferir: o teste falha antes do T040 (módulo não existe) e `npm test -- test/lib/site-origin.test.ts` fica verde depois.
- [X] T040 [US1] Criar `src/lib/site-origin.ts` (sonnet)
  - Arquivos: criar `src/lib/site-origin.ts`.
  - Regras: exportar `OFFICIAL_ORIGIN = 'https://espia.diuliano.com'`, `LEGACY_HOSTS = ['espia.espia.workers.dev']` e `inviteOrigin(location: { origin: string; host: string })`. Host oficial ou antigo devolve `OFFICIAL_ORIGIN`; qualquer outro host devolve `location.origin`. Sem acesso a `window` dentro da função. Faz o T039 passar.
  - Conferir: `npm test -- test/lib/site-origin.test.ts` verde; `npm run check` verde.
- [X] T041 [US1] Usar `inviteOrigin` no convite da store e do mock (sonnet)
  - Arquivos: `src/lib/stores/room.svelte.ts` (linha 518), `src/mocks/room.ts` (linha 308).
  - Regras: em `room.svelte.ts:518`, trocar `inviteUrl: location.origin + roomPath(this.code)` por `inviteOrigin(location) + roomPath(this.code)`. Em `src/mocks/room.ts:308`, trocar `` `${location.origin}/s/${MOCK_ROOM_ID}` `` por `` `${inviteOrigin(location)}/s/${MOCK_ROOM_ID}` ``. Importar de `../site-origin` (ajustando o caminho). O texto mostrado em `src/ui/pages/RoomPage.svelte:33` já deriva de `view.inviteUrl`, então não muda. A cópia (`copyInvite` em `room.svelte.ts:341-346`) e a cópia automática ao criar sala fixa (`room.svelte.ts:384-387`) usam `inviteUrl`; conferir que não leem `location.origin` por fora (`grep location.origin src`).
  - Conferir: `npm run check`, `npm test`, `npm run build` verdes. O mock `room` aberto em `localhost` mostra o endereço da própria página (FR-007); só em host oficial ou antigo mostra `espia.diuliano.com/s/...`. Anotar isso no relatório do T043.
- [X] T042 [P] [US1] Trocar o endereço antigo nos documentos que o citam como endereço do site (haiku lista, sonnet edita)
  - Arquivos: `specs/003-espia-desktop/contracts/deep-link.md`, `specs/005-seletor-nativo-desktop/contracts/native-capture.md`, `specs/007-fechar-para-bandeja/contracts/tray.md`, `specs/011-salas-fixas/contracts/local-storage.md`, `specs/015-atualizar-app/contracts/update.md`, `DEFINICOES.md`.
  - Regras: o haiku roda `grep -n "espia.espia.workers.dev"` nesses seis arquivos e devolve a lista de linhas com a frase em volta. O sonnet troca para `espia.diuliano.com` só onde o texto cita o endereço antigo como o endereço do site. Ficam como estão: onde o texto explica que o endereço antigo continua aceito (por exemplo, links antigos), registros históricos de publicação, `specs/*/relatorio.md`, `specs/*/research.md`, `quickstart.md` e `.wrangler/`. Não editar `src-tauri/src/links.rs` (congelado; ele aceita o endereço antigo de propósito, `LEGACY_HOST`). Dúvida: deixar como está e anotar na resposta.
  - Conferir: o `grep` nos seis arquivos só devolve as linhas que explicam o endereço antigo; FR-008 atendido.
- [X] T043 [US1] Conferir no mock e no site local os cinco cenários da história (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US1").
  - O que fazer: com `npm run dev` e `npm run dev:worker`, conferir os cinco cenários de aceite da história 1 da spec: (1) sala aberta pelo endereço oficial: link copiado e link mostrado começam com `espia.diuliano.com`; (2) sala aberta pelo endereço antigo: idem; (3) app já instalado: idem, sem instalar nada (só depois do T044); (4) link antigo de sala abre a mesma sala com os mesmos passos de hoje; (5) criar sala fixa copia sozinho e, ao colar, o link usa o endereço oficial. No site local (`localhost`) o convite usa o endereço da própria página (FR-007).
  - O que registrar: um "passou/falhou" por cenário e o texto copiado. Prova SC-001 e SC-002 (parcial, antes do deploy).
- [X] T044 [US1] Publicar o site e conferir pelo endereço oficial, pelo antigo e pelo app instalado (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US1, publicação").
  - O que fazer: a pessoa roda `npm run deploy` (nenhum agente roda). Depois copia o convite nas salas abertas por `https://espia.diuliano.com`, por `https://espia.espia.workers.dev` e pelo app instalado 0.1.10.
  - O que registrar: os três textos copiados e a data da publicação. Prova SC-001 (100% dos convites com o endereço oficial) e SC-002 e a história 1 inteira.

**Checkpoint**: a história 1 funciona sozinha. O convite sai sempre com o endereço oficial, inclusive no app que já está instalado.

---

## Phase 4: User Story 3 - Transmitir pelo app nativo (Priority: P1)

**Goal**: abrir o app nativo, entrar na sala, clicar em Transmitir e escolher uma janela no seletor com miniaturas; quem assiste pelo navegador não instala nada; o app usa menos processador do que hoje e o jogo não perde mais FPS.

**Independent Test**: na mesma máquina e no mesmo jogo, transmitir 1080p60 para três pessoas no navegador com o app instalado hoje e com o app nativo. Comparar FPS do jogo, uso de processador do app e a imagem e o som recebidos.

- [ ] T045 [P] [US3] Copiar a captura para `espia-capture` sem a entrega à página (haiku copia, sonnet adapta)
  - Arquivos: `app/crates/espia-capture/src/{mod.rs,sources.rs,thumbs.rs,video.rs,audio.rs,procs.rs,discord.rs}` e `app/crates/espia-capture/Cargo.toml`.
  - Origem: `src-tauri/src/capture/{mod,sources,thumbs,video,audio,procs,discord}.rs` (somente leitura; `publish.rs` não é copiado).
  - Regras: o haiku copia os sete arquivos com o mesmo conteúdo e os testes que eles têm. O sonnet remove tudo o que entrega à página (buffers compartilhados, `PostWebMessageAsJson`, comandos e tipos do Tauri), ajusta `use` e `Cargo.toml` (crate `windows` 0.62 com as mesmas features do `src-tauri/Cargo.toml`; mesmas versões onde fizer sentido), e deixa a captura expondo funções Rust puras. Não alterar nada em `src-tauri/`.
  - Conferir: `cargo check -p espia-capture` e `cargo test -p espia-capture` passam; `git status` não mostra mudança em `src-tauri/`.
- [ ] T046 [US3] Saída da captura por canal com relógio comum (sonnet)
  - Arquivos: `app/crates/espia-capture/src/output.rs`; ajustes em `mod.rs`.
  - Origem: `src/lib/capture-timing.ts` (`CaptureTiming`, `MAX_MEDIA_AGE_US = 200_000`), `src/lib/native-capture.ts` (`FrameMsg`, `SOUND_MAX_QUEUE_MS = 200`, `shouldDropSound`).
  - Regras: quadros NV12 e som PCM saem por canal (`tokio::sync::mpsc` com limite; no vídeo fica só o quadro mais novo, como em `src-tauri/src/capture/publish.rs`) para `espia-media`, cada item com carimbo de tempo do mesmo relógio; mídia mais velha que 200 ms (200 000 µs) é descartada; fila de som acima de 200 ms descarta. Escrever primeiro os testes (portando `test/lib/capture-timing.test.ts` e `test/lib/native-capture.test.ts`, na parte que não é ponte do Tauri).
  - Conferir: `cargo test -p espia-capture` passa.
- [ ] T047 [US3] Seletor real: telas, janelas, miniaturas, "tocando", qualidade, som e avisos (sonnet)
  - Arquivos: `app/crates/espia/src/go_live.rs`; `app/crates/espia-core/src/sources.rs` (regras puras de lista).
  - Origem: `src/lib/native-capture.ts` (`listSources`, `pollSources`, `POLL_MS = 2_000`, `sourceKey`, `matchLastSource`, `AudioWarn`), `src/lib/audio-source.ts` (`decideAudio`, `nativeAudioText`), `src/ui/components/GoLiveModal.svelte`, `src/ui/components/SourceCard.svelte`, `src/lib/prefs.ts` (`loadGoLivePrefs`, `DEFAULT_GO_LIVE`).
  - Regras: lista de telas e janelas atualizada a cada 2 s com miniaturas; selo "tocando" nas janelas com som; qualidade 720p ou 1080p e 30 ou 60 fps; padrão `{ resolution: 1080, fps: 30, audio: true, surface: 'window' }`; última escolha lembrada em `device.json` (`golive`) e reencontrada por `sourceKey`/`matchLastSource`; avisos de som indisponível (`unsupported`, `many_discords`) com os textos de `nativeAudioText`. Testes de regra pura primeiro, portando `test/lib/native-capture.test.ts` (lista) e `test/lib/audio-source.test.ts`.
  - Conferir: `cargo test` passa; `npm run app:mock -- room --scene <cena do seletor>` continua abrindo.
- [ ] T048 [US3] Envio de vídeo pela ligação (sonnet)
  - Arquivos: `app/crates/espia-media/src/send_video.rs`.
  - Origem: `src/lib/peer.ts` (fonte de vídeo da ligação, dois canais de envio reaproveitados, `preferHardwareCodec` na linha 403, `orderCodecs`), `src/lib/quality.ts`.
  - Regras: uma fonte de vídeo da `libwebrtc` alimentada pelos quadros do T046; os canais de envio de vídeo e de som são reaproveitados entre transmissões, nunca recriados a cada uma; H.264 preferido (ordem de codecs com H.264 primeiro); só STUN. Sem `ViewMessage` válida nada é enviado (T050 liga a regra). Escrever primeiro o teste da ordem de codecs (porte de `test/lib/peer-codecs.test.ts`).
  - Conferir: `cargo test -p espia-media` passa.
- [ ] T049 [US3] Envio de som (sonnet)
  - Arquivos: `app/crates/espia-media/src/send_audio.rs`.
  - Origem: R8 do `research.md`; `src/lib/native-capture.ts` (`soundBytes`).
  - Regras: 48 kHz, estéreo, blocos de 10 ms para a fonte de som da ligação; som PCM vindo do T046. "Som de voz" usa o processamento de voz do motor se a crate `libwebrtc` expuser (não confirmado); sem isso, vale só "Som de jogo" e o seletor diz por quê (texto: "Som de voz ainda não funciona no app. Use Som de jogo."). Escrever primeiro o teste que corta o PCM em blocos de 10 ms exatos (480 amostras por canal).
  - Conferir: `cargo test -p espia-media` passa.
- [ ] T050 [US3] Regra do Assistir no envio (sonnet)
  - Arquivos: `app/crates/espia-media/src/send_policy.rs`.
  - Origem: `src/lib/quality.ts` (`sendParams`), `src/lib/peer.ts` (`encodeView`, `decodeView`, revisão da `ViewMessage`), `src/lib/room-view.ts` (`viewModes`).
  - Regras: parâmetros por ligação conforme o modo da `ViewMessage`: `full` = qualidade efetiva (`resolution`, `fps`, `maxBitrate`); `thumb` = miniatura 360p, 15 fps, 500 kbps (`scaleResolutionDownBy = resolution / 360`); `hidden` e `off` = `active: false` (vídeo parado; o som segue em `hidden`, como no site). Sem `ViewMessage` válida, nada é enviado, nem imagem nem som (FR-021). `ViewMessage` com revisão velha é ignorada. Escrever os testes primeiro, portando `test/lib/quality.test.ts` (parte de `sendParams`) e `test/lib/view-modes.test.ts`.
  - Conferir: `cargo test -p espia-media send_policy` passa; um teste prova que uma ligação sem `ViewMessage` não envia nada.
- [ ] T051 [US3] `live` e `unlive` com `requestId` e `broadcastSeq` (sonnet)
  - Arquivos: `app/crates/espia-core/src/room/broadcast.rs` (regra pura) e ligação em `app/crates/espia/src/broadcast.rs`.
  - Origem: `src/shared/media.ts` (`nextBroadcastOrder`, `isRequestId`), trecho de `src/lib/stores/room.svelte.ts` que envia `live`/`unlive` e trata `live-ack`; `test/lib/media-policy.test.ts`.
  - Regras: cada `live` e `unlive` leva `requestId` (UUID v4) e `broadcastSeq` (primeiro é 0, depois +1); o app só considera a transmissão no ar depois do `live-ack`; respostas com ordem velha ou `requestId` repetido são ignoradas, como `nextBroadcastOrder`. Testes primeiro, no redutor.
  - Conferir: `cargo test -p espia-core` passa; no site local, o Chrome vê o app ao vivo e depois não.
- [ ] T052 [US3] Qualidade por número de espectadores e troca de qualidade ao vivo (sonnet)
  - Arquivos: `app/crates/espia-media/src/quality_apply.rs`; ligação em `app/crates/espia/src/broadcast.rs`.
  - Origem: `src/lib/quality.ts` (`effectiveQuality`, `countFullViewers`, `isAdjusted`).
  - Regras: recalcular a qualidade quando o número de espectadores em modo `full` muda ou a pessoa troca a qualidade no seletor, e reaplicar os parâmetros por ligação sem reiniciar a transmissão; 1 a 4 cheios = escolhida; 5 a 6 = teto 4 Mbps e 720p60 ou 1080p30; 7 ou mais = 720p30; miniaturas e escondidos não contam. Testes primeiro, sobre a função pura de `espia-core::quality`.
  - Conferir: `cargo test` passa; no site local, abrir e fechar espectadores no Chrome muda os parâmetros no log.
- [ ] T053 [US3] Ajuste automático por processador com os números do motor (sonnet)
  - Arquivos: `app/crates/espia-media/src/stats.rs`; ligação em `app/crates/espia-core/src/cpu_governor.rs` (já portado no T020).
  - Origem: `src/lib/cpu-governor.ts` (`DOWN_AFTER = 10`, `UP_AFTER = 60`, `MAX_STEPS = 2`), `src/lib/media-stats.ts` (`MediaStats`, `StatsRow`, `StreamMetrics`).
  - Regras: ler do motor as estatísticas de envio (limitação por processador, tempo de codificação) uma vez por segundo, alimentar o `CpuGovernor` e, quando ele pedir, descer ou subir um degrau (máximo 2) e reaplicar a qualidade (T052). O sinal de "processador no limite" é `qualityLimitationReason` igual a `cpu` nas estatísticas de envio do motor, o mesmo campo que o site lê (`SendStats.limitation` em `src/lib/peer.ts`). Se a crate expõe esse campo é o item (e) do gate G1.
  - Conferir: `cargo test` passa; testes puros do governador continuam verdes.
- [ ] T054 [US3] Nome do programa transmitido (sonnet)
  - Arquivos: `app/crates/espia-core/src/live_label.rs` (já portado; ajustar) e `app/crates/espia-capture/src/program_name.rs`.
  - Origem: `src/lib/live-label.ts` (`liveLabel`, `LabelInput`), `src/shared/protocol.ts` (`LIVE_LABEL_MAX = 60`), `src-tauri/src/capture/procs.rs` (nome do programa por janela).
  - Regras: o rótulo enviado em `live.label` é o nome do programa (por exemplo o executável sem `.exe`, formatado como o site), nunca o título da janela; até 60 caracteres; tela inteira e câmera seguem as regras de `liveLabel`. Testes primeiro (portar `test/lib/live-label.test.ts`).
  - Conferir: `cargo test` passa; no site local, o Chrome mostra o nome do programa, sem o título da janela.
- [ ] T055 [US3] Prévia parada da própria transmissão e ver a própria tela em destaque (sonnet)
  - Arquivos: `app/crates/espia/src/self_preview.rs`; `app/crates/espia-core/src/snapshot.rs` (regra pura).
  - Origem: `src/lib/snapshot.ts` (`SNAPSHOT_MS = 5_000`, `SNAPSHOT_WIDTH = 320`, `fitWidth`, `grabSnapshot`), `test/lib/snapshot.test.ts`.
  - Regras: a cada 5 s uma imagem parada com largura 320 da própria transmissão, usada na lista de pessoas; ver a própria tela em destaque usa os quadros da captura, sem passar pela rede. Testes primeiro (porte de `fitWidth`).
  - Conferir: `cargo test` passa; no app, a própria transmissão aparece em destaque e como prévia.
- [ ] T056 [US3] Placa de vídeo que não ajuda: transmite em qualidade menor e avisa (sonnet)
  - Arquivos: `app/crates/espia-media/src/encoder_fallback.rs`; texto em `app/crates/espia-core/src/notices.rs`.
  - Origem: FR-013 da `spec.md`; resultado do G1 (item f) em `relatorio.md`.
  - Regras: se a codificação por placa não estiver disponível, transmitir pelo processador em qualidade menor (degrau definido no T053) e mostrar este aviso, com este texto exato (guardado em `notices.rs`): "Sua placa de vídeo não ajudou aqui. A transmissão vai numa qualidade menor." Nunca tela preta sem explicação.
  - Conferir: `cargo test` passa (decisão pura testada); abrir o mock com a cena do aviso.
- [ ] T057 [US3] Parar, trocar de janela, transmitir de novo, sem sobra (sonnet)
  - Arquivos: `app/crates/espia/src/broadcast.rs`, `app/crates/espia-capture/src/mod.rs`.
  - Origem: trecho de `src/lib/stores/room.svelte.ts` que para e troca a captura; `src-tauri/src/capture/mod.rs`.
  - Regras: parar solta captura de vídeo, captura de som e fontes da ligação; trocar de janela e transmitir de novo duas vezes seguidas não deixa captura, thread nem som duplicado; fechar a janela para a bandeja não para a transmissão. Escrever primeiro um teste do estado de ciclo de vida (parado, abrindo, no ar, parando).
  - Conferir: `cargo test` passa; no roteiro do T058, conferir o Gerenciador de Tarefas sem sobra.
- [ ] T058 [US3] Roteiro da história 3 do quickstart, com medição de SC-004 e SC-005 (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US3").
  - O que fazer: seguir o roteiro da história 3 em `quickstart.md`, com captura real do jogo de referência, na mesma máquina do T008, com três pessoas no navegador (computador e celular, incluindo Safari).
  - O que registrar: (1) FPS do jogo sem transmitir e transmitindo 1080p60 para 3 pessoas, comparado ao G0; (2) uso de processador do app, comparado ao G0; (3) os 7 cenários de aceite da história 3 (um "passou/falhou" cada, incluindo quem não clicou em Assistir não receber nada, e placa de vídeo que não ajuda). Prova SC-004 (perda de FPS não maior que a de hoje) e SC-005 (processador cai pelo menos pela metade).
  - Gate para a troca: junto com T068 e T079, libera o app nativo a substituir o de antes.

**Checkpoint**: a história 3 funciona sozinha. O app nativo transmite para o navegador, com a regra do Assistir e as metas de SC-004 e SC-005 medidas.

---

## Phase 5: User Story 4 - Assistir pelo app nativo (Priority: P1)

**Goal**: no app nativo, clicar em Assistir e ver a imagem e ouvir o som com a mesma qualidade do navegador; ajustar volume, tela cheia ou janela flutuante; parar de assistir quando quiser.

**Independent Test**: com a mesma transmissão 1080p60, assistir lado a lado no navegador e no app nativo. Comparar o tempo até a imagem aparecer, o atraso, a fluidez, o som em sincronia e o comportamento em rede ruim.

- [ ] T059 [US4] Recepção de vídeo: quadros de cada transmissão até a tela (sonnet)
  - Arquivos: `app/crates/espia-media/src/recv_video.rs`; ligação em `app/crates/espia-ui/src/video_surface.rs` (recebe só RGBA e metadados de `espia-core`, nunca tipo da `libwebrtc`).
  - Origem: R3 do `research.md` (imagem RGBA por quadro na primeira versão); `src/ui/components/StreamTile.svelte` (como o vídeo entra no tile).
  - Regras: cada transmissão assistida entrega quadros da `libwebrtc` convertidos para RGBA e mostrados na Slint; só quadros de transmissões com `ViewMessage` `full` ou `thumb` enviada; limitar a taxa de entrega à tela ao necessário (não desenhar mais quadros que o monitor mostra). Textura direto na placa de vídeo só se o G2 mostrou custo alto (ver `relatorio.md`).
  - Conferir: `cargo check`, `cargo test` passam; no site local, o app mostra o vídeo do Chrome.
- [ ] T060 [US4] Recepção de som: misturador com volume por pessoa e saída (sonnet)
  - Arquivos: `app/crates/espia-media/src/recv_audio.rs`, `app/crates/espia-media/src/mixer.rs`.
  - Origem: R8; `src/lib/prefs.ts` (`loadVolume`, `saveVolume`: volume 0 a 1 por apelido, chave em minúsculas).
  - Regras: o som de cada pessoa entra num misturador com volume por pessoa e sai por `cpal`; volume entre 0 e 1, lembrado em `device.json` (`volumes`) pelo apelido em minúsculas; cada transmissão tem seu volume. Escrever primeiro os testes do misturador (soma, volume 0, volume 1, sem estouro: valor limitado ao intervalo válido).
  - Conferir: `cargo test -p espia-media mixer` passa; no site local, o app toca o som do Chrome.
- [ ] T061 [US4] Assistir e Parar de assistir: estado, modos de vista e `ViewMessage` (sonnet)
  - Arquivos: `app/crates/espia-core/src/watch_state.rs` (já portado; ligar), `app/crates/espia/src/watch.rs`.
  - Origem: `src/lib/watch-state.ts` (`chooseWatch`, `pruneWatching`, estados `opening`, `playing`, `unavailable`), `src/lib/room-view.ts` (`viewModes`), `src/lib/peer.ts` (`encodeView`).
  - Regras: ao entrar numa sala nada é recebido; Assistir manda `ViewMessage` `full`, Parar de assistir manda `off`; destaque e grade em `full`, miniaturas em `thumb`; cada mudança aumenta a `revision`. Nada de imagem nem som antes do clique (FR-021). Testes primeiro (portar `test/lib/watch-state.test.ts` e `test/lib/view-modes.test.ts`).
  - Conferir: `cargo test` passa; no site local, entrar na sala com transmissão não recebe nada até clicar em Assistir.
- [ ] T062 [US4] Destaque, grade e miniaturas no palco (sonnet)
  - Arquivos: `app/crates/espia-ui/ui/components/{stage,stream-tile}.slint` (trocar o vídeo falso pelo real), `app/crates/espia/src/stage.rs`.
  - Origem: `src/ui/components/Stage.svelte`, `src/ui/components/StreamTile.svelte`.
  - Regras: mesmas regras de disposição do site (destaque, grade, miniaturas); clicar numa miniatura destaca; volume por transmissão no tile; a tela só recebe view-models.
  - Conferir: `npm run app:mock -- room` abre todas as cenas do palco; no site local, o app mostra duas transmissões em destaque e miniatura.
- [ ] T063 [US4] Tela cheia (sonnet)
  - Arquivos: `app/crates/espia-ui/ui/components/fullscreen.slint`, `app/crates/espia/src/fullscreen.rs`.
  - Origem: `src/ui/components/StreamTile.svelte` (botão "Tela cheia" no computador), `src/ui/components/Viewer.svelte` (só como referência de controles; zoom de celular não entra).
  - Regras: a transmissão ocupa o monitor onde a janela está; `Esc` e o botão saem; sair não para de assistir; a recepção não muda ao abrir a tela cheia (como a regra FR-005 da 018).
  - Conferir: `cargo test` passa; roteiro curto: entrar e sair da tela cheia com a transmissão do Chrome.
- [ ] T064 [US4] Janela flutuante (sonnet)
  - Arquivos: `app/crates/espia-ui/ui/pip.slint`, `app/crates/espia/src/pip.rs`.
  - Origem: botão de janela flutuante em `src/ui/components/StreamTile.svelte` e `pip` em `src/lib/types.ts` (`RoomActions`).
  - Regras: janela própria, sempre no topo, com a transmissão escolhida e controle de volume; fechar a janela flutuante não para de assistir; só uma por vez, como no site.
  - Conferir: `cargo test` passa; roteiro curto: abrir e fechar a janela flutuante.
- [ ] T065 [US4] Queda e volta: recupera a imagem e mantém o que estava sendo assistido (sonnet)
  - Arquivos: `app/crates/espia/src/reconnect.rs`, `app/crates/espia-core/src/room/reconnect.rs`.
  - Origem: `src/lib/signaling.ts` (reconexão e retomada), `src/lib/stores/room.svelte.ts` (estado `link`: `ok`, `reconnecting`, `failed`), `src/ui/components/ReconnectPill.svelte`, `ReconnectFail.svelte`.
  - Regras: na queda, mostrar a pílula de reconexão; na volta, retomar com a mesma identidade (sessão de `device.json`, 24 h), reabrir as ligações, reenviar a `ViewMessage` do que estava sendo assistido e recuperar a imagem sozinho; falha final mostra `ReconnectFail`. Testes primeiro, no redutor.
  - Conferir: `cargo test` passa; roteiro curto: desligar e religar a rede com uma transmissão sendo assistida.
- [ ] T066 [US4] Diagnóstico de mídia com os números do motor (sonnet)
  - Arquivos: `app/crates/espia/src/diagnostics.rs`; `app/crates/espia-ui/ui/components/media-diagnostics.slint` (ligar aos dados reais).
  - Origem: `src/lib/media-stats.ts` (`MediaStats`, `StatsSample`), `src/ui/components/MediaDiagnostics.svelte`; `test/lib/media-stats.test.ts`.
  - Regras: mesmos números e rótulos do site (quadros por segundo, perda, atraso, codificador usado), vindos das estatísticas da `libwebrtc`; o campo que não existir no motor aparece como "indisponível", sem inventar valor. Testes primeiro (porte de `test/lib/media-stats.test.ts`).
  - Conferir: `cargo test` passa; `npm run app:mock -- room --scene <cena do diagnóstico>` continua abrindo.
- [ ] T067 [US4] Janela escondida ou minimizada: para de desenhar e pede `hidden` (sonnet)
  - Arquivos: `app/crates/espia/src/visibility.rs`.
  - Origem: `src/lib/room-view.ts` (`viewModes`, entrada de visibilidade), `src/lib/peer.ts` (`encodeView`), `test/lib/view-modes.test.ts`.
  - Regras: janela minimizada ou escondida na bandeja: parar de desenhar quadros e mandar `hidden` para as transmissões assistidas (o som continua), como no site; ao voltar, mandar `full` ou `thumb` de novo. Testes primeiro, no redutor.
  - Conferir: `cargo test` passa; roteiro curto: minimizar, ouvir o som continuar e ver o processador cair.
- [ ] T068 [US4] Roteiro da história 4 do quickstart, com SC-008 e SC-009 (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US4").
  - O que fazer: seguir o roteiro da história 4 em `quickstart.md`: a mesma transmissão 1080p60 assistida lado a lado no navegador e no app nativo, por 30 minutos.
  - O que registrar: (1) tempo entre clicar em Assistir e ver a imagem, e o atraso, nos dois, em rede boa; (2) som e imagem em sincronia ao longo de 30 minutos, com a maior diferença vista (meta: até 80 ms); (3) os seis cenários de aceite (um "passou/falhou" cada); (4) rede ruim: se a imagem se recupera sozinha. Prova SC-008 e SC-009.

**Checkpoint**: a história 4 funciona sozinha. O app nativo assiste com a mesma qualidade do navegador.

---

## Phase 6: User Story 5 - O mesmo Espia, sem perder função (Priority: P1)

**Goal**: instalar o app nativo e usar como sempre: mesmas telas do site, seletor com miniaturas, som da janela sem vazar o Discord, câmera, bandeja, `espia://`, Suas salas e salas fixas.

**Independent Test**: percorrer a lista de paridade (FR-012) no app nativo e marcar cada item como igual, melhor ou pior em relação ao app instalado hoje. Comparar cada tela com a tela equivalente do site, lado a lado.

- [ ] T069 [P] [US5] Câmera, placa de captura e entradas de som (sonnet)
  - Arquivos: `app/crates/espia-capture/src/camera.rs`, `app/crates/espia-capture/src/audio_input.rs`.
  - Origem: `src/lib/camera.ts` (`listDevices`, `openPreview`, `startCamera`, `isBusy`, `CameraError`: `busy`, `notfound`, `other`), `src/lib/devices.ts` (`isCaptureCard`, `audioRole`, `suggestAudio`, `previewIds`, `PREVIEW_LIMIT = 4`); `test/lib/camera.test.ts`, `test/lib/devices.test.ts`.
  - Regras: câmera e placa de captura com `nokhwa` (Media Foundation); entradas de som com `cpal`; "Som de voz" e "Som de jogo" (`AudioRole`); até 4 prévias ao mesmo tempo; câmera ocupada devolve `busy`, sem câmera `notfound`. Os quadros saem pelo canal do T046. Testes de regra pura primeiro (porte dos dois testes TS).
  - Conferir: `cargo test -p espia-capture` passa; roteiro curto na máquina da pessoa com uma câmera.
- [ ] T070 [P] [US5] Bandeja (sonnet)
  - Arquivos: `app/crates/espia-shell/src/tray.rs`.
  - Origem: `src-tauri/src/tray.rs` (`parse_prefs`, `prefs_json`, `on_close`, `on_quit`, `CloseAction`; campos `closeOnX` e `trayNoticeShown` de `tray.json`), `specs/007-fechar-para-bandeja/contracts/tray.md`; testes do próprio `tray.rs` (`prefs_ida_e_volta`, `x_esconde_por_padrao_mesmo_ao_vivo`, `x_fecha_com_a_opcao_e_pergunta_ao_vivo`, `sair_pergunta_so_ao_vivo`).
  - Regras: bandeja pela crate `tray-icon` (R16); ícone, menu, ícone de "ao vivo" quando transmitindo, "Parar transmissão", fechar para a bandeja (padrão: o X esconde, mesmo ao vivo), aviso da primeira vez (`trayNoticeShown`), confirmação ao sair transmitindo. As regras `on_close` e `on_quit` e o `tray.json` são portados sem mudança de comportamento; o app escondido transmitindo continua transmitindo. Portar os testes primeiro.
  - Conferir: `cargo test -p espia-shell tray` passa; roteiro curto: fechar pelo X, ver a bandeja, reabrir.
- [ ] T071 [P] [US5] Instância única e link `espia://` (sonnet)
  - Arquivos: `app/crates/espia-shell/src/single_instance.rs`, `app/crates/espia-shell/src/links.rs`.
  - Origem: `src-tauri/src/links.rs` (`room_path`, `room_path_from_args`, `is_internal`, `valid_code`, `ORIGIN`, `HOST`, `LEGACY_HOST`, seus testes) e o trecho de `src-tauri/src/lib.rs` com `tauri_plugin_single_instance` e `tauri_plugin_deep_link` (linhas 147 a 170); `specs/003-espia-desktop/contracts/deep-link.md`.
  - Regras: `room_path` aceita `espia://s/<código>` e `https://espia.diuliano.com/s/<código>` e o endereço antigo `https://espia.espia.workers.dev/s/<código>` (links antigos continuam entrando, FR-002), com código em minúsculas válido (`xxx-xxx`, mesmo alfabeto); outro formato devolve `None`. Instância única pelo mutex `Local\dev.espia.app.instance`; a segunda instância passa o link pelo named pipe `\.\pipe\dev.espia.app.link` e sai; a primeira abre a sala. Confirmação ao trocar de sala transmitindo. `is_internal` não é portado (não há motor de página). Regra: qualquer link `http` ou `https` clicado numa tela abre no navegador padrão (T072); as regras de `room_path` (os dois hosts aceitos, `espia://s/<código>`) são portadas. Portar os testes de `links.rs` primeiro. Não alterar `src-tauri/`.
  - Conferir: `cargo test -p espia-shell links` passa; roteiro curto: com o app aberto, abrir um `espia://s/<código>` e ver a sala.
- [ ] T072 [P] [US5] Posição e tamanho da janela e links externos no navegador padrão (sonnet)
  - Arquivos: `app/crates/espia-shell/src/window_state.rs`, `app/crates/espia-shell/src/open_external.rs`.
  - Origem: `data-model.md` (`window.json`); R9 e R16.
  - Regras: gravar posição e tamanho em `window.json` (`{ "x", "y", "width", "height", "maximized" }`, ver `data-model.md`) ao mover, redimensionar e fechar; tamanho mínimo 720×480 e padrão 1280×800 centralizada; ao abrir, validar contra os monitores atuais (se a posição salva cair fora de todos os monitores, centralizar a janela); escala de 125%, 150% e 200% e mais de um monitor sem perder nitidez. Links externos (qualquer `http` ou `https` clicado numa tela) abrem no navegador padrão, nunca dentro do app. Testes primeiro para a regra de "cabe nos monitores".
  - Conferir: `cargo test -p espia-shell` passa; roteiro curto: mover a janela, fechar, abrir.
- [ ] T073 [US5] Sala fixa: renomear, apagar, sala apagada e sala vencida (sonnet)
  - Arquivos: `app/crates/espia/src/owner.rs`; `app/crates/espia-core/src/room/owner.rs`.
  - Origem: `src/lib/api.ts` (`renameRoom`, `deleteRoom`, `ownerErrorText`, `OwnerFailure`: `forbidden`, `gone`, `name`, `rate-limited`, `network`), `src/routes/RoomRoute.svelte` e `src/lib/stores/room.svelte.ts` (mensagens `room-renamed` e `room-deleted`), `src/lib/saved-rooms.ts` (`renameSaved`, `forgetRoom`); `test/lib/saved-rooms.test.ts`.
  - Regras: só quem tem a chave de dono (64 hex em `device.json`) vê renomear e apagar; nome até 40; apagar tira a sala de Suas salas (a chave de dono nunca é descartada por corte da lista, só por apagar); sala apagada por outro e sala vencida mostram o aviso do site. Testes primeiro (porte de `test/lib/saved-rooms.test.ts`).
  - Conferir: `cargo test` passa; roteiro curto contra o site local.
- [ ] T074 [US5] Remover pessoa e o aviso "O dono te tirou da sala" (sonnet)
  - Arquivos: `app/crates/espia/src/remove_person.rs`.
  - Origem: `src/lib/api.ts` (`removePerson`), `src/ui/components/RemoveDialog.svelte`, mensagem `removed` do servidor e fechamento `kicked` de `src/lib/signaling.ts`.
  - Regras: o dono remove uma pessoa pela lista (com confirmação); a pessoa removida vê "O dono te tirou da sala" (texto exato do site) e volta para a tela inicial.
  - Conferir: `cargo test` passa; roteiro curto contra o site local com dois participantes.
- [ ] T075 [US5] Avisos da sala (toasts) e textos iguais aos do site (haiku confere textos, sonnet corrige)
  - Arquivos: `app/crates/espia-core/src/notices.rs`, `app/crates/espia-ui/ui/**/*.slint`.
  - Origem: textos em `src/lib/stores/room.svelte.ts` (por exemplo "Link copiado — é só mandar pra galera" na linha 563 e "Sala criada — fixe o link no grupo" na linha 386), `src/lib/api.ts` (`ownerErrorText`), `src/ui/**/*.svelte`.
  - Regras: o haiku extrai uma tabela "texto do site, arquivo do site, texto do app, arquivo do app" e devolve só as diferenças; o sonnet corrige o app para igualar o site. Nunca "Go Live" nem "compartilhar" em texto de interface.
  - Conferir: a tabela do haiku vazia depois da correção; `cargo test` passa.
- [ ] T076 [US5] Comparação lado a lado de cada tela com o site e correções (pessoa aponta, sonnet corrige)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US5, telas"); correções em `app/crates/espia-ui/`.
  - O que fazer: a pessoa abre cada tela do site (`?mock=<tela>&scene=<cena>`) e a mesma no app (`npm run app:mock -- <tela> --scene <cena>`) lado a lado, para as telas `home`, `join`, `room` e `notfound`, e aponta diferenças de cor, fonte, texto, ícone, castor ou posição de ação. O sonnet corrige cada apontamento.
  - O que registrar: uma linha por tela e cena com "igual" ou o apontamento e a correção. Prova SC-010 (nenhuma diferença apontada em 100% das telas).
- [ ] T077 [US5] Lista de paridade (haiku)
  - Arquivos: criar `specs/019-app-rust-convite/checklists/paridade.md`.
  - Origem: FR-012 da `spec.md`.
  - Regras: uma linha por item do FR-012, no formato `- [ ] <item> | igual / melhor / pior | observação`; sem juntar nem cortar itens; copiar o texto do item como está na spec.
  - Conferir: o número de linhas é igual ao número de itens da lista do FR-012 (contar na spec e no arquivo).
- [ ] T078 [US5] Sala mista: site, app de antes e app nativo, todas as combinações (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US5, sala mista").
  - O que fazer: numa mesma sala, com uma pessoa no site, uma no app de antes (0.1.10) e uma no app nativo, repetir cada combinação de quem transmite e quem assiste (site, app de antes e app nativo, cada um transmitindo para os outros dois).
  - O que registrar: uma linha por combinação ("passou/falhou", com imagem e som). Prova FR-014 e o cenário 3 da história 5.
- [ ] T079 [US5] Percorrer a lista de paridade e registrar (pessoa)
  - Arquivos: `specs/019-app-rust-convite/checklists/paridade.md`, `specs/019-app-rust-convite/relatorio.md` (seção "US5, paridade").
  - O que fazer: percorrer cada item da lista (T077) no app nativo e marcar "igual", "melhor" ou "pior" em relação ao app de antes.
  - O que registrar: o arquivo da lista preenchido e, em `relatorio.md`, o resultado final. Prova SC-007 (100% igual ou melhor). Item "pior" bloqueia a troca (T109).
  - Gate para a troca: junto com T058 e T068, libera o app nativo a substituir o de antes.

**Checkpoint**: a história 5 funciona sozinha. O app nativo tem toda a função do app de antes, conferida item a item.

---

## Phase 7: User Story 2 - Chegar pelo endereço antigo sem perder nada (Priority: P2)

**Goal**: abrir o site pelo endereço antigo, ou instalar o app nativo, e passar a usar o endereço oficial com salas fixas, chaves de dono, apelido e preferências presentes.

**Independent Test**: num navegador e num app com salas fixas, uma sala própria e preferências guardadas no endereço antigo, abrir o endereço antigo no navegador e instalar o app nativo por cima. Conferir o endereço final, a lista Suas salas, as ações de dono e as preferências nos dois.

> A parte de site (T080–T083) não depende da Phase 2 e pode sair junto com a Phase 3. A parte de app (T084–T086) depende da Phase 2 (T014, T019, T024) e do T085.

- [ ] T080 [P] [US2] Teste de `origin-move` (sonnet)
  - Arquivos: criar `test/lib/origin-move.test.ts`.
  - Origem: `contracts/convite-e-endereco.md`; R12 do `research.md`; `src/lib/saved-rooms.ts` (`ROOM_ID_RE` vem de `src/lib/room-id.ts`).
  - Regras: casos de empacotar, desempacotar, juntar, validar, teto e idempotência: (a) pacote `{ "v": 1, "items": { "<chave>": "<valor cru>" } }` em base64url sem preenchimento `=`, teto de 16.384 caracteres do texto codificado; se passar, descarta nesta ordem `espia:session:*`, `espia:volume:*`, `espia:golive`, `espia:appinvite` (`espia:rooms`, `espia:owner:*` e `espia:nickname` sempre vão); se ainda passar, a ida acontece sem `#mv=` e nada é apagado no endereço antigo; (b) desempacotar pacote inválido devolve nada, sem erro; (c) junção só acrescenta, nunca troca valor existente (`espia:nickname`, `espia:golive`, `espia:volume:*`, `espia:owner:*`, `espia:session:*`, `espia:appinvite`); (d) `espia:rooms`: união por código, ordem por `lastUsed` do mais recente, corte em 5; no mesmo código dos dois lados fica a entrada de maior `lastUsed`, com o nome dela; chave de dono nunca descartada, mesmo de sala fora da lista; (e) validação: código `xxx-xxx`, chave de dono com 64 hex, nome de sala até 40, apelido até 24, volume entre 0 e 1; item inválido ignorado em silêncio; (f) repetir a junção não duplica nem apaga.
  - Conferir: o teste falha antes do T081 e fica verde depois.
- [ ] T081 [US2] Criar `src/lib/origin-move.ts` (sonnet)
  - Arquivos: criar `src/lib/origin-move.ts`.
  - Origem: `src/lib/prefs.ts` (`KEY` das chaves `espia:*`), `src/lib/saved-rooms.ts` (`SAVED_ROOMS_MAX = 5` vem de `src/shared/protocol.ts`), `src/lib/room-id.ts` (`ROOM_ID_RE`).
  - Regras: funções puras (sem `window`): empacotar o `localStorage` num pacote `#mv=`, desempacotar e juntar, com as regras do T080. Faz o T080 passar.
  - Conferir: `npm test -- test/lib/origin-move.test.ts` verde; `npm run check` verde.
- [ ] T082 [US2] `src/main.ts`: sair do endereço antigo e importar no oficial (sonnet)
  - Arquivos: `src/main.ts`.
  - Origem: R12; `src/main.ts` de hoje (monta `App` e chama `installDesktopHook`); `src/lib/desktop.ts` (detecção do app de antes por `'__TAURI_INTERNALS__' in window`).
  - Regras: antes de montar a aplicação: (1) no host antigo (`LEGACY_HOSTS` de `src/lib/site-origin.ts`), fora do app de antes (`'__TAURI_INTERNALS__' in window` é falso) e sem `?mock`: empacotar as chaves `espia:*` e `location.replace` para `https://espia.diuliano.com` + caminho + busca + `#mv=<base64url>`; (2) no oficial, se houver `#mv=`, desempacotar, juntar e limpar o fragmento com `history.replaceState`. Nunca navegar no app de antes (apps instalados dependem do endereço antigo, FR-006). Sem `localStorage` (janela anônima), navegar mesmo assim, sem pacote (cenário de borda). O fragmento não vai ao servidor (FR-005). O Worker não muda.
  - Conferir: `npm run check`, `npm test` e `npm run build` verdes; no `npm run dev`, `?mock` não redireciona.
- [ ] T083 [US2] Conferir no navegador os cenários 1, 2, 3 e 6 e que o app instalado não é redirecionado (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US2, site").
  - O que fazer: com o site publicado (a regra é pelo nome do endereço, então a prova local são os testes do T080 e a prova real é o endereço antigo publicado), conferir: (1) abrir o endereço antigo chega ao oficial com salas fixas, chaves de dono, apelido e preferências; (2) link antigo de sala chega à entrada da mesma sala, em no máximo dois passos; (3) com salas nos dois endereços, as listas são juntadas sem repetir e sem perder chave de dono (máximo 5 salas); (4) o app instalado (0.1.10) abre e entra em salas sem ser redirecionado; (6) abrir o endereço antigo de novo não apaga nem duplica.
  - O que registrar: um "passou/falhou" por cenário. Prova SC-002, SC-003 (parte do navegador) e FR-003 a FR-006. O cenário 5 é o T086.
- [ ] T084 [US2] `import_webview.rs`: importação do app de antes (sonnet)
  - Arquivos: `app/crates/espia-shell/src/import_webview.rs`.
  - Origem: R10 do `research.md` e resultado do G3 (T014, sonda em `app/spikes/webview-data/`); regras de junção de `app/crates/espia-core/src/device.rs` (T019).
  - Regras: ler o `localStorage` do motor de página em `%LOCALAPPDATA%\dev.espia.app\EBWebView\Default\Local Storage\leveldb`, sempre a partir de uma cópia da pasta, com `rusty-leveldb`; origens `https://espia.espia.workers.dev` e `https://espia.diuliano.com`; chaves `espia:*`; juntar com as regras do T019 e marcar `importedFromWebview: true`; nunca apagar a pasta do motor (o app de antes continua funcionando); nada vai ao servidor. Falha de leitura com a pasta presente não impede abrir o app: abre com o que tem, não mostra nada e deixa `importedFromWebview` em `false`, para tentar de novo a cada abertura. Se a pasta do motor não existe, grava `importedFromWebview: true`. Testes primeiro, com uma pasta `leveldb` de fixture criada no teste.
  - Conferir: `cargo test -p espia-shell import_webview` passa.
- [ ] T085 [US2] Primeira abertura do app nativo: importar, reaproveitar `tray.json`, converter a posição (sonnet)
  - Arquivos: `app/crates/espia/src/first_run.rs`.
  - Origem: R9, R10; `app/crates/espia-shell/src/storage.rs` (T024) e `import_webview.rs` (T084); `src-tauri/src/tray.rs` (formato de `tray.json`).
  - Regras: na primeira abertura (sem `device.json`, ou com `importedFromWebview: false`), importar o app de antes (T084); reaproveitar o `tray.json` existente em `%APPDATA%\dev.espia.app\` sem reescrevê-lo; converter a posição e o tamanho de janela do app de antes para `window.json`, a partir de `%APPDATA%\dev.espia.app\.window-state.json` (formato não confirmado; se não der para ler, vale o padrão de 1280×800 centralizada); só uma vez. Importação que falhou com a pasta presente é tentada de novo a cada abertura (T084). Testes primeiro, com pasta temporária.
  - Conferir: `cargo test` passa; segunda abertura não importa de novo.
- [ ] T086 [US2] Conferir o cenário 5 com um perfil real e SC-003 (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US2, app").
  - O que fazer: num computador com o app de antes e 5 salas fixas (2 delas próprias) guardadas, instalar o app nativo por cima e abrir.
  - O que registrar: as 5 salas aparecem, as 2 ações de dono funcionam (renomear), o apelido e a preferência de fechar para a bandeja estão lá, e os convites saem com `espia.diuliano.com`. Prova SC-003 (app), FR-004 e FR-015.

**Checkpoint**: a história 2 funciona sozinha, no navegador e no app.

---

## Phase 8: User Story 6 - App leve (Priority: P2)

**Goal**: deixar o app aberto o dia inteiro ocupando menos memória do que hoje, como um processo só, abrindo rápido.

**Independent Test**: medir a memória do app com o roteiro de medição do projeto, numa sala assistindo, numa sala transmitindo, parado numa sala e na bandeja fora de sala. Medir o tempo de abertura. Comparar com o app instalado hoje na mesma máquina.

- [ ] T087 [US6] Fora de sala e escondido na bandeja: soltar motor de vídeo e telas (sonnet)
  - Arquivos: `app/crates/espia/src/lifecycle.rs`, `app/crates/espia-media/src/lib.rs`, `app/crates/espia-ui/src/lib.rs`.
  - Origem: `specs/009-memoria-do-app/spec.md` (meta de 40 MB na bandeja), SC-006 da `spec.md`.
  - Regras: ao sair da sala, desligar `espia-media` e liberar o motor de vídeo; ao esconder na bandeja, destruir a janela e as telas da Slint e deixar só a bandeja e o laço; ao reabrir, recriar. Meta: no máximo 40 MB na bandeja fora de sala. O que não puder ser solto sem quebrar: registrar em comentário e no `relatorio.md` pelo principal.
  - Conferir: `cargo test` passa; `npm run memoria -- -Processo Espia -Nativo` (T006) na bandeja fora de sala.
- [ ] T088 [US6] Tempo de abertura até a tela inicial em até 2 s (sonnet)
  - Arquivos: `app/crates/espia/src/startup.rs` e o que o perfil de abertura apontar.
  - Regras: medir de `main` até a tela inicial visível (log de tempo por etapa, ativado por uma variável de ambiente); cortar o que atrasa: carregar `libwebrtc`, `tokio` e leitura de arquivos só depois da primeira tela, quando possível. Meta SC-011: tela inicial em até 2 s depois de aberto.
  - Conferir: o log de tempo mostra o total; `cargo test` passa.
- [ ] T089 [P] [US6] Nome, ícone e versão do executável e um processo só (sonnet)
  - Arquivos: `app/crates/espia/build.rs`, `app/crates/espia/assets/icon.ico` (copiar de `src-tauri/icons/icon.ico`), `app/crates/espia/Cargo.toml`.
  - Origem: `src-tauri/tauri.conf.json` (`productName` "Espia", `mainBinaryName` "Espia", identificador `dev.espia.app`), `src-tauri/icons/`.
  - Regras: recurso do Windows com nome de produto "Espia", descrição, ícone e versão `0.2.0` (R15); o binário se chama `Espia.exe`; um processo só no Gerenciador de Tarefas (FR-023). Não editar `src-tauri/`.
  - Conferir: `cargo build --release -p espia` gera `Espia.exe`; as propriedades do arquivo mostram nome e versão.
- [ ] T090 [US6] Medir memória nas quatro situações e o tempo de abertura (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US6").
  - O que fazer: com `npm run memoria -- -Processo Espia -Nativo` no app nativo e `npm run memoria` no app de antes, na mesma máquina: (1) sala assistindo uma transmissão 1080p; (2) sala transmitindo; (3) parado numa sala; (4) bandeja fora de sala; e cronometrar da abertura até a tela inicial (5 aberturas, anotar a maior).
  - O que registrar: as quatro memórias do app nativo e do app de antes, e os cinco tempos. Metas: até 150 MB assistindo 1080p (SC-006), até 40 MB na bandeja fora de sala (SC-006), abaixo da do app de antes, e tela inicial em até 2 s (SC-011).
- [ ] T091 [US6] Se passar das metas: perfil de memória e correções (principal)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` e o que o perfil apontar em `app/`.
  - Regras: só se o T090 mostrar número acima da meta. Fazer o perfil de memória (por crate), listar os maiores consumidores, corrigir os de maior ganho e pedir nova medição ao T090. Se o T090 passou, registrar "não necessário" e fechar.
  - Conferir: o T090 repetido dentro das metas, ou o registro "não necessário".

**Checkpoint**: a história 6 funciona sozinha. O app nativo cumpre as metas de memória e de abertura.

---

## Phase 9: User Story 7 - Trocar de versão sem susto (Priority: P2)

**Goal**: o app nativo avisa quando tem versão nova e atualiza sem a pessoa procurar o instalador; se a versão nova der problema, dá para voltar ao app de antes ou ao site sem perder as salas.

**Independent Test**: publicar uma versão de teste mais nova e conferir o aviso e a atualização. Entrar numa sala com uma versão velha demais e conferir o aviso. Instalar o app de antes por cima do nativo e conferir que funciona.

- [ ] T092 [US7] Gerar o par de chaves da atualização e guardar a privada fora do repositório (pessoa)
  - Arquivos: `app/crates/espia-shell/src/update_key.pub` (só a chave pública, que não é segredo); chave privada em `%USERPROFILE%\.espia\update.key` (nunca no repositório).
  - O que fazer: a pessoa gera o par minisign no próprio computador (por exemplo `minisign -G -p update.pub -s %USERPROFILE%\.espia\update.key`), guarda a privada em lugar seguro com cópia, e coloca só o texto da chave pública no arquivo indicado. Nenhum agente gera, lê ou escreve a chave privada.
  - O que registrar: em `relatorio.md`, apenas "par gerado em <data>; privada em `%USERPROFILE%\.espia\update.key`" (sem o conteúdo). Prova FR-022 e o princípio VI da constituição.
- [ ] T093 [P] [US7] Configuração do instalador (sonnet)
  - Arquivos: `app/packaging/` (configuração do `cargo-packager`, textos, `hooks.nsh`).
  - Origem: R15; `src-tauri/nsis/{PortugueseBR.nsh,hooks.nsh,header.bmp,sidebar.bmp}` e `src-tauri/tauri.conf.json` (bloco `bundle`).
  - Regras: NSIS por usuário (sem permissão de administrador), mesmo identificador `dev.espia.app` e mesmo nome "Espia" para instalar por cima do app de antes; registrar `espia://`; textos em português reaproveitados de `src-tauri/nsis/`; versão do app nativo 0.2.0; sem regra de firewall (R16). `cargo-packager` não está confirmado como suficiente: se não cobrir algo, usar o plano B de R15 (script NSIS próprio em `app/packaging/`) e registrar o motivo em `research.md`.
  - Conferir: o comando de montagem gera `Espia-setup` na pasta de saída em `app/`; ler o script e ver os pontos acima.
- [ ] T094 [P] [US7] `scripts/release-app.mjs` e script `app:release` (sonnet)
  - Arquivos: criar `scripts/release-app.mjs`; editar `package.json` (script `app:release`) e `public/_headers` (`Cache-Control: no-cache` para `/download/latest.json`).
  - Origem: `scripts/copy-installer.mjs` (como o instalador é copiado hoje), `specs/015-atualizar-app/contracts/update.md`, `contracts/atualizacao.md`.
  - Regras: montar o instalador (T093), assinar com a chave privada de `%USERPROFILE%\.espia\update.key` (o script lê o caminho, nunca imprime a chave), copiar para `public/download/` e escrever `public/download/latest.json` com `version`, `notes`, `pub_date`, `minimum`, `url` e `signature`. Sem a chave, o script para com mensagem clara. O script não roda `npm run deploy` nem `git push`.
  - Conferir: `node scripts/release-app.mjs --help` descreve as etapas; `package.json` continua válido.
- [ ] T095 [US7] `update.rs`: consultar, baixar, conferir assinatura, instalar e reabrir (sonnet)
  - Arquivos: `app/crates/espia-shell/src/update.rs`.
  - Origem: `contracts/atualizacao.md`; `specs/015-atualizar-app/contracts/update.md` (referência histórica).
  - Regras: `GET https://espia.diuliano.com/download/latest.json` (campos `version`, `notes`, `pub_date`, `minimum`, `url`, `signature`); baixar o instalador; conferir a assinatura minisign com a crate `minisign-verify` e a chave pública embutida (T092) antes de instalar, e recusar se não bater; recusar também se o `url` não começar com `https://espia.diuliano.com/download/`; instalar e reabrir. A consulta acontece ao abrir o app e a cada 6 horas com ele aberto. Versão igual ou menor que a instalada: sem aviso. Escrever os testes primeiro, com `latest.json` e assinaturas de fixture (assinatura boa, ruim, versão igual, `minimum` acima da instalada).
  - Conferir: `cargo test -p espia-shell update` passa.
- [ ] T096 [US7] Aviso de versão nova nas telas, com cenas de dados falsos novas (sonnet)
  - Arquivos: `app/crates/espia-ui/ui/components/update-notice.slint`, `app/crates/espia-ui/src/mocks/{home,room}.rs` (cenas novas), `app/crates/espia/src/update_ui.rs`.
  - Origem: R14; `src/ui/components/UpdateRequired.svelte`.
  - Regras: aviso "Tem versão nova" com "Atualizar agora" e "Depois"; abaixo de `minimum` o aviso não tem "Depois". Cenas novas no mock, com e sem "Depois" (`home`/`update-available` com "Depois" e `home`/`update-minimum` sem "Depois", como em `contracts/dados-falsos.md`). Mock antes da lógica: primeiro as cenas, depois a ligação ao `update.rs`.
  - Conferir: `npm run app:mock -- home --scene update-available` e `--scene update-minimum` abrem; `cargo test` passa (o teste do T033 aceita cenas só do app).
- [ ] T097 [US7] Versão velha demais para a sala (sonnet)
  - Arquivos: `app/crates/espia/src/update_required.rs`.
  - Origem: fechamento 4426 (`CLOSE.update`) em `src/shared/protocol.ts`; `SignalingEnd` `update` em `src/lib/signaling.ts`; `src/ui/components/UpdateRequired.svelte`; `app/crates/espia-net/src/signaling.rs` (T023).
  - Regras: o fechamento 4426 mostra "Atualize o Espia" com o botão de atualizar (o mesmo do T096), em vez de falhar em silêncio; sem rede de atualização, a frase explica o que fazer. Testes primeiro, no redutor.
  - Conferir: `cargo test` passa; no site local com `mediaProtocol` forçado errado, o app mostra a tela.
- [ ] T098 [US7] Nunca atualizar transmitindo; voltar para a sala depois (sonnet)
  - Arquivos: `app/crates/espia/src/update_guard.rs`; `app/crates/espia-shell/src/storage.rs` (`resume.json`).
  - Origem: R14; modelo `resume.json` (`{ "room": "<código>", "hidden": bool, "at": ms }`).
  - Regras: nunca baixar nem instalar durante uma transmissão própria; o aviso espera a pessoa decidir; ao instalar, gravar `resume.json`; ao reabrir, voltar para a sala e para o estado (escondida ou não); `resume.json` vale 5 minutos e é apagado ao ser lido, como na 015; durante a transmissão o aviso aparece só com o texto, e os botões aparecem quando ela termina; "Depois" fecha o aviso até a próxima abertura do app. Testes primeiro.
  - Conferir: `cargo test` passa; um teste prova que transmitindo não há download.
- [ ] T099 [US7] `public/download/` com o app de antes ao lado do nativo (sonnet)
  - Arquivos: `public/download/` (copiar), `src/lib/app-invite.ts` (conferir, sem mudar o destino).
  - Origem: R13 do `research.md`; `src/lib/app-invite.ts`; `test/lib/app-invite.test.ts`.
  - Regras: copiar o `public/download/Espia-setup.exe` de hoje (0.1.10, montado em 2026-11-05) para `public/download/Espia-setup-0.1.10.exe`; o nativo continua entrando só no T109 (aqui é preparar o procedimento e, em teste, a pasta). `src/lib/app-invite.ts` continua apontando para `Espia-setup.exe`. `public/download` está no `.gitignore`; não commitar. Este é um passo de preparação: não trocar `Espia-setup.exe` antes do T109.
  - Conferir: os dois arquivos existem; `npm test -- test/lib/app-invite.test.ts` verde.
- [ ] T100 [US7] Conferir instalar por cima, atualizar e voltar (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "US7").
  - O que fazer: (1) instalar o app nativo por cima do app de antes e medir o tempo numa internet de 10 Mbps; (2) com uma versão de teste mais nova publicada (a pessoa roda `npm run app:release` e o deploy), conferir o aviso e atualizar em até 3 cliques, sem abrir o site, voltando à sala; (3) entrar numa sala com versão velha demais e ver "Atualize o Espia"; (4) estando transmitindo, ver que nada é interrompido; (5) instalar o app de antes (`Espia-setup-0.1.10.exe`) por cima do nativo e ver que funciona, com salas e preferências.
  - O que registrar: um "passou/falhou" por cenário 1 a 5, o tempo de instalação e a contagem de cliques. Prova SC-013 (menos de 2 minutos) e SC-014 (até 3 cliques).

**Checkpoint**: a história 7 funciona sozinha. Dá para atualizar sem susto e voltar ao app de antes.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: documentos, revisão de segurança, limpeza, semana de uso e troca.

- [ ] T101 [P] Registrar o app nativo em `DEFINICOES.md` e `README.md` (sonnet)
  - Arquivos: `DEFINICOES.md`, `README.md`.
  - Regras: registrar o app nativo (novo) e o app de antes, a pasta `app/`, a Slint, a `libwebrtc`, os comandos `npm run app:dev`, `app:check`, `app:test`, `app:mock`, `app:release`, o endereço oficial para convite, os gates e o que mudou do `research.md`. Vocabulário do produto (sala, sala fixa, Suas salas, Transmitir, Assistir). Não apagar o registro histórico.
  - Conferir: os dois arquivos citam `app/`, Slint e `libwebrtc`.
- [ ] T102 [P] Montar o app nativo no `release.yml` (sonnet)
  - Arquivos: `.github/workflows/release.yml`.
  - Origem: o próprio arquivo (hoje usa `tauri-apps/tauri-action@v0` e copia `src-tauri/target/release/bundle/nsis/Espia_*_x64-setup.exe` para `Espia-setup.exe`).
  - Regras: ou montar o app nativo no lugar do Tauri (usando `scripts/release-app.mjs`, sem chave privada no CI: a assinatura continua feita pela pessoa), ou marcar o fluxo como desativado com um comentário que explica. Nenhum segredo novo no CI.
  - Conferir: o YAML continua válido (ler inteiro).
- [ ] T103 [P] Emenda PATCH da constituição: registrar `app/` e as escolhas (principal)
  - Arquivos: `.specify/memory/constitution.md`.
  - Regras: registrar a pasta `app/`, a Slint 1.17 e a `libwebrtc` (as escolhas que a constituição mandava fazer no plano), subir o patch da versão (de 4.0.0) e tirar o Sync Impact Report temporário. Texto que a constituição já fixa não muda. O arquivo hoje está modificado na árvore de trabalho; confirmar com a pessoa antes de sobrescrever mudanças que não sejam desta feature.
  - Conferir: a versão e a data estão no cabeçalho; não sobra o Sync Impact Report temporário.
- [ ] T104 [P] Marcar a spec 015 como substituída (haiku)
  - Arquivos: `specs/015-atualizar-app/spec.md`.
  - Regras: acrescentar no topo uma linha "Substituída pela 019 (App nativo em Rust): ver `specs/019-app-rust-convite/contracts/atualizacao.md`. Esta spec fica como referência histórica." Não mudar o resto.
  - Conferir: a linha está no topo e o arquivo continua legível.
- [ ] T105 Revisão de segurança (principal)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "Segurança"); correções em `app/`.
  - Regras: conferir que nenhum segredo está no executável (procurar por chaves, tokens, caminhos pessoais em `app/` e no `Espia.exe`); que a única chave embutida é a pública (T092); que os acessos do app ao computador ficam dentro da lista da constituição (tela, janelas, câmera, entradas de som, rede, `%APPDATA%\dev.espia.app\`, leitura do perfil do app de antes só na primeira abertura); que a chave de dono nunca vai ao servidor nem a registro; que `src-tauri/` não mudou; que o Worker e `src/shared/protocol.ts` não mudaram (`git diff --stat` sobre esses caminhos).
  - Conferir: a seção "Segurança" lista cada item como "ok" ou o que foi corrigido.
- [ ] T106 Apagar `app/spikes/` (haiku)
  - Arquivos: apagar `app/spikes/`; ajustar `app/Cargo.toml` e `app/README.md` se citarem as sondas.
  - Regras: só depois de G1, G2 e G3 registrados no `relatorio.md` e do T105. Não apagar `relatorio.md`.
  - Conferir: `cargo check --workspace` em `app/` passa sem `app/spikes/`.
- [ ] T107 Rodar o `quickstart.md` inteiro (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "Quickstart").
  - O que fazer: percorrer todas as histórias do `quickstart.md` na ordem, no app nativo e no site.
  - O que registrar: um "passou/falhou" por roteiro e a data. Falha reabre a tarefa da história correspondente.
- [ ] T108 Uma semana de uso do grupo (pessoa)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "Uma semana").
  - O que fazer: o grupo usa o app nativo por uma semana, com o app de antes ainda para baixar.
  - O que registrar: quantas pessoas usaram, quem precisou voltar ao app de antes ou ao site e o motivo (travamento, imagem pior, som faltando). Prova SC-012 (ninguém precisa voltar).
- [ ] T109 Troca: `Espia-setup.exe` passa a ser o nativo e publicar (pessoa)
  - Arquivos: `public/download/Espia-setup.exe`, `public/download/Espia-setup-0.1.10.exe`, `public/download/latest.json`, `specs/019-app-rust-convite/relatorio.md`.
  - O que fazer: só depois de T058, T068 e T079 passarem e de T108 aceito: a pessoa roda `npm run app:release` (copia o nativo para `Espia-setup.exe`, mantém `Espia-setup-0.1.10.exe`) e `npm run deploy` (nenhum agente roda).
  - O que registrar: data da troca, versão publicada, os dois links de download funcionando. Atualizar `relatorio.md` com o estado final de cada SC.
- [ ] T110 Depois da troca aceita: registrar a remoção de `src-tauri/` e da ponte (principal)
  - Arquivos: `specs/019-app-rust-convite/relatorio.md` (seção "Depois da troca").
  - Regras: só registrar, nesta feature, a lista do que sai numa feature futura: `src-tauri/`, `src/lib/native-capture.ts`, `src/lib/desktop.ts` e os contratos da ponte (`specs/005-seletor-nativo-desktop/contracts/native-capture.md`). Não apagar nada agora: `src-tauri/` fica congelado e intacto até a troca ser aceita e a feature futura existir.
  - Conferir: a seção existe, lista os quatro itens e diz que a remoção é de outra feature.

**Checkpoint**: tudo registrado; o app nativo substitui o de antes, ou a troca está bloqueada por um item citado.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sem dependências. T001 antes de T002, T003 e T006 que dependem do workspace; T004, T005, T007 e T008 podem ir já.
- **Foundational (Phase 2)**: depende da Phase 1. Bloqueia as histórias 3 a 7 (parte de app). Gate vermelho (T010, T012, T014) para o que vem depois na mesma trilha.
- **US1 (Phase 3)**: não depende de nada. É o MVP e pode ser publicada sozinha, no mesmo dia.
- **US2 site (T080–T083)**: não depende de nada. Pode sair junto com a US1.
- **US3 (Phase 4) e US4 (Phase 5)**: dependem da Phase 2 (até T037) e podem andar em paralelo depois de T037.
- **US5 (Phase 6)**: depende de US3 e US4.
- **US6, US7 e a parte de app da US2 (T084–T086)**: dependem de US5 (e de T085 depender de T084, T019, T024).
- **Polish (Phase 10)**: depois das histórias desejadas. T109 só depois de T058, T068, T079 e T108.

### Troca do app

O app nativo só substitui o de antes para o grupo quando US3, US4 e US5 passam: **T058, T068 e T079**. Até lá, `public/download/Espia-setup.exe` continua sendo o instalador de 2026-11-05 (R13).

### User Story Dependencies

- **US1**: independente de tudo.
- **US2 (site)**: independente. **US2 (app)**: precisa do núcleo (T019, T024) e do G3 (T014).
- **US3**: precisa da Phase 2 inteira (T037, T038).
- **US4**: precisa da Phase 2 inteira (T037, T038). Pode andar junto com a US3.
- **US5**: precisa de US3 e US4.
- **US6**: precisa de US3 e US4 (medir o app completo).
- **US7**: precisa de T092 (chaves) e do núcleo de arquivos (T024).

### Within Each Phase

- Os testes vêm antes da implementação de cada parte e devem falhar antes dela.
- Primeiro o mock, depois a lógica real (constituição): T028–T033 antes de T035 e T036.
- Phase 2: G1 (T009→T010), G2 (T011→T012), G3 (T013→T014) são três trilhas paralelas. Núcleo: T015–T019 em paralelo; T020 depois de T015, T017 e T018; T021 depois de T020; T022 e T023 depois de T015; T034 depois de T021 a T033; T035 e T036 depois de T034; T037 depois de T021 e T023; T038 depois de T035 a T037.
- US3: T045→T046; T047 em paralelo com T048–T052; depois T053–T057; T058 por último.
- US4: T059, T062–T064 (palco); T060 (som); T061, T065, T067 (estado); T066 (diagnóstico); T068 por último.
- US5: T069, T070, T071, T072 em paralelo; depois T073–T075; T076–T079 por último.
- Gates vermelhos param tudo o que vem depois na mesma trilha.
- Mesmo arquivo, mesma sequência: T047 e T062 tocam `go_live.rs` e `stage.rs` (arquivos diferentes), mas T031 e T032 acrescentam em `mocks/room.rs`, então vão em sequência. T048–T052 tocam `broadcast.rs` em T051, T052 e T057, então vão em sequência entre si.

### Parallel Opportunities

- Phase 1: T002 a T006 em paralelo depois de T001.
- Phase 2: T009, T011, T013 em paralelo; T015 a T019 em paralelo; T022, T023, T024 em paralelo; T028, T029, T030 em paralelo.
- Phase 3 e Phase 7 (site) em paralelo com tudo o resto.
- Phase 4 e Phase 5 em paralelo depois de T037.
- Phase 6: T069 a T072 em paralelo.
- Phase 8 e 9: T089, T093, T094 em paralelo.
- Phase 10: T101 a T104 em paralelo.

---

## Parallel Example: Phase 2 (núcleo puro)

```text
Task: "T015 protocol.rs e media_policy.rs com fixtures em app/crates/espia-core e app/fixtures/protocol"
Task: "T016 test/shared/protocol-fixtures.test.ts"
Task: "T017 room_id.rs, room_name.rs, route.rs em app/crates/espia-core"
Task: "T018 view_model.rs em app/crates/espia-core"
Task: "T019 device.rs em app/crates/espia-core"
```

## Parallel Example: User Story 1

```text
Task: "T039 test/lib/site-origin.test.ts"
Task: "T042 trocar o endereço antigo nos documentos (lista e edição)"
```

## Parallel Example: User Story 5

```text
Task: "T069 camera.rs e audio_input.rs em app/crates/espia-capture"
Task: "T070 tray.rs em app/crates/espia-shell"
Task: "T071 single_instance.rs e links.rs em app/crates/espia-shell"
Task: "T072 window_state.rs e open_external.rs em app/crates/espia-shell"
```

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Completar a Phase 3 (US1). Ela não depende da Phase 2.
2. Parar e validar: T043 no mock e no site local.
3. Publicar (T044, pela pessoa). A parte de site da US2 (T080–T083) pode sair junto.

### Incremental Delivery

1. US1 (+ site da US2): convite no endereço oficial e ida do endereço antigo. Publicável no mesmo dia.
2. Phase 1 e Phase 2: base do app nativo, com os gates G0 a G3 verdes antes de construir o resto.
3. US3 e US4 em paralelo: transmitir e assistir.
4. US5: paridade, medida item a item.
5. US2 (app), US6 e US7: importação, leveza e atualização.
6. Polish: documentos, segurança, uma semana de uso, troca.

O app nativo só substitui o de antes depois que **T058, T068 e T079** passam. Gate vermelho para a trilha: parar, registrar em `relatorio.md` e replanejar com o plano B da decisão.

---

## Divisão por subagentes

**Regra**: o modelo principal decide, escreve a instrução fechada de cada lote e revisa. Um lote é uma execução de um subagente, com arquivos que nenhum outro lote em execução toca. Cada lote termina com o gate: `npm run check` e `npm run build` no site; `cargo check` e `cargo test` em `app/` no app; e o mock da tela abrindo, quando for tarefa de tela. O subagente não roda `git`, `npm run deploy` nem escreve segredo, e não toca em `src-tauri/`.

**Executores**:

- **haiku**: vasculhar, copiar, renomear, conferir listas e textos.
- **sonnet**: escrever código e documentos, com instrução fechada.
- **principal**: desenho do redutor da sala, do laço principal e da ligação entre pessoas; decisão dos gates; revisão de cada lote; segurança.
- **pessoa**: medições, testes com captura real e com o grupo, chave de assinatura, publicação.

### Phase 1: Setup

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| A | T001 | sonnet | `app/Cargo.toml`, `app/crates/*`, `app/README.md`, `.gitignore` | nenhum (vem primeiro) |
| B1 | T002, T005, T006 | sonnet | `.github/workflows/ci.yml`, `docs/adr/0002-app-nativo-em-rust.md`, `docs/adr/0001-conexao-direta-e-franquia-de-midia.md`, `scripts/memoria.ps1` | B2, T007, T008 (T002 depois de A) |
| B2 | T003, T004 | haiku | `package.json`, `app/fixtures/protocol/README.md` | B1, T007, T008 (T003 depois de A) |
| P1 | T007 | principal | `specs/019-app-rust-convite/research.md` | B1, B2, T008 |
| P2 | T008 | pessoa | `specs/019-app-rust-convite/relatorio.md` | A, B1, B2, P1 |

### Phase 2: Foundational

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| G | T009, T011, T013 | sonnet (três execuções, uma por prova) | `app/spikes/webrtc-probe/`, `app/spikes/ui-probe/`, `app/spikes/webview-data/` | C1, N, S |
| G-gates | T010, T012, T014 | pessoa e principal | `relatorio.md`, `research.md` | cada gate depois da sua prova |
| C1 | T015, T016, T017, T018, T019 | sonnet (cinco execuções em paralelo) | `app/crates/espia-core/src/{protocol,media_policy,room_id,room_name,route,view_model,device}.rs`, `app/fixtures/protocol/*.json`, `test/shared/protocol-fixtures.test.ts` | G, N, S, U1 |
| C2 | T020 | sonnet | `app/crates/espia-core/src/{quality,cpu_governor,watch_state,live_label,audio_source,devices,join_view}.rs` | G, N, S, U1 (depois de C1) |
| C3 | T021 | principal desenha, sonnet escreve | `app/crates/espia-core/src/room/` | G, N, S, U1 (depois de C2) |
| N | T022, T023 | sonnet (duas execuções) | `app/crates/espia-net/src/{api,signaling}.rs` | G, C1, S, U1 (depois de T015) |
| S | T024 | sonnet | `app/crates/espia-shell/src/storage.rs` | G, C1, N, U1 |
| U1 | T025, T026, T027 | sonnet (em sequência) | `app/crates/espia-ui/build.rs`, `ui/tokens.slint`, `ui/components/*`, `src/mocks/{mod,registry}.rs`, `app/crates/espia/src/cli.rs` | G, C1, N, S |
| U2 | T028, T029, T030 | sonnet (três execuções em paralelo) | `espia-ui/ui/pages/{home,join,notfound,offline}.slint`, `ui/components/{saved-rooms-panel,saved-room,delete-room-dialog,name-dialog}.slint`, `src/mocks/{home,join,notfound}.rs` | G, C*, N, S (depois de U1) |
| U3 | T031 (dois lotes: estrutura, depois cenas) | sonnet | `espia-ui/ui/pages/room.slint`, `ui/components/{stage,stream-tile,...}.slint`, `src/mocks/room.rs` | G, C*, N, S, U2 (U3 estrutura e U2 não compartilham arquivo) |
| U4 | T032 | sonnet | `espia-ui/ui/components/{go-live-modal,source-card}.slint`, `src/mocks/room.rs` | G, C*, N, S (depois de U3, mesmo `room.rs`) |
| U5 | T033 | sonnet | `app/crates/espia-ui/tests/scene_parity.rs` | G, C*, N, S (depois de U3 e U4) |
| E | T034, T035, T036, T037, T038 | principal desenha T034 e T037, sonnet escreve; T038 pessoa | `app/crates/espia/src/*`, `app/crates/espia-media/src/{peer,mesh,view_channel}.rs` | nenhum, em sequência (depois de C3, N, S, U1 a U5) |

### Phase 3: US1

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| S1 | T039, T040, T041 | sonnet | `test/lib/site-origin.test.ts`, `src/lib/site-origin.ts`, `src/lib/stores/room.svelte.ts`, `src/mocks/room.ts` | S2, qualquer fase de app |
| S2 | T042 | haiku lista, sonnet edita | `specs/003-espia-desktop/contracts/deep-link.md`, `specs/005-seletor-nativo-desktop/contracts/native-capture.md`, `specs/007-fechar-para-bandeja/contracts/tray.md`, `specs/011-salas-fixas/contracts/local-storage.md`, `specs/015-atualizar-app/contracts/update.md`, `DEFINICOES.md` | S1 |
| P | T043, T044 | pessoa | `relatorio.md` | depois de S1 e S2 |

### Phase 4: US3

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| K | T045, T046 | haiku copia, sonnet adapta | `app/crates/espia-capture/src/*` | fase 5 (V, A, W, D) |
| L | T047 | sonnet | `app/crates/espia/src/go_live.rs`, `app/crates/espia-core/src/sources.rs` | M (depois de K) |
| M | T048, T049, T050, T051, T052 | sonnet (em sequência) | `app/crates/espia-media/src/{send_video,send_audio,send_policy,quality_apply}.rs`, `app/crates/espia-core/src/room/broadcast.rs`, `app/crates/espia/src/broadcast.rs` | L (depois de K) |
| R | T053, T054, T055, T056, T057 | sonnet (em sequência) | `app/crates/espia-media/src/{stats,encoder_fallback}.rs`, `app/crates/espia-capture/src/program_name.rs`, `app/crates/espia/src/self_preview.rs`, `app/crates/espia-core/src/{snapshot,notices}.rs` | fase 5 (depois de L e M) |
| P | T058 | pessoa | `relatorio.md` | depois de R |

### Phase 5: US4

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| V | T059, T062, T063, T064 | sonnet (em sequência) | `app/crates/espia-media/src/recv_video.rs`, `espia-ui/src/video_surface.rs`, `ui/components/{stage,stream-tile,fullscreen,pip}.slint`, `app/crates/espia/src/{stage,fullscreen,pip}.rs` | A, W, D, fase 4 |
| A | T060 | sonnet | `app/crates/espia-media/src/{recv_audio,mixer}.rs` | V, W, D |
| W | T061, T065, T067 | sonnet (em sequência) | `app/crates/espia/src/{watch,reconnect,visibility}.rs`, `app/crates/espia-core/src/room/reconnect.rs` | V, A, D |
| D | T066 | sonnet | `app/crates/espia/src/diagnostics.rs`, `espia-ui/ui/components/media-diagnostics.slint` | V, A, W |
| P | T068 | pessoa | `relatorio.md` | depois de V, A, W, D |

### Phase 6: US5

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| Q1 | T069 | sonnet | `app/crates/espia-capture/src/{camera,audio_input}.rs` | Q2, Q3, Q4 |
| Q2 | T070 | sonnet | `app/crates/espia-shell/src/tray.rs` | Q1, Q3, Q4 |
| Q3 | T071 | sonnet | `app/crates/espia-shell/src/{single_instance,links}.rs` | Q1, Q2, Q4 |
| Q4 | T072 | sonnet | `app/crates/espia-shell/src/{window_state,open_external}.rs` | Q1, Q2, Q3 |
| Q5 | T073, T074 | sonnet (em sequência) | `app/crates/espia/src/{owner,remove_person}.rs`, `app/crates/espia-core/src/room/owner.rs` | depois de Q1 a Q4 |
| Q6 | T075 | haiku confere textos, sonnet corrige | `app/crates/espia-core/src/notices.rs`, `app/crates/espia-ui/ui/**` | depois de Q5 |
| Q7 | T077 | haiku | `specs/019-app-rust-convite/checklists/paridade.md` | Q5, Q6 |
| P | T076, T078, T079 | pessoa aponta, sonnet corrige (T076); pessoa (T078, T079) | `relatorio.md`, `checklists/paridade.md`, `app/crates/espia-ui/` (correções) | depois de Q6 e Q7 |

### Phase 7: US2

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| Site | T080, T081, T082 | sonnet (em sequência) | `test/lib/origin-move.test.ts`, `src/lib/origin-move.ts`, `src/main.ts` | App (T084, T085), fases de app |
| App | T084, T085 | sonnet (em sequência) | `app/crates/espia-shell/src/import_webview.rs`, `app/crates/espia/src/first_run.rs` | Site (depois de T014, T019, T024) |
| P | T083, T086 | pessoa | `relatorio.md` | T083 depois de Site; T086 depois de App |

### Phase 8: US6

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| X1 | T089 | sonnet | `app/crates/espia/build.rs`, `app/crates/espia/assets/icon.ico`, `app/crates/espia/Cargo.toml` | fase 9 (T093, T094) |
| X2 | T087, T088 | sonnet (em sequência) | `app/crates/espia/src/{lifecycle,startup}.rs`, `app/crates/espia-media/src/lib.rs`, `app/crates/espia-ui/src/lib.rs` | X1 |
| P | T090 | pessoa | `relatorio.md` | depois de X1 e X2 |
| M | T091 | principal | `relatorio.md`, `app/` (só se o T090 passar das metas) | depois de P |

### Phase 9: US7

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| K0 | T092 | pessoa | `app/crates/espia-shell/src/update_key.pub`, `%USERPROFILE%\.espia\update.key` | T093, T094 |
| Y1 | T093 | sonnet | `app/packaging/` | Y2, T089 |
| Y2 | T094 | sonnet | `scripts/release-app.mjs`, `package.json` | Y1, T089 |
| Y3 | T095, T096, T097, T098, T099 | sonnet (em sequência) | `app/crates/espia-shell/src/update.rs`, `espia-ui/ui/components/update-notice.slint`, `src/mocks/{home,room}.rs`, `app/crates/espia/src/{update_ui,update_required,update_guard}.rs`, `public/download/` | depois de K0, Y1, Y2 |
| P | T100 | pessoa | `relatorio.md` | depois de Y3 |

### Phase 10: Polish

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| Z1 | T101, T102 | sonnet (duas execuções) | `DEFINICOES.md`, `README.md`, `.github/workflows/release.yml` | Z2, Z3 |
| Z2 | T103 | principal | `.specify/memory/constitution.md` | Z1, Z3 |
| Z3 | T104 | haiku | `specs/015-atualizar-app/spec.md` | Z1, Z2 |
| Z4 | T105, T106 | principal (T105), haiku (T106) | `relatorio.md`, `app/spikes/` | depois de Z1 a Z3 |
| P | T107, T108, T109 | pessoa | `relatorio.md`, `public/download/*` | em sequência, depois de Z4 |
| F | T110 | principal | `relatorio.md` | depois de T109 |

### Instrução fechada que o principal dá a um lote

```text
Objetivo: <uma frase do que o lote entrega>.
Tarefas: <IDs, com o texto completo de cada uma copiado deste tasks.md>.
Arquivos que pode tocar: <lista exata; nenhum outro arquivo, e nunca src-tauri/, src/worker/ nem src/shared/protocol.ts>.
Arquivos de origem (só leitura): <caminhos e funções ou linhas citados nas tarefas>.
Regras: português do Brasil; limites exatos das tarefas; sem git, sem deploy, sem instalar, sem segredo; testes antes da implementação.
Gate: <npm run check e npm run build | cargo check e cargo test em app/> e <mock abrindo: npm run app:mock -- <tela> --scene <cena>>.
O que devolver: lista de arquivos criados e mudados, resultado de cada comando do gate (saída resumida), pontos em que a tarefa ou o código eram ambíguos e como foram resolvidos, e o que ficou para a pessoa fazer.
```

---

## Notes

- Nenhuma tarefa mexe no Worker, em `src/shared/protocol.ts` nem em `src-tauri/` antes do T110 (e o T110 só registra).
- Abrir, fechar ou aproximar nunca altera `watching`, `stopped`, `focusedId` nem `viewModes()` no site (FR-005 da 018). No app, a regra do Assistir (FR-021) vale nos dois sentidos.
- Textos novos em português informal, com "Transmitir" e "Assistir". Nunca "Go Live" nem "compartilhar" em texto de interface.
- O app nativo fala o protocolo que já existe: `Origin: https://espia.diuliano.com`, `mediaProtocol: 2`, `PING` exato.
- A spec 015 (atualização) é referência histórica; o contrato novo é `contracts/atualizacao.md`.
- Fazer commit depois de cada tarefa ou grupo lógico, só quando pedido.

---

## Resumo

Contagem feita sobre as linhas de tarefa deste arquivo.

- **Total**: 110 tarefas (T001 a T110).
- **Por fase**: Phase 1 Setup 8; Phase 2 Foundational 30; Phase 3 US1 6; Phase 4 US3 14; Phase 5 US4 10; Phase 6 US5 11; Phase 7 US2 7; Phase 8 US6 5; Phase 9 US7 9; Phase 10 Polish 10.
- **Por história**: US1 6; US2 7; US3 14; US4 10; US5 11; US6 5; US7 9 (total 62; as outras 48 são Setup, Foundational e Polish).
- **Tarefas [P]**: 33.
- **Por executor, como escrito na linha da tarefa**: `sonnet` 73; `pessoa` 16; `principal` 5; `haiku` 5; `principal desenha, sonnet escreve` 3; `pessoa e principal` 3; `sonnet, com revisão do principal` 1; `pessoa aponta, sonnet corrige` 1; `haiku lista, sonnet edita` 1; `haiku copia, sonnet adapta` 1; `haiku confere textos, sonnet corrige` 1.
- **Por executor principal (primeira palavra)**: sonnet 74; pessoa 20; haiku 8; principal 8.
