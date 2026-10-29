# Implementation Plan: Transmissão leve

**Branch**: `010-transmissao-leve` (trabalho direto na `main`, como na 004–009) | **Date**: 2026-10-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/010-transmissao-leve/spec.md`

## Summary

Três problemas medidos na 009 (app 0.1.6, seletor nativo, 1080p60, 3 pessoas): memória que cresce 12–14 MB/min **com som**, CPU da página em 27–30 % e upload com pico acima do teto. O plano:

1. **Diagnóstico primeiro** (R1): build de debug + CDP (heap, métricas, trace, `getStats`). Confirma o que cresce, para onde vai a CPU e quanto cada conexão realmente envia.
2. **Som (US1)**: hoje são 100 mensagens por segundo, de 3,8 KB, cada uma com um `eval` e um pedido IPC, e escritas sem fila nem descarte (R2). Correção:
   - pedaços de 40 ms (4× menos mensagens);
   - a página descarta pedaço quando o gerador tem fila ou o pedaço está atrasado mais de 200 ms;
   - carimbo de tempo pelo relógio, se R1 mostrar deriva.
   - Plano B (anel compartilhado, como o vídeo) só se o lote não bastar.
3. **Upload (US2)**: o teto entra **junto com a faixa** (`sendEncodings` no `addTransceiver`), em vez de depois da negociação. O log de 1 s com o `bytesSent` real é a régua: a coluna do Gerenciador soma envio e recebimento.
4. **CPU (US3)**:
   - o Rust pula quadros iguais ao anterior e o ritmo fica exato (R4);
   - **ajuste automático sem aviso**: o motor diz quando está limitando por CPU (`qualityLimitationReason`). Com 10 s no limite desce um degrau; com 60 s de folga sobe um (R5).

Sem comando nativo novo e sem emenda: só muda o formato do som **dentro** da ponte de captura da 005.

## Technical Context

**Language/Version**: TypeScript 6 + Svelte 5 (site); Rust 1.90+ (app)

**Primary Dependencies**: as mesmas. WebRTC do motor (`getStats`, `sendEncodings`), WebCodecs (`AudioData`, `VideoFrame`), `MediaStreamTrackGenerator`. Playwright-core (scratchpad, só para o diagnóstico).

**Storage**: nada.

**Testing**:
- Vitest:
  - `quality.ts`: `stepDown`, `stepUp` e `effectiveQuality` com `cpuSteps`;
  - `cpu-governor.ts`: os tempos de 10 s e 60 s;
  - `native-capture.ts`: descarte de som por fila e atraso, com uma ponte falsa;
  - `peer.ts`: `sendEncodings` no `addTransceiver`, se testável com o falso que já existe.
- `cargo test`: tamanho do pedaço de som, regra de "quadro igual".
- Diagnóstico R1 e roteiro T1–T6 ([quickstart.md](./quickstart.md)).

**Target Platform**: US1 só no app (Windows). US2 e US3 no app e no site (Chrome/Edge).

**Project Type**: SPA + Worker (sem mudança no Worker) + casca desktop `src-tauri/`

**Performance Goals**: SC-1001 (≤ 3 MB/min com som), SC-1003 (≤ 21 Mbps de envio com 3 pessoas em 1080p60), SC-1004 (CPU ≤ ~22 %), SC-1005 (≤ 15 s para descer, ≤ 75 s para subir).

**Constraints**:
- som sincronizado e sem atraso crescente;
- o ajuste por CPU não pode piscar, pela histerese de 10 s e 60 s;
- sem servidor de mídia (II);
- sem aviso novo.

**Scale/Scope**:
- **site**: `native-capture.ts` (som), `peer.ts` (teto no início + log de envio), novo `cpu-governor.ts`, `quality.ts` (degraus), `room.svelte.ts` (liga o governor ao `applyQuality`);
- **Rust**: `capture/audio.rs` (`CHUNK_FRAMES`), `capture/video.rs` (pular quadro igual, ritmo).

## Constitution Check

| Princípio / restrição | Situação |
|---|---|
| I. Simplicidade | ✅ Nada novo para quem usa. O ajuste é automático e sem aviso. |
| II. Sem servidor de mídia | ✅ Continua P2P em malha. Só reduz o custo de cada conexão. |
| III. Efêmero e sem contas | ✅ Nada guardado. |
| IV. Camada visual separada | ✅ O governor e os degraus ficam em `src/lib/`. A tela só lê o `adjusted` que já existe. |
| V. Telas mockáveis | ✅ Sem tela nova. O indicador "ajustada" já tem mock. |
| VI. Segredos | ✅ Nada. |
| Restrição técnica do app (só as APIs listadas) | ✅ Nenhum comando novo. O formato do som muda dentro da ponte de captura da 005 ([contracts/native-capture-delta.md](./contracts/native-capture-delta.md)). |

**Re-check pós-design**: ✅.

## Project Structure

### Documentation (this feature)

```text
specs/010-transmissao-leve/
├── spec.md
├── plan.md                         # este arquivo
├── research.md                     # R1–R6
├── data-model.md                   # degraus, CpuGovernor, som
├── quickstart.md                   # diagnóstico + roteiro T1–T6
├── contracts/native-capture-delta.md
├── checklists/requirements.md
└── tasks.md                        # /speckit-tasks
```

### Source Code (repository root)

```text
src/lib/
├── quality.ts                      # + stepDown/stepUp, effectiveQuality(chosen, fullViewers, cpuSteps)
├── cpu-governor.ts                 # NOVO: tick(limited) → down/up (10 s / 60 s)
├── native-capture.ts               # som: descarte por fila/atraso, contagem no log de 10 s
├── peer.ts                         # sendEncodings no addTransceiver; stats de envio (bytesSent, qualityLimitationReason)
└── stores/room.svelte.ts           # timer de 1 s: stats → governor → applyQuality; reset na troca manual

src-tauri/src/capture/
├── audio.rs                        # CHUNK_FRAMES 480 → 1920
└── video.rs                        # pular quadro igual (hash de amostras); ritmo exato

test/lib/quality.test.ts, cpu-governor.test.ts, native-capture.test.ts
```

**Structure Decision**: as regras (degraus, histerese, descarte de som) são funções puras com testes. O store só junta stats → regra → `applyQuality`, o caminho que já existe para o ajuste por espectadores.

## Ordem de entrega

1. **Diagnóstico R1** (scratchpad): heap e métricas com e sem som; trace de CPU; `getStats` por conexão. Anotar no `relatorio.md` e ajustar R2–R4 se a causa for outra.
2. **US1**: som em 40 ms (Rust) + descarte na página. Medir T1/T2 no debug. Deploy do site (compatível com o app 0.1.6).
3. **US2**: teto no `addTransceiver` + log de envio. Medir T3.
4. **US3**:
   - pular quadro igual e ritmo (Rust);
   - `quality.ts` com degraus, `cpu-governor.ts` e a ligação no store;
   - medir T4–T6.
5. Versão 0.1.7: build, roteiro completo, relatório.
