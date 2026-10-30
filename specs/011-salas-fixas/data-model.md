# Data Model: Salas fixas e chamada que sobrevive a quedas

## No servidor (storage do Durable Object de cada sala)

Chaves novas marcadas com ★. Todas, menos `fixed` e `tombstone`, são apagadas quando a sala esvazia de vez (como hoje).

| Chave | Tipo | Quando existe |
|---|---|---|
| ★ `roster` | `Record<id, Member>` | Enquanto há alguém conectado ou na tolerância. Espelha os attachments (R1). |
| `ghosts` | `Record<id, { member, until }>` | Quem caiu e pode voltar até `until` (agora 60 s). |
| `reservedUntil` | `number` | 60 s depois de criar, até o criador conectar. |
| ★ `holdUntil` | `number` | Sala rápida: 5 min depois que a última pessoa conectada **caiu** (R2). |
| `joins` | `number` | Contador de tons/ordem. |
| ★ `fixed` | `FixedRoom` | Sala fixa, até expirar ou ser apagada. |
| ★ `tombstone` | `{ reason: 'deleted' \| 'expired'; until: number }` | 30 dias depois de apagar/expirar (R5). Sem nome. |

```ts
interface Member { id; nickname; tone; live; token; seq; turn }   // igual a hoje

interface FixedRoom {
  name: string;        // 1–40, normalizado (R10)
  ownerHash: string;   // SHA-256 hex da ownerKey (R4)
  createdAt: number;
  lastUsed: number;    // último join (não resume, não prévia, não status)
}
```

### Existe?

```
exists = tombstone ausente E (
  há socket com join  OU  ghosts não vazio  OU  reservedUntil > agora
  OU holdUntil > agora  OU  fixed presente )
```

### Transições

```
                 POST /api/rooms {kind:'quick'}           POST /api/rooms {kind:'fixed', name}
                          │                                          │
                     [reservada 60 s]                          [fixa, vazia] ◄──────────────┐
                          │ join                                     │ join (lastUsed = agora)  │
                          ▼                                          ▼                          │
                      [com gente] ── última cai (não sai) ──► rápida: [segurada 5 min]          │
                          │                                     fixa:  [fixa, vazia] ───────────┘
                          │ último sai (4001) / segurada vence sem ninguém
                          ▼
            rápida: apaga tudo (404)        fixa: volta pra [fixa, vazia]

  [fixa, vazia] ── 30 dias sem join ──► lápide 'expired' (410) ── 30 dias ──► nada (404)
  [fixa, qualquer] ── DELETE do dono ──► todos recebem room-deleted / 4410 ► lápide 'deleted' (410)
```

Reinício do DO (publicação): no construtor, `roster − sockets vivos` vira `ghosts` com `until = agora + GRACE`. Se não sobrou ninguém conectado, cria `holdUntil` (rápida). Marca o alarme.

### Alarme (um só)

Próximo de: `ghosts[].until`, `reservedUntil`, `holdUntil`, `tombstone.until`, e, se sala fixa sem ninguém, `fixed.lastUsed + 30 dias`.

## No aparelho (`localStorage` da origem do site; o app tem o seu)

| Chave | Valor |
|---|---|
| ★ `espia:rooms` | `SavedRoom[]`, no máximo 5, ordenado por `lastUsed` desc |
| ★ `espia:owner:<code>` | `ownerKey` (hex). Não sai quando a sala sai da lista; sai quando a sala é apagada/expira |
| `espia:session:<code>` (sessionStorage) | igual a hoje (resume) |

```ts
interface SavedRoom {
  code: string;
  name: string;      // último nome conhecido (atualizado pelo status / room-renamed)
  lastUsed: number;  // última entrada neste aparelho
}
// "Sua" = existe espia:owner:<code>.
```

Regras:
- Entrar numa sala **fixa** (welcome com `room.kind === 'fixed'`) → upsert com `lastUsed = agora`. Passou de 5 → remove o de menor `lastUsed`.
- "Tirar da lista" / "Tirar de Suas salas" → remove o item (a chave de dono fica).
- Status `gone` → o item continua na lista, mostrado como "Esta sala não existe mais", até a pessoa tirar.
- Status `ok` com nome diferente → atualiza `name`.

## View-models (src/lib/types.ts)

```ts
type SavedRoomState = 'loading' | 'ok' | 'gone';
interface SavedRoomView {
  code: string; name: string; own: boolean; tone: Tone;   // tom derivado do código (estável)
  state: SavedRoomState;
  people: number; streaming: number;
  recent: string | null;        // "usada ontem", "há 3 dias"…
  menuOpen?: boolean;
}
interface SavedRoomsView { groups: { label: string | null; rooms: SavedRoomView[] }[]; note: string }

HomeView += { rooms: SavedRoomsView | null; fixedDialog?: NameDialogView; deleteDialog?: DeleteDialogView }
interface NameDialogView { mode: 'create' | 'rename'; value: string; busy: boolean; error: string | null }
interface DeleteDialogView { name: string; people: number; busy: boolean }

RoomView += {
  room: { kind: 'quick' } | { kind: 'fixed'; name: string; own: boolean };
  link: 'ok' | 'reconnecting' | 'failed';
  ownerMenu?: boolean; nameDialog?: NameDialogView; deleteDialog?: DeleteDialogView;
}
JoinView += { fixedName?: string }        // selo "Sala fixa" + título (F8)

type GoneReason = 'quick' | 'deleted' | 'kicked' | 'expired';
interface NotFoundView { reason: GoneReason; name: string | null; saved: boolean }
```

Funções puras novas (testáveis sem tela): `orderSavedRooms`, `groupSavedRooms`, `recentLabel(ms)`, `savedRoomSub(view)`, `normalizeRoomName`, `rememberRoom(list, room, now)` (limite 5).
