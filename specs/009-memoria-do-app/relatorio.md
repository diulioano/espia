# Relatório: Memória do app (009)

Medições com `scripts/memoria.ps1` (conjunto de trabalho privado da árvore do Espia, a coluna "Memória" do Gerenciador).

## Base (v0.1.5 instalado)

| Situação | Total | Fonte |
|---|---|---|
| Numa sala, minimizado, transmitindo | 243,3 MB | print do Gerenciador |
| Numa sala, minimizado, 2 min depois de parar | 228,3 MB | print do Gerenciador |
| Aberto (situação não anotada) | 395,7 MB (página 300, GPU 52, motor 18, utilitários 17, Espia.exe 8) | `memoria.ps1` |

## Build de debug (2026-10-29, automático)

| Teste | Resultado |
|---|---|
| T902: `destroy()` da janela 10 s depois de abrir | motor 7 → 0 processos em < 2 s; `espia.exe` vivo com 8,0 MB |
| Segunda abertura com `espia://s/k7f-q2x` sem janela | janela "Espia" recriada no mesmo processo; motor de volta em < 2 s; a segunda instância saiu |
| X (WM_CLOSE na janela do Espia) na tela inicial | janela escondida; regra `ReleaseAfter(30s)`; motor solto entre 30 e 35 s; **7,6 MB, 0 processos do motor** (SC-903 ✅) |

## App 0.1.6 instalado (pessoa usuária, 2026-10-29)

| Teste | Resultado |
|---|---|
| Transmitindo para 3 pessoas | pico de ~600 MB (motor) |
| Fechar no X e deixar na bandeja | ~30 MB (SC-903 ✅, ≤ 40 MB) |

### Transmitindo para 3 pessoas (captura nativa), página do motor (`memoria.ps1` a cada 5 s)

| Situação | Crescimento da página |
|---|---|
| Com som (15:06, 1 min) | 387 → 401 MB, ~14 MB/min, constante |
| Sem som (15:11–15:14, 3 min) | sobe e desce com a coleta de lixo; depois de um salto de ~75 MB (provável troca de quem assiste), ~2–3 MB/min |
| Com som de novo (15:16–15:19, 3,5 min) | média 374 → 411 MB, ~12 MB/min, constante |
| Parar a transmissão | a página caiu de ~400 para ~270 MB (o acumulado é solto ao parar) |

Conclusão: **vazamento ligado ao som da captura nativa** (~12–14 MB/min, ~750 MB/h). Fica para a feature 010. Ao parar, tudo é solto (US3 ok).

Ainda faltam (pessoa usuária): clique no ícone da bandeja para reabrir, M1–M4, M6 (som com memória baixa), M8 e M9.
