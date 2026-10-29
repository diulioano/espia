# Implementation Plan: Seletor de tela nativo no app desktop

**Branch**: `005-seletor-nativo-desktop` (trabalho direto na `main`, como na 004) | **Date**: 2026-10-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-seletor-nativo-desktop/spec.md`

## Summary

No app desktop, o seletor "Escolha o que compartilhar" fica igual ao design (`tela-esperada.png`). Cada aba mostra uma grade de janelas ou monitores com miniaturas que se atualizam a cada 2 s, e as janelas com som ganham o selo "tocando som". Depois do clique em Transmitir, a captura começa sem o seletor do Windows.

Como o WebView2 não deixa a página escolher a fonte do `getDisplayMedia` (research R1), quem lista e captura é o **app em Rust**:

- **Listagem**: `EnumWindows`/`EnumDisplayMonitors`, miniaturas por `PrintWindow`/`BitBlt` + JPEG pelo WIC.
- **Vídeo**: Windows.Graphics.Capture, reduzido e convertido para NV12 na GPU.
- **Entrega dos quadros**: buffers compartilhados do WebView2 (anel de 3 slots) + aviso por `Channel` do Tauri. A página monta um `MediaStreamTrackGenerator`, e o track entra na mesma malha P2P de hoje.
- **Som de janela**: loopback de processo do WASAPI, incluindo só a árvore do programa da janela.
- **Som de tela inteira** (Q1: B): o PC inteiro **excluindo** a árvore do Discord. Com mais de um Discord aberto, ou num Windows sem suporte, vai sem som e com aviso.

No site, nada muda. A ponte só existe para a origem do Espia, dentro do app, e exige a emenda 1.2.0 da constituição.

## Technical Context

**Language/Version**: Rust 1.90+ (edição 2024) no app; TypeScript 6 + Svelte 5 no site

**Primary Dependencies**:
- **App**: Tauri 2.12 (comandos + `ipc::Channel` + capability remota), `windows` 0.62 com novas features (`Graphics_Capture`, `Graphics_DirectX_Direct3D11`, `Win32_Graphics_{Direct3D11,Dxgi,Gdi,Dwm,Imaging}`, `Win32_Media_Audio`, `Win32_System_{Com,Threading,ProcessStatus,WinRT_Graphics_Capture,WinRT_Direct3D11}`, `Win32_Storage_FileSystem`), `webview2-com` 0.39 (`ICoreWebView2Environment12`, `ICoreWebView2_17`) e `base64`.
- **Site**: `@tauri-apps/api` (só `core`).

**Storage**: `localStorage` (prefs da 004 + `lastSourceKey`). Nada no servidor.

**Testing**: `cargo test` (funções puras), Vitest (ponte falsa, regra de som, Worker com `'pc'`), mocks do seletor e roteiro manual N1–N8 ([quickstart.md](./quickstart.md))

**Target Platform**: Windows 10 1903+ / 11 x64 com WebView2 ≥ 1.0.1661. O som por processo depende de o loopback de processo funcionar (2004+ na prática); sem ele, a transmissão vai sem som e com aviso.

**Project Type**: SPA + Worker (existente) + casca desktop `src-tauri/` (cresce: captura nativa)

**Performance Goals**:
- grade visível em ≤ 1 s;
- prévias a cada 2 s ±0,5 s;
- 1080p60 sem quadros perdidos por cópia num PC médio;
- custo extra da ponte ≤ 10% de CPU a 1080p60.

**Constraints**:
- instalador: referência de ~+5 MB, flexível (SC-506, decisão de 2026-10-29);
- miniaturas e quadros nunca saem do PC;
- ponte só para a origem do site;
- nunca enviar som com o Discord junto;
- nenhuma regressão no site.

**Scale/Scope**:
- **Rust**: ~5 módulos novos (`capture/{mod,sources,thumbs,video,audio,discord}.rs`).
- **Site**: 1 módulo de lógica (`native-capture.ts`) + a evolução do `GoLiveModal` e do `room.svelte.ts`.
- **Mocks**: 4 cenas.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio / restrição | Situação |
|---|---|
| I. Simplicidade (site sem instalar nada) | ✅ O site continua igual (US4). Só o app opcional ganha o seletor nativo. |
| II. Sem servidor de mídia | ✅ A mídia continua P2P. Só muda de onde vem o track de quem transmite. |
| III. Efêmero e sem contas | ✅ Nada novo no servidor. As prefs ganham só `lastSourceKey` no `localStorage`. |
| IV. Camada visual separada | ✅ A ponte e o `PickerState` vivem em `src/lib/native-capture.ts`. O `GoLiveModal` recebe o view-model pronto e emite eventos. |
| V. Telas mockáveis | ✅ Cenas `pickerwindows`, `pickerscreens`, `pickerempty` e `pickerfallback` com dados falsos (FR-515). |
| VI. Segredos fora do navegador | ✅ Sem segredos novos. A ponte é limitada por origem (R9). |
| Restrição técnica: "App desktop … nenhuma API nativa exposta à página" | ❌ **Violada** → emenda 1.2.0 abaixo. |

**Emenda proposta (MINOR → 1.2.0)**: a restrição do app desktop passa a ser "Tauri 2 + WebView2, só Windows, carrega o site publicado; expõe à página **apenas** a ponte de captura (`specs/005-seletor-nativo-desktop/contracts/native-capture.md`: listar fontes, miniaturas, iniciar/parar captura de vídeo e som), **apenas** para a origem do site, na janela principal. Qualquer outra API nativa exige emenda." Registrar no Sync Impact Report, com o motivo: o pedido da pessoa usuária de 2026-10-29.

**Re-check pós-design**: ✅ com a emenda aplicada. O contrato tem 5 comandos, uma origem e nenhum acesso a arquivos ou shell.

## Project Structure

### Documentation (this feature)

```text
specs/005-seletor-nativo-desktop/
├── spec.md
├── plan.md                    # este arquivo
├── research.md                # R1–R11
├── data-model.md              # CaptureSource, PickerState, AudioPlan, textos
├── quickstart.md              # automático, mocks, roteiro N1–N8, tamanho
├── contracts/
│   └── native-capture.md      # comandos, buffers compartilhados, áudio, AudioKind 'pc'
├── checklists/requirements.md
└── tasks.md                   # /speckit-tasks
```

### Source Code (repository root)

```text
src/
├── shared/protocol.ts                # AudioKind + 'pc'
├── worker/validate.ts                # aceita 'pc' (via AUDIO_KINDS)
├── lib/
│   ├── native-capture.ts             # NOVO: detecção, capture_list/polling, montagem do MediaStream
│   │                                 #       (NV12 → VideoFrame → MediaStreamTrackGenerator; PCM → AudioData)
│   ├── capture.ts                    # startCapture escolhe nativo × getDisplayMedia
│   ├── audio-source.ts               # textos do modo nativo, 'pc' no audioLabelText
│   ├── prefs.ts / types.ts           # lastSourceKey, PickerState
│   └── stores/room.svelte.ts         # picker no estado da sala; constrainCapture → capture_set_quality
├── ui/components/
│   ├── GoLiveModal.svelte            # grade (modo native) ou cartão "próximo passo" (modo browser)
│   └── SourceCard.svelte             # NOVO: cartão com miniatura, ícone, selo, ✓
└── mocks/room.ts, MockRoom.svelte    # cenas picker*

src-tauri/
├── Cargo.toml                        # features do windows, base64
├── build.rs                          # AppManifest::commands([...5 comandos])
├── capabilities/capture.json         # NOVO: remote.urls = origem do site; janela main
└── src/
    ├── lib.rs                        # registra os comandos e o estado da sessão
    └── capture/                      # NOVO
        ├── mod.rs                    # comandos Tauri, sessão, conferência da origem
        ├── sources.rs                # EnumWindows/Monitors, filtro (puro + testes), ícones
        ├── thumbs.rs                 # PrintWindow/BitBlt → WIC JPEG
        ├── video.rs                  # WGC → VideoProcessor NV12 → anel de buffers compartilhados
        ├── audio.rs                  # loopback de processo, AudioPlan (pura + testes), sessões "tocando"
        └── discord.rs                # raízes do Discord (puro + testes) + vigia de 1 s

test/lib/native-capture.test.ts       # NOVO (ponte falsa)
test/lib/audio-source.test.ts         # + 'pc'
test/worker/room.test.ts              # + live com 'pc'
.specify/memory/constitution.md       # emenda 1.2.0
```

**Structure Decision**: a mesma estrutura de antes (SPA + Worker + `src-tauri/`). A captura nativa fica isolada em `src-tauri/src/capture/`, e o lado do site em um único módulo de lógica, `src/lib/native-capture.ts`. A UI só troca o conteúdo do modal conforme `PickerState.mode`.

## Ordem de entrega (princípio V, mock → real)

1. Emenda 1.2.0 da constituição.
2. **Visual**: `SourceCard` + `GoLiveModal` modo `native` + 4 cenas de mock. Comparar com `tela-esperada.png`.
3. **Protocolo**: `'pc'` no `AudioKind` + Worker + testes.
4. **Rust**: listagem + miniaturas + selo (`capture_list`) e a capability. A grade passa a mostrar dados reais.
5. **Rust**: vídeo (WGC → anel) + `native-capture.ts` montando o track. Transmitir sem som.
6. **Rust**: som de janela (include) e som de tela (exclude Discord + vigia) + `AudioPlan`.
7. Qualidade no ar, fallback (FR-511), `lastSourceKey` (US5).
8. Build, medição do instalador, roteiro N1–N8, versão 0.1.3 e deploy.

## Riscos

| Risco | Mitigação |
|---|---|
| A cópia NV12 a 1080p60 pesar em PC fraco | Medir no passo 5. Plano B (R5): H.264 no Rust + WebCodecs. O passo 7 já força 720p30 quando há muitos espectadores (`quality.ts`). |
| `MediaStreamTrackGenerator` sair do Chromium/WebView2 | Hoje está disponível. Alternativa: `VideoTrackGenerator` num Worker (padrão W3C), com a mesma ponte. |
| Loopback de processo indisponível no Windows 10 de alguém | A transmissão vai sem som e com aviso. Nunca cai para o som do PC inteiro. |
| Jogo em tela cheia exclusiva sai preto | O aviso sugere o modo janela/sem bordas (edge case da spec). |
| A capability remota abrir mais do que o necessário | A lista fechada de 5 comandos no `build.rs`, a origem exata e a conferência da URL antes de postar buffers. Revisar com `/security-review`. |

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| A ponte nativa exposta à página (restrição técnica do app) | É o único jeito de ter a grade com miniaturas e capturar sem o seletor do Windows dentro do WebView2 (R1) | Electron: +100 MB e som do PC com o Discord junto. `--auto-select-desktop-capture-source`: vale para o processo inteiro e é fixado ao abrir. Manter o seletor do Windows: é o que a pessoa pediu para trocar. |
| Pipeline de vídeo próprio (WGC + buffers compartilhados) | O `getDisplayMedia` não aceita uma fonte escolhida fora do seletor nativo | WebRTC nativo em Rust duplicaria a malha inteira. H.264 + WebCodecs fica como plano B. |
