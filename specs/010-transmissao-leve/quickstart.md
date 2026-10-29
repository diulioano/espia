# Quickstart: Transmissão leve (010)

## Automático

```powershell
npm run check; npm run build; npm test     # quality (degraus), cpu-governor, som (descarte)
cd src-tauri; cargo test                   # tamanho do pedaço de som, pular quadro igual
```

## Diagnóstico (R1, scratchpad, build de debug)

```powershell
$env:ESPIA_BROWSER_ARGS = '--remote-debugging-port=9222'; npm run desktop:dev
# outro terminal: node <scratchpad>/diag-010.mjs   (heap, métricas, trace, getStats)
```

Antes de corrigir: anotar no `relatorio.md` o que cresce no heap, a divisão da CPU e o `bytesSent` por conexão.

## Roteiro manual (app instalado, site publicado)

Base (009): com som, a página cresce 12–14 MB/min; CPU 27–30 %; upload com pico de 45 Mbps (coluna de rede = envio + recebimento).

| # | Passos | Esperado |
|---|---|---|
| T1 | Transmitir uma janela **com som** (nativo, 1080p60) para 1 pessoa por 15 min; `npm run memoria` a cada minuto | crescimento médio ≤ 3 MB/min do minuto 2 ao 15 (SC-1001) |
| T2 | Em T1, quem assiste compara o som com a imagem no começo e no fim | sem atraso que cresce (SC-1002) |
| T3 | 3 pessoas, 1080p60, 5 min, com gente entrando, saindo e trocando miniatura/tela cheia; log `[espia] envio` de 1 s | nenhum intervalo de 5 s acima de 21 Mbps de envio (SC-1003) |
| T4 | Igual à medição de base (3 pessoas, 1080p60, nativo); CPU da página no Gerenciador | ≤ ~22 % (SC-1004) |
| T5 | Transmitir 1080p60 e abrir algo pesado no PC (ou o jogo mais pesado) | o indicador de qualidade mostra a descida em ≤ 15 s, **sem aviso**; ao fechar o programa pesado, volta em ≤ 75 s (SC-1005) |
| T6 | Trocar a qualidade na mão durante T5 | a escolha nova vira o teto; o ajuste recomeça dela |

Números no `relatorio.md` da feature.
