# Implementation Plan: Transmitir sem pesar no jogo, codificando uma vez pela placa de vídeo

**Branch**: `020-codificar-pela-placa` (trabalho direto na `main`, como nas features anteriores) | **Date**: 2026-11-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/020-codificar-pela-placa/spec.md`

## Summary

1. **Provar antes de construir (G4, G5, G6)**: uma sonda nova, a partir da sonda do G1 da 019, comprime vídeo de teste pela placa de vídeo e o entrega pelo caminho de vídeo já codificado da `libwebrtc` 0.4.0. Primeiro para uma pessoa no Chrome (G4), depois para três, com entrada no meio, parar de assistir, troca de qualidade e troca de caminho (G5). Por fim, captura real com jogo, lado a lado com o app de hoje (G6).
2. **Comprimir uma vez pela placa (R1 a R5)**: o app passa a ser o dono da compressão. A textura que a captura já produz na placa vai direto ao codificador de H.264 do Windows, e o mesmo quadro comprimido é repartido por todas as ligações.
3. **Duas qualidades (R6 a R9)**: cheia e reduzida, cada uma comprimida só enquanto alguém a recebe. As regras de quem recebe o quê, de quando pedir quadro inteiro e de como reagir à internet de cada espectador são funções puras em `espia-core`, com teste.
4. **Sem placa capaz (R11)**: o app cai sozinho no caminho pelo processador, que o G1 já provou, na qualidade escolhida, com o ajuste automático por processador que já existe.
5. **O que a pessoa vê (R12)**: nenhuma opção nova. Só uma linha no Diagnóstico de mídia.
6. **Medição (R15)**: script novo que mede o processador em núcleos, para comparar app de hoje e app nativo sem depender da máquina.

Esta feature decide como a fase de transmitir da 019 comprime o vídeo. Ela não muda o site, o Worker nem o protocolo.

## Technical Context

**Language/Version**: Rust 1.90, edição 2024 (workspace `app/`). PowerShell 5.1 para o script de medição.

**Primary Dependencies**: `libwebrtc` =0.4.0 (caminho de vídeo já codificado), `windows` 0.62 (Media Foundation, Direct3D 11, DXGI), `tokio`. Nenhuma dependência nova em relação à 019.

**Storage**: N/A. Nada é guardado no aparelho nem no servidor.

**Testing**:
- `cargo test` em `espia-core` para as regras puras (qualidade por espectador, quadro inteiro a pedido, internet fraca, troca de caminho).
- `cargo test` em `espia-media` para o que não precisa de placa (montagem do quadro, estado do publicador com codificador falso).
- Sonda e roteiro manual em [quickstart.md](./quickstart.md) para o que depende de placa, rede e jogo. Teste automático nunca captura a tela de verdade.

**Target Platform**: Windows 10 e 11, x64, com placa de vídeo NVIDIA, AMD ou Intel. Sem placa capaz: caminho pelo processador.

**Project Type**: app desktop nativo (workspace `app/`), mais um script de medição.

**Performance Goals**: SC-002 a SC-010 da spec. Em resumo: metade do processador do app de hoje com 3 pessoas em 1080p60; crescimento de no máximo 25% entre 1 e 7 pessoas; pelo menos 53 quadros por segundo para quem assiste; primeira imagem em até 3 segundos.

**Constraints**:
- conexão direta entre as pessoas, sem servidor de mídia;
- um canal de vídeo por pessoa na negociação, como o site espera;
- nenhuma opção nova para quem transmite;
- nunca tirar a faixa de um canal ativo (defeito achado no G1);
- sem mudança no protocolo, no Worker e no site.

**Scale/Scope**: até 9 espectadores por transmissão (sala de 10); no máximo 2 compressões pela placa por transmissão; estimativa de 1.500 a 2.500 linhas de Rust novas, mais a sonda.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Situação |
|-----------|----------|
| I. Simplicidade | ✅ Transmitir tem os mesmos passos. Nenhuma opção, pergunta ou termo técnico novo (FR-014). A regra do Assistir vale igual (FR-008). |
| II. Sem servidor de mídia | ✅ Malha direta mantida. Só muda quem comprime, dentro do aparelho de quem transmite. A qualidade continua baixando com o número de espectadores (R10). A parte "um servidor repassa" da ideia original ficou fora. |
| III. Nada sobre as pessoas | ✅ Nada novo é guardado nem enviado ao servidor. |
| IV. Camada visual separada | ✅ As regras ficam em `espia-core` (puras) e a compressão em `espia-media`. A tela só recebe a linha do diagnóstico como dado pronto, na lista de linhas que o painel já mostra. |
| V. Telas mockáveis | ✅ A linha nova do Diagnóstico de mídia ganha cena de dados falsos antes da lógica real. |
| VI. Segredos fora do cliente | ✅ Nenhum segredo envolvido. |
| Restrições técnicas | ✅ O acesso ao computador continua sendo o que transmitir exige: captura de imagem e a placa de vídeo para comprimi-la. Navegadores alvo continuam recebendo H.264. A biblioteca de ligação de vídeo é a mesma da 019; a mudança é registrada em `DEFINICOES.md`. |
| Fluxo | ✅ Mock antes da lógica na única tela tocada. Gate do app: `cargo check` e `cargo test` em `app/`. Protocolo sem mudança. |

**Re-check pós-design**: ✅ sem violações. O desenho de duas qualidades mantém um canal de vídeo por pessoa, então o site e os navegadores não percebem diferença na negociação.

## Project Structure

### Documentation (this feature)

```text
specs/020-codificar-pela-placa/
├── plan.md                 # este arquivo
├── spec.md
├── investigacao.md         # levantamento que originou a feature
├── research.md             # R1–R18
├── data-model.md           # transmissão, qualidades, espectador, caminho de compressão, medição
├── quickstart.md           # gates G4–G6 e roteiro por história
├── relatorio.md            # criado na primeira medição; recebe G4–G6
├── contracts/
│   ├── codificador.md      # interface do codificador e o que cada caminho garante
│   ├── qualidades.md       # regras de cheia e reduzida, quadro inteiro, internet fraca
│   ├── caminho.md          # escolha e troca entre placa e processador
│   └── medicao.md          # scripts/processador.ps1 e o que registrar
├── checklists/
│   └── requirements.md
└── tasks.md                # /speckit-tasks
```

### Source Code (repository root)

```text
app/
├── crates/
│   ├── espia-core/src/
│   │   ├── tiers.rs                 # novo: regras puras de qualidade por espectador, quadro inteiro, internet fraca, caminho
│   │   └── quality.rs               # existente: effective_quality e send_params continuam a fonte dos números
│   ├── espia-media/src/
│   │   ├── encode/
│   │   │   ├── mod.rs               # novo: interface do codificador e quadro comprimido
│   │   │   ├── hardware.rs          # novo: codificador de H.264 da placa (Windows)
│   │   │   └── software.rs          # novo: caminho pelo processador (quadros crus para a biblioteca)
│   │   └── publish.rs               # novo: fontes e faixas por qualidade, ligação de cada canal à faixa certa
│   └── espia-ui/
│       ├── ui/components/media-diagnostics.slint   # linha nova
│       └── src/mocks/room.rs                       # cena com a linha nova
└── spikes/
    └── encode-probe/                # novo: prova G4 e G5 (apagada no fim, como as outras sondas)

scripts/
└── processador.ps1                  # novo: processador em núcleos, média de 60 s
```

**Structure Decision**: tudo dentro do workspace `app/` criado pela 019. As regras vão para `espia-core`, que não conhece mídia; a compressão e a publicação vão para `espia-media`. A sonda fica em `app/spikes/`, fora do workspace de produção, como as do G1 a G3.

## Gates

Provas antes de construir. Se um gate falhar, parar, registrar em `relatorio.md` e replanejar com o plano B da decisão. Não seguir adiante com gate vermelho.

| Gate | Onde | Passa se |
|------|------|----------|
| **Leitura do app de hoje** (pessoa) | Diagnóstico de mídia do site, com o app 0.1.10 transmitindo | Registrado qual compressão o app de hoje usa. Só informa; não bloqueia. |
| **G4, uma pessoa** | `app/spikes/encode-probe/` | (a) acha e inicia o codificador de H.264 da placa; (b) o Chrome recebe 1920×1080 em H.264 com som; (c) pelo menos 53 quadros por segundo no Chrome; (d) o Diagnóstico da sonda mostra que a compressão não é a de software da biblioteca; (e) processador da sonda em núcleos, média de 60 s, registrado; (f) atraso entre entregar o quadro e receber o comprimido registrado. |
| **G5, três pessoas** | `app/spikes/encode-probe/` | (a) três pessoas recebem o mesmo vídeo; (b) processador com três em até 25% acima do de uma; (c) quem entra no meio vê imagem em até 3 s; (d) os outros não congelam mais de 0,5 s nessa entrada; (e) parar de assistir e voltar funciona sem derrubar a sonda; (f) trocar um espectador para a reduzida e de volta funciona; (g) forçar a troca para o caminho pelo processador mantém a transmissão. |
| **G6, com jogo** (pessoa) | app nativo com captura real, ou a sonda com captura | Lado a lado com o app de hoje, na mesma sessão, com o número de pessoas assistindo conferido: perda de FPS do jogo não maior que a do app de hoje; processador em no máximo metade. |

Planos B: G4 devolve a decisão à R2 da 019 (biblioteca modificada ou outra biblioteca). G5 itens (a) e (b): uma fonte por ligação (R5). G5 item (f): dois canais por ligação entre apps (R7). G5 item (g): parar e reabrir a transmissão sozinho (R11). G6: entrada pela memória (R3) e revisão das metas.

## Entrega em etapas

- **Etapa 1**: `scripts/processador.ps1`, leitura do app de hoje e as regras puras de `tiers.rs` com teste. Nada disso depende de placa.
- **Etapa 2**: sonda e G4. Depois G5.
- **Etapa 3** (só com G5 verde): `encode/` e `publish.rs` em `espia-media`, ligados à fase de transmitir da 019. G6.
- **Etapa 4**: reduzida para internet fraca (história 5) e caminho pelo processador com troca no meio (história 6).
- **Etapa 5**: conferência com o grupo, incluindo outras marcas de placa e iPhone.

A tarefa T037 da 019 (ligação entre pessoas sem mídia) pode andar em paralelo com as etapas 1 e 2.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Dois caminhos de compressão (placa e processador) | FR-012 e FR-013: quem não tem placa capaz, ou tem a placa ocupada, continua transmitindo sem pergunta. | Só o caminho pela placa deixaria parte do grupo sem transmitir. Só o caminho pelo processador é o que o G1 mostrou ser mais pesado que o app de hoje. |
| Duas qualidades comprimidas ao mesmo tempo | Com um vídeo para todos, miniatura e internet fraca precisam de uma versão menor (FR-006, FR-010). | Uma qualidade só faria a miniatura gastar a internet de todos e deixaria o espectador mais fraco puxar a qualidade dos outros. |
