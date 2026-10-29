---
description: "Tarefas da 004: seletor de tela (janela/tela inteira, com ou sem som)"
---

# Tasks: Seletor de tela

Implementada junto com a [002](../002-audio-sem-discord/tasks.md) (T001–T017 de lá).

- [x] T401 `GoLivePrefs.surface: 'window' | 'monitor'` em `src/lib/types.ts`; lembrar em `src/lib/prefs.ts` (padrão `'window'`)
- [x] T402 `startCapture`: `video.displaySurface = prefs.surface`; tela inteira não pede áudio
- [x] T403 `GoLiveModal.svelte` no layout do `SourcePicker`: abas, cartão da escolha com selo com/sem som, faixa com som/resolução/FPS, rodapé com dica e "Transmitir" (FR-401..406)
- [x] T404 Tela inteira trava o som e explica; volta para Janelas com a escolha anterior (FR-403/404)
- [x] T405 Mocks: `modal` (A1), `modalscreen` (A2), `modalmute` (A3), `livescreen`; `MockRoom` simula o som
- [x] T406 `npm run check` + `npm test` (58 testes) verdes
- [x] T407 E2E (Edge headless, só aba de teste pelo título, cor conferida): janela+som → 1 faixa e "com som"; sem som → 0 faixas; tela inteira → 0 faixas; preferências lembradas; sem rolagem em 390px
- [ ] T408 Publicar o site (`npm run deploy`). **O app desktop carrega o site publicado**: sem isso o seletor novo não aparece nem no app.
- [ ] T409 Teste manual com o Discord ligado (roteiro M1–M5 da 002, quickstart)
- [ ] T410 (futuro, opcional) Som do PC inteiro **menos o Discord** no app desktop via loopback de processo do Windows (`PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE`), injetado como faixa no WebView. Só se a galera sentir falta do som com tela inteira.
