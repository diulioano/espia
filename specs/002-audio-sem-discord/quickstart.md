# Quickstart — validar a 002

## Automático

```bash
npm run check   # svelte-check + tsc (app e Worker)
npm test        # inclui a tabela de decisão (data-model.md) e o contrato do `live`
```

E2E (Playwright + Edge instalado, scratchpad): captura **só abas de teste**, escolhidas pelo título
(`--auto-select-tab-capture-source-by-title`). Confere a cor do vídeo recebido antes de qualquer
captura de tela. Não usar `--use-fake-ui-for-media-stream`: ele captura a tela de verdade.
- Aba de teste com som → quem assiste recebe faixa de áudio; subtítulo "· com som"; card "com som da aba".
- "Compartilhar áudio" desligado → nenhuma faixa de áudio enviada; "sem som".

O Edge sem janela entrega o som da aba mudo (sem dispositivo de saída). Por isso o **conteúdo** do som só se valida no teste manual.

## Manual (o que importa: SC-201..204)

Pré-requisito: PC com Chrome/Edge atual no Windows, numa chamada de voz do Discord com alguém falando; um jogo ou vídeo tocando numa janela; um segundo PC (ou celular) assistindo em https://espia.espia.workers.dev.

| # | Quem transmite escolhe | Esperado em quem assiste (tirando o mudo) | Esperado em quem transmite |
|---|---|---|---|
| M1 | **Janela** do jogo, áudio ligado | Som do jogo; **nenhuma** voz do Discord | Card "com som da janela" |
| M2 | **Aba** com YouTube, áudio ligado | Som do vídeo; nenhuma voz do Discord | Card "com som da aba" |
| M3 | **Tela inteira**, áudio ligado | Nenhum som | Card "sem som (tela inteira)" + toast com a dica |
| M4 | Janela, áudio **desligado** no modal | Nenhum som | Card "sem som", sem toast |
| M5 | Aba, depois "compartilhar esta aba" para outra aba | Som da aba nova | Card continua "com som da aba" |

**T-Q1 (pesquisa R3):** no M1, anotar o rótulo da faixa de áudio da janela (console: `[espia] áudio:` vai logar superfície e rótulo). Se o rótulo for genérico de sistema, ajustar a heurística.
