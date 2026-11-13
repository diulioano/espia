# Contrato: escolha e troca do caminho de compressão (R11, R12)

**Feature**: [spec.md](../spec.md) · **Modelo**: [data-model.md](../data-model.md) · **Decisões**: [research.md](../research.md)

Quem comprime a transmissão, placa de vídeo ou processador, é escolhido sozinho pelo app, por transmissão. A pessoa nunca escolhe, nunca é perguntada e nunca lê um aviso técnico (FR-012 a FR-014). A escolha e a troca são uma máquina de estados pura em `app/crates/espia-core/src/tiers.rs`. Quem lê o Windows e a placa é `espia-media/src/encode/`; a máquina só recebe fatos ("não achou", "não iniciou", "erro", "quadro saiu", "passou 1 s sem quadro").

## Situações e o caminho

| Situação | Caminho |
|---|---|
| Achou o codificador de H.264 por hardware e ele iniciou | placa |
| Não achou codificador por hardware | processador, desde o começo |
| Achou, mas ele não iniciou | processador, desde o começo |
| Placa ocupada por outro programa que grava ou transmite e recusa mais uma compressão | cai como "não iniciou" ou, se já estava transmitindo, como erro (ver gatilhos) |
| Duas placas (notebook): a janela capturada está numa delas | a placa onde está a janela (spec, casos de borda) |
| Placa funcionando, e já está no caminho da placa | placa, sem troca |
| Computador de outra marca de placa (AMD, Intel) em que o codificador do Windows não serve | processador, até haver solução específica (R2). Não confirmado: depende de testar cada marca |

## Gatilhos de troca no meio da transmissão

Só existe uma direção: da placa para o processador.

| Gatilho | Regra |
|---|---|
| Erros de compressão | **três erros seguidos**. Um quadro que sai bem zera a contagem. |
| Falta de quadro | **nenhum quadro comprimido em 2 segundos**, contados só enquanto alguma qualidade está ativa (sem ninguém recebendo, nada é comprimido, então não há o que esperar; R18) |

O caminho pelo processador é o que o G1 já provou: quadros crus e o codificador de software da biblioteca, um por espectador. Não há teto próprio: vale a qualidade escolhida, e o ajuste automático por processador (regra da 010) baixa a qualidade se o computador não der conta (R18).

Depois de trocar, a transmissão fica no processador até acabar. A transmissão seguinte tenta a placa de novo (R18 de [research.md](../research.md)).

## O que a pessoa vê

- O indicador de qualidade que já existe, mostrando a qualidade efetiva. No caminho pelo processador ele pode baixar sozinho, pelo ajuste automático por processador.
- No Diagnóstico de mídia (só neste aparelho), uma linha **"Compressão: placa de vídeo"** ou **"Compressão: processador"**, com o nome do codificador (R12, FR-018).
- Mais nada. Nenhum aviso, nenhuma pergunta, nenhuma opção ("usar placa de vídeo" foi rejeitada na R12).

## Plano B da troca

Se a troca do canal de repasse para o codificador de software, sem renegociar, não funcionar (**não confirmado**, G5 item g): a transmissão é parada e reaberta sozinha, com pausa de até 2 segundos, sem a pessoa clicar em nada.

## O que NÃO pode acontecer

| Proibido | Por quê |
|---|---|
| Pergunta nova ao começar ou durante a transmissão | FR-014, constituição I |
| Opção nova sobre placa de vídeo ou compressão | FR-014, R12 |
| Termo técnico na tela (nome de codificador fora do painel de diagnóstico, "placa", "hardware", "software") | FR-014 |
| Transmissão que cai e não volta | FR-013: a transmissão continua pelo outro caminho |
| A pessoa precisar clicar em Transmitir de novo | FR-013 |
| Espectador que perde o som por causa da troca | FR-016 |
| Qualquer envio a quem não clicou em Assistir, durante ou depois da troca | FR-008 |

## Máquina de estados

```text
escolhendo
  evento "iniciou"           -> placa
  evento "não achou"         -> processador
  evento "não iniciou"       -> processador

placa
  evento "erro"              -> erros seguidos + 1
                                 se erros seguidos = 3 -> processador
  evento "quadro saiu"       -> erros seguidos = 0; tempo sem quadro = 0
  evento "1 s sem quadro"    -> tempo sem quadro + 1 (só com qualidade ativa)
                                 se tempo sem quadro = 2 -> processador
  evento "sem qualidade ativa" -> tempo sem quadro = 0 (não conta)

processador
  qualquer evento            -> processador
```

## Casos de teste da máquina de estados

| Estado | Eventos | Estado final |
|---|---|---|
| escolhendo | iniciou | placa |
| escolhendo | não achou | processador |
| escolhendo | não iniciou | processador |
| placa | erro, erro | placa (2 erros) |
| placa | erro, erro, erro | processador |
| placa | erro, erro, quadro saiu, erro, erro | placa (a contagem zerou) |
| placa | erro, quadro saiu, erro, quadro saiu, erro | placa |
| placa | 1 s sem quadro | placa |
| placa | 1 s sem quadro, 1 s sem quadro | processador (2 s) |
| placa | 1 s sem quadro, quadro saiu, 1 s sem quadro | placa (zerou) |
| placa | sem qualidade ativa por 10 s | placa (não conta) |
| placa | sem qualidade ativa, depois qualidade ativa e 1 s sem quadro | placa |
| processador | iniciou, quadro saiu, erro, erro, erro | processador (não volta) |
| processador | qualquer sequência | processador |

Casos de teste da qualidade no caminho pelo processador (sem teto próprio):

| Entrada | Saída |
|---|---|
| escolhida 1080p60, 1 a 4 `Full`, caminho processador | 1080p60 (a escolhida) |
| escolhida 720p30, caminho processador | 720p30 |
| escolhida 1080p60, caminho placa | qualidade efetiva de `effective_quality` |
| escolhida 1080p60, caminho processador, processador limitado por 10 s | 1080p30 (um degrau a menos) |

Casos que dependem de placa e de rede (não são teste automático):

| Caso | Onde é visto |
|---|---|
| trocar para o processador mantém a transmissão sem a pessoa fazer nada | G5 item (g) |
| a pausa na troca não passa de 2 s | G5 item (g), plano B |
| computador sem placa capaz transmite em pelo menos 720p30 em 100% das tentativas | SC-010, história 6 |
| outra marca de placa | conferência com o grupo |
