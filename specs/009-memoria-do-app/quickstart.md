# Quickstart: Memória do app (009)

## Automático

```powershell
npm run check; npm run build; npm test          # site: reportRoom e os avisos do store
cd src-tauri; cargo test                         # engine_action, tempos, prefs da 007
```

## Régua: `scripts/memoria.ps1`

```powershell
powershell -File scripts/memoria.ps1            # total do Espia + detalhe por tipo de processo
powershell -File scripts/memoria.ps1 -Watch     # uma linha a cada 5 s (ver a memória cair)
```

Mede a mesma coluna "Memória" do Gerenciador (conjunto de trabalho privado). Conta só a árvore do `Espia.exe`. O WhatsApp e a Pesquisa do Windows, que usam o mesmo motor, ficam de fora.

## Roteiro manual (app instalado, versão nova; site já publicado)

**Base** (prints de 2026-10-29, v0.1.5): numa sala, minimizado, sem transmitir = **~228 MB**. Anote também a base com a janela aberta.

| # | Passos | Esperado |
|---|---|---|
| M1 | Transmitir uma janela (nativo) 10 min com outra pessoa assistindo, parar pelo botão, esperar 30 s | total ≤ base + 50 MB (SC-901) |
| M2 | Repetir M1 com a câmera e com a tela pelo seletor do navegador; repetir **saindo da sala** no meio da transmissão | idem; o aviso de câmera/microfone do Windows some |
| M3 | 5 ciclos de transmitir 1 min e parar | variação ≤ 50 MB entre o 1º e o 5º (SC-902) |
| M4 | Assistir alguém 10 min e sair da sala | volta para perto da base |
| M5 | Na tela inicial, X; esperar 1 min. Repetir: numa sala, X, sair da sala pelo ícone→abrir→sair→X (ou a sala acabar escondido) | ≤ 40 MB, 0 processos do motor (SC-903) |
| M6 | Numa sala com alguém transmitindo com som: **minimizar**, esperar 1 min; depois X, esperar 1 min | continua na sala (o outro vê você na lista), **o som continua**, memória ≤ a da janela aberta (SC-903a) |
| M7 | Depois de M5: clicar no ícone; depois um link `espia://s/…`; depois o atalho. Também: X e reabrir antes de 30 s | volta em ≤ 2 s, na tela inicial / na sala do link (SC-903b); antes de 30 s, volta sem recarregar |
| M8 | Gerenciador, aba Processos, com a janela aberta | o motor do Espia agrupado com o castor ("WebView2 Manager"); WhatsApp e Pesquisa fora do grupo |
| M9 | Transmitindo: minimizar e X, 5 min cada, outra pessoa assiste. Depois, com o navegador **fechado**, clicar num link externo no Espia (abre o navegador) e escolher "Sair". Depois abrir de novo e "Finalizar tarefa" no Espia | sem queda (SC-906); 0 processos do Espia 5 s depois de cada saída; **o navegador continua aberto** (SC-905) |

Anote os números de cada linha no `relatorio.md` da feature.
