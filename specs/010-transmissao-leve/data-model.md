# Data Model: Transmissão leve (010)

Tudo em memória da página. Nada é gravado.

## Degrau de qualidade (`src/lib/quality.ts`, puro)

```text
stepDown(q):  1080p60 → 1080p30 → 720p30;  720p60 → 720p30;  720p30 → 720p30
stepUp(q, chosen): o inverso, nunca acima de `chosen`
```

`effectiveQuality(chosen, fullViewers, cpuSteps)`: aplica a regra de espectadores que já existe e depois `cpuSteps` vezes `stepDown`. Vale o mais baixo. O `maxBitrate` vem da tabela `BITRATE` da qualidade final, com o teto de espectadores.

## CpuGovernor (`src/lib/cpu-governor.ts`, puro, com testes)

| Campo | Tipo | Regra |
|---|---|---|
| `steps` | 0..2 | degraus baixados por CPU |
| `limitedFor` | segundos | segundos seguidos com alguma conexão em tela cheia `qualityLimitationReason === 'cpu'` |
| `freeFor` | segundos | segundos seguidos sem nenhuma em `'cpu'` |

```text
tick(limited: boolean, canGoDown: boolean, canGoUp: boolean) -> 'down' | 'up' | null
  limited  → freeFor = 0; limitedFor++; se limitedFor ≥ 10 e canGoDown → steps++, limitedFor = 0, 'down'
  !limited → limitedFor = 0; freeFor++; se freeFor ≥ 60 e canGoUp e steps > 0 → steps--, freeFor = 0, 'up'
reset() → steps = 0, contadores = 0   (troca de qualidade na mão, fim da transmissão)
```

Sem conexões em tela cheia: `tick` não é chamado (os contadores ficam parados).

## Som da ponte nativa (`src/lib/native-capture.ts`)

| Nome | Valor |
|---|---|
| Pedaço | 1.920 quadros (40 ms) do Rust; a página aceita qualquer tamanho |
| Atraso máximo | 200 ms → descarta |
| Fila | `audioWriter.desiredSize <= 0` → descarta |
