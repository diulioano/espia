# Implementation Plan: Reduzir atraso da imagem

**Branch**: `main` | **Date**: 2026-11-04 | **Spec**: [spec.md](./spec.md)

**Input**: `specs/016-reduzir-atraso-imagem/spec.md`

O setup retornou `BRANCH=016-reduzir-atraso-imagem` como identificador da feature, derivado de feature.json. `git branch --show-current` confirmou `main`; nenhum branch foi criado ou trocado.

## Summary

Reduzir o atraso da imagem, relatado em cerca de 10 s em alguns casos, com meta inicial de p95 ≤ 1 s em condições estáveis e recuperação em ≤ 5 s. O relato ainda não foi reproduzido nesta fase.

A pesquisa encontrou um descompasso de relógios na captura nativa: a página fixa o zero antes de inicializar a captura, o Rust começa a contagem depois da preparação e o áudio usa a chegada do primeiro pacote. Também há lacunas na política de descarte e na observabilidade da recepção. O plano combina:

1. Reproduzir e medir o atraso real com contador visual e evento de flash/som, correlacionando estatísticas de captura, envio e recepção.
2. Corrigir o mapeamento dos relógios e a recuperação do áudio; limitar trabalho pendente da ponte nativa e invalidar callbacks de sessões encerradas.
3. Aplicar preferência de baixa latência aos receivers quando suportada, preservando o comportamento padrão nos demais motores. Medir o resultado, pois a preferência não garante esvaziamento imediato.
4. Proteger operações assíncronas de troca/parada de mídia e validar retorno de visibilidade, miniatura/destaque, PiP e reconexão.

Sem mudanças no Worker, mensagens de sinalização, canal ctl, comandos nativos, arquitetura, telas ou passos para assistir. Não se considera a correção concluída sem os ensaios reais de SC-001–006.

## Technical Context

**Language/Version**: TypeScript 6, Svelte 5; Rust edition 2024, rust-version 1.90.

**Primary Dependencies**: Vite 8, Vitest 4, Tauri 2, WebView2, windows 0.62, WebRTC/getStats, VideoFrame/AudioData e MediaStreamTrackGenerator já utilizados. Sem nova dependência de produção.

**Storage**: apenas estado transitório por sessão e janela limitada de diagnóstico em memória. Sem persistência, coleta remota ou conteúdo de mídia gravado.

**Testing**: Vitest com fake Bridge/relógio/writers/RTCPeerConnection; testes Rust de mailbox e cancelamento; ensaios reais em dois ou mais aparelhos. Gates de implementação: npm run check, npm test, npm run build; cargo test e desktop:build se o Rust mudar.

**Target Platform**: transmissão de tela/janela em Chrome/Edge desktop e app Windows; câmera nos caminhos existentes, inclusive celular; recepção em navegadores modernos. APIs opcionais sempre detectadas.

**Project Type**: SPA com Worker de sinalização e app desktop opcional.

**Performance Goals**: SC-001 p95 ≤ 1 s e nenhuma amostra ≥ 5 s nos trechos estáveis; SC-002 p95 ≤ Discord + 500 ms; SC-003/004 recuperação ≤ 5 s; SC-005 diferença A/V ≤ 200 ms; SC-006 sem reativação depois de encerrar.

**Constraints**: preservar CPU governor 10 s/60 s, qualidade por espectadores, teto de upload, áudio em segundo plano e exclusão do som do Discord. Um espectador ruim não pode rebaixar a qualidade global dos outros. Não substituir ausência de métricas por zero.

**Scale/Scope**: salas de até 10 pessoas; ensaio prolongado com 1 e 3 espectadores, regressão funcional com sala cheia. Alterações concentradas em captura/peer/store e auxiliares puros de tempo/estatísticas.

## Constitution Check

*Gate avaliado antes da pesquisa e reavaliado após o design.*

| Princípio / restrição | Antes | Depois do design |
|---|---|---|
| I. Simplicidade | PASS | Sem conta, instalação obrigatória, configuração ou aviso novo. |
| II. Sem servidor de mídia | PASS | WebRTC P2P e TURN existentes, até 10 pessoas. |
| III. Nada sobre as pessoas | PASS | Diagnóstico local limitado, sem nomes, IPs, SDP ou envio ao servidor. |
| IV. Camada visual separada | PASS | Relógios, estatísticas e política em src/lib; UI continua recebendo estado pronto. |
| V. Telas mockáveis | PASS | Sem estado visual novo; regressão dos mocks existentes. |
| VI. Segredos fora do navegador | PASS | Nenhum segredo ou mudança em credenciais. |
| APIs nativas restritas | PASS | Ajustes internos à ponte de captura existente, mantendo comandos e formatos. |

Nenhuma violação, emenda ou mudança em decisões de DEFINICOES.md. A implementação que venha a exigir outro contrato deve atualizar os artefatos antes de prosseguir.

## Project Structure

### Documentation (this feature)

```text
specs/016-reduzir-atraso-imagem/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/latency.md
└── checklists/requirements.md
```

`tasks.md` será criado por /speckit-tasks, fora desta fase.

### Source Code (repository root)

```text
src/lib/
├── native-capture.ts          # relógios, descarte, lifecycle dos geradores
├── capture-timing.ts          # novo: regras puras de tempo da sessão
├── media-stats.ts             # novo: normalização de deltas e janela limitada
├── peer.ts                   # stats inbound/outbound, preferência de recepção, setLocal seguro
├── mesh.ts                   # agregação e descarte por link
└── stores/room.svelte.ts      # coleta também para quem só assiste, integração e cleanup
src-tauri/src/capture/
├── mod.rs                    # publicação limitada e cancelável para WebView
├── video.rs                  # emissão do quadro mais recente e invalidação
└── audio.rs                  # limite do backlog antes da publicação
src/ui/components/StreamTile.svelte  # inspeção/regressão A/V e PiP; sem lógica WebRTC
scripts/latency-source.html   # novo: contador e flash/beep para ensaio local
test/lib/                    # timing, stats, native-capture e lifecycle do peer
```

**Structure Decision**: regras puras ficam separadas dos adaptadores. Mantêm-se os caminhos de captura nativa e do navegador. O store coordena recursos sem duplicar a coleta de envio existente.

## Phase 0 — Research

Concluída em [research.md](./research.md). As decisões técnicas estão resolvidas; a causa específica dos 10 s e o alcance das metas são itens de validação empírica, não fatos assumidos. As pesquisas de captura nativa e peer/reprodução foram delegadas em paralelo conforme o workflow.

## Phase 1 — Design

- [data-model.md](./data-model.md): relógio de captura, mailbox, amostras e identidade de sessão.
- [contracts/latency.md](./contracts/latency.md): compatibilidade, invariantes e limites das métricas.
- [quickstart.md](./quickstart.md): reprodução, matriz de aceitação e regressão.

### Sequência para a futura implementação

1. Construir a régua local e coletar uma linha de base, especialmente com som ligado/desligado e inicialização demorada. Registrar evidências em relatorio.md, sem inventar resultados.
2. Cobrir o descompasso de relógio por teste com startup de 2 s e 10 s; implementar mapeamento e recuperação do áudio.
3. Limitar publicação de quadros/PCM e proteger encerramento/troca de sessão; medir novamente. Validar compatibilidade com formatos antigos.
4. Adicionar métricas de recepção e preferência de baixo buffering detectável; proteger setLocal/close contra operações atrasadas.
5. Executar matriz completa. Se o atraso persistir, usar as métricas para localizar a etapa e revisar o design antes de escolher reset de reprodução ou mudança de formato. Não declarar as metas atendidas por unit tests ou pelo valor solicitado ao receiver.

Uma preferência de receiver pode recuperar lentamente e a ponte antiga não informa a idade exata do PCM. Essas limitações não relaxam SC-001–006: falha no ensaio impede considerar a implementação concluída.

## Ajuste de execução — 2026-11-04

Com apenas um ambiente conectado, sem sessões em outros aparelhos/Discord/instrumento A/V, a baseline externa T008 não é executável aqui. Ela permanece pendente para aceitação; as correções de relógio/lifecycle podem avançar após instrumentação T005–T007 com regressões determinísticas. Não confundir esses testes com reprodução ponta a ponta dos 10 s. A revisão inicial fe6b282 permite executar a comparação antes/depois quando o ambiente estiver disponível. A ordem interna das tarefas de código permanece; gates externos não bloqueiam os incrementos seguintes, mas impedem declarar US1/US2/US3 ou a feature aprovadas.

## Ajuste de estrutura realizado

A fila nativa foi extraída para src-tauri/src/capture/publish.rs, módulo interno de capture/mod.rs, para verificar limites/terminalidade sem COM. Nenhum contrato externo mudou. As dependências de aceitação externas permanecem pendentes; o estado detalhado está em relatorio.md e tasks.md.
