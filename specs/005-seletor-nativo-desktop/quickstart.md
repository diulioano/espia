# Quickstart — validar a 005

## Pré-requisitos

- Windows 11 (e, se der, um Windows 10 22H2 para conferir a borda amarela e o loopback).
- `npm ci`, Rust 1.90+.
- O site com a 005 publicado (`npm run deploy`) **ou** o app de debug apontando para o Worker local:
  ```powershell
  npm run dev:worker   # outro terminal
  $env:ESPIA_START = "http://127.0.0.1:8787/"; npm run desktop:dev
  ```

## Automático

```powershell
npm run check; npm test          # regra de som com 'pc', ponte falsa, Worker aceitando 'pc'
cd src-tauri; cargo test         # filtro de janelas, raízes do Discord, AudioPlan, tamanhos
```

## Mocks (visual, sem app)

Abra `/?mock=room&scene=pickerwindows`, `pickerscreens`, `pickerempty` e `pickerfallback`. Compare com `tela-esperada.png`: contagem nas abas, "Prévias atualizam a cada 2s", ✓ no cartão escolhido, selo "tocando som" e o texto do som citando a janela.

## Manual no app (roteiro N1–N8)

| # | Passos | Esperado |
|---|---|---|
| N1 | Abra o Hollow Knight (ou outro jogo), um vídeo no Firefox e o VS Code. Clique em **Transmitir**. | A grade mostra as três janelas com miniatura, programa e ícone. O Espia não aparece. "Janelas N" bate com a grade. |
| N2 | Espere 4 s. Feche o VS Code. | As miniaturas mudam. O cartão do VS Code some em até 2 s. |
| N3 | Escolha o vídeo do Firefox com som ligado e clique em **Transmitir**. | **Nenhum** seletor do Windows aparece. Numa segunda máquina, a sala mostra o vídeo com som e "· com som". |
| N4 | Durante uma chamada do Discord, repita o N3. | Quem assiste ouve o vídeo e **nenhuma** voz do Discord (SC-503). |
| N5 | Aba **Telas inteiras**, som ligado, chamada do Discord ativa, música tocando no Spotify. | Quem assiste ouve o Spotify e **não** ouve o Discord (SC-503b). O card mostra "com som do PC (sem Discord)". |
| N6 | Com o N5 no ar, feche e reabra o Discord e entre de novo na chamada. | O Discord continua fora do som. |
| N7 | Abra o Discord **e** o Discord PTB e transmita a tela inteira com som. | A transmissão vai sem som, com o aviso "Mais de um Discord aberto…". |
| N8 | Com uma janela no ar, feche essa janela. | A transmissão acaba sozinha, como hoje. |

## Fallback e site

- **No Chrome** (sem o app): o seletor fica idêntico ao da v0.1.2, com o cartão "Você escolhe a janela no próximo passo" (US4).
- **App v0.1.2 já instalado + site novo**: o mesmo fluxo da v0.1.2, sem erro (FR-511).

## Tamanho

Rode `npm run desktop:build` e compare `Espia_0.1.3_x64-setup.exe` com o da 0.1.2 (1,13 MB). O limite é +5 MB (SC-506).

## Console

`[espia] captura nativa:` mostra a fonte, o tamanho, o fps, a `AudioPlan` e os quadros descartados por segundo.
