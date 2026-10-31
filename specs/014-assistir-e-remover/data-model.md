# Data model — 014

Nada novo é guardado em disco: nem no servidor, nem no `localStorage`. Tudo abaixo vive na memória da sala aberta.

## Store da sala (`src/lib/stores/room.svelte.ts`)

| Campo | Tipo | Regra |
|---|---|---|
| `unwatched` | `Map<peerId, startedAt>` | Entra com "Parar de assistir". Sai com "Assistir", `peer-live` null ou com outro `startedAt`, ou `peer-left` (R4). Sobrevive à queda. |
| `pageVisible` | `boolean` | `document.visibilityState === 'visible'`, atualizado no `visibilitychange` (R2). |
| `pipId` | `string \| null` | Tela em picture-in-picture, avisada pelo tile (R3). |
| `selfOnStage` | `boolean` | A própria transmissão grande no palco (R6). Volta a `false` com Esc, outro foco, outro clique na prévia ou fim da transmissão. |
| `snapshot` | `ImageBitmap \| null` | Último quadro da própria transmissão, 320 px de largura, a cada 5 s com a página visível. O anterior é fechado. Volta a `null` no fim da transmissão. |
| `removeDialog` | `{ peerId, name } \| null` | Confirmação "Tirar Lu da sala?". |

### Transições do modo de cada tela (`viewModes`, R5)

```
não assistida ───────────────────────────────► hidden
página escondida (e não é o PiP) ────────────► hidden
celular: selecionada ► full · outras ► hidden
foco: focada ► full · outras ► thumb
grade / única ► full
```

## View-models (`src/lib/types.ts`)

- `StreamView`:
  - `+ watching: boolean` (`false` = cartão "Assistir", sem `<video>`);
  - `+ you?: boolean` (a própria tela no palco: sempre muda, sem volume).
- `MyBroadcastView`:
  - `+ snapshot: ImageBitmap | null`;
  - `+ onStage: boolean`;
  - `viewers` passa a contar quem me vê com modo ≠ `hidden`.
- `ParticipantView`: `+ removable: boolean` (só para o dono numa sala fixa, nunca em `you`).
- `RoomView`:
  - `+ removeDialog?: { name: string; busy: boolean; error?: string }`;
  - `+ notice?: 'removed'` na tela inicial (toast "O dono te tirou da sala").
- `RoomActions`:
  - `+ watch(id, on: boolean)`;
  - `+ pip(id | null)`;
  - `+ toggleSelfStage()`;
  - `+ askRemove(peerId)`, `+ confirmRemove()`, `+ cancelRemove()`.

## Servidor (Durable Object)

Não guarda nada novo. `removePerson` só apaga dados que já existem (membro no `roster`/fantasmas e as credenciais TURN).
