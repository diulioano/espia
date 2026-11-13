# Contrato: medição do processador e o que registrar (R15)

**Feature**: [spec.md](../spec.md) · **Modelo**: [data-model.md](../data-model.md) · **Decisões**: [research.md](../research.md)

O script `scripts/processador.ps1` ainda não existe. Este contrato diz o que ele precisa fazer e o que toda medição registra em `relatorio.md`. Ele segue o estilo de `scripts/memoria.ps1`: PowerShell 5.1, só lê, não muda nada no computador, imprime em português.

## Por que núcleos, e por que conferir quem assiste

A porcentagem do processador depende de quantos núcleos a máquina tem: 2,2% numa máquina de 24 núcleos lógicos não é o mesmo que 2,2% numa de 8. Por isso a medição é em **núcleos** (porcentagem da máquina × núcleos lógicos ÷ 100), em média de 60 segundos, porque uma leitura de um instante engana.

Conferimos quem está **assistindo** porque quem não assiste não recebe vídeo e não custa compressão. Na medição do app de hoje com "3 pessoas" (G0), parte delas podia não estar assistindo, e o 0,53 núcleo pode ser de 1 ou 2 espectadores (ressalva do dono, em research.md, "Contexto").

## Parâmetros

Nomes propostos, no estilo de `memoria.ps1` (`-Processo`, `-Nativo`, `-Interval`). Podem mudar quando o script for escrito.

| Parâmetro | Padrão | O que faz |
|---|---|---|
| (nenhum) | | Mede a **árvore** de processos do `Espia.exe`: o app e os processos do motor de página (WebView2) que ele abriu, como `memoria.ps1` sem `-Nativo`. É o caso do app de hoje (0.1.10). |
| `-Nativo` | desligado | Mede só o processo, sem descer para os filhos. É o caso do app nativo e da sonda. |
| `-Processo <nome>` | `Espia` | Nome do processo sem `.exe`. Vale no lugar do padrão. |
| `-Duracao <s>` | 60 | Duração da medição, em segundos. |
| `-Intervalo <s>` | 1 (R18) | De quanto em quanto tempo lê o processador. |

Se não houver processo com o nome pedido, o script avisa ("... não está rodando") e sai com erro, como `memoria.ps1`.

## O que imprime

Uma linha de resumo, no fim:

| Campo | O que é |
|---|---|
| Núcleos médios | média das leituras, em núcleos (porcentagem da máquina × núcleos lógicos ÷ 100) |
| Pico | a maior leitura, em núcleos |
| Porcentagem da máquina | a média, em % da máquina inteira |
| Núcleos lógicos | número usado na conta (lido do computador) |
| Amostras | quantas leituras entraram na média |

Exemplo do formato (valores inventados, só para mostrar a forma):

```text
Espia (árvore): 0,53 núcleo médio, pico 0,81 | 2,2% da máquina | 24 núcleos lógicos | 60 amostras
```

Pontos que a escrita do script define, conferindo contra o Gerenciador de Tarefas na primeira medição:

- Qual contador do Windows usa e como o normaliza (o do Gerenciador de Tarefas é em porcentagem da máquina).
- O que acontece se um processo da árvore nasce ou morre durante a medição.

## Exemplos de linha de comando

Antes de rodar: a transmissão já precisa estar em andamento e com as pessoas assistindo (veja "O que toda medição registra").

| Para medir | Comando |
|---|---|
| App de hoje (0.1.10), árvore do `Espia.exe` | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/processador.ps1` |
| App de hoje, 30 s em vez de 60 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/processador.ps1 -Duracao 30` |
| App nativo, um processo só | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/processador.ps1 -Nativo` |
| Sonda (um processo) | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/processador.ps1 -Processo encode-probe -Nativo` |

O nome do executável da sonda (`encode-probe`) é o da pasta `app/spikes/encode-probe/` e **não está confirmado** até a sonda existir. A sonda do G1 é `webrtc-probe`. Se o script ganhar um atalho em `package.json` (como `npm run memoria`), isso ainda não está decidido.

## O que TODA medição registra em `relatorio.md`

`relatorio.md` é criado na primeira medição. Cada medição ganha uma linha ou bloco com:

| Campo | O que anotar |
|---|---|
| Data | AAAA-MM-DD |
| Máquina | processador, núcleos lógicos, placa de vídeo |
| Jogo | nome, ou "sem jogo" |
| Qualidade | resolução e quadros por segundo da transmissão. Anotar a **efetiva** (com 5 ou mais pessoas em tela cheia ela cai) |
| Pessoas assistindo | quantas estavam **assistindo naquele momento** |
| Como isso foi conferido | por exemplo, cada pessoa confirmou que estava com a tela aberta e não escondida, e o Diagnóstico de mídia mostrava vídeo enviado com taxa maior que zero para cada uma. Sem conferência, a medição não serve de comparação |
| FPS do jogo sem transmitir | contador do próprio jogo |
| FPS do jogo transmitindo | contador do próprio jogo |
| Aplicativo medido | app de hoje, app nativo ou sonda |
| Núcleos | média de 60 s e pico, do script |
| Atraso entre entregar o quadro e receber o comprimido | só nas medições do G4 (item f) |
| Passou ou falhou | para cada item do gate, quando houver |

Regras:

- A comparação que vale é **lado a lado, na mesma sessão**, na mesma máquina e com o mesmo jogo, com o mesmo número de pessoas assistindo nas duas pontas (FR-017).
- A base de 2026-11-12 (0,53 núcleo, LoL, 300 para 200 a 220 FPS) é uma leitura de um instante com "até 3 pessoas". Serve de ordem de grandeza, não de base final.
- Os critérios de comparação (SC-002: no máximo metade do processador do app de hoje; SC-003: perda de FPS não maior que a do app de hoje; SC-004: crescimento de no máximo 25% entre 1 e 7 pessoas) usam sempre núcleos.
