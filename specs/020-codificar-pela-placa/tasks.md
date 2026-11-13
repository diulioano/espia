---

description: "Task list for 020 — Transmitir sem pesar no jogo, codificando uma vez pela placa de vídeo"
---

# Tasks: Transmitir sem pesar no jogo, codificando uma vez pela placa de vídeo

**Input**: Design documents from `/specs/020-codificar-pela-placa/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/codificador.md](./contracts/codificador.md), [contracts/qualidades.md](./contracts/qualidades.md), [contracts/caminho.md](./contracts/caminho.md), [contracts/medicao.md](./contracts/medicao.md), [quickstart.md](./quickstart.md), [investigacao.md](./investigacao.md)

**Tests**: o plano pede testes das regras puras (`cargo test` em `espia-core`, escritos a partir das tabelas de `contracts/qualidades.md` e `contracts/caminho.md`) e do que não precisa de placa em `espia-media` (montagem do quadro e estado do publicador, com codificador falso). Os testes vêm antes da implementação de cada parte e devem falhar antes dela. O que depende de placa, rede, jogo ou do grupo é sonda e roteiro manual do `quickstart.md`, feito pela pessoa.

**Ordem da constituição (v4.0.0)**: na única tela tocada (Diagnóstico de mídia), primeiro a cena de dados falsos (T018), depois o dado real (T023).

**Gate de cada tarefa**:

- **App** (`app/`): `npm run app:check` e `npm run app:test` sem erros. Em tarefa de tela, a cena abrindo no mock (`npm run app:mock -- room --scene <cena>`).
- **Sonda** (`app/spikes/encode-probe/`): `cargo build --release -j 6` na pasta da sonda, sem erro.
- **Script**: o comando de conferência escrito na tarefa.
- **Tarefa da pessoa**: o registro em `specs/020-codificar-pela-placa/relatorio.md` é o gate.

**Regras de segurança (valem para todas as tarefas)**:

- Teste automático nunca captura a tela de verdade nem usa `--use-fake-ui-for-media-stream`. A sonda gera o vídeo (barras e contador). Captura real só no roteiro manual da pessoa.
- Teste automático nunca exige placa de vídeo: o CI roda em máquina sem placa. Teste que precisa de placa é `#[ignore]` e roda à mão.
- `npm run deploy`, `git push` e escrever segredo são feitos pela pessoa. Nenhum agente roda esses comandos.
- `src-tauri/` está congelado (regra da 019): ler é permitido, mudar não.
- O Worker (`src/worker/`), `src/shared/protocol.ts` e o site (`src/`) não mudam nesta feature.
- Nenhuma opção, pergunta, aviso ou termo técnico sobre placa de vídeo ou compressão aparece para a pessoa (FR-014). A única exceção é a linha do Diagnóstico de mídia.
- Parar de enviar é sempre `active=false`. Nunca tirar a faixa de um canal ativo (`set_track(None)` derrubou a sonda do G1).
- Compilar com `-j 6` e nunca dois builds ao mesmo tempo (o computador já reiniciou com builds pesados em paralelo).

**Gate vermelho para o que vem depois**: se G4 (T014), G5 (T015) ou G6 (T024) falhar, parar, registrar em `relatorio.md` e replanejar com o plano B da decisão (`plan.md`, "Gates"). Nenhuma tarefa das fases 4 a 8 começa antes do G5 verde.

## Format: `- [ ] T001 [P] [US1] Descrição com caminho de arquivo (executor)`

- **[P]**: pode rodar em paralelo, porque mexe em arquivo diferente e não depende de tarefa pendente.
- **[Story]**: só nas fases de história (Fases 3 a 8). US1 saber se funciona antes de construir, US2 transmitir sem derrubar o jogo, US3 mais gente assistindo não pesa mais, US4 quem assiste não percebe diferença, US5 internet fraca de um não estraga a dos outros, US6 computador sem placa boa continua transmitindo.
- **(executor)**: último item da linha. `haiku` vasculha, copia e confere lista. `sonnet` escreve código ou documento com instrução fechada. `principal` decide, desenha, revisa gate. `pessoa` mede, testa com jogo, com captura real ou com o grupo.
- Cada tarefa traz, em sub-itens, `Arquivos`, `Origem`, `Regras` e `Conferir`. Tarefa da pessoa traz `O que fazer`, `O que registrar` e o critério de passe.
- Caminhos relativos à raiz do repositório. Identificadores em Rust seguem o resto da crate (nomes em inglês, comentários em português).

## Path Conventions

- App nativo: `app/` (workspace Rust da 019). Regras puras em `app/crates/espia-core`, compressão e publicação em `app/crates/espia-media`, captura em `app/crates/espia-capture`, tela em `app/crates/espia-ui`, binário em `app/crates/espia`.
- Sonda: `app/spikes/encode-probe/` (crate própria, fora do workspace, apagada no fim).
- Medição: `scripts/processador.ps1`.
- Biblioteca de ligação: `~/.cargo/registry/src/*/libwebrtc-0.4.0` e `webrtc-sys-0.3.53` (só leitura).

## O que esta feature espera da 019

Tarefas de [019/tasks.md](../019-app-rust-convite/tasks.md) que precisam estar feitas antes de tarefas daqui. Nenhuma delas é refeita aqui.

| Tarefa da 019 | O que entrega | Quem daqui espera |
|---|---|---|
| T037 | ligação entre pessoas em `espia-media` (`peer.rs`, `mesh.rs`), com a `libwebrtc` atrás de um trait | T022, T027, T030, T032, T033 |
| T045, T046 | captura em `espia-capture` e saída por canal com relógio comum | T021, T032 |
| T047 a T052 (ajustadas pelo T017 daqui) | seletor, envio, `live`/`unlive` e qualidade ligados ao app | T024 (G6 com o app nativo) |
| T066 | `app/crates/espia/src/diagnostics.rs` com as linhas do motor | T023 (se ainda não existir, o T023 cria só a parte dele) |

As fases 1, 2 e 3 daqui não dependem de nada da 019 além do que já está feito (workspace, `quality.rs`, `cpu_governor.rs`, sonda do G1).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: o medidor de processador, o relatório e a leitura do app de hoje. Nada aqui depende de placa.

- [X] T001 [P] Criar `scripts/processador.ps1`, que mede o processador em núcleos (sonnet)
  - Arquivos: criar `scripts/processador.ps1`.
  - Origem: `scripts/memoria.ps1` (função `Get-Tree`, que monta a árvore do `Espia.exe`; parâmetros `-Processo` e `-Nativo`; mensagem e `exit 1` quando o processo não existe); [contracts/medicao.md](./contracts/medicao.md); `app/spikes/webrtc-probe/src/metrics.rs` (conta por tempo de processador).
  - Regras: PowerShell 5.1, só lê, não muda nada no computador. Parâmetros: sem parâmetro mede a **árvore** do `Espia.exe` (o app e os processos do motor de página, como `memoria.ps1`); `-Nativo` mede só o processo, sem filhos; `-Processo <nome>` (padrão `Espia`, sem `.exe`); `-Duracao <s>` (padrão 60); `-Intervalo <s>` (padrão 1). Conta: a cada intervalo, somar `KernelModeTime + UserModeTime` (`Win32_Process`, unidades de 100 ns) dos processos medidos; núcleos da leitura = variação do tempo de processador ÷ variação do relógio. Só entra na conta o processo presente nas duas leituras seguidas (processo que nasce ou morre no meio não gera número negativo nem salto). Núcleos lógicos lidos do computador (`[Environment]::ProcessorCount`). Porcentagem da máquina = núcleos ÷ núcleos lógicos × 100. Saída: uma linha de resumo no fim, com núcleos médios, pico, porcentagem da máquina, núcleos lógicos e amostras, na forma `Espia (arvore): 0,53 nucleo medio, pico 0,81 | 2,2% da maquina | 24 nucleos logicos | 60 amostras` (rótulo `(nativo)` com `-Nativo`). Mensagens sem acento, como em `memoria.ps1`, porque o PowerShell 5.1 lê arquivo sem BOM como ANSI. Sem processo com o nome pedido: avisar `<nome> nao esta rodando` e sair com código 1. Não criar atalho em `package.json` (não decidido).
  - Conferir: sem o app aberto, `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/processador.ps1` avisa e sai com 1; `... -File scripts/processador.ps1 -Processo powershell -Nativo -Duracao 5` imprime a linha de resumo com cerca de 5 amostras.
- [X] T002 [P] Criar o esqueleto de `relatorio.md` (haiku)
  - Arquivos: criar `specs/020-codificar-pela-placa/relatorio.md`.
  - Origem: [contracts/medicao.md](./contracts/medicao.md) ("O que TODA medição registra"); [plan.md](./plan.md) ("Gates"); [quickstart.md](./quickstart.md).
  - Regras: seções, nesta ordem: "Leitura do app de hoje", "G4, uma pessoa" (itens (a) a (f), um por linha, resultado vazio), "G5, três pessoas" (itens (a) a (g)), "G6, com jogo", "História 2", "História 3", "História 4", "História 5", "História 6", "Conferência com o grupo", "Decisões". No topo, o modelo de bloco de medição com os 12 campos do contrato: data (AAAA-MM-DD), máquina (processador, núcleos lógicos, placa de vídeo), jogo, qualidade efetiva, pessoas assistindo, como isso foi conferido, FPS do jogo sem transmitir, FPS do jogo transmitindo, aplicativo medido, núcleos (média de 60 s e pico), atraso entre entregar o quadro e receber o comprimido (só G4), passou ou falhou. Nenhum número inventado: campos vazios ficam vazios.
  - Conferir: o arquivo tem as 11 seções e o modelo com os 12 campos.
- [ ] T003 Ler qual compressão o app de hoje usa e conferir o script contra o Gerenciador de Tarefas (pessoa)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "Leitura do app de hoje").
  - O que fazer: seguir a seção 2 do [quickstart.md](./quickstart.md): app 0.1.10 transmitindo 1080p60, pelo menos uma pessoa assistindo, Ctrl+Shift+D, ler a linha **Encoder**. Na mesma transmissão, rodar `scripts/processador.ps1` (T001) e comparar a porcentagem da máquina com a do Gerenciador de Tarefas (soma de `Espia` e do motor de página).
  - O que registrar: o texto exato do Encoder (ou "—"), a qualidade, quantas pessoas assistiam e como foi conferido; núcleos médios e pico do script e a leitura do Gerenciador; se o atalho abriu dentro do app 0.1.10 (item "a confirmar" do quickstart).
  - Critério de passe: só informa, não bloqueia. Se o script e o Gerenciador divergirem muito, reabrir o T001 antes do G4.

**Checkpoint**: dá para medir processador em núcleos e há onde anotar.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: as regras puras em `app/crates/espia-core/src/tiers.rs`: quem recebe o quê, quadro inteiro a pedido, internet de cada espectador e caminho de compressão. Sem rede, sem placa, sem relógio (cada passo de tempo é uma leitura por segundo, como em `cpu_governor.rs`).

**⚠️ CRITICAL**: bloqueia as histórias 2 a 6. Não bloqueia a história 1: a sonda não usa `tiers.rs`, e o plano (Etapa 1) deixa esta fase andar em paralelo com ela. Nada desta fase é ligado ao app antes do G5 verde.

As cinco tarefas mexem no mesmo arquivo e vão em sequência. O contrato `contracts/qualidades.md` foi alinhado à R18 em 2026-11-12.

- [X] T004 Desenho e tipos de `tiers.rs`; quem recebe o quê, qualidades ativas e ajuste da reduzida (principal desenha, sonnet escreve)
  - Arquivos: criar `app/crates/espia-core/src/tiers.rs`; `app/crates/espia-core/src/lib.rs` (declarar `pub mod tiers;`).
  - Origem: [contracts/qualidades.md](./contracts/qualidades.md) seções 1, 2 e 3; [data-model.md](./data-model.md) ("Qualidade", "Espectador de uma transmissão"); R6 e R18 de [research.md](./research.md); `app/crates/espia-core/src/quality.rs` (`ViewMode`, `Quality`, `effective_quality`, `count_full_viewers`, `wants_video`, `wants_sound`, `send_params`).
  - Regras: o principal escreve antes, num comentário no topo do arquivo, os tipos e quem chama cada função (qualidade: cheia ou reduzida; o que um espectador recebe: cheia, reduzida ou nada; espectador: modo de visão opcional mais "banda fraca"; ajuste da reduzida: miniatura ou internet fraca). Tabela "quem recebe": `Full` com banda boa = cheia; `Full` com banda fraca = reduzida; `Thumb` = reduzida (a banda não muda nada); `Hidden`, `Off` e sem modo = nada. Qualidade ativa: a cheia se pelo menos um recebe a cheia; a reduzida se pelo menos um recebe a reduzida; sem ninguém, nenhuma. Constantes novas: reduzida de miniatura = **360p, 15 quadros por segundo, 500 kbps**; reduzida de internet fraca = **720p, 30 quadros por segundo, 1,5 Mbps**. Ajuste: "internet fraca" se há pelo menos um `Full` fraco na reduzida, senão "miniatura"; nunca os dois (só existe uma reduzida, então a miniatura recebe os 720p30 enquanto durar). Quando a cheia já é 720p30 ou menor, a reduzida de internet fraca tem o mesmo tamanho e os mesmos quadros por segundo da cheia, com taxa de 1,5 Mbps ou metade da taxa da cheia, o que for menor (R18). A cheia é sempre `effective_quality(escolhida, espectadores em Full, degraus)`; `Full` fraco continua contando no teto por espectadores. Escrever os testes primeiro, uma asserção por linha das tabelas de casos das seções 1 (7 casos), 2 (6) e 3 (8). No caso "escolhida 720p30, 1 `Full` fraco" a reduzida fica em **1,25 Mbps** (metade de 2,5).
  - Conferir: `cargo test -p espia-core tiers` passa, com 21 casos; `cargo tree -p espia-core` não ganhou dependência.
- [X] T005 Ações por canal: parar, voltar e trocar de qualidade (sonnet)
  - Arquivos: `app/crates/espia-core/src/tiers.rs`.
  - Origem: [contracts/qualidades.md](./contracts/qualidades.md) seções 4 e 5; R7 de [research.md](./research.md).
  - Regras: função pura que recebe o estado anterior e o novo de cada espectador e devolve a lista de ações do publicador: ligar ou desligar o canal de vídeo (`active`), apontar o canal para a faixa da cheia ou da reduzida, pedir quadro inteiro de uma qualidade, ativar ou parar uma qualidade, ligar ou cortar o som. `Hidden` = `active=false`, faixa mantida, som continua. `Off` = `active=false`, faixa mantida, sem som (o som segue `wants_sound`). Voltar a `Full` = `active=true` mais pedido de quadro inteiro da cheia. Trocar de qualidade muda só o canal daquele espectador e pede quadro inteiro da qualidade nova; a qualidade de destino é ativada antes ou junto. Último espectador de uma qualidade saindo dela: a qualidade para. **Não existe ação "tirar faixa"**: o tipo das ações não tem essa variante. Testes primeiro: os 5 casos da seção 4 e os 4 da seção 5.
  - Conferir: `cargo test -p espia-core tiers` passa; um teste percorre todas as transições entre os seis estados de modo e confere que nenhuma ação remove faixa.
- [X] T006 Quadro inteiro a pedido, com junção de 300 ms (sonnet)
  - Arquivos: `app/crates/espia-core/src/tiers.rs`.
  - Origem: [contracts/qualidades.md](./contracts/qualidades.md) seção 6; [data-model.md](./data-model.md) ("Pedido de quadro inteiro"); R8 e R18.
  - Regras: estado por qualidade (a cheia e a reduzida têm contas separadas). Entrada: "há pedido" e o instante em milissegundos (passado por quem chama; sem relógio aqui). Saída: o próximo quadro sai inteiro ou não. Havendo pedido, o próximo quadro sai inteiro. No máximo **um quadro inteiro a cada 300 ms por qualidade**. O pedido que chega dentro dos 300 ms é **adiado** até a janela passar, **nunca descartado**. O primeiro quadro de uma qualidade que acabou de ficar ativa é inteiro. Não há quadro inteiro periódico. Constante nova: 300 ms. Testes primeiro, os 7 casos da seção 6 (pedidos em 0, 100 e 200 dão um quadro inteiro em 0 e outro em 300; pedidos em 0 e 299 dão o segundo em 300; 60 s sem pedido não dão nenhum além do primeiro; pedido na cheia não faz a reduzida emitir).
  - Conferir: `cargo test -p espia-core tiers` passa.
- [X] T007 Internet de cada espectador e taxa da cheia (sonnet)
  - Arquivos: `app/crates/espia-core/src/tiers.rs`.
  - Origem: [contracts/qualidades.md](./contracts/qualidades.md) seção 7 (inclusive "Casos de teste a mais"); [data-model.md](./data-model.md) ("Transições de um espectador em tela cheia"); R9 e R18.
  - Regras: um passo por segundo, com a banda de saída disponível de cada espectador e a taxa nominal da cheia (`max_bitrate` da qualidade efetiva). Constantes novas: descer com banda abaixo de **60%** da taxa nominal por **5 s** seguidos, ou abaixo de **90%** por **15 s** seguidos; voltar com banda em **90%** ou mais por **15 s** seguidos; tentativa de volta a cada **30 s** na reduzida. A base dos 60% e dos 90% é sempre a taxa **nominal**, nunca a já baixada. "Todos abaixo": se todos os que recebem a cheia estão abaixo de 90%, ninguém desce; um espectador sozinho nunca desce. Taxa da cheia: a **nominal**; só no caso "todos abaixo" ela é, a cada segundo, o menor valor entre a nominal e **90% da menor banda** entre quem recebe a cheia, e sobe de volta pela mesma conta. Assim um espectador mediano não baixa a taxa dos outros (FR-010). Tentativa de volta que não dá certo: vale de novo a regra de descer. `Thumb` não entra na regra de banda nem na conta de "todos abaixo". Modo que volta a `Full` recomeça a regra do zero. Testes primeiro: os 11 casos da tabela principal e os 4 "a mais". O caso "A na reduzida com 5,0 Mbps por 60 s" se lê como: até o segundo 29 continua na reduzida sem tentativa, e no segundo 30 há a tentativa. A confirmar na história 5: quem tem banda fixa entre 60% e 90% alterna entre a reduzida e a tentativa; se incomodar, dobrar o intervalo da tentativa a cada falha.
  - Conferir: `cargo test -p espia-core tiers` passa, com 15 casos desta regra.
- [X] T008 Caminho de compressão: máquina de estados e convivência com `effective_quality` (sonnet)
  - Arquivos: `app/crates/espia-core/src/tiers.rs`.
  - Origem: [contracts/caminho.md](./contracts/caminho.md) ("Máquina de estados", "Casos de teste"); [contracts/qualidades.md](./contracts/qualidades.md) seção 8; [data-model.md](./data-model.md) ("Caminho de compressão"); R10, R11 e R18; `app/crates/espia-core/src/cpu_governor.rs` (`CpuGovernor::tick`).
  - Regras: estados `escolhendo`, `placa`, `processador`. Eventos (fatos que `espia-media` informa): iniciou, não achou, não iniciou, erro, quadro saiu, 1 s sem quadro, sem qualidade ativa. `escolhendo` + iniciou = placa; + não achou ou não iniciou = processador. Em `placa`: **3 erros seguidos** = processador (um quadro que sai zera a contagem); **nenhum quadro comprimido em 2 s** = processador, contados só com alguma qualidade ativa (sem qualidade ativa, o tempo zera). `processador` é final nesta transmissão: nenhum evento volta à placa. Codificador reaberto por recusar a taxa ao vivo mais de uma vez em 60 s conta como um erro (R18). Qualidade: nos dois caminhos vale `effective_quality`; o caminho pelo processador não tem teto próprio (R18), e é o ajuste automático por processador que baixa a qualidade quando o computador não dá conta. "Limitado": quadros que o codificador não comprimiu a tempo contam como limitado para o `CpuGovernor` (10 s limitado baixa um degrau, 60 s de folga sobe um). Constantes novas: 3 erros, 2 s. Testes primeiro: os 14 casos da máquina de estados, os 4 da qualidade no caminho pelo processador e os 5 da seção 8 de `qualidades.md`.
  - Conferir: `cargo test -p espia-core tiers` passa; `npm run app:check` e `npm run app:test` verdes.

**Checkpoint**: regras puras com testes verdes. Nada ligado ao app ainda.

---

## Phase 3: User Story 1 - Saber se funciona antes de construir (Priority: P1) 🎯 MVP

**Goal**: uma sonda pequena prova, com números da máquina do dono, que o vídeo comprimido uma vez pela placa chega ao navegador de uma e de três pessoas e gasta pouco processador. Se falhar, o trabalho para ali.

**Independent Test**: rodar a sonda contra o site local (`npm run dev:worker`) com vídeo de teste, primeiro com uma pessoa no Chrome e depois com três. Conferir a imagem e anotar o processador de cada caso em `relatorio.md`.

- [X] T009 [P] [US1] Esqueleto da sonda `app/spikes/encode-probe/`, a partir da sonda do G1 (haiku copia, sonnet adapta)
  - Arquivos: criar `app/spikes/encode-probe/{Cargo.toml,.cargo/config.toml,src/main.rs,src/args.rs,src/media.rs,src/metrics.rs,src/proto.rs}`.
  - Origem: `app/spikes/webrtc-probe/` (os mesmos sete arquivos; só leitura).
  - Regras: o haiku copia os arquivos. O sonnet ajusta: pacote `encode-probe` (o executável precisa se chamar `encode-probe.exe`, o nome que `scripts/processador.ps1 -Processo encode-probe -Nativo` espera); `[workspace]` vazio no `Cargo.toml`; `libwebrtc = "=0.4.0"`; `.cargo/config.toml` com `+crt-static`; apelido padrão "Sonda 020"; acrescentar `windows = "0.62"` com as features `Win32_Foundation`, `Win32_Media_MediaFoundation`, `Win32_Graphics_Direct3D`, `Win32_Graphics_Direct3D11`, `Win32_Graphics_Dxgi`, `Win32_Graphics_Dxgi_Common` e `Win32_System_Com` (acrescentar outra só se o compilador pedir). O comportamento da sonda do G1 continua inteiro (`--backend software` é usado no G5 item g). Não mudar nada em `app/spikes/webrtc-probe/`. `app/Cargo.toml` já exclui `spikes`.
  - Conferir: `cargo build --release -j 6` em `app/spikes/encode-probe/` compila; `.\target\release\encode-probe.exe --help` roda; `git status` não mostra mudança em `app/spikes/webrtc-probe/`.
- [X] T010 [US1] Codificador de H.264 da placa na sonda (sonnet, com revisão do principal)
  - Arquivos: criar `app/spikes/encode-probe/src/hw_encoder.rs` e `app/spikes/encode-probe/src/texture.rs`; `src/main.rs` e `src/args.rs` (opção nova).
  - Origem: [contracts/codificador.md](./contracts/codificador.md) ("Interface", "Configuração obrigatória do caminho pela placa", "Erros"); R2, R3 e R4 de [research.md](./research.md); `src-tauri/src/capture/video.rs` (criação do dispositivo Direct3D 11 e da textura NV12; só leitura); crate `windows` 0.62 (`Win32::Media::MediaFoundation`).
  - Regras: achar o codificador com `MFTEnumEx` e `MFT_ENUM_FLAG_HARDWARE` (entrada NV12, saída H.264); imprimir o nome de cada um achado (`MFT_FRIENDLY_NAME_Attribute`) e qual iniciou. Entrada por textura: `MFCreateDXGIDeviceManager`, `MF_SA_D3D11_AWARE`, `MFCreateDXGISurfaceBuffer`. Em `texture.rs`, o quadro NV12 gerado no processador é enviado para uma textura (na sonda se prova a ligação, não a captura). O codificador por hardware é assíncrono: destravar com `MF_TRANSFORM_ASYNC_UNLOCK` e seguir os eventos (`METransformNeedInput` e o de saída pronta). Configuração obrigatória, os sete itens do contrato: baixa latência ligada (`CODECAPI_AVLowLatencyMode`); nenhum quadro B; perfil Constrained Baseline (`eAVEncH264VProfile_ConstrainedBase`); taxa constante com alvo que muda ao vivo (`CODECAPI_AVEncCommonRateControlMode`, `CODECAPI_AVEncCommonMeanBitRate`); quadro inteiro só a pedido (`CODECAPI_AVEncMPVGOPSize` muito longo, `CODECAPI_AVEncVideoForceKeyFrame`); tamanho e quadros por segundo os pedidos (padrão 1920×1080, 60); entrada por textura no mesmo dispositivo. Interface do contrato: iniciar, entregar, mudar alvo, pedir quadro inteiro, receber, parar. Cada quadro devolvido traz os bytes, se é inteiro ou parcial, o tempo de captura em microssegundos e o atraso entre entregar e receber. No primeiro quadro inteiro, conferir e imprimir uma linha: tem códigos de início? traz os parâmetros de sequência (SPS e PPS) junto? Se não trouxer, acrescentá-los antes de cada quadro inteiro (plano B da R4) e dizer isso no log. Propriedade que o codificador recusa: uma linha de log com o nome dela, sem derrubar a sonda (só a falha de início é fatal). Opção nova `--encode-only <segundos>`: gera e comprime sem entrar em sala e imprime quadros entregues e recebidos, atraso médio e maior, quantos quadros inteiros e o tamanho médio deles. Cada nome da API é conferido no código da crate `windows` 0.62 e na documentação da Microsoft antes de usar; o que não existir ou se comportar diferente fica anotado para o `LEIA-ME.md` (T012). O principal revisa antes do T014.
  - Conferir: `cargo build --release -j 6` compila; `encode-probe.exe --encode-only 10` na máquina do dono imprime o nome do codificador, perto de 600 quadros recebidos e a linha sobre SPS e PPS. Numa máquina sem codificador por hardware, diz "não achou" e sai sem travar.
- [X] T011 [US1] Entregar o vídeo comprimido pelo caminho já codificado, uma fonte para todas as ligações (sonnet)
  - Arquivos: `app/spikes/encode-probe/src/main.rs`, `src/media.rs`, `src/args.rs`.
  - Origem: `libwebrtc-0.4.0/src/video_source.rs` (`NativeVideoSource::new_encoded`, `capture_encoded_frame`, `take_keyframe_request`, `take_rate_control_request`, `EncodedRateControl`), `src/video_frame.rs` (`EncodedVideoFrame { codec, payload, timestamp_us, frame_type, resolution, frame_metadata }`, `EncodedVideoCodec::H264`, `EncodedFrameType::{Key, Delta}`), `src/rtp_sender.rs` (`set_video_encoder_backend(VideoEncoderBackend::PreEncoded)`), `webrtc-sys-0.3.53/src/passthrough_video_encoder.cpp`; `app/spikes/webrtc-probe/src/main.rs` (trecho que liga a faixa ao canal, perto das linhas 900 a 935); R1, R5 e R8.
  - Regras: valor novo `--backend placa`, que passa a ser o padrão. Nele: **uma** fonte `new_encoded` (1920×1080) e **uma** faixa para a cheia, usadas por todas as ligações (R5); o canal de vídeo de cada ligação recebe `set_video_encoder_backend(PreEncoded)` e a faixa; a ordem de codecs vem de `video_encoder_codecs(VideoEncoderBackend::PreEncoded)`, com H.264 primeiro; o log mostra o `profile-level-id` que a resposta aceitou. A cada quadro: ler `take_keyframe_request()`; havendo pedido, o próximo quadro sai inteiro, com no máximo um a cada 300 ms (pedido dentro da janela é adiado, não descartado); `take_rate_control_request()` é só impresso. Parar de enviar é `active=false`; nunca `set_track(None)` num canal ativo. `--backend software` continua igual ao da sonda do G1. Estatísticas: a linha `[envio vídeo]` ganha `origem=placa|biblioteca` e o nome do codificador, além do `encoderImplementation` que a biblioteca informa; linha nova `[codificador]` a cada `--stats-secs` com quadros entregues e recebidos, atraso médio e maior, quadros inteiros, tamanho médio do quadro inteiro e erros. Som igual ao da sonda do G1.
  - Conferir: `cargo build --release -j 6` compila; `--encode-only 10` continua funcionando; com o site local e uma sala, a sonda entra e anuncia a transmissão (a imagem no Chrome é conferida no T014).
- [X] T012 [US1] Comandos da prova de três pessoas e o `LEIA-ME.md` da sonda (sonnet)
  - Arquivos: `app/spikes/encode-probe/src/main.rs`, `src/args.rs`; criar `app/spikes/encode-probe/LEIA-ME.md`.
  - Origem: [quickstart.md](./quickstart.md) (G4 e G5, coluna "O que fazer"); [plan.md](./plan.md) ("Gates"); R6, R7, R11 e R18; `app/spikes/webrtc-probe/LEIA-ME.md` (formato).
  - Regras: (1) Reduzida: segunda fonte, segunda faixa e segundo codificador, criados só quando alguém precisa e parados quando ninguém recebe; 360p, 15 quadros por segundo, 500 kbps; opção `--reduzida-fraca` usa 720p, 30, 1,5 Mbps. (2) Regra do Assistir pelo `ViewMessage`: `full` recebe a cheia; `thumb` recebe a reduzida (troca de faixa no canal daquela ligação); `hidden` e `off` são `active=false`; sem `ViewMessage`, nada. (3) Comandos pelo teclado: `peers` (lista quem recebe o quê), `tier <início do id> full|reduced` (força um espectador, sem esperar regra de banda), `fail` (simula a falha do codificador da placa e troca todas as ligações para o caminho pelo processador, sem renegociar), `fail --reabrir` (plano B: para e reabre a transmissão sozinha). (4) Opção `--sem-placa`: começa já no caminho pelo processador. (5) Em toda troca de caminho, imprimir a duração da pausa em milissegundos. (6) `LEIA-ME.md`: como compilar; preparação; os itens (a) a (f) do G4 e (a) a (g) do G5, cada um com o comando exato e o que observar; o que foi achado na API da Media Foundation no T010; o que a sonda não prova sozinha.
  - Conferir: `cargo build --release -j 6` compila; `--help` lista as opções novas; o `LEIA-ME.md` tem os 13 itens, cada um com comando.
- [X] T013 [US1] Trocar os "a definir pela sonda" do `quickstart.md` pelos comandos reais (haiku)
  - Arquivos: `specs/020-codificar-pela-placa/quickstart.md`.
  - Origem: `app/spikes/encode-probe/LEIA-ME.md` (T012).
  - Regras: em cada linha de G4 e G5 que diz `a definir pela sonda`, pôr o comando do `LEIA-ME.md`. Conferir que o nome do executável (`encode-probe`) é o usado nos comandos de `scripts/processador.ps1`. Não mudar critérios nem números.
  - Conferir: `grep -n "a definir pela sonda" specs/020-codificar-pela-placa/quickstart.md` não devolve nada.
- [ ] T014 [US1] Rodar o G4 e registrar cada item; decidir R1, R2 e R4 (pessoa e principal)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "G4, uma pessoa"); `research.md` (R1, R2, R4 e a tabela "O que ficou sem confirmação": marcar "confirmado" ou "plano B", com data); `contracts/codificador.md` (itens "não confirmado" que o G4 resolve).
  - O que fazer: a pessoa roda a sonda contra o site local com um Chrome assistindo, seguindo o `LEIA-ME.md`; o principal decide.
  - Critérios de passe (todos): (a) acha e inicia o codificador de H.264 da placa; (b) o Chrome recebe 1920×1080 em H.264 com som (`chrome://webrtc-internals`); (c) pelo menos **53 quadros por segundo** no Chrome, por pelo menos 1 minuto; (d) o diagnóstico da sonda mostra que a compressão não é a de software da biblioteca (não é OpenH264; `origem=placa`); (e) processador da sonda em núcleos, média de 60 s, registrado (`scripts/processador.ps1 -Processo encode-probe -Nativo`); (f) atraso entre entregar o quadro e receber o comprimido registrado, média e maior.
  - O que registrar: um número ou "passou/falhou" por item, no formato de medição do T002, com quantas pessoas assistiam e como foi conferido; se o quadro inteiro traz SPS e PPS; se a taxa muda ao vivo. Prova SC-001 (parte de uma pessoa) e FR-001.
  - Se não passar: parar, registrar o motivo (FR-002) e voltar à decisão R2 da 019 (biblioteca modificada ou outra biblioteca). Nenhuma tarefa depois desta começa.
- [ ] T015 [US1] Rodar o G5 e registrar cada item; decidir R5, R7, R8 e R11 (pessoa e principal)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "G5, três pessoas"); `research.md` (R5, R7, R8, R11 e a tabela do fim); `contracts/caminho.md` ("Plano B da troca"); `data-model.md` onde citar "não confirmado" resolvido aqui.
  - O que fazer: três Chromes (ou três aparelhos) assistindo à sonda, seguindo o `LEIA-ME.md`; o principal decide.
  - Critérios de passe (todos): (a) três pessoas recebem o mesmo vídeo (1920×1080, H.264, imagem e som); (b) processador com três em até **25%** acima do de uma (duas medições de 60 s); (c) quem entra no meio vê imagem em até **3 s**; (d) os outros não congelam mais de **0,5 s** nessa entrada (`freezeCount` e `totalFreezesDuration`); (e) parar de assistir e voltar funciona sem derrubar a sonda; (f) trocar um espectador para a reduzida e de volta funciona, sem mexer nos outros; (g) forçar a troca para o caminho pelo processador mantém a transmissão, sem ninguém clicar em nada.
  - O que registrar: um número ou "passou/falhou" por item; o tamanho do quadro inteiro em 1080p; a duração da pausa no item (g). Prova SC-001 (parte de três pessoas), FR-001 e os cenários 2 a 4 da história 1.
  - Se não passar: parar e registrar. Planos B: (a) ou (b), uma fonte por ligação (R5); (d), aumentar o intervalo mínimo entre quadros inteiros (R8); (f), dois canais por ligação entre apps (R7); (g), parar e reabrir a transmissão sozinho, com pausa de até 2 s (R11). Plano B adotado muda as tarefas das fases 4 a 8: o principal as ajusta neste arquivo antes de seguir.

**Checkpoint**: a prova terminou com resultado registrado para uma e para três pessoas (SC-001). Só com G5 verde começam as fases seguintes.

---

## Phase 4: User Story 2 - Transmitir sem derrubar o jogo (Priority: P1)

**Goal**: transmitindo 1080p60 pelo app nativo, o jogo perde menos quadros do que perde hoje e o app usa no máximo metade do processador do app de hoje, com os mesmos passos para começar a transmitir.

**Independent Test**: na mesma máquina, sessão e jogo, transmitir 1080p60 para 3 pessoas pelo app de hoje e pelo app nativo. Comparar o FPS do jogo e o processador em núcleos (`scripts/processador.ps1`).

- [ ] T016 [P] [US2] Sonda com captura real, para provar cedo a R3 (sonnet, pessoa roda)
  - Arquivos: criar `app/spikes/encode-probe/src/capture.rs`; `app/spikes/encode-probe/src/main.rs`, `src/args.rs`, `LEIA-ME.md`; `specs/020-codificar-pela-placa/relatorio.md` (seção "G6, com jogo", item "prévia").
  - Origem: `src-tauri/src/capture/video.rs` (`Windows.Graphics.Capture` e o processador de vídeo da placa, que escreve a textura NV12; só leitura); R3 de [research.md](./research.md); [plan.md](./plan.md) ("Gates", G6: "ou a sonda com captura").
  - Regras: opção `--janela <título>`: captura a janela que a pessoa indicar e entrega a textura NV12 direto ao codificador, no mesmo dispositivo gráfico, sem cópia para a memória. Opção `--pela-memoria`: faz o plano B da R3 (copia para a memória e envia para uma textura), para comparar. Sem `--janela`, a sonda continua gerando o vídeo. Nenhum teste automático usa `--janela`: captura real só com a pessoa, na janela que ela escolher. O `LEIA-ME.md` ganha os dois comandos.
  - Conferir: `cargo build --release -j 6` compila. A pessoa roda com o jogo aberto e uma pessoa assistindo, nos dois modos, e registra: se captura e codificador dividiram o dispositivo sem travar, o FPS do jogo e os núcleos da sonda (`scripts/processador.ps1 -Processo encode-probe -Nativo`). Não substitui o G6 (T024): só antecipa a resposta sobre a R3, antes de escrever `encode/` e `publish.rs`. Se não dividirem, o T020 e o T021 já partem da entrada pela memória.
- [ ] T017 [P] [US2] Registrar na 019 o que muda nas tarefas de transmitir (principal)
  - Arquivos: `specs/019-app-rust-convite/tasks.md` (texto de T048, T050, T052, T053 e T056; sem mudar caixas marcadas), `specs/019-app-rust-convite/research.md` (R2, um adendo).
  - Origem: R17 de [research.md](./research.md); resultado do G5 em `relatorio.md`.
  - Regras: R2 da 019 continua (mesma biblioteca), com o adendo "a compressão é do app" e link para a 020. T037 não muda. T048 passa a publicar por `espia-media/src/publish.rs` e `encode/` (quadro cru só no caminho pelo processador). T050: no caminho pela placa a regra do Assistir é por qualidade (`espia-core::tiers`); `send_params` por ligação vale só no caminho pelo processador. T052: a qualidade efetiva vai para o codificador. T053: no caminho pela placa, "limitado" é quadro que o codificador não comprimiu a tempo (R10). T056: já ajustado em 2026-11-12 (sem aviso e sem teto no processador); só conferir que cita a 020. As metas SC-004 e SC-005 da 019 são as SC-003 e SC-002 daqui.
  - Conferir: as cinco tarefas da 019 citam a 020; o T056 continua sem aviso.
- [ ] T018 [P] [US2] Cenas de dados falsos da linha "Compressão" no Diagnóstico de mídia (sonnet)
  - Arquivos: `app/crates/espia-ui/src/mocks/room.rs` (cenas e o teste que conta as cenas), `app/crates/espia-ui/src/bind/room.rs` (entregar as linhas a `RoomState.diag`), `app/crates/espia-ui/ui/pages/room.slint` (o painel poder abrir já aberto numa cena), e o ponto de `app/crates/espia-ui/src/ui.rs` ou `src/mocks/mod.rs` em que a cena aberta é aplicada.
  - Origem: [data-model.md](./data-model.md) ("Linha do diagnóstico"); R12 e R18; `app/crates/espia-ui/ui/components/media-diagnostics.slint` e `room-state.slint` (`DiagLine { label, value }`, `RoomState.diag`, hoje sem ninguém que o preencha).
  - Regras: a linha entra na lista `DiagLine` que o painel já recebe; **`view_model.rs` não muda** (R18). Como a cena da sala é só um `RoomView`, as linhas do diagnóstico são estado de apresentação da cena: uma função em `mocks/room.rs` devolve as linhas por `id` de cena, e o mock as aplica em `RoomState.diag` e abre o painel. A mesma entrada (`bind/room.rs`) serve depois ao dado real (T023). Três cenas só do app, na tela `room`: `diag-placa`, com a linha de rótulo `Compressão` e valor `placa de vídeo (<nome do codificador>)`; `diag-processador`, com valor `processador (<nome do codificador>)`; e `diag-desconhecido`, com valor `—`, como as outras linhas quando o dado falta. Nome desconhecido: só `placa de vídeo` ou `processador`. Os nomes das cenas falsas são inventados e ficam claros como exemplo. Nenhuma opção, botão ou chave: a linha só informa. O teste `quarenta_e_sete_do_site_mais_uma_do_app` passa a contar 51 cenas e é renomeado. O teste de paridade (`tests/scene_parity.rs`) permite cenas só do app e não muda.
  - Conferir: as cenas `diag-placa`, `diag-processador` e `diag-desconhecido` (`npm run app:mock -- room --scene <cena>`) abrem com o painel visível e a linha certa; `cargo test -p espia-ui` passa.
- [ ] T019 [P] [US2] Interface do codificador e montagem do quadro em `espia-media` (sonnet)
  - Arquivos: criar `app/crates/espia-media/src/encode/mod.rs`; `app/crates/espia-media/src/lib.rs` (declarar o módulo).
  - Origem: [contracts/codificador.md](./contracts/codificador.md) ("Interface", "O que o codificador recebe", "O que o codificador devolve", "Erros"); [data-model.md](./data-model.md) ("Quadro comprimido"); `app/spikes/encode-probe/src/hw_encoder.rs` (o que o G4 confirmou sobre o formato).
  - Regras: trait do codificador com as seis operações do contrato (iniciar com configuração, entregar quadro, mudar alvo em bps, pedir quadro inteiro, receber quadro comprimido ou nenhum, parar). Um codificador serve **uma qualidade**. Configuração: largura, altura, quadros por segundo (15, 30 ou 60) e taxa alvo. Quadro de entrada: textura NV12 na placa ou NV12 em memória. Quadro comprimido, com os campos do modelo: codec H.264; parte (inteiro ou parcial); dados (H.264 com códigos de início, parâmetros de sequência junto de cada quadro inteiro); tamanho em bytes; tamanho da imagem; tempo de captura em microssegundos; qualidade (cheia ou reduzida). Erros como valores, um por linha da tabela "Erros" do contrato: não achou, não iniciou, erro ao comprimir, alvo não aceito. Funções puras de montagem: separar as unidades do H.264 pelos códigos de início, dizer se o quadro é inteiro, dizer se traz SPS e PPS, e acrescentá-los antes do quadro inteiro quando faltarem (plano B da R4). Codificador falso para testes (saídas, erros e recusa de alvo roteirizados), usado também pelos testes do publicador. Testes primeiro, com bytes de exemplo escritos no próprio teste.
  - Conferir: `cargo test -p espia-media encode` passa; a crate continua sem dependência de `espia-ui`.
- [ ] T020 [US2] Codificador da placa em `espia-media` (sonnet, com revisão do principal)
  - Arquivos: criar `app/crates/espia-media/src/encode/hardware.rs`; `app/crates/espia-media/Cargo.toml` (crate `windows` 0.62 com as features do T009).
  - Origem: `app/spikes/encode-probe/src/{hw_encoder.rs,texture.rs}` como ficaram depois do G4 e do G5; [contracts/codificador.md](./contracts/codificador.md) ("Configuração obrigatória", "Erros e o que o chamador faz"); R2, R3, R4, R10 e R18; [contracts/caminho.md](./contracts/caminho.md) ("Situações e o caminho").
  - Regras: cumpre o trait do T019. Configuração obrigatória, os sete itens: baixa latência ligada; nenhum quadro B; perfil Constrained Baseline; taxa constante com alvo que muda ao vivo; quadro inteiro só a pedido; tamanho e quadros por segundo da qualidade; entrada por textura no mesmo dispositivo gráfico da captura. Aceita também NV12 em memória, enviado para uma textura (câmera, e plano B da R3). Alvo que o codificador recusa mudar ao vivo: reabrir com a taxa nova e o próximo quadro sai inteiro; cada reabertura é informada a quem chama (a regra de "mais de uma por minuto" é do T008). Janela capturada que muda de tamanho: reabrir no tamanho novo, próximo quadro inteiro. Placa no limite: contar os quadros que não saíram a tempo e informar "limitado" a cada segundo (R10; se o G6 mostrar que não há esse sinal, usar o atraso entre entregar e receber). Duas placas: procurar o codificador da placa em que está o dispositivo da captura (`MFT_ENUM_ADAPTER_LUID`); se não houver ali, "não achou". Placa ocupada por outro programa que grava ou transmite: aparece como "não iniciou" ou como erro, sem tratamento especial. Nome do codificador lido do Windows, para a linha do diagnóstico. Nenhum erro vira aviso para a pessoa. Teste que precisa de placa: `#[ignore]`, com nome começando por `placa_`, rodado à mão. O principal revisa.
  - Conferir: `npm run app:check` e `npm run app:test` verdes sem placa; na máquina do dono, `cargo test -p espia-media --manifest-path app/Cargo.toml -- --ignored placa_` comprime 120 quadros 1080p60 e o primeiro sai inteiro.
- [ ] T021 [US2] Saída da captura em textura, no mesmo dispositivo gráfico (sonnet)
  - Arquivos: `app/crates/espia-capture/src/video.rs`, `app/crates/espia-capture/src/output.rs` (criados pelos T045 e T046 da 019).
  - Origem: R3 de [research.md](./research.md); `src-tauri/src/capture/video.rs` (o processador de vídeo da placa escreve a textura NV12 `output` e depois copia para `staging` e para a memória; só leitura); `app/spikes/encode-probe/src/capture.rs` e o resultado da prévia (T016).
  - Regras: além da saída em memória do T046 da 019, a captura entrega a textura NV12 já reduzida e convertida, com o dispositivo gráfico em que ela vive e o carimbo do mesmo relógio. Quando quem consome é o codificador da placa, a cópia para a memória não é feita (é o peso que esta feature tira). A cópia para a memória continua disponível para a prévia parada (T055 da 019) e para o caminho pelo processador. A captura não escreve numa textura que o codificador ainda segura: a entrega usa um pequeno conjunto de texturas, copiadas dentro da placa. Mídia mais velha que 200 ms continua descartada (regra do T046 da 019). Se captura e codificador não puderem dividir o dispositivo (não confirmado, G6), o plano B é a entrada pela memória, e a tarefa registra isso sem apagar a saída em textura.
  - Conferir: `cargo check -p espia-capture` e `cargo test -p espia-capture` (em `app/`) passam; `git status` não mostra mudança em `src-tauri/`.
- [ ] T022 [US2] Publicador: uma fonte por qualidade, repartida por todas as ligações (principal desenha, sonnet escreve)
  - Arquivos: criar `app/crates/espia-media/src/publish.rs`; `app/crates/espia-media/src/lib.rs`.
  - Origem: R5, R7, R8 e R10; [data-model.md](./data-model.md) ("Transmissão publicada", "Quadro comprimido", "Pedido de quadro inteiro"); `app/crates/espia-core/src/tiers.rs` (T004 a T008); `encode/mod.rs` (T019); `app/spikes/encode-probe/src/main.rs` (como a sonda liga fonte, faixa e canal); o trait da ligação criado no T037 da 019 (`peer.rs`).
  - Regras: o principal escreve antes, num comentário no topo, quem é dono das fontes, das faixas e dos codificadores, e como as ações de `tiers` viram chamadas. A `libwebrtc` fica atrás de trait, como no T037, para o teste usar ligações falsas. Para cada qualidade ativa: uma fonte de vídeo já codificado, uma faixa e um codificador; no máximo duas ao mesmo tempo (FR-006). Esta tarefa entrega a **cheia**; a reduzida entra no T027. O canal de vídeo de cada ligação usa o repasse (`PreEncoded`) e aponta para a faixa da qualidade daquele espectador. Um quadro é comprimido uma vez e entregue uma vez à fonte (FR-004). O tempo do quadro é o carimbo que ele traz da captura, o mesmo relógio do som (FR-016). O primeiro quadro de uma qualidade que acabou de ficar ativa é inteiro. A cada quadro, ler `take_keyframe_request()` de cada fonte e aplicar a regra do T006. `take_rate_control_request()` só é guardado para o diagnóstico, não decide nada. A configuração do codificador vem de `effective_quality(escolhida, espectadores em Full, degraus)`; quando ela muda, mudar o alvo ou reabrir, e o próximo quadro sai inteiro. O sinal "limitado" do codificador alimenta o `CpuGovernor` uma vez por segundo. Sem ninguém recebendo, nenhuma qualidade fica ativa e nada é comprimido. Parar de enviar é `active=false`; nunca tirar a faixa de um canal ativo. Testes primeiro, com o codificador falso e ligações falsas: (1) três espectadores em `Full` geram uma compressão por quadro e três canais na mesma faixa; (2) nenhum espectador gera nenhuma compressão; (3) pedido de quadro inteiro faz o próximo quadro sair inteiro, e dois pedidos dentro de 300 ms dão o segundo adiado; (4) de 4 para 5 espectadores em `Full` com 1080p60 escolhido, o codificador passa a 720p60 e 4 Mbps e o próximo quadro sai inteiro; (5) nenhuma sequência de eventos produz "tirar faixa".
  - Conferir: `cargo test -p espia-media publish` passa; `npm run app:check` e `npm run app:test` verdes.
- [ ] T023 [US2] Linha "Compressão" com o dado real (sonnet)
  - Arquivos: `app/crates/espia/src/diagnostics.rs` (do T066 da 019; se ainda não existir, criar só com o que esta tarefa precisa).
  - Origem: T018 (a entrada das linhas em `espia-ui`); `publish.rs` (caminho em uso e nome do codificador); R12; FR-018.
  - Regras: durante uma transmissão própria, a lista de linhas do Diagnóstico de mídia ganha a linha de rótulo `Compressão`, com o valor nos mesmos formatos das cenas do T018: `placa de vídeo (<nome>)`, `processador (<nome>)` ou `—`. O nome vem do Windows (placa) ou da biblioteca (processador). A linha muda sozinha quando o caminho troca (T033). Sem transmissão própria, a linha não aparece. Nenhum outro lugar da tela mostra "placa", "processador" ou nome de codificador (FR-014). Se o T066 da 019 vier depois, ele acrescenta as outras linhas na mesma lista.
  - Conferir: `npm run app:check` e `npm run app:test` verdes; as três cenas do diagnóstico continuam abrindo.
- [ ] T024 [US2] G6 e roteiro da história 2, lado a lado com o app de hoje (pessoa e principal)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seções "G6, com jogo" e "História 2"); `research.md` (R3, R10 e a tabela do fim).
  - Depende de: T016 a T023 (com a prévia do T016 já registrada) e das tarefas T045 a T052 da 019, já ajustadas pelo T017, para o app nativo transmitir com captura real.
  - O que fazer: seguir "G6" e "História 2" do [quickstart.md](./quickstart.md). Mesma máquina, mesmo jogo, mesma sessão, 1080p60 com som, as mesmas 3 pessoas, conferidas nos dois lados. Em cada app: FPS do jogo sem transmitir, FPS transmitindo e `scripts/processador.ps1` (árvore para o app de hoje, `-Nativo` para o novo). No app nativo: mudar o jogo de cena parada para muito movimento; mudar o tamanho da janela capturada no meio; medir a memória com `npm run memoria -- -Nativo`; olhar os passos até a transmissão começar.
  - Critérios de passe: perda de FPS do jogo no app nativo não maior que a do app de hoje (SC-003); núcleos do app nativo em no máximo metade dos do app de hoje (SC-002); memória dentro da meta da 019 (SC-012); imagem fluida na mudança de cena (cenário 3); mesmos passos de hoje, sem pergunta nova e sem opção sobre placa de vídeo (cenário 4, FR-014).
  - O que registrar: os dois blocos de medição completos, com a qualidade efetiva e as pessoas assistindo; se captura e codificador dividiram o dispositivo gráfico; qual sinal de "placa no limite" apareceu; a pausa na mudança de tamanho da janela. Prova FR-003 e FR-017.
  - Se não passar: parar e registrar. Plano B: entrada pela memória (R3) e revisão das metas com a pessoa.

**Checkpoint**: o app nativo transmite comprimindo uma vez pela placa, com o ganho medido lado a lado.

---

## Phase 5: User Story 3 - Mais gente assistindo não pesa mais (Priority: P1)

**Goal**: de 1 a 7 pessoas assistindo, o processador do app quase não muda, e ninguém percebe quando alguém sai.

**Independent Test**: transmitir e medir o processador do app, em núcleos, com 1, 3 e 7 pessoas em tela cheia.

- [ ] T025 [US3] Testes do publicador com a sala cheia, e correções (sonnet)
  - Arquivos: `app/crates/espia-media/src/publish.rs` (testes e o que eles apontarem).
  - Origem: cenários 1 a 3 da história 3 e "Edge Cases" da [spec.md](./spec.md) ("Muitas pessoas entrando em sequência"); [contracts/qualidades.md](./contracts/qualidades.md) seções 6 e 8; `effective_quality` em `quality.rs`.
  - Regras: testes novos, com codificador falso e ligações falsas: (1) de 1 a 7 espectadores em `Full`, as compressões por quadro continuam uma por qualidade ativa; (2) com 1080p60 escolhido, a cheia é 1080p60 com 1 a 4, 720p60 e 4 Mbps com 5 e 6, 720p30 e 2,5 Mbps com 7, e o codificador é reconfigurado a cada mudança; (3) um espectador sai e os canais dos que ficam não recebem nenhuma ação (nem troca de faixa, nem `active`, nem quadro inteiro); (4) cinco pessoas entram em 1 s e saem no máximo quatro quadros inteiros (em 0, 300, 600 e 900 ms), sem nenhum pedido descartado; (5) nove espectadores (sala de 10) continuam com uma compressão por qualidade. Corrigir em `publish.rs` o que falhar.
  - Conferir: `cargo test -p espia-media publish` passa.
- [ ] T026 [US3] Roteiro da história 3 e medição de SC-004 (pessoa)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "História 3").
  - O que fazer: seguir "História 3" do [quickstart.md](./quickstart.md): 60 s de `scripts/processador.ps1 -Nativo` com 1, com 3 e com 7 pessoas em tela cheia, conferidas; uma pessoa sai; olhar o indicador de qualidade com 5, 6 e 7. Sem sete pessoas reais, usar abas e aparelhos e anotar que não eram pessoas.
  - Critério de passe: de 1 a 7, os núcleos crescem no máximo **25%** (SC-004); nada muda para quem fica quando alguém sai; a regra de qualidade por número de espectadores continua aparecendo no indicador (FR-011).
  - O que registrar: três blocos de medição, cada um com a qualidade efetiva (ela cai com 5 ou mais), e um "passou/falhou" por cenário.

**Checkpoint**: o custo de processador não cresce com a plateia.

---

## Phase 6: User Story 4 - Quem assiste não percebe diferença (Priority: P1)

**Goal**: quem assiste pelo Chrome no computador, pelo app, pelo Chrome no celular ou pelo Safari no iPhone recebe imagem e som como hoje; a miniatura continua atualizando; parar de assistir para de receber.

**Independent Test**: com uma pessoa transmitindo pelo app nativo, assistir nos quatro aparelhos e conferir imagem, som, miniatura, parar e voltar a assistir.

- [ ] T027 [US4] Reduzida para miniatura e regra do Assistir no publicador (sonnet)
  - Arquivos: `app/crates/espia-media/src/publish.rs`.
  - Origem: `tiers.rs` (T004 e T005); [contracts/qualidades.md](./contracts/qualidades.md) seções 1, 2, 4 e 5; FR-006, FR-007 e FR-008; `wants_sound` em `quality.rs`.
  - Regras: aplicar as ações do T005. `Thumb` recebe a reduzida: segunda fonte, segunda faixa e segundo codificador, em **360p, 15 quadros por segundo, 500 kbps**, existindo só enquanto alguém a recebe. `Hidden`, `Off` e sem modo: `active=false` no canal de vídeo, com a faixa mantida. O som segue `wants_sound`: só `Off` o corta. Voltar a `Full`: `active=true` e quadro inteiro da cheia. Todos em miniatura: o codificador da cheia para. Ninguém assistindo: os dois param. `ViewMessage` com revisão velha é ignorada (mesma regra do T050 da 019). Testes primeiro, com codificador falso e ligações falsas: as linhas das seções 4 e 5 do contrato no nível do publicador; "1 `Full` e 1 `Thumb`" dão dois codificadores; "2 `Thumb`" dão só o da reduzida; quem não mandou `ViewMessage` não recebe nenhum quadro nem som.
  - Conferir: `cargo test -p espia-media publish` passa.
- [ ] T028 [US4] Espectador cujo navegador não aceita o formato (principal desenha, sonnet escreve)
  - Arquivos: `specs/020-codificar-pela-placa/research.md` (decisão nova, R19) e `contracts/codificador.md`; `app/crates/espia-core/src/tiers.rs` (regra pura) e `app/crates/espia-media/src/publish.rs`.
  - Origem: "Edge Cases" da [spec.md](./spec.md) ("Quem assiste por um navegador que não aceita o formato"); `src/lib/peer.ts` (`orderCodecs`, o que o site negocia hoje); a ordem de codecs da sonda (`proto.rs`, `order_codecs`).
  - Regras: o plano não decide este caso, e o site não muda nesta feature, então não há como mostrar aviso do lado de quem assiste. O principal decide e registra como R19 antes de o sonnet escrever. Recomendado: a ligação cuja resposta não aceita H.264 recebe, só ela, pelo caminho da biblioteca (quadro cru, no codec que ela aceitou, com `send_params` por ligação), e as outras continuam no caminho pela placa. A detecção olha os codecs negociados daquela ligação. Nunca tela preta sem explicação. Teste primeiro: ligação sem H.264 cai no caminho compatível; ligação com H.264 não muda.
  - Conferir: R19 escrita com data; `cargo test -p espia-core tiers` e `cargo test -p espia-media publish` passam.
- [ ] T029 [US4] Roteiro da história 4 nos quatro aparelhos (pessoa)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "História 4").
  - O que fazer: seguir os nove passos de "História 4" do [quickstart.md](./quickstart.md), com uma pessoa transmitindo pelo app nativo e espectadores no Chrome do computador, no app, no Chrome do celular e no Safari do iPhone.
  - Critérios de passe: imagem e som nos quatro (SC-011); primeira imagem em até **3 s**, inclusive entrando no meio (SC-006); pelo menos **53 quadros por segundo** em 1080p60 em tela cheia, com até 4 pessoas em tela cheia (SC-005); ninguém congela mais de **0,5 s** quando outro entra (SC-007); som e imagem a até **80 ms** em 10 minutos (SC-008); miniatura atualizando em qualidade menor; parar de assistir ou esconder a janela para de receber; quem não clicou em Assistir não recebe nada (FR-008).
  - O que registrar: um "passou/falhou" por passo e por aparelho, com os números. O iPhone depende de alguém do grupo: sem o aparelho, registrar "não testado" e levar para o T037.

**Checkpoint**: quem assiste não perdeu nada, em nenhum aparelho testado.

---

## Phase 7: User Story 5 - Internet fraca de um não estraga a dos outros (Priority: P2)

**Goal**: quando a internet de um espectador fica fraca, só ele passa à qualidade reduzida, e volta sozinho quando melhora.

**Independent Test**: com três pessoas assistindo em tela cheia, limitar a internet de uma e observar a qualidade que cada uma recebe.

- [ ] T030 [US5] Ler a banda de cada ligação e aplicar a regra de internet fraca (principal decide, sonnet escreve)
  - Arquivos: `app/crates/espia-media/src/publish.rs`; `specs/020-codificar-pela-placa/research.md` (R18, uma linha nova).
  - Origem: `tiers.rs` (T007); R9 e R18; [contracts/qualidades.md](./contracts/qualidades.md) seções 3 e 7; `libwebrtc-0.4.0/src/stats.rs` (`RtcStats::CandidatePair`, campo `available_outgoing_bitrate`).
  - Regras: uma vez por segundo, ler das estatísticas de cada ligação a banda de saída disponível do par em uso e entregar à regra do T007. Espectador marcado fraco: o canal dele passa a apontar para a faixa da reduzida, no ajuste de internet fraca (**720p, 30 quadros por segundo, 1,5 Mbps**; as miniaturas recebem o mesmo vídeo enquanto durar). Volta à cheia quando a regra mandar, inclusive pela tentativa a cada 30 s. A taxa da cheia segue a conta do T007 e vai ao codificador como alvo. Estatística ausente numa leitura: não conta como fraca nem como boa. Ponto que o plano deixa em aberto e que o principal decide antes, registrando na R18: quando a internet de **quem transmite** é o limite, só a taxa baixa, e o indicador de qualidade mostra resolução e quadros por segundo; a spec pede que o indicador mostre a mudança (cenário 3). Recomendado: taxa da cheia abaixo de 60% da nominal por 10 s conta como "limitado" e baixa um degrau pela regra que já existe. Testes primeiro, com bandas roteirizadas em ligações falsas: só o fraco troca de faixa; os outros não recebem nenhuma ação; todos fracos e ninguém troca; a volta acontece.
  - Conferir: `cargo test -p espia-media publish` passa; a decisão do indicador está na R18 com data.
- [ ] T031 [US5] Roteiro da história 5 e medição de SC-009 (pessoa)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "História 5"); `research.md` (R6 e R9: confirmar os números ou adotar o plano B).
  - O que fazer: seguir os sete passos de "História 5" do [quickstart.md](./quickstart.md). Escolher como limitar a internet de uma pessoa (roteador, 4G com sinal fraco ou programa de limite de banda) e anotar o método.
  - Critérios de passe: só a pessoa limitada passa à reduzida (FR-010); as outras duas ficam em pelo menos **53 quadros por segundo** em 1080p60 (SC-009); tirando o limite, ela volta sozinha à cheia; limitando quem transmite, a qualidade baixa para todos e o indicador mostra.
  - O que registrar: quanto tempo levou para descer e para voltar, e se a volta foi pela estimativa de banda ou pela tentativa a cada 30 s; a qualidade que a reduzida entregou; o custo da segunda compressão (núcleos do app e FPS do jogo).
  - Se não passar: planos B da R6 (reduzida sempre em 360p) e da R9 (volta só por tentativa periódica); o principal ajusta as constantes do T004 e do T007.

**Checkpoint**: o espectador mais fraco não puxa os outros para baixo.

---

## Phase 8: User Story 6 - Computador sem placa boa continua transmitindo (Priority: P2)

**Goal**: sem placa de vídeo capaz, ou com a placa falhando no meio, a transmissão acontece pelo processador, com os mesmos passos e sem pergunta nova.

**Independent Test**: com o uso da placa desligado para o teste (`ESPIA_SEM_PLACA`, só em build de depuração), transmitir e conferir que alguém assiste; depois, forçar a falha com uma transmissão pela placa em andamento.

- [ ] T032 [P] [US6] Caminho pelo processador em `espia-media` (sonnet)
  - Arquivos: criar `app/crates/espia-media/src/encode/software.rs`.
  - Origem: [contracts/codificador.md](./contracts/codificador.md) ("O que cada caminho garante"); R11; `app/spikes/webrtc-probe/src/main.rs` (fonte de quadros crus e `--backend software`); `send_params` em `quality.rs`; saída em memória da captura (T046 da 019).
  - Regras: cumpre o trait do T019, entregando o quadro cru e não devolvendo quadro comprimido: quem comprime é a biblioteca, uma vez por espectador que recebe vídeo. Qualidade sob medida por ligação, com `send_params` (cheia, miniatura 360p, 15 quadros por segundo e 500 kbps; `hidden` e `off` com `active=false`). Sem teto próprio: a qualidade é a efetiva (escolhida, número de espectadores e ajuste automático por processador). Sem "cheia" e "reduzida" neste caminho. A regra do Assistir vale igual: nada a quem não clicou em Assistir. Testes primeiro, com ligações falsas: 1080p60 escolhido continua 1080p60; os parâmetros por modo; nenhum envio sem `ViewMessage`.
  - Conferir: `cargo test -p espia-media encode` passa.
- [ ] T033 [US6] Escolher o caminho no começo e trocar no meio da transmissão (sonnet, com revisão do principal)
  - Arquivos: `app/crates/espia-media/src/publish.rs`.
  - Origem: `tiers.rs` (T008); [contracts/caminho.md](./contracts/caminho.md) (inteiro, inclusive "O que NÃO pode acontecer"); R11 e R18; resultado do item (g) do G5 em `relatorio.md`.
  - Regras: o publicador informa os fatos à máquina de estados do T008 e obedece ao estado. No começo: não achou ou não iniciou, transmissão pelo processador desde o primeiro quadro, sem aviso. No meio: **3 erros seguidos** ou **2 s sem quadro comprimido** com alguma qualidade ativa trocam todas as ligações para o caminho do T032, sem renegociar se o G5 (g) confirmou, ou parando e reabrindo sozinho com pausa de até 2 s (plano B). Depois da troca, a transmissão fica no processador até acabar; a seguinte tenta a placa de novo. A qualidade continua a escolhida, e o ajuste automático por processador a baixa se o computador não der conta; o indicador mostra. O som não é interrompido pela troca. Nada é enviado a quem não clicou em Assistir, durante ou depois. A pessoa não clica em nada e não lê nenhum aviso. Variável de ambiente `ESPIA_SEM_PLACA`: força o caminho pelo processador, **só em build de depuração**; em build de release é ignorada. Nunca uma opção na tela. Testes primeiro, com o codificador falso roteirizado: três erros trocam; dois erros e um quadro não trocam; 2 s sem quadro com qualidade ativa trocam; sem qualidade ativa não trocam; depois da troca nada volta à placa; a transmissão seguinte começa em `escolhendo`. O principal revisa.
  - Conferir: `cargo test -p espia-media publish` passa; `npm run app:check` e `npm run app:test` verdes.
- [ ] T034 [US6] Roteiro da história 6 e SC-010 (pessoa)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "História 6").
  - O que fazer: seguir os sete passos de "História 6" do [quickstart.md](./quickstart.md), num computador sem placa capaz ou com `ESPIA_SEM_PLACA` num build de depuração: transmitir 10 vezes; conferir o indicador e a linha do diagnóstico; com uma transmissão pela placa em andamento, forçar a falha.
  - Critérios de passe: a transmissão começa nas 10 tentativas, com os mesmos passos, em pelo menos 720p30 (SC-010); nenhum aviso técnico nem escolha nova (FR-012, FR-014); na falha, a transmissão continua com no máximo uma pausa curta, sem transmitir de novo (FR-013); a linha do diagnóstico passa de "placa de vídeo" para "processador" (FR-018).
  - O que registrar: um "passou/falhou" por passo, a duração da pausa na troca e a qualidade mostrada no indicador.

**Checkpoint**: ninguém do grupo deixa de transmitir por causa da placa.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: registro, conferência com o grupo, revisão e limpeza.

- [ ] T035 [P] Registrar a compressão pela placa em `DEFINICOES.md` (sonnet)
  - Arquivos: `DEFINICOES.md`.
  - Origem: [plan.md](./plan.md) (Constitution Check, linha "Restrições técnicas"); R1, R6, R11 e R12.
  - Regras: registrar, no vocabulário do produto: o app nativo comprime o vídeo uma vez, pela placa de vídeo, e manda o mesmo vídeo a todos pela conexão direta; duas qualidades (cheia e reduzida); sem placa capaz, o app usa o processador, sozinho, na qualidade escolhida, que baixa sozinha se o computador não der conta; a linha "Compressão" do Diagnóstico de mídia; `scripts/processador.ps1`; `ESPIA_SEM_PLACA` (só depuração). Nenhum servidor de mídia. Não apagar registro histórico. O T101 da 019 mexe no mesmo arquivo: não rodar os dois juntos.
  - Conferir: o arquivo cita a 020, as duas qualidades e o script.
- [ ] T036 [P] Fechar nos documentos da 020 o que foi confirmado (principal)
  - Arquivos: `specs/020-codificar-pela-placa/{research.md,data-model.md,quickstart.md}` e `contracts/{codificador.md,qualidades.md,caminho.md,medicao.md}`.
  - Regras: cada linha das tabelas "O que ficou sem confirmação" e cada "não confirmado" ou "a confirmar" vira "confirmado em AAAA-MM-DD", "plano B adotado" ou "pendente", com link para a seção do `relatorio.md`. Onde `data-model.md` ainda diz "ainda não está decidido" sobre a linha do diagnóstico, pôr a forma do T018.
  - Conferir: `grep -n -i "não confirmado\|a confirmar\|ainda não está decidido" specs/020-codificar-pela-placa` só devolve linhas marcadas "pendente", com o motivo.
- [ ] T037 Conferência com o grupo: outras marcas de placa e aparelhos (pessoa)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "Conferência com o grupo").
  - O que fazer: levantar quais placas o grupo tem. Em pelo menos um computador de cada marca (NVIDIA, AMD, Intel), transmitir pelo app nativo e ler a linha "Compressão". Assistir pelo Safari no iPhone e pelo Chrome no celular.
  - O que registrar: por computador, marca e modelo da placa, o valor da linha "Compressão", a qualidade e se a imagem chegou; por aparelho, imagem e som. Marca em que o codificador do Windows não serve fica no caminho pelo processador até haver solução (R2), e isso fica escrito. Prova FR-015 e SC-011 fora da máquina do dono.
- [ ] T038 Revisão de regras e de segurança (principal)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "Decisões"); correções em `app/`.
  - Regras: conferir que nenhuma tela mostra "placa", "hardware", "software" ou nome de codificador fora do Diagnóstico de mídia (procurar em `app/crates/espia-ui/ui/` e nos textos de `espia-core`); que `ESPIA_SEM_PLACA` não tem efeito em build de release; que nada novo é guardado em disco nem enviado ao servidor; que nenhuma chamada tira a faixa de um canal ativo (procurar `set_track(None)` em `app/crates/`); que o Worker, `src/shared/protocol.ts`, o site e `src-tauri/` não mudaram (`git diff --stat` nesses caminhos). Registrar que a pendência de patente do H.264 com o codificador do sistema (R16) segue junto da pendência de licenças da 019 (T093 de lá).
  - Conferir: a seção lista cada item como "ok" ou o que foi corrigido.
- [ ] T039 Apagar `app/spikes/encode-probe/` (haiku)
  - Arquivos: apagar `app/spikes/encode-probe/`.
  - Regras: só depois de G4, G5 e G6 registrados no `relatorio.md` e do T038. Não apagar as outras sondas (são da 019, T106 de lá). Não apagar `relatorio.md`.
  - Conferir: `npm run app:check` passa; `quickstart.md` e `relatorio.md` continuam legíveis sem a pasta (os comandos da sonda ficam como registro histórico).
- [ ] T040 Rodar o `quickstart.md` inteiro (pessoa)
  - Arquivos: `specs/020-codificar-pela-placa/relatorio.md` (seção "Decisões", item "Quickstart").
  - O que fazer: rodar os gates automáticos da seção 1 (`npm run check`, `npm test`, `npm run build`, `npm run app:check`, `npm run app:test`) e percorrer os roteiros das histórias 2 a 6 na ordem, com o app nativo.
  - O que registrar: um "passou/falhou" por roteiro e a data. Falha reabre a tarefa da história correspondente.

**Checkpoint**: tudo registrado; o que ficou pendente está escrito com o motivo.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sem dependências. T001 e T002 em paralelo; T003 depois dos dois.
- **Foundational (Phase 2)**: sem dependências além do que a 019 já entregou. T004 a T008 em sequência (mesmo arquivo). Pode andar em paralelo com a Phase 3.
- **US1 (Phase 3)**: T009 pode começar já. T014 precisa do T001 (medição). É o MVP e é o gate de tudo o que vem depois.
- **US2 (Phase 4)**: precisa do G5 verde (T015), da Phase 2 e das tarefas T037, T045 e T046 da 019. O T024 precisa também das T047 a T052 da 019.
- **US3 (Phase 5)**: precisa do T022. A medição (T026) precisa do T024.
- **US4 (Phase 6)**: precisa do T022. O roteiro (T029) precisa do T024.
- **US5 (Phase 7)**: precisa do T027 (a reduzida existe).
- **US6 (Phase 8)**: T032 precisa do T019; T033 precisa do T022, do T027 e do T032.
- **Polish (Phase 9)**: depois das histórias desejadas. T039 só depois de G4, G5, G6 e do T038.

### User Story Dependencies

- **US1**: independente. Decide se as outras existem.
- **US2**: precisa da US1 aprovada.
- **US3**: precisa da US2 (o publicador). Não depende de US4 a US6.
- **US4**: precisa da US2. Não depende da US3.
- **US5**: precisa da US4 (a reduzida).
- **US6**: precisa da US2 e da US4 (regra do Assistir no publicador). Não depende da US5.

### Within Each User Story

- Os testes vêm antes da implementação de cada parte e devem falhar antes dela.
- Primeiro a cena de dados falsos (T018), depois o dado real (T023).
- O principal escreve o desenho (T004, T022) ou a decisão (T028, T030) antes de o sonnet escrever.
- Mesmo arquivo, mesma sequência: T004 a T008 e T028 mexem em `tiers.rs`; T022, T025, T027, T028, T030 e T033 mexem em `publish.rs`; T010 a T012 mexem em `src/main.rs` da sonda. Nunca dois ao mesmo tempo.
- Tarefa da pessoa fecha a história. Gate vermelho para o que vem depois.

### Parallel Opportunities

- Phase 1: T001 e T002.
- Phase 2 inteira em paralelo com a Phase 3 (arquivos diferentes): por exemplo T004 com T009.
- Phase 4: T016, T017, T018 e T019 em paralelo logo depois do G5.
- Phase 8: T032 em paralelo com as tarefas das fases 5, 6 e 7 (arquivo próprio).
- Phase 9: T035 e T036.
- A T037 da 019 pode andar em paralelo com as fases 1, 2 e 3 daqui (plano, "Entrega em etapas").

---

## Parallel Example: Setup e Foundational com a sonda

```text
Task: "T001 scripts/processador.ps1"
Task: "T002 specs/020-codificar-pela-placa/relatorio.md (esqueleto)"
Task: "T004 app/crates/espia-core/src/tiers.rs (tipos, quem recebe, ativas, ajuste)"
Task: "T009 app/spikes/encode-probe/ (esqueleto a partir da sonda do G1)"
```

## Parallel Example: User Story 2

```text
Task: "T016 app/spikes/encode-probe/src/capture.rs (sonda com captura real)"
Task: "T017 specs/019-app-rust-convite/tasks.md e research.md (o que muda na 019)"
Task: "T018 cenas do diagnóstico em app/crates/espia-ui"
Task: "T019 app/crates/espia-media/src/encode/mod.rs (interface, quadro, codificador falso)"
```

## Parallel Example: User Story 6

```text
Task: "T032 app/crates/espia-media/src/encode/software.rs"
Task: "T027 app/crates/espia-media/src/publish.rs (reduzida e regra do Assistir, da US4)"
```

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Phase 1: `scripts/processador.ps1`, `relatorio.md` e a leitura do app de hoje.
2. Phase 3: sonda, G4 e G5.
3. **Parar e decidir**: com G5 verde, seguir. Com gate vermelho, registrar e voltar à decisão R2 da 019. Este é o resultado que a história 1 promete: saber cedo.

### Incremental Delivery

1. Etapa 1 do plano: Phase 1 e Phase 2 (nada depende de placa).
2. Etapa 2: Phase 3 (G4, depois G5).
3. Etapa 3, só com G5 verde: Phase 4 (codificador, publicador, G6). Depois Phase 5 e Phase 6, que medem e completam o caminho pela placa.
4. Etapa 4: Phase 7 (internet fraca) e Phase 8 (sem placa capaz).
5. Etapa 5: Phase 9, com a conferência do grupo.

Cada história termina num roteiro da pessoa com números no `relatorio.md`.

---

## Divisão por subagentes

**Regra**: o modelo principal decide, escreve a instrução fechada de cada lote e revisa. Um lote é uma execução de um subagente, com arquivos que nenhum outro lote em execução toca. O subagente não roda `git`, `npm run deploy` nem escreve segredo, não toca em `src-tauri/`, no site nem no Worker, e compila com `-j 6`, um build por vez.

| Lote | Tarefas | Executor | Arquivos | Paralelo com |
|---|---|---|---|---|
| A | T001 | sonnet | `scripts/processador.ps1` | B, C, D |
| B | T002 | haiku | `specs/020-codificar-pela-placa/relatorio.md` | A, C, D |
| P1 | T003 | pessoa | `relatorio.md` | C, D (depois de A e B) |
| C | T004 a T008 | principal desenha o T004; sonnet, em sequência | `app/crates/espia-core/src/{tiers.rs,lib.rs}` | A, B, D |
| D | T009 a T012 | haiku copia; sonnet, em sequência; principal revisa o T010 | `app/spikes/encode-probe/` | A, B, C |
| D2 | T013 | haiku | `quickstart.md` | depois de D |
| G | T014, T015 | pessoa e principal | `relatorio.md`, `research.md`, contratos | depois de A, D e D2 |
| E0 | T016 | sonnet; a pessoa roda | `app/spikes/encode-probe/` | E1, E2, E3 (depois de G) |
| E1 | T017 | principal | `specs/019-app-rust-convite/{tasks.md,research.md}` | E2, E3 (depois de G) |
| E2 | T018 | sonnet | `app/crates/espia-ui/` | E1, E3 |
| E3 | T019, T020 | sonnet, em sequência; principal revisa o T020 | `app/crates/espia-media/src/encode/{mod.rs,hardware.rs}`, `Cargo.toml` | E1, E2, E4 |
| E4 | T021 | sonnet | `app/crates/espia-capture/src/{video.rs,output.rs}` | E2, E3 (depois dos T045 e T046 da 019) |
| E5 | T022, T023 | principal desenha o T022; sonnet | `app/crates/espia-media/src/publish.rs`, `app/crates/espia/src/diagnostics.rs` | depois de C, E2 e E3 |
| P2 | T024 | pessoa e principal | `relatorio.md`, `research.md` | depois de E1 a E5 |
| F | T025, T027, T028, T030, T033 | sonnet, em sequência (mesmo `publish.rs`); principal decide T028 e T030 e revisa o T033 | `app/crates/espia-media/src/publish.rs`, `app/crates/espia-core/src/tiers.rs`, `research.md` | H |
| H | T032 | sonnet | `app/crates/espia-media/src/encode/software.rs` | F (antes do T033) |
| P3 | T026, T029, T031, T034 | pessoa | `relatorio.md` | cada uma depois das tarefas da sua história |
| Z | T035, T036, T038, T039 | sonnet (T035), principal (T036, T038), haiku (T039) | `DEFINICOES.md`, documentos da 020, `app/spikes/encode-probe/` | T035 com T036; T039 por último |
| P4 | T037, T040 | pessoa | `relatorio.md` | depois de P3 |

A instrução fechada de cada lote segue o modelo de [019/tasks.md](../019-app-rust-convite/tasks.md) ("Instrução fechada que o principal dá a um lote").

---

## Notes

- Esta feature não muda o site, o Worker, o protocolo nem `src-tauri/`. Quem transmite pelo navegador continua como está.
- A conexão continua direta entre as pessoas, sem servidor de mídia (constituição II, ADR 0001).
- Os números das regras (60%, 90%, 5 s, 15 s, 30 s, 300 ms) são ponto de partida. As histórias 1 e 5 confirmam ou trocam.
- A sonda nunca captura a tela. Captura real só com a pessoa, na janela que ela escolher.
- Fazer commit depois de cada tarefa ou grupo lógico, só quando pedido.

---

## Resumo

Contagem feita sobre as linhas de tarefa deste arquivo.

- **Total**: 40 tarefas (T001 a T040).
- **Por fase**: Phase 1 Setup 3; Phase 2 Foundational 5; Phase 3 US1 7; Phase 4 US2 9; Phase 5 US3 2; Phase 6 US4 3; Phase 7 US5 2; Phase 8 US6 3; Phase 9 Polish 6.
- **Por história**: US1 7; US2 9; US3 2; US4 3; US5 2; US6 3 (total 26; as outras 14 são Setup, Foundational e Polish).
- **Tarefas [P]**: 10 (T001, T002, T009, T016, T017, T018, T019, T032, T035, T036).
- **Por executor, como escrito na linha da tarefa**: `sonnet` 15; `pessoa` 7; `principal desenha, sonnet escreve` 3; `sonnet, com revisão do principal` 3; `pessoa e principal` 3; `principal` 3; `haiku` 3; `principal decide, sonnet escreve` 1; `haiku copia, sonnet adapta` 1; `sonnet, pessoa roda` 1.
