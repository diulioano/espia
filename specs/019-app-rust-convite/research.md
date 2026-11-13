# Research: App nativo em Rust e convite no endereço oficial

**Feature**: [spec.md](./spec.md) · **Plan**: [plan.md](./plan.md) · **Date**: 2026-11-11

Números de custo e de tamanho vêm de [custo-migracao.md](./custo-migracao.md). Tudo o que está marcado "não confirmado" continua assim até o gate ou a tarefa que o resolve (ver o fim deste arquivo).

## Contexto

Decisão de 2026-11-11: o app para Windows vira inteiro nativo em Rust, sem motor de página. O site (Svelte) e o servidor (Worker + Durable Object, TypeScript) ficam como estão.

Isso substitui dois trechos de documentos anteriores:

- a frase "A migração ampla do motor para Rust foi adiada" de `docs/adr/0001-conexao-direta-e-franquia-de-midia.md`;
- a prioridade de 2026-11-05 em `docs/design/transmissao-motor-transporte.md` ("Uma migração ampla para Rust não é o próximo passo escolhido").

O resto do ADR 0001 continua valendo: conexão direta como padrão, sem SFU, sem TURN pago, meta de atraso de até 500 ms na maior parte do tempo, duas transmissões com até sete espectadores.

A constituição 4.0.0 já permite o app nativo e manda escolher no plano a pasta do código, a ferramenta de interface e a biblioteca de ligação de vídeo, registrando em `DEFINICOES.md`. R1, R2 e R3 fazem essa escolha.

## Fatos do código de hoje

- **Convite**: `src/lib/stores/room.svelte.ts:518` faz `inviteUrl: location.origin + roomPath(this.code)`.
  - Cópia em `src/lib/stores/room.svelte.ts:341-346` (`copyInvite`).
  - Cópia automática ao criar sala fixa: `src/routes/HomeRoute.svelte:186` (`created: true`) e `src/lib/stores/room.svelte.ts:384-387`.
  - Texto mostrado: `src/ui/pages/RoomPage.svelte:33`.
  - Mock: `src/mocks/room.ts:308`.
- **O que o navegador guarda por endereço** (`src/lib/prefs.ts`, `src/lib/saved-rooms.ts`):
  - `espia:nickname` (texto);
  - `espia:golive` (JSON das últimas escolhas de transmissão);
  - `espia:volume:<apelido>` (0 a 1);
  - `espia:owner:<código>` (chave de 64 hex);
  - `espia:rooms` (lista de `{code, name, lastUsed}`, no máximo 5, `SAVED_ROOMS_MAX`);
  - `espia:session:<código>` (`{peerId, resumeToken, at}`, vale 24 h);
  - `espia:appinvite` (aviso de baixar o app dispensado).
- **Servidor**: `sameOrigin` em `src/worker/index.ts:137-148` exige cabeçalho `Origin` com o mesmo host do pedido, em `POST /api/rooms`, `POST /api/rooms/status`, ações de dono e `GET /ws/:code`. Não há host fixo no Worker. A mesma sala responde nos dois endereços.
- **Protocolo**: `src/shared/protocol.ts` (6 mensagens do cliente, 15 do servidor, códigos de fechamento, `PING` exato) e `src/shared/media.ts` (`MEDIA_PROTOCOL = 2`, `directIceServers` só STUN, `nextBroadcastOrder`). O servidor fecha com 4426 quem não manda `mediaProtocol: 2`.
- **Ligação entre pessoas**:
  - uma `RTCPeerConnection` por par (`src/lib/mesh.ts:148`, `src/lib/peer.ts:88`);
  - negociação perfeita (educado = `myId < id`);
  - canal de dados `ctl` com `ViewMessage` versão 2 (`src/lib/peer.ts`, `encodeView`/`decodeView`: modos `full`, `thumb`, `hidden`, `off`);
  - dois canais de envio reaproveitados (vídeo e som);
  - parâmetros por ligação (`sendParams` em `src/lib/quality.ts:88-93`: miniatura 360p 15 fps 500 kbps; `hidden` e `off` param o envio);
  - preferência por H.264 (`src/lib/peer.ts:403`);
  - candidatos de retransmissão descartados (`src/lib/peer.ts:204`);
  - sem `ViewMessage` válida, nada é enviado.
- **Qualidade**: `src/lib/quality.ts:49-62` (1 a 4 espectadores cheios = escolhida; 5 a 6 = teto 4 Mbps e 720p60 ou 1080p30; 7 ou mais = 720p30). Ajuste por processador em `src/lib/cpu-governor.ts`.
- **App de antes** (`src-tauri/src`, 3.252 linhas): `capture/{mod,video,audio,sources,thumbs,procs,discord,publish}.rs`, `tray.rs`, `engine.rs`, `links.rs`, `share_bar.rs`, `lib.rs`.
  - Identificador `dev.espia.app`, produto "Espia", instalador NSIS por usuário, textos em `src-tauri/nsis/`.
  - Preferência da bandeja em `tray.json`, na pasta de configuração do app (`%APPDATA%\dev.espia.app\`).
  - Dados do motor de página em `%LOCALAPPDATA%\dev.espia.app\EBWebView\`.
- **Instalador publicado**: 0.1.10, montado em 2026-11-05, abre o endereço antigo. `public/download/Espia-setup.exe` é esse arquivo.
- **Feature 015** (atualização): tem plano e contrato (`specs/015-atualizar-app/contracts/update.md`) e 0 de 36 tarefas feitas.
- **Testes de hoje**: Vitest em `test/lib` (lógica pura) e `test/worker`. `cargo test` em `src-tauri`. CI em `.github/workflows/ci.yml`.

## R1 — Pasta e forma do código

**Decisão**: workspace Rust novo em `app/`, com sete crates. `src-tauri/` fica congelado e sem mudança até a troca.

```text
app/
├── Cargo.toml                 # workspace
├── crates/
│   ├── espia-core/            # lógica pura: protocolo, sala (redutor), qualidade, view-models, dados do aparelho. Sem rede, mídia, tela
│   ├── espia-net/             # sinalização (WebSocket) e API HTTP
│   ├── espia-media/           # ligação de vídeo: par, malha, envio, recepção, som
│   ├── espia-capture/         # captura no Windows: telas, janelas, miniaturas, som por janela, câmera, entradas de som
│   ├── espia-shell/           # bandeja, instância única, link espia://, janela, avisos do Windows, arquivos do aparelho, atualização
│   ├── espia-ui/              # telas em Slint; depende só de espia-core (view-models). Cenas de dados falsos
│   └── espia/                 # binário Espia.exe: junta tudo
├── fixtures/protocol/         # JSON do protocolo conferido pelo site e pelo app
├── packaging/                 # instalador
└── spikes/                    # provas dos gates G1, G2, G3 (apagadas no fim)
```

**Motivo**: a regra IV da constituição (tela não fala com rede, captura nem mídia) passa a ser garantida pelo grafo de dependências, porque `espia-ui` só enxerga `espia-core`. Também deixa subagentes trabalharem em paralelo sem tocar no mesmo arquivo.

**Alternativas consideradas**: uma crate só (descartada pelos dois motivos acima).

## R2 — Ligação de vídeo: crate `libwebrtc` da LiveKit (0.4.x)

**Decisão**: usar a crate `libwebrtc` da LiveKit (0.4.x), que embrulha o mesmo motor do Chrome.

**Motivo**: recepção pronta (fila de recepção, sincronia de som e imagem, controle de banda), a mesma negociação do navegador e uma codificação por ligação como hoje. Com isso a regra do Assistir e a miniatura por espectador continuam iguais ao site, sem inventar mecanismo novo.

**Alternativas consideradas**:
- `str0m` 0.25: recepção, sincronia e retransmissão ficariam por nossa conta; há registro de falha de H.264 com Safari em níveis altos.
- `webrtc-rs`: sem controle de banda completo.

**Risco / não confirmado**:
- codificação pela placa de vídeo no Windows na versão oficial;
- montagem no Windows;
- tamanho do executável;
- se a crate expõe tudo de que precisamos: parâmetros por ligação, preferência de codec, canal de dados, estatísticas, processamento de voz.

**Licença** (conferida em 2026-11-11, T007):
- A crate `libwebrtc` é Apache-2.0 em todas as versões publicadas, inclusive a 0.4.0 (fonte: campo `license` em `https://crates.io/api/v1/crates/libwebrtc`; repositório `livekit/rust-sdks`).
- O código do WebRTC que ela embrulha é BSD de 3 cláusulas (fonte: `LICENSE` em `https://webrtc.googlesource.com/src`). Distribuir em forma binária exige levar junto o aviso de direitos autorais, a lista de condições e a isenção de garantia, e não usar o nome do Google para promover o app.
- A crate não compila o WebRTC: `webrtc-sys` baixa um binário pronto de `github.com/livekit/rust-sdks/releases` (fonte: `webrtc-sys/build/src/lib.rs`, que cita um `LICENSE.md` dentro do pacote baixado).
- **O que o projeto precisa cumprir**: um arquivo de licenças de terceiros dentro do instalador, com o texto da Apache-2.0 e o aviso da LiveKit, e o texto BSD do WebRTC com os avisos das bibliotecas que vêm dentro dele. Nenhum termo lido impede distribuir `Espia.exe` ao grupo.
- **Não confirmado**: o conteúdo do `LICENSE.md` do binário baixado (quais bibliotecas de terceiros ele lista) e se o codificador H.264 embutido traz obrigação de patente para um app gratuito. O primeiro se confere no G1, ao baixar o binário; o arquivo de licenças do instalador (T093) parte dele.

**Gate / plano B**: o G1 decide.
- Plano B: versão modificada com codificador do Windows (referência: `RafaelrainBR/rust-sdks`, PR 3, não confirmado).
- Plano C: `str0m` com codificador próprio, o que exige replanejar as fases de mídia.

## R3 — Interface: Slint 1.17

**Decisão**: Slint 1.17. Vídeo na tela: a primeira versão usa imagem RGBA por quadro. Textura direto na placa de vídeo só se o G2 mostrar custo alto.

**Motivo**: estável (1.x), telas declarativas que aguentam o design Espia (fontes próprias, cantos, sombras, animação), bandeja embutida e memória baixa em relato não confirmado.

**Alternativas consideradas**:
- `egui`: visual utilitário; 72 a 189 MB medidos por terceiros.
- `iced`: 0.x; 133 a 152 MB.
- GPUI: sem publicação oficial utilizável.

**Risco / não confirmado**:
- Memória baixa da Slint: relato não confirmado.

**Licença** (conferida em 2026-11-11, T007):
- A crate `slint` 1.17.0 sai sob três licenças à escolha: `GPL-3.0-only`, `LicenseRef-Slint-Royalty-free-2.0` ou `LicenseRef-Slint-Software-3.0` (comercial). Fonte: campo `license` em `https://crates.io/api/v1/crates/slint/1.17.0`. A mais nova publicada hoje é a 1.18.1, com o mesmo campo.
- O Espia usa a gratuita, Slint Royalty-free 2.0 (fonte: `LICENSES/LicenseRef-Slint-Royalty-free-2.0.md` no repositório `slint-ui/slint`, ramo principal). Ela cobre app de computador, de celular e de web, e não cobre sistema embarcado. O Espia é app de computador.
- **O que o projeto precisa cumprir**, uma das duas atribuições: (a) mostrar o componente `AboutSlint` numa tela "Sobre" alcançável pelo menu principal (sem tela "Sobre", na tela de abertura); ou (b) pôr o selo da Slint numa página pública, de preferência a de download do app. Além disso: não distribuir a Slint sozinha, não expor a API da Slint a terceiros e não tirar os avisos de licença do código dela.
- **Decisão**: cumprir pela (b), com o selo na página de download do site, e também citar a Slint na tela "Sobre" do app. O Espia não tem menu principal, então a (b) é a que se encaixa sem mexer no desenho das telas. Entra junto com o arquivo de licenças do instalador (T093).
- Nenhum termo lido impede distribuir `Espia.exe` ao grupo. O plano B `iced` não é necessário por licença.
- **Não confirmado**: se o texto da Royalty-free 2.0 no ramo principal é idêntico ao que acompanha a 1.17.0 (o nome e a versão da licença são os mesmos), e os requisitos visuais do selo (tamanho, link).

**Gate / plano B**: o G2 decide. Plano B: `iced`.

## R4 — Lógica da sala como redutor puro

**Decisão**: `espia-core::room` recebe eventos (mensagens do servidor, eventos de ligação, ações da pessoa, relógio) e devolve estado e efeitos. É o porte de `src/lib/stores/room.svelte.ts`, `room-view.ts`, `watch-state.ts`, `quality.ts`, `cpu-governor.ts`, `live-label.ts`, `audio-source.ts`, `devices.ts` e `join-view.ts`. O binário executa os efeitos. Os testes de `test/lib` dessas regras são portados para `cargo test`.

**Motivo**: lógica pura é testável sem UI, rede ou mídia (constituição IV), e é a parte que porta com menos risco.

**Alternativas consideradas**: nenhuma registrada.

## R5 — Protocolo espelhado e conferido dos dois lados

**Decisão**: tipos `serde` em `espia-core::protocol` espelham `src/shared/protocol.ts` e `src/shared/media.ts`. Exemplos JSON em `app/fixtures/protocol/` são lidos por um teste Vitest (`test/shared/protocol-fixtures.test.ts`, com os validadores de `src/worker/validate.ts`) e por `cargo test`. O app manda `Origin` igual à origem do endereço base em uso (`https://espia.diuliano.com`) e `mediaProtocol: 2`. A variável `ESPIA_SERVER` (exemplo `http://127.0.0.1:8787`) troca o endereço base só em build de depuração; release usa sempre o endereço oficial. O app nunca manda `turn-refresh` e ignora entradas de `ice-servers` que não são STUN. Nenhuma mudança no Worker.

**Motivo**: o app nativo não atualiza junto com o site (constituição, Fluxo). As fixtures pegam desvio de protocolo nos dois lados antes de chegar ao grupo.

**Alternativas consideradas**: nenhuma registrada.

## R6 — Rede

**Decisão**: `tokio` como motor assíncrono, `tokio-tungstenite` para o WebSocket e `reqwest` com `rustls` para a API. Keep-alive com o `PING` exato, reconexão e retomada como em `src/lib/signaling.ts`.

**Motivo**: o comportamento de sinalização do app tem de ser o do site, que já é conferido pelo Worker.

**Alternativas consideradas**: nenhuma registrada.

## R7 — Captura

**Decisão**: copiar `src-tauri/src/capture/{sources,thumbs,video,audio,procs,discord}.rs` para `app/crates/espia-capture/src/`.
- Sai tudo o que entrega à página: `publish.rs`, buffers compartilhados, `PostWebMessageAsJson` e comandos Tauri.
- Quadros NV12 e som PCM passam a sair por canal para `espia-media`.
- Câmera e placa de captura: crate `nokhwa` (Media Foundation).
- Entradas de som: `cpal`.
- O `src-tauri` não muda.

**Motivo**: a captura de hoje já é Rust e funciona; o que muda é o destino dos quadros.

**Alternativas consideradas**: nenhuma registrada.

## R8 — Som

**Decisão**:
- Envio: 48 kHz, estéreo, blocos de 10 ms para a fonte de som da ligação.
- Recepção: o som de cada pessoa vai para um misturador com volume por pessoa e sai por `cpal`.
- Sincronia com a imagem é item do G1 (SC-009).
- "Som de voz" usa o processamento de voz do motor se a crate expuser.

**Motivo**: manter o volume por pessoa do site e a sincronia com a imagem.

**Alternativas consideradas**: nenhuma registrada.

**Risco / não confirmado**: se a crate expõe o processamento de voz. Sem isso, vale só "Som de jogo" e o seletor diz por quê.

**Gate / plano B**: o G1 mede a sincronia (item g) e mostra o que a crate expõe.

## R9 — Dados do aparelho no app nativo

**Decisão**: pasta `%APPDATA%\dev.espia.app\` (a mesma do app de antes). Arquivos:
- `device.json` (modelo em [data-model.md](./data-model.md));
- `tray.json` (formato de hoje, reaproveitado);
- `window.json` (posição e tamanho);
- `resume.json` (volta à sala depois de atualizar).

A escrita é atômica (arquivo temporário e renomear). Nada disso vai ao servidor.

**Motivo**: a mesma pasta permite instalar por cima do app de antes e reaproveitar `tray.json`. Atende o princípio III da constituição (dados do aparelho não vão ao servidor).

**Alternativas consideradas**: nenhuma registrada.

## R10 — Trazer os dados do app de antes

**Decisão**: na primeira abertura, o app nativo lê o `localStorage` do motor de página em `%LOCALAPPDATA%\dev.espia.app\EBWebView\Default\Local Storage\leveldb`, a partir de uma cópia da pasta, com a crate `rusty-leveldb`.
- Origens lidas: `https://espia.espia.workers.dev` e `https://espia.diuliano.com`.
- Chaves: as `espia:*`.
- Junta com as regras do R12 e marca `importedFromWebview: true`.
- Se a importação falha, o app abre com o que tem, sem mostrar nada, e tenta de novo a cada abertura enquanto `importedFromWebview` for `false` e a pasta existir. Se a pasta não existe, marca `importedFromWebview: true`.
- Não apaga a pasta, para o app de antes continuar funcionando se a pessoa voltar.

**Motivo**: sem isso, quem instala o app nativo por cima chega com Suas salas vazia e sem as chaves de dono (FR-004, FR-015).

**Alternativas consideradas**: exportação manual pelo site (plano B abaixo).

**Risco / não confirmado**: o formato das chaves do `localStorage` do Chromium não foi confirmado.

**Gate**: G3 passou em 2026-11-11 (T014): salas fixas, 4 chaves de dono e apelido lidos de uma cópia do perfil real. O formato das chaves do Chromium descrito no `LEIA-ME.md` da sonda vale para o WebView2 instalado. O plano B (exportação manual pelo site) não é necessário.

## R11 — Convite

**Decisão**: módulo novo `src/lib/site-origin.ts` com `OFFICIAL_ORIGIN = 'https://espia.diuliano.com'`, `LEGACY_HOSTS = ['espia.espia.workers.dev']` e `inviteOrigin(location)`.
- Host oficial ou antigo devolve o oficial.
- Qualquer outro host (desenvolvimento, teste) devolve `location.origin`.
- `src/lib/stores/room.svelte.ts:518` e `src/mocks/room.ts:308` passam a usar `inviteOrigin`.

**Motivo**: o convite é montado num lugar só. Vale na hora para os apps já instalados, porque eles carregam o site. O host de desenvolvimento continua funcionando (FR-007).

**Alternativas consideradas**: nenhuma registrada.

## R12 — Ida do endereço antigo para o oficial (só no navegador)

**Decisão**:
- No host antigo, fora do app de antes (`'__TAURI_INTERNALS__' in window` é falso) e sem `?mock`, antes de montar a aplicação o site empacota as chaves `espia:*` e navega com `location.replace` para `https://espia.diuliano.com` + caminho + busca + `#mv=<base64url do JSON>`.
- JSON: `{ "v": 1, "items": { "<chave>": "<valor cru do localStorage>" } }`. Base64url sem preenchimento `=`. Teto de 16.384 caracteres do texto codificado. Se passar, descarta nesta ordem: `espia:session:*`, `espia:volume:*`, `espia:golive`, `espia:appinvite`; `espia:rooms`, `espia:owner:*` e `espia:nickname` sempre vão. Se ainda passar, a ida acontece sem `#mv=` e nada é apagado no endereço antigo.
- No oficial, `src/main.ts` importa antes de montar a aplicação e limpa o fragmento com `history.replaceState`.
- Junção: só acrescenta, nunca troca valor que já existe no oficial (`espia:nickname`, `espia:golive`, `espia:volume:*`, `espia:owner:*`, `espia:session:*`, `espia:appinvite`).
  - `espia:rooms`: união por código, ordena por `lastUsed` do mais recente para o mais antigo, corta em 5. Com o mesmo código dos dois lados, fica a entrada de maior `lastUsed`, com o nome dela.
  - Chave de dono nunca é descartada, mesmo de sala fora da lista.
- Validação de cada item: código de sala no formato `xxx-xxx` de `src/lib/room-id.ts`, chave de dono com 64 hex, nome de sala até 40, apelido até 24, volume entre 0 e 1. Item inválido é ignorado em silêncio.
- É idempotente: repetir não duplica nem apaga.
- O fragmento não vai ao servidor (FR-005). O servidor não redireciona nada, porque os apps instalados dependem do endereço antigo (FR-006).
- Regras puras em `src/lib/origin-move.ts` (empacotar, desempacotar, juntar), com teste em `test/lib/origin-move.test.ts`.

**Motivo**: `localStorage` é separado por endereço; sem levar os dados, a pessoa chega ao oficial com a lista vazia e sem poder apagar as salas que criou (FR-004). Fragmento de URL não é enviado ao servidor, então as chaves de dono não passam por ele (FR-005, constituição III).

**Alternativas consideradas**: redirecionar no servidor. Descartada: os apps já instalados só aceitam o endereço antigo e ficariam presos na tela de carregamento (ver [custo-migracao.md](./custo-migracao.md)).

## R13 — App de antes

**Decisão**: não publicar instalador novo dele. `public/download/Espia-setup.exe` continua sendo o de 2026-11-05 até o app nativo passar nas histórias 3, 4 e 5. Na troca, `Espia-setup.exe` vira o nativo e o de antes fica em `public/download/Espia-setup-0.1.10.exe`.

**Motivo**: um app de antes apontando para o endereço oficial perderia Suas salas dentro do app, e ele está congelado.

**Alternativas consideradas**: instalador novo do app de antes com o endereço oficial (descartada pelo motivo acima).

## R14 — Atualização do app nativo

**Decisão**:
- `GET https://espia.diuliano.com/download/latest.json`, no formato do contrato da 015 adaptado: `version`, `notes`, `pub_date`, `minimum`, `url`, `signature`.
- Instalador assinado com minisign e conferido com a crate `minisign-verify` antes de instalar. Chave pública embutida no executável (não é segredo). Chave privada fora do repositório, em `%USERPROFILE%\.espia\update.key`, gerada e guardada pela pessoa.
- O app consulta `latest.json` na abertura e a cada 6 horas enquanto está aberto.
- Aviso "Tem versão nova" com "Atualizar agora" e "Depois". "Depois" fecha o aviso até a próxima abertura do app, quando ele volta (não há memória de versão pulada). Abaixo de `minimum`, o aviso não tem "Depois". Fechamento 4426 do servidor mostra "Atualize o Espia" com o mesmo botão.
- Nunca baixa nem instala durante uma transmissão própria. Depois de instalar, reabre e volta para a sala (`resume.json`, válido por 5 minutos e apagado ao ser lido, como na 015).
- A spec 015 passa a ser referência histórica. O contrato novo é `contracts/atualizacao.md`.

**Motivo**: o app nativo deixa de receber as mudanças do site na hora (spec, Assumptions); a constituição exige origem verificada e proíbe interromper transmissão (VI e Restrições técnicas).

**Alternativas consideradas**: nenhuma registrada além de partir do contrato da 015.

## R15 — Instalador

**Decisão**: NSIS por usuário, feito com `cargo-packager`. Mesmo identificador `dev.espia.app` e mesmo nome "Espia", para instalar por cima do app de antes. Registra `espia://`. Textos em português reaproveitados de `src-tauri/nsis/`. Versão do app nativo: 0.2.0.

**Motivo**: instalar por cima preserva os dados do aparelho e a preferência da bandeja (FR-015).

**Alternativas consideradas**: script NSIS próprio em `app/packaging/` (plano B).

**Risco / não confirmado**: não confirmado que o `cargo-packager` cobre tudo.

**Gate / plano B**: a tarefa T093 (`app/packaging/`) resolve. Plano B: script NSIS próprio em `app/packaging/`.

## R16 — Casca do Windows

**Decisão**: bandeja pela crate `tray-icon`, instância única (mutex `Local\dev.espia.app.instance` e named pipe `\.\pipe\dev.espia.app.link` para repassar o link), `espia://`, avisos do Windows e posição da janela. Sem barra "está compartilhando", porque não há motor de página. Sem regra de firewall no instalador (instalação por usuário não tem permissão); o G1 anota se o Windows mostra o aviso.

**Motivo**: manter o que o app faz hoje (FR-012) com as peças que sobram sem o motor de página.

**Alternativas consideradas**: a bandeja embutida da Slint 1.17.

**Gate / plano B**: o G1 (item j) anota o aviso de firewall.

## R17 — Dados falsos no app

**Decisão**: `Espia.exe --mock <tela> --scene <cena>`; `--mock` sozinho abre o índice.
- Telas: `home`, `join`, `room`, `notfound` e a nova `offline` (cena `default`). O app é só para desktop: a tela `mobile` e toda cena marcada `device: 'mobile'` ficam só no site.
- São 69 cenas portadas (home 13, join 5, room 47, notfound 4, contando os seis `watch-*` gerados), mais 3 novas: `home`/`update-available`, `home`/`update-minimum` e `offline`/`default`.
- As cenas são portadas de `src/mocks/{home,join,room,notfound}.ts` para `app/crates/espia-ui/src/mocks/`.
- Um teste do app lê esses arquivos do site e confere que todo `id` de cena não marcado `device: 'mobile'` existe no app.

**Motivo**: constituição V; o teste de paridade impede que as duas interfaces se afastem.

**Alternativas consideradas**: nenhuma registrada.

## R18 — Design

**Decisão**: `app/crates/espia-ui/build.rs` lê `src/ui/tokens.css` e gera os valores para a Slint. Fontes Caprasimo e Figtree embutidas a partir dos pacotes `@fontsource` já instalados. Ícones Lucide como SVG. Textos copiados do site.

**Motivo**: `src/ui/tokens.css` continua a fonte única do design (constituição IV).

**Alternativas consideradas**: nenhuma registrada.

## R19 — Medição

**Decisão**: `scripts/memoria.ps1` ganha um modo para o app nativo (um processo só). A linha de base do app de hoje é medida antes (T008) e registrada em `specs/019-app-rust-convite/relatorio.md`.

**Motivo**: SC-004 a SC-006 comparam com uma medição nova do app de hoje, na mesma máquina (spec, Assumptions).

**Alternativas consideradas**: nenhuma registrada.

## R20 — Qualidade por espectador e Assistir

**Decisão**: igual ao site. `ViewMessage` versão 2 pelo canal `ctl`, parâmetros por ligação conforme `sendParams`, nada enviado sem mensagem válida. H.264 preferido. Só STUN, candidatos de retransmissão descartados.

**Motivo**: a regra do Assistir (FR-021) e a miniatura por espectador continuam iguais, e o app interopera com o site e com o app de antes (FR-014).

**Alternativas consideradas**: nenhuma registrada. Uma codificação única para todos os espectadores perderia a qualidade por espectador (ver [custo-migracao.md](./custo-migracao.md), opção A).

**Gate / plano B**: o G1 (item e) confere que os parâmetros por ligação existem na crate e que as estatísticas de envio trazem `qualityLimitationReason` (o sinal de processador no limite, o mesmo campo que o site lê em `SendStats.limitation`, `src/lib/peer.ts`).

## R21 — Documentos

**Decisão**: ADR novo `docs/adr/0002-app-nativo-em-rust.md`. `DEFINICOES.md`, `README.md`, `.github/workflows/release.yml` e a pasta do app na constituição são atualizados nas tarefas finais.

**Motivo**: a constituição 4.0.0 manda registrar as escolhas de pasta, ferramenta de interface e ligação de vídeo em `DEFINICOES.md`.

**Alternativas consideradas**: nenhuma registrada.

## O que ficou sem confirmação

| Item | Onde | Resolve |
|------|------|---------|
| Codificação pela placa de vídeo no Windows na versão oficial da `libwebrtc` | R2 | G1 (item f), T009, T010 |
| Montagem da `libwebrtc` no Windows | R2 | G1 (item a) |
| Tamanho do executável | R2 | G1 (item i registra a memória e o tamanho do executável) e T090 |
| A crate expõe parâmetros por ligação, preferência de codec, canal de dados, estatísticas e processamento de voz | R2, R8 | G1 (itens d e e) |
| Plano B com versão modificada (`RafaelrainBR/rust-sdks`, PR 3) | R2 | só se o G1 falhar; T010 |
| Conteúdo do `LICENSE.md` do binário do WebRTC baixado pela `webrtc-sys`; obrigação de patente do H.264 embutido | R2 | G1 (ao baixar o binário), T093 |
| Memória baixa da Slint | R3 | G2 (memória em até 150 MB), T011, T012 |
| Texto da Slint Royalty-free 2.0 que acompanha a 1.17.0 igual ao do ramo principal; requisitos visuais do selo | R3 | T093 |
| "Som de voz" com processamento de voz do motor | R8 | G1; sem isso, só "Som de jogo" |
| Formato das chaves do `localStorage` do Chromium | R10 | G3, T013, T014 |
| `cargo-packager` cobre tudo (NSIS por usuário, `espia://`, instalar por cima) | R15 | T093 |
| Aviso de firewall do Windows na primeira transmissão | R16 | G1 (item j) |
| Quanto do peso de transmitir vem da captura e quanto vem da página; codificador usado pelo motor de hoje | custo-migracao.md | G0, T008 |
