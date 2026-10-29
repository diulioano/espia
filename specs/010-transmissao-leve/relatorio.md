# Relatório: Transmissão leve (010)

Medições no build de debug (`scratchpad/diag-010.mjs`): o app transmite só a janela de teste "espia-som-teste" (tom de 440 Hz) pela captura nativa, 1080p, com 1 Edge headless assistindo. A memória é a da página do motor (`memoria.ps1`). "Sem DevTools" = o CDP é desconectado depois de começar a transmitir, porque o agente de rede do DevTools guarda cada pedido e infla a medida.

## Diagnóstico (T1001–T1002)

| Medida | Resultado |
|---|---|
| Com som, DevTools ligado | página 208 → 349 MB em ~3 min (~50 MB/min); heap do JS 9 → 28 MB |
| Sem som, DevTools ligado | estável (~355 MB) |
| Fotos do heap (2 min, com som) | +12.618 `blink::NetworkResourcesData::ResourceData` (~105/s = 1 por pedaço de som): o registro do DevTools de cada pedido de busca do canal do Tauri |
| Com som, **sem DevTools** (app 0.1.6) | ~8 MB/min (163 → 180 MB em 2 min) |

Causa: no Tauri 2.12, cada mensagem de um canal vira um script executado na página (`eval`). Bytes ≥ 1 KB também viram um pedido de busca (`fetch`). Com 100 pedaços de som por segundo, a página cresce. O vídeo (60 avisos de quadro por segundo por `eval`) explica o resíduo de 2–3 MB/min sem som.

## Correção (US1)

| Tentativa | Com som, sem DevTools |
|---|---|
| App 0.1.6 (bytes crus → `eval` + `fetch` por pedaço) | ~8 MB/min |
| Base64 em JSON (só `eval`, ~5 KB por script) | **~70 MB/min** (pior: os scripts são guardados) |
| **Canal de mensagens do WebView2** (`PostWebMessageAsJson`), som e avisos de quadro | **estável ~95–100 MB em 4 min**, também com a janela mudando a cada quadro |

Som conferido: quem assiste ouve o mesmo nível de quem transmite (RMS 0,0213 nos dois). Vídeo chega (1080p30).
