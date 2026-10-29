# Implementation Plan: Seletor de tela

**Spec**: [spec.md](./spec.md) | **Date**: 2026-10-28

## Summary

O modal "Transmitir" ganha o layout do `SourcePicker` do design. A aba escolhida (Janelas / Telas
inteiras) vira a dica `displaySurface` do `getDisplayMedia`, e a regra de som da 002 decide o que sai:
janela → som só dela (`windowAudio: 'window'`), aba → som da aba, tela inteira → nada
(`systemAudio: 'exclude'` + rede de segurança que descarta a faixa antes da malha).

## Technical Context

Igual à 002: TypeScript 6, Svelte 5, `getDisplayMedia`, Worker + DO. Sem dependência nova.

## Constitution Check

| Princípio | Situação |
|---|---|
| I. Simplicidade | ✅ Continua uma etapa no app + o seletor nativo. Nada de voz/chat. |
| II. Sem servidor de mídia | ✅ |
| III. Efêmero | ✅ A aba escolhida fica só no `localStorage` (preferência). |
| IV. Camada visual separada | ✅ Regra em `src/lib/audio-source.ts` + `capture.ts`; o modal só muda `GoLivePrefs`. |
| V. Telas mockáveis | ✅ Cenas `modal` (A1), `modalscreen` (A2), `modalmute` (A3), `livescreen`. |
| VI. Segredos fora do navegador | ✅ |

## Mudanças

```text
src/shared/protocol.ts            AudioKind; LiveInfo.audio e `live` usam AudioKind
src/worker/validate.ts            aceita AudioKind e boolean legado (converte)
src/lib/audio-source.ts           NOVO: decideAudio + audioLabelText (puro)
src/lib/capture.ts                displaySurface, systemAudio:'exclude', windowAudio:'window', applyAudioRule
src/lib/types.ts                  ShareSurface; GoLivePrefs.surface; MyBroadcastView.audioKind/audioReason
src/lib/prefs.ts                  lembra a aba
src/lib/stores/room.svelte.ts     aplica a regra antes da malha; reavalia em configurationchange/addtrack/removetrack; toast FR-208; subtítulo "· com som/sem som"
src/ui/components/GoLiveModal.svelte  layout SourcePicker
src/ui/pages/RoomPage.svelte      card com audioLabelText
src/mocks/room.ts, MockRoom.svelte cenas novas
test/lib/audio-source.test.ts     tabela do data-model da 002
test/worker/room.test.ts          contrato do `live` (novo + legado + 'system' recusado)
```

## Riscos

| Risco | Mitigação |
|---|---|
| WebView2/Chrome ignoram `displaySurface` | É só a dica de qual aba abre; a regra de som vale para o que vier. |
| `windowAudio` não suportado (navegador antigo) | Janela vai sem som (FR-204), nunca com o som do sistema. |
| Rótulo do som de janela parecido com "System" | Seria descartado (seguro). Confirmar no teste manual T019 da 002. |
| A grade de miniaturas do design | Fora do alcance da plataforma (spec, análise de viabilidade). |
