# Data Model — Espia

## Entidades de domínio

### Room (Durable Object, só em memória)
| Campo | Tipo | Regra |
|---|---|---|
| `code` | string `xxx-xxx` (research R2) | gerado no **servidor** em `POST /api/rooms`; DO nomeado por `idFromName(code)` |
| `reservedUntil` | number \| null | reserva de 60 s após a criação, até o criador conectar |
| participantes | sockets aceitos + `serializeAttachment(PeerAttachment)` | máx. 10; ordem de entrada preservada; sobrevive à hibernação |
| `pending` | `Map<peerId, { attachment, timer }>` | quem caiu e ainda está nos 20 s de tolerância |
| `nextTone` | number | rodízio `deep, accent, sage, sand` |

A sala "existe" se tem ≥ 1 participante, alguém em tolerância ou reserva ativa. Fora disso → 404 / `room-not-found`.

### Peer (público — vai para os outros)
| Campo | Tipo |
|---|---|
| `id` | string (UUID gerado pelo servidor) |
| `nickname` | string (1–24, higienizado) |
| `tone` | `'accent' \| 'sage' \| 'sand' \| 'deep'` |
| `live` | `null \| { resolution: 720\|1080, fps: 30\|60, audio: boolean, startedAt: number }` |

### PeerAttachment (privado — só no servidor)
`Peer` + `resumeToken` (aleatório, 128 bits) + `turnUsername` (para revogar) + `joinedAt`.

### Estado de cada conexão P2P (navegador)
| Campo | Tipo |
|---|---|
| `pc` | `RTCPeerConnection` |
| `ctl` | `RTCDataChannel` |
| `polite` | boolean (`meuId < idDoOutro`) |
| `makingOffer`, `ignoreOffer` | flags da perfect negotiation |
| `senders` | senders da minha transmissão para esta pessoa |
| `viewMode` | `'full' \| 'thumb' \| 'hidden'` — como **essa pessoa** está vendo a minha tela |
| `remoteStream` | `MediaStream \| null` — a transmissão dela para mim |

## View-models (contrato UI ↔ lógica/mocks) — `src/lib/types.ts`

```ts
type Tone = 'accent' | 'sage' | 'sand' | 'deep';
type TileStatus = 'ok' | 'loading' | 'failed';

interface ParticipantView { id: string; name: string; tone: Tone; you: boolean; live: boolean }
// "assistindo" é derivado: watchingId(view) em src/lib/room-view.ts

interface StreamView {
  id: string;            // = peerId
  name: string; tone: Tone;
  quality?: string;      // "1080p · 60 fps"
  caption?: string;      // placeholder do mock ("tela de Lu · Hollow Knight")
  status: TileStatus;
  muted: boolean; volume: number;   // 0..1
  stream?: MediaStream;  // ausente no mock
}

interface ToastView { id: string; kind: 'join'|'leave'|'live'|'copied'|'error'; text: string; action?: string }

interface MyBroadcastView { resolution: 720|1080; fps: 30|60; audio: boolean; startedAt: number; viewers: number; stream?: MediaStream }

interface RoomView {
  roomId: string; inviteUrl: string;
  status: 'connecting' | 'ready';
  participants: ParticipantView[];  // você primeiro
  capacity: number;                 // 5
  streams: StreamView[];            // sem a sua
  focusedId: string | null;
  myBroadcast: MyBroadcastView | null;
  copied: boolean;
  goLiveOpen: boolean;
  goLivePrefs: { resolution: 720|1080; fps: 30|60; audio: boolean };
  permissionDenied: boolean;
  toasts: ToastView[];
  canShare: boolean;
}
```

Derivações na UI: título/subtítulo do header, layout do palco (vazio/único/grade/foco) e
"Vendo X" saem de `RoomView` em funções puras (`src/lib/room-view.ts`), usadas igualmente por mock e real.

## Preferências locais (`localStorage`, chave `espia:*`)
| Chave | Valor |
|---|---|
| `espia:nickname` | string |
| `espia:golive` | `{ resolution, fps, audio }` (padrão 1080/30/true) |
| `espia:volume:<nickname>` | number 0..1 |
