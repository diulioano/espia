# Implementation Plan: Transmissão sob demanda

**Branch**: `main` | **Date**: 2026-11-05 | **Spec**: [spec.md](spec.md)

**Input**: `specs/017-transmissao-sob-demanda/spec.md`. O setup resolveu o identificador lógico `017-transmissao-sob-demanda`; `git branch --show-current` confirma `main`. Nenhuma branch foi criada.

## Summary

Preservar captura, site, app e malha direta. Substituir a lista negativa de quem parou por escolhas explícitas por transmissão. Nenhum track de envio pode ser ligado sem autorização válida para aquela transmissão. Mostrar cartões Assistir desde o primeiro estado, inclusive durante negociação.

A release opera somente com STUN, sem emitir ou renovar TURN, enquanto não houver prova de orçamento e interrupção efetivos, conforme FR-017. Não construir um contador que prometa controle de cobrança sem poder exercê-lo. SFU fica em pesquisa documental; implementação e ensaio remoto dependem de emenda constitucional, decisão de criptografia e prova de proteção de custo.

## Technical Context

**Language/Version**: TypeScript 6.0.3; Rust edition 2024, MSRV 1.90, nativo preservado.
**Primary Dependencies**: Svelte 5.57, Vite 8.3, Tauri 2.12, WebView2 e WebRTC existentes. Versões declaradas no projeto, não proposta de atualização.
**Storage**: escolhas somente na memória da permanência na sala; sem nova persistência de participantes. Salas e retomada usam armazenamento existente. Sem novo banco de consumo nesta release.
**Testing**: Vitest e ambiente Worker existente; `npm run check`, `npm test`, `npm run build`; ensaios em aparelhos reais. `cargo test` e build desktop somente se a implementação tocar Rust ou configuração nativa.
**Target Platform**: navegadores suportados e app Windows existentes; celular com uma recepção ativa por vez.
**Project Type**: SPA, Worker e Durable Object por sala; app desktop opcional.
**Performance Goals**: p95 visual por recepção ≤500 ms; início ≤3 s; recuperação ≤5 s; diferença A/V ≤200 ms; parar ≤2 s. Metas a validar, sem ganho presumido.
**Constraints**: até dez pessoas; duas transmissões e sete espectadores; sem cobrança excedente de mídia; sem conteúdo ou identidade em diagnósticos; sem migração do motor.
**Scale/Scope**: 14 entregas simultâneas no máximo escolhido. Com teto atual de 720p30/2,5 Mbps para sete espectadores completos, aproximadamente 17,5 Mbps de vídeo de upload por transmissor, além dos protocolos e som. É cálculo de capacidade, não resultado medido.

## Constitution Check

Revisão pós-remediação de 2026-11-05: **PASS com a constituição 3.0.0 vigente**. O PASS anterior com divergência foi retirado; a emenda constitucional ocorreu separadamente e precede a implementação. Nenhuma exceção constitucional é assumida.

| Princípio | Verificação e resolução |
| --- | --- |
| I, simplicidade | Constituição 3.0.0 exige até dois passos para entrar e clique por transmissão alheia. O desenho atende à regra vigente, sem conta ou instalação obrigatória. |
| II, sem servidor de mídia | Malha direta mantida; STUN-only é compatível. SFU não integra a entrega executável. Pesquisa documental não é adoção; ensaio real depende de emenda prévia. |
| III, privacidade | Escolhas locais em memória; nenhuma persistência nova de pessoa. Sem ledger nesta release. |
| IV, separação visual | Store produz estado; UI emite Assistir/Parar. Peer concentra autorização de envio. |
| V, mocks | Cartão inicial, parado, abrindo, ativo, indisponível e retomada previstos em mocks antes da lógica real. |
| VI, segredos | Não levar chaves ao cliente; não emitir credenciais de relay nesta release. |

A emenda separada do princípio I foi concluída na remediação: versão 3.0.0 e Sync Impact Report registrados. O princípio II permanece intacto.

## Project Structure

### Documentation (this feature)

```text
specs/017-transmissao-sob-demanda/
  spec.md
  plan.md
  research.md
  data-model.md
  quickstart.md
  checklists/requirements.md
  contracts/watch.md
  contracts/signaling.md
  contracts/media-policy.md
```

`tasks.md` será gerado pelo próximo comando.

### Source Code (repository root)

```text
src/lib/watch-state.ts                 # novo: escolhas e transições puras
src/lib/stores/room.svelte.ts          # integração, projeção e retomada
src/lib/room-view.ts                   # modos derivados de escolha positiva
src/lib/mesh.ts                        # transmissão atual por peer
src/lib/peer.ts                        # tracks nulos até autorização
src/lib/quality.ts                     # perfis de conteúdo e tetos existentes
src/lib/cpu-governor.ts                # pressão local, separada da rede de um receptor
src/lib/signaling.ts                   # negociação de versão e política
src/lib/types.ts
src/shared/media.ts                   # novo: identidade, ordem e política puras
src/shared/protocol.ts
src/worker/validate.ts
src/worker/room.ts
src/worker/turn.ts                     # política STUN-only antes de qualquer API
src/ui/components/StreamTile.svelte
src/mocks/room.ts
test/lib/                             # estado, lifecycle, modos e qualidade
test/worker/room.test.ts               # versão, retomada e ausência de TURN
scripts/latency-source.html            # régua existente da 016
```

**Structure Decision**: reutilizar as camadas existentes. Uma função de estado pequena substitui a semântica invertida de `unwatched`. Não adicionar barramento, adaptador genérico de transportes, servidor SFU ou Durable Object global sem uso comprovado.

## Phase 0: Research

Concluída em [research.md](research.md). Decisões: escolha positiva por transmissão; consentimento versionado; tracks ausentes antes de consentir; STUN-only enquanto custo não for controlável; SFU documental; comparação real antes de otimizações nativas. Unknowns foram resolvidos como decisões ou condições de aceitação, não como aprovação de desempenho.

## Phase 1: Design and delivery sequence

1. Conferir constituição 3.0.0 e registrar decisões novas em DEFINICOES.md durante implementação. Aplicar STUN-only em mintIceServers, join/resume/refresh antes de anunciar mediaProtocol:2/direct-only; testar zero emissão com segredos presentes. Criar mocks dos estados do contrato de visualização.
2. Implementar reducer de escolha positiva e identidade da transmissão; renovar escolha somente por clique ou retomada da mesma transmissão na mesma permanência.
3. Proteger o primeiro envio e todas as substituições: controle abre sem mídia, tracks nulos até autorização válida; mudança de transmissão invalida grants antigos. Serializar mutações já existentes e invalidar resultados atrasados.
4. Publicar contratos compatíveis como uma unidade: cliente anuncia versão, servidor recusa clientes antigos e conexões antigas são drenadas. Evitar coexistência silenciosa com emissores que enviam por padrão.
5. Auditar a política STUN-only já aplicada nos fundamentos, validar filtros e indisponibilidade no cliente e planejar drenagem de credenciais anteriores; não declarar custo protegido retroativamente.
6. Medir baseline e candidato em rodadas equivalentes, usando 016 como referência. Explorar primeiro hints de conteúdo nativos, limites por receptor e aceleração efetiva; não adicionar classificação de pixels ou mover codificação para Rust sem evidência.
7. Registrar resultado por recepção. Só concluir eficácia após os ensaios reais. Pesquisa SFU permanece condicional e não bloqueia entregar assistir sob demanda com transporte direto.

### Quality and latency ownership

`quality.ts` mantém tetos por espectadores realmente ativos e o máximo escolhido. O perfil usa `MediaStreamTrack.contentHint` quando disponível: `detail`/`text` favorecem resolução, `motion` fluidez. Na ausência do hint, câmera usa movimento e captura de tela usa detalhe. Isso é uma heurística de origem, não um classificador de cenas; testar texto e movimento na mesma janela e registrar falhas.

O motor ajusta congestionamento por peer. Limitação de rede de um receptor não reduz a captura global. O governor global responde apenas a pressão de codificação local confirmada. Preservar inicialmente seus limiares de 10 s para reduzir e 60 s para subir; só reajustar se a baseline demonstrar necessidade. A meta de recuperação não é satisfeita apenas pela escolha desses limiares.

Nenhuma condição interna de `getStats()` equivale a medição visual ponta a ponta. `encoderImplementation` e `powerEfficientEncoder` precisam confirmar hardware onde disponíveis; preferência por H.264 não prova uso da GPU.

### Conditional scope and acceptance

FR-014–017 e SC-005 usam nesta release o ramo seguro de retransmissão desabilitada. O ramo de saldo positivo da história 3 não é declarado exercitado. Habilitá-lo exige orçamento verificável de toda a conta, reservas concorrentes e teste de expiração/interrupção. A entrega deve declarar essa limitação.

História 4 entrega baseline e protocolo de comparação. Se SFU não passar pelos gates de governança, custo e criptografia, o relatório marca comparação remota pendente. Não criar uma SFU que nunca possa ser ativada apenas para preencher uma tarefa.

## Complexity Tracking

Nenhuma violação constitucional remanescente após a emenda separada. O controle de ordem da transmissão guarda somente um maior broadcastSeq, o requestId correspondente, lastStartedAt e o estado live/ended na identidade já existente do membro; não cria histórico ilimitado de pedidos.

O aceite aplica integralmente SC-003 e SC-007 revisados: 1.800 instantes, cobertura instrumental ≥99%, lacunas máximas de 2 s, fluidez ≥27 fps, interrupções ≤1% sem pausa >1 s e regressão de FPS do jogo ≤5%. A ferramenta distingue lacuna de medição de falta de vídeo. Gates de desempenho real e eventual SFU permanecem condições de aceitação, não resultados deste planejamento.


## Registro de execução local

A implementação desenvolveu e testou os perfis de conteúdo por hints, sem retunar tetos nem limiares 10/60 s, enquanto a baseline real T028 segue indisponível. A revisão 0.1.9 anterior está preservada para comparação. Preparação local não aprova eficácia nem autoriza publicação; o aceite e a ativação comparativa dependem das medições previstas. A supressão TURN foi efetivada nos fundamentos antes de welcome v2. Detalhes e limitações em relatorio.md.
