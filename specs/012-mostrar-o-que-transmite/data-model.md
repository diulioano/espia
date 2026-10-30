# Data Model: Mostrar o que cada pessoa está transmitindo

## Protocolo (src/shared/protocol.ts)

```ts
export const LIVE_LABEL_MAX = 60;

interface LiveInfo {
  resolution; fps; audio; source; startedAt;
  /** specs/012: "Hollow Knight", "Tela inteira", "Aba do navegador", "Câmera · C920". Ausente = cliente antigo. */
  label?: string;
}

ClientMessage 'live' += { label?: string }
```

Validação no servidor (`src/worker/validate.ts`): mesma limpeza do apelido, 1–60 caracteres; vazio ou inválido → sem `label` (o `live` continua valendo).

## Função pura (src/lib/live-label.ts)

```ts
type LabelInput =
  | { kind: 'native-window'; app: string }
  | { kind: 'native-monitor'; title: string; monitors: number }
  | { kind: 'browser'; surface: 'monitor' | 'window' | 'browser' | undefined }
  | { kind: 'camera'; title: string | undefined; facing?: 'user' | 'environment' };

liveLabel(input): string | undefined
```

| Entrada | Saída |
|---|---|
| native-window, app "Hollow Knight" | "Hollow Knight" |
| native-window, app "" | "Janela" |
| native-monitor, monitors 1 | "Tela inteira" |
| native-monitor, "Tela 2", monitors 2 | "Tela 2" |
| browser monitor / window / browser | "Tela inteira" / "Janela" / "Aba do navegador" |
| browser undefined | undefined |
| camera "HD Pro Webcam C920" | "Câmera · HD Pro Webcam C920" |
| camera sem título, facing user / environment | "Câmera frontal" / "Câmera traseira" |
| qualquer coisa > 60 | cortada em 60 com "…" |

## View-models (src/lib/types.ts)

- `StreamView.label?: string`
- `ParticipantView.liveLabel?: string`
- `MyBroadcastView.label?: string` (substitui o uso do `sourceTitle` no cartão "Você está ao vivo")
