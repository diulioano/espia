# Implementation Plan: App nativo em Rust e convite no endereço oficial

**Branch**: `019-app-rust-convite` (trabalho direto na `main`, como nas features anteriores) | **Date**: 2026-11-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/019-app-rust-convite/spec.md`

## Summary

1. **Convite e endereço no site (R11, R12)**: o convite sai sempre com `espia.diuliano.com`, por `inviteOrigin` em `src/lib/site-origin.ts`.
   - No navegador, quem abre o endereço antigo é levado ao oficial com os dados do aparelho no fragmento `#mv=`, que não vai ao servidor.
   - O servidor não redireciona nada, porque os apps já instalados dependem do endereço antigo.
2. **Três provas antes de construir (G1, G2, G3)**: ligação de vídeo (`libwebrtc`), interface (Slint) e leitura dos dados do app de antes. Mais a linha de base G0, medida no app de hoje.
   - Gate vermelho para tudo que vem depois na mesma trilha, e o plano B da decisão entra no lugar.
3. **App nativo em `app/` (R1 a R8, R16 a R18, R20)**: workspace Rust com sete crates.
   - `espia-ui` só enxerga `espia-core`, então a regra IV da constituição vale pelo grafo de dependências.
   - Ligação de vídeo com `libwebrtc`, interface em Slint 1.17, captura reaproveitada de `src-tauri/`.
   - Protocolo e Worker sem mudança; o app espelha `src/shared/protocol.ts` e é conferido por fixtures.
4. **Dados do aparelho e importação (R9, R10)**: arquivos JSON em `%APPDATA%\dev.espia.app\`, e importação do `localStorage` do app de antes na primeira abertura.
5. **Atualização e instalador (R13 a R15)**: `latest.json` com assinatura minisign, aviso com "Atualizar agora", instalador NSIS por cima do app de antes, versão 0.2.0. O app de antes não ganha instalador novo.
6. **Entrega em etapas**: primeiro o convite e a ida de endereço (site, no mesmo dia); o app nativo só substitui o de antes quando as histórias 3, 4 e 5 passam.

## Technical Context

**Language/Version**: Rust 1.90, edição 2024 (app nativo). TypeScript 6 e Svelte 5 no site, só nas histórias 1 e 2.

**Primary Dependencies**:
- App nativo: `slint` 1.17, `libwebrtc` 0.4.x, `tokio`, `tokio-tungstenite`, `reqwest` (rustls), `serde`/`serde_json`, `windows` 0.62, `nokhwa`, `cpal`, `rusty-leveldb`, `minisign-verify`, `tray-icon`.
- Ferramenta: `cargo-packager`.
- Site: nenhuma dependência nova.

**Storage**: arquivos JSON em `%APPDATA%\dev.espia.app\` no app nativo; `localStorage` no site (ver [data-model.md](./data-model.md)).

**Testing**:
- `cargo test` nas crates puras, com o porte dos testes de `test/lib`.
- Vitest para `site-origin`, `origin-move` e as fixtures do protocolo.
- Roteiro manual em [quickstart.md](./quickstart.md).

**Target Platform**: Windows 10 e 11, x64 (app nativo). Site como hoje.

**Project Type**: SPA + Worker + app desktop nativo, no mesmo repositório.

**Performance Goals**: SC-004 a SC-014 da spec.

**Constraints**:
- sem mudança no protocolo nem no Worker;
- o app de antes continua funcionando;
- nenhum segredo no executável.

**Scale/Scope**:
- 14 a 22 mil linhas de Rust novas (estimativa de [custo-migracao.md](./custo-migracao.md));
- 69 cenas de dados falsos portadas, mais 3 novas desta feature (aviso de versão com e sem "Depois" e a tela sem conexão);
- 100 a 200 linhas no site.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Situação |
|-----------|----------|
| I. Simplicidade | ✅ App nativo com os mesmos dois passos, Assistir explícito, mesmos textos. O site continua sem instalar nada. |
| II. Sem servidor de mídia | ✅ Malha direta mantida, só STUN. O app interopera com os navegadores alvo (G1). |
| III. Nada sobre as pessoas | ✅ Dados do aparelho só em `%APPDATA%\dev.espia.app\`. A ida de endereço usa fragmento, que não vai ao servidor. |
| IV. Camada visual separada | ✅ `espia-ui` depende só de `espia-core`. Tokens gerados de `src/ui/tokens.css`. |
| V. Telas mockáveis | ✅ `--mock <tela> --scene <cena>` com as mesmas cenas do site, conferidas por teste. |
| VI. Segredos fora do cliente | ✅ Nenhum segredo no executável. Atualização conferida por assinatura; chave privada fora do repositório. |
| Restrições técnicas | ✅ Acesso do app limitado ao que transmitir exige. A importação lê `%LOCALAPPDATA%\dev.espia.app\EBWebView\`, que é pasta de dados do próprio app (mesmo identificador); conferido na T105. App de antes congelado, sem comando novo na ponte. Convite só no endereço oficial. Pendência da constituição resolvida aqui: pasta `app/`, Slint, `libwebrtc`. |
| Fluxo | ✅ Mock antes da lógica em cada tela. Gate do app: `cargo check` e `cargo test` em `app/`. Protocolo sem mudança. |

**Re-check pós-design**: ✅ sem violações. A escolha da ferramenta de interface e da ligação de vídeo continua sujeita aos gates G1 e G2; se um plano B for usado, a decisão é registrada em `DEFINICOES.md` e em [research.md](./research.md).

## Project Structure

### Documentation (this feature)

```text
specs/019-app-rust-convite/
├── plan.md                    # este arquivo
├── research.md                # R1–R21
├── data-model.md              # device.json, #mv=, latest.json, resume.json, tray.json, window.json, estado da sala, ViewMessage
├── quickstart.md              # gates, mocks, roteiro por história
├── relatorio.md               # criado na T008 (G0), recebe G1–G3 e as medições
├── custo-migracao.md          # levantamento que originou a decisão
├── contracts/
│   ├── convite-e-endereco.md  # inviteOrigin, host → origem, #mv=, junção e validação
│   ├── app-nativo.md          # linha de comando, instância única, arquivos, bandeja, acessos
│   ├── sinalizacao-nativa.md  # uso do protocolo existente sem mudá-lo
│   ├── atualizacao.md         # latest.json, assinatura, fluxo e regras
│   └── dados-falsos.md        # --mock, telas, cenas e regra de paridade
├── checklists/
│   ├── requirements.md
│   └── paridade.md            # itens do FR-012, um por linha (T077)
└── tasks.md                   # /speckit-tasks
```

### Source Code (repository root)

```text
app/                                      # novo: workspace do app nativo
├── Cargo.toml                            # workspace
├── crates/
│   ├── espia-core/                       # lógica pura: protocolo, sala (redutor), qualidade, view-models, dados do aparelho. Sem rede, mídia, tela
│   ├── espia-net/                        # sinalização (WebSocket) e API HTTP
│   ├── espia-media/                      # ligação de vídeo: par, malha, envio, recepção, som
│   ├── espia-capture/                    # captura no Windows: telas, janelas, miniaturas, som por janela, câmera, entradas de som
│   ├── espia-shell/                      # bandeja, instância única, link espia://, janela, avisos do Windows, arquivos do aparelho, atualização
│   ├── espia-ui/                         # telas em Slint; depende só de espia-core (view-models). Cenas de dados falsos
│   └── espia/                            # binário Espia.exe: junta tudo
├── fixtures/protocol/                    # JSON do protocolo conferido pelo site e pelo app
├── packaging/                            # instalador
└── spikes/                               # provas dos gates G1, G2, G3 (apagadas no fim)

src/                                      # site: só as histórias 1 e 2
├── lib/
│   ├── site-origin.ts                    # novo: OFFICIAL_ORIGIN, LEGACY_HOSTS, inviteOrigin
│   ├── origin-move.ts                    # novo: empacotar, desempacotar, juntar (#mv=)
│   └── stores/room.svelte.ts             # linha 518 usa inviteOrigin
├── main.ts                               # sair do endereço antigo e importar no oficial, antes de montar
└── mocks/room.ts                         # linha 308 usa inviteOrigin

test/
├── lib/
│   ├── site-origin.test.ts               # novo
│   └── origin-move.test.ts               # novo
└── shared/
    └── protocol-fixtures.test.ts         # novo: fixtures contra src/worker/validate.ts

scripts/
├── memoria.ps1                           # modo para o app nativo (um processo só)
└── release-app.mjs                       # novo: montar, assinar, copiar, escrever latest.json

public/download/                          # Espia-setup.exe, Espia-setup-0.1.10.exe, latest.json
src-tauri/                                # app de antes: congelado, sem mudança até a troca
```

**Structure Decision**: o app nativo vive em `app/`, fora de `src/` e de `src-tauri/`, como a constituição 4.0.0 pede ("pasta própria"). A separação em sete crates faz o grafo de dependências garantir a regra IV (ver R1 em [research.md](./research.md)). O site muda pouco, só o necessário para o convite, a ida de endereço e as fixtures.

## Gates

Provas antes de construir o resto. Se um gate falhar, parar, registrar em `relatorio.md` e replanejar as fases afetadas com o plano B da decisão. Não seguir adiante com gate vermelho.

| Gate | Onde | Passa se |
|------|------|----------|
| **G0, linha de base** (pessoa) | app de hoje, mesma máquina e jogo | Registrar em `relatorio.md`: FPS do jogo sem transmitir e transmitindo 1080p60 para 3 pessoas; processador do app; memória numa sala e na bandeja; qual codificador o motor usa (o site já mostra em Diagnóstico de mídia). |
| **G1, ligação** | `app/spikes/webrtc-probe/` | (a) compila no Windows com `cargo build --release`; (b) entra numa sala do site local e o Chrome recebe vídeo 1080p60 H.264 e som; (c) recebe vídeo e som do Chrome; (d) troca `ViewMessage` com o site pelo canal `ctl`; (e) muda parâmetros por ligação (bitrate, fps, redução de resolução, ativo) e lê `qualityLimitationReason` nas estatísticas de envio; (f) usa a placa de vídeo para codificar, ou o processador do processo fica em até metade do G0; (g) som e imagem não se afastam mais de 80 ms em 10 minutos; (h) Safari no iPhone recebe; (i) memória do processo e tamanho do executável anotados; (j) anotado se o Windows mostra aviso de firewall. |
| **G2, interface** | `app/spikes/ui-probe/` | A tela da sala com dados falsos fica igual ao site na comparação lado a lado; mostra um vídeo 1080p60 e duas miniaturas sem engasgar; a memória privada do processo, com o motor do G1 carregado, fica em até 150 MB. |
| **G3, dados do app de antes** | `app/spikes/webview-data/` | Lê salas fixas, chaves de dono e apelido de uma cópia do perfil real do app instalado. |

Planos B: G1 decide R2 (versão modificada da `libwebrtc`, depois `str0m`); G2 decide R3 (`iced`); G3 decide R10 (exportação manual pelo site, que exige replanejar).

## Entrega em etapas

- **MVP**: Phase 3 (história 1) sozinha, publicável no mesmo dia. A parte de site da história 2 (T080 a T083) pode sair junto.
- O app nativo só substitui o de antes para o grupo quando as histórias 3, 4 e 5 passam (T058, T068, T079).
- **Ordem**: Phase 1 → Phase 2 → histórias 3 e 4 (podem andar em paralelo depois de T037) → história 5 → histórias 6 e 7 e a parte de app da história 2.
- A história 1 não depende de nada. Gates vermelhos param tudo o que vem depois na mesma trilha.
- **Troca**: `Espia-setup.exe` passa a ser o nativo e o de antes fica em `public/download/Espia-setup-0.1.10.exe` (R13). Até lá, `public/download/Espia-setup.exe` continua sendo o de 2026-11-05.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Sete crates no workspace `app/` | A regra IV (tela não fala com rede, captura nem mídia) passa a ser garantida pelo grafo de dependências, porque `espia-ui` só enxerga `espia-core`. Também deixa subagentes trabalharem em paralelo sem tocar no mesmo arquivo. | Uma crate só deixaria a regra IV depender de disciplina, e os lotes de trabalho se cruzariam nos mesmos arquivos. |
| Duas interfaces (site e app nativo) | Decisão de 2026-11-11, já refletida na constituição 4.0.0. O app só pesa menos se não carregar motor de página. | Manter o app de antes como único app não entrega as metas de peso da spec (FR-009, SC-005, SC-006). Só mover o envio para o nativo deixa o motor de página carregado e a memória igual (ver [custo-migracao.md](./custo-migracao.md)). |
