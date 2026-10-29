# Implementation Plan: Som da transmissão sem vazar o Discord

**Branch**: `002-audio-sem-discord` | **Date**: 2026-10-28 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-audio-sem-discord/spec.md`

## Summary

Quem transmite a tela inteira hoje manda o som do PC inteiro, inclusive a chamada do Discord. A
correção é capturar só o som da superfície escolhida:

- Aba: som da aba.
- Janela: som só da janela, com `windowAudio: 'window'`.
- Tela inteira: sem som, com `systemAudio: 'exclude'`.

Uma rede de segurança descarta o som se o navegador ignorar essas dicas. A decisão vira uma função
pura e testável. O tipo de som (`tab`/`window`/`none`) passa a ser anunciado no `live`, e a
interface mostra o que está saindo. Detalhes em [research.md](./research.md).

## Technical Context

**Language/Version**: TypeScript 6, Svelte 5 (runes)

**Primary Dependencies**: APIs do navegador (`getDisplayMedia`, `MediaStreamTrack`); Cloudflare Workers + Durable Objects (já existentes)

**Storage**: N/A

**Testing**: Vitest (`@cloudflare/vitest-plugin`) para a regra pura e o Worker; Playwright + Edge instalado (scratchpad) para e2e com abas de teste; teste manual com Discord

**Target Platform**: Chrome/Edge desktop atual no Windows (transmitir); qualquer navegador (assistir)

**Project Type**: SPA + Worker (projeto único, ver estrutura)

**Performance Goals**: reavaliar o som em < 100 ms após troca de superfície; sem custo extra na malha

**Constraints**: as opções do navegador são dicas e não garantias. Nada de som do sistema pode sair, nem por um instante (descartar antes de entregar à malha).

**Scale/Scope**: ~6 arquivos alterados, 1 novo módulo, ~10 testes novos

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Situação |
|---|---|
| I. Simplicidade | ✅ Nenhum passo a mais. Só textos e um aviso. Nada de voz/chat. |
| II. Sem servidor de mídia | ✅ O áudio continua P2P; o Worker só valida o rótulo. |
| III. Efêmero | ✅ Nada persistido. |
| IV. Camada visual separada | ✅ Regra em `src/lib/audio-source.ts`; a UI recebe `audioKind` pronto no view-model. |
| V. Telas mockáveis | ✅ Cenas do mock da sala ganham os rótulos novos (`live` com som de janela/tela inteira). |
| VI. Segredos fora do navegador | ✅ Sem mudança. |

Sem violações. Re-check pós-design: igual.

## Project Structure

### Documentation (this feature)

```text
specs/002-audio-sem-discord/
├── spec.md
├── plan.md                    # este arquivo
├── research.md                # R1–R6
├── data-model.md              # AudioKind + tabela de decisão
├── quickstart.md              # testes automáticos + roteiro manual com Discord
├── contracts/signaling-delta.md
├── checklists/requirements.md
└── tasks.md                   # /speckit-tasks
```

### Source Code (repository root)

```text
src/
├── shared/protocol.ts          # LiveInfo.audio: AudioKind
├── worker/validate.ts          # aceita AudioKind e boolean legado (converte)
├── lib/
│   ├── audio-source.ts         # NOVO: decideAudio(wanted, surface, label) → {kind, drop, warn}
│   ├── capture.ts              # systemAudio:'exclude', windowAudio:'window'; aplica decideAudio; reavalia em configurationchange
│   ├── types.ts                # MyBroadcastView.audioKind/audioReason
│   └── stores/room.svelte.ts   # anuncia kind; toast FR-208; subtítulo de quem assiste
├── ui/components/GoLiveModal.svelte   # texto FR-206
├── ui/pages/RoomPage.svelte           # card FR-207
└── mocks/room.ts                      # cenas com os rótulos novos
test/
├── lib/audio-source.test.ts    # tabela do data-model
└── worker/room.test.ts         # contrato do `live` (novo + legado)
```

**Structure Decision**: mantém a estrutura da 001 (SPA + Worker num repositório só). Só entra um módulo novo (`audio-source.ts`), puro, para a regra ficar testável sem navegador.

## Riscos

| Risco | Mitigação |
|---|---|
| O navegador ignora `windowAudio` e manda o som do sistema com a janela | Heurística pelo rótulo da faixa (R3), confirmada no teste manual T-Q1. Se não der para detectar, a janela continua dependendo da dica (Chrome 141+ honra). |
| A pessoa perde o som ao usar tela inteira | Texto no modal antes e toast com a dica depois (SC-203). |
| Abas abertas antes do deploy mandam `audio: boolean` | Worker converte (contrato). |

## Complexity Tracking

Nenhuma violação a justificar.
