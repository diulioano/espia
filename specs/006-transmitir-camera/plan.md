# Implementation Plan: Transmitir câmera ou placa de captura

**Branch**: `006-transmitir-camera` (trabalho direto na `main`, como nas anteriores) | **Date**: 2026-10-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/006-transmitir-camera/spec.md`

## Summary

O seletor ganha a aba **Câmeras**, no site, no app e no celular. A captura usa `getUserMedia` com o `deviceId` escolhido, e por isso **não precisa de nada nativo**: o app 0.1.3 já serve, basta publicar o site.

- **Permissão e prévias**: a permissão só é pedida no clique em "Permitir câmeras". Os dispositivos aparecem com prévia ao vivo (até 4 ao mesmo tempo).
- **Som (Q1: C)**: qualquer entrada, sugerida pelo mesmo `groupId`/nome do vídeo, com dois perfis. "Som de jogo" vai cru (placa) e "Som de voz" vai com cancelamento de eco (microfone). O som só existe junto da câmera.
- **Celular (Q3: B)**: só a aba Câmeras, frontal/traseira, em 720p30.
- **Protocolo**: o `live` ganha `source: 'camera'` e o `AudioKind` ganha `'device'`.
- **Fora (Q2: C)**: tela + câmera ao mesmo tempo.
- **Constituição**: emenda 1.3.0 (som de câmera permitido; celular transmite câmera).

## Technical Context

**Language/Version**: TypeScript 6 + Svelte 5 (site). Nada em Rust.

**Primary Dependencies**: as atuais. Sem dependências novas.

**Storage**: `localStorage` (nomes do último vídeo/som e perfil de som). Nada no servidor.

**Testing**: Vitest (`src/lib/devices.ts` puro + Worker), mocks das cenas novas, E2E no Edge com câmera **falsa** (`--use-fake-device-for-media-stream`) e roteiro manual C1–C7 com hardware.

**Target Platform**: Chrome/Edge desktop, app desktop (WebView2) e Chrome/Safari no celular (só câmera).

**Project Type**: SPA + Worker (existente).

**Performance Goals**:
- seletor com as prévias em ≤ 1,5 s depois da permissão;
- placa a 1080p60 com a mesma fluidez de uma janela de jogo (SC-602).

**Constraints**:
- câmera nunca liga sem clique (SC-603);
- nenhum som sem câmera (SC-604);
- prévias desligadas ao fechar o seletor;
- nada de captura real de tela nos testes.

**Scale/Scope**:
- 1 módulo de lógica novo (`devices.ts`);
- a evolução de `capture.ts`, `room.svelte.ts`, `GoLiveModal`/`SourceCard` e do protocolo;
- 7 cenas de mock.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio / restrição | Situação |
|---|---|
| I. Simplicidade / fora do escopo (voz) | ❌ → **emenda 1.3.0**: voz como recurso próprio continua fora, e o som de uma câmera transmitida (inclusive microfone) passa a ser permitido enquanto a câmera estiver no ar. |
| II. Sem servidor de mídia | ✅ A mesma malha P2P. |
| III. Efêmero e sem contas | ✅ Só preferências locais. |
| IV. Camada visual separada | ✅ A lógica de dispositivos fica em `src/lib/devices.ts` + store. A UI recebe `CameraView` pronto. |
| V. Telas mockáveis | ✅ 7 cenas novas (FR-610). |
| VI. Segredos fora do navegador | ✅ Nada novo. |
| Restrição: "transmitir: Chrome/Edge desktop" | ❌ → emenda 1.3.0: "câmera: também celular". |
| Restrição do app (ponte de captura só) | ✅ Nenhuma API nativa nova. A permissão de câmera é a do próprio WebView2. |

**Emenda proposta (MINOR → 1.3.0)**: as duas mudanças acima, com o motivo "pedido de 2026-10-29 (spec 006, Q1: C e Q3: B)".

**Re-check pós-design**: ✅ com a emenda aplicada.

## Project Structure

### Documentation (this feature)

```text
specs/006-transmitir-camera/
├── spec.md · plan.md · research.md (R1–R9) · data-model.md · quickstart.md
├── contracts/signaling-live.md
├── checklists/requirements.md
└── tasks.md              # /speckit-tasks
```

### Source Code (repository root)

```text
src/
├── shared/protocol.ts              # AudioKind + 'device'; live/LiveInfo + source
├── worker/validate.ts, room.ts     # aceita/repassa source (padrão 'screen')
├── lib/
│   ├── devices.ts                  # NOVO, puro: suggestAudio, audioRole, audioConstraints,
│   │                               #   isCaptureCard, previewBudget(4)
│   ├── camera.ts                   # NOVO: permissão, enumerateDevices, devicechange, prévias, startCamera()
│   ├── capture.ts                  # canShareScreen / canShareCamera
│   ├── audio-source.ts             # 'device' nos textos
│   ├── prefs.ts / types.ts         # surface 'camera', lastVideoLabel/AudioLabel/AudioRole, CameraView
│   └── stores/room.svelte.ts       # aba Câmeras: prévias, escolha, transmitir câmera, source no live
├── ui/components/
│   ├── GoLiveModal.svelte          # 3ª aba + painel de câmeras (permitir/lista/negada/vazia),
│   │                               #   seletor de entrada de som + Voz/Jogo + dica do fone
│   └── SourceCard.svelte           # prévia ao vivo (<video>) e estado "em uso"
├── ui/pages/RoomPage, MobileRoomPage, Stage/StreamTile  # "câmera de …"; botão no celular
└── mocks/room.ts, MockRoom.svelte  # 7 cenas

test/lib/devices.test.ts            # NOVO
test/worker/room.test.ts            # + source / 'device'
.specify/memory/constitution.md     # 1.3.0
```

**Structure Decision**: tudo no site, na mesma estrutura. Uma camada pura (`devices.ts`) concentra as decisões testáveis, e `camera.ts` fala com o navegador.

## Ordem de entrega (mock → real)

1. Emenda 1.3.0.
2. Protocolo (`source`, `'device'`) + Worker + testes.
3. `devices.ts` puro + testes.
4. Visual: a aba Câmeras + o seletor de som + as cenas de mock.
5. `camera.ts` + store: permissão, lista, prévias e transmitir com o som escolhido.
6. Celular: `canShareCamera`, a aba única, frontal/traseira, 720p30.
7. Quem assiste: "câmera de …". Prefs lembradas.
8. E2E com câmera falsa, deploy e roteiro C1–C7.

## Riscos

| Risco | Mitigação |
|---|---|
| Drivers de placa que não abrem duas vezes (prévia + transmissão) | Fechar as prévias antes de abrir para transmitir (R3). |
| Heurística voz × jogo errar | Padrão `voice` (seguro contra eco) + troca manual com um clique. |
| Webcam com microfone embutido sugerido como jogo | O `groupId` só sugere a entrada. O perfil vem do nome ("mic" → voz). |
| iOS Safari pausar a câmera em segundo plano | Tratado como fonte encerrada (R6). |
| Ouvir a pessoa duas vezes (Espia + Discord) | Dica fixa no seletor com entrada de voz. Mudo por pessoa para quem assiste. |

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Princípio I (voz fora do escopo) | A pessoa quer compartilhar a câmera com a própria voz (Q1: C) | Só o som da placa (Q1: B) foi recusado pela pessoa. A emenda limita a voz a "junto de uma câmera no ar". |
| Restrição "transmitir só no desktop" | Celular transmitindo a câmera (Q3: B) | Manter o celular só assistindo foi recusado (Q3: B). |
