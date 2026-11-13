# Contrato: convite no endereço oficial e ida do endereço antigo (R11, R12)

Vale só no site (Svelte), no navegador e nos apps já instalados (que carregam o site). Não muda o Worker nem o protocolo. O app nativo não usa nada daqui: ele já nasce com o endereço oficial e traz os dados pela importação (ver `contracts/app-nativo.md`).

## `inviteOrigin` (`src/lib/site-origin.ts`, módulo novo)

```ts
export const OFFICIAL_ORIGIN = 'https://espia.diuliano.com';
export const LEGACY_HOSTS = ['espia.espia.workers.dev'] as const;
export function inviteOrigin(loc: Pick<Location, 'hostname' | 'origin'>): string;
```

Devolve a origem que vai no convite. A tabela é a regra inteira:

| Host da página | Resultado |
|---|---|
| `espia.diuliano.com` (oficial) | `OFFICIAL_ORIGIN` |
| `espia.espia.workers.dev` (antigo, o único de `LEGACY_HOSTS`) | `OFFICIAL_ORIGIN` |
| `localhost` ou `127.0.0.1`, com qualquer porta | `loc.origin` (o teste local continua possível, FR-007) |
| qualquer outro host | `loc.origin` |

O tipo de `loc` acima (`hostname` e `origin` de `Location`) é o que a função lê; o teste da tarefa T039 fixa esse tipo e a tabela.

### Onde é usada

| Lugar | Hoje | Passa a ser |
|---|---|---|
| `src/lib/stores/room.svelte.ts:518` | `inviteUrl: location.origin + roomPath(this.code)` | `inviteOrigin(location) + roomPath(this.code)` |
| `src/mocks/room.ts:308` | origem fixa nas cenas | `inviteOrigin(location)` |

`RoomView.inviteUrl` alimenta os três pontos que mostram ou copiam o convite: `copyInvite` (`room.svelte.ts:341-346`), a cópia automática ao criar sala fixa (`HomeRoute.svelte:186` com `created: true` e `room.svelte.ts:384-387`) e o texto mostrado em `src/ui/pages/RoomPage.svelte:33`. Mudar `inviteUrl` basta para os três (FR-001).

Os apps já instalados carregam o site, então a mudança vale para eles na hora, sem instalar nada.

## Pacote `#mv=`

Levado no fragmento do endereço, na ida do endereço antigo para o oficial.

```text
https://espia.diuliano.com/s/k7f-q2x#mv=<base64url do JSON>
```

JSON (UTF-8):

```json
{ "v": 1, "items": { "<chave>": "<valor cru do localStorage>" } }
```

Exemplo, com apelido, uma sala fixa e a chave de dono dela (a chave aqui é só de exemplo):

```json
{"v":1,"items":{"espia:nickname":"Rafa","espia:rooms":"[{\"code\":\"k7f-q2x\",\"name\":\"Noite de jogo\",\"lastUsed\":1762860000000}]","espia:owner:k7f-q2x":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}
```

Codificado em base64url:

```text
#mv=eyJ2IjoxLCJpdGVtcyI6eyJlc3BpYTpuaWNrbmFtZSI6IlJhZmEiLCJlc3BpYTpyb29tcyI6Ilt7XCJjb2RlXCI6XCJrN2YtcTJ4XCIsXCJuYW1lXCI6XCJOb2l0ZSBkZSBqb2dvXCIsXCJsYXN0VXNlZFwiOjE3NjI4NjAwMDAwMDB9XSIsImVzcGlhOm93bmVyOms3Zi1xMngiOiJhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhIn19
```

O base64url vai sem preenchimento `=` no fim.

### Chaves empacotadas

As `espia:*` que o site guarda (`src/lib/prefs.ts`, `src/lib/saved-rooms.ts`): `espia:nickname`, `espia:golive`, `espia:volume:<apelido>`, `espia:owner:<código>`, `espia:rooms`, `espia:session:<código>`, `espia:appinvite`. Outras chaves do `localStorage` nunca entram.

### Empacotar (`src/lib/origin-move.ts`)

1. Lê as chaves `espia:*` do `localStorage` do endereço antigo (valor cru, sem reinterpretar).
2. Monta `{ "v": 1, "items": { … } }` e codifica em base64url.
3. Teto de 16.384 caracteres do texto codificado. Se passar, descarta nesta ordem, parando quando couber: todos os `espia:session:*`, depois todos os `espia:volume:*`, depois `espia:golive`, depois `espia:appinvite`. `espia:rooms`, `espia:owner:*` e `espia:nickname` sempre vão. Se ainda passar do teto, a ida acontece sem `#mv=` e nada é apagado no endereço antigo.
4. Sem `localStorage` (janela anônima, dados bloqueados): pacote vazio, e a ida acontece do mesmo jeito (edge case da spec).

O teto é 16 × 1024 = 16.384 caracteres.

### Desempacotar e validar (no endereço oficial)

1. Lê `#mv=`; sem o fragmento, não faz nada.
2. Decodifica base64url e interpreta o JSON. Texto inválido, JSON inválido ou `v` diferente de `1`: ignora o pacote inteiro.
3. Valida cada item. Item inválido é ignorado em silêncio, sem derrubar os outros:

| Item | Regra |
|---|---|
| código de sala (em `espia:rooms`, `espia:owner:*`, `espia:session:*`) | formato `xxx-xxx` de `src/lib/room-id.ts` (`ROOM_ID_RE`) |
| chave de dono | 64 hex minúsculos |
| nome de sala | até 40 |
| apelido | até 24 |
| volume | entre 0 e 1 |

4. Junta (abaixo) e grava no `localStorage` do endereço oficial.
5. Limpa o fragmento com `history.replaceState`, antes de montar a aplicação.

### Junção

Só acrescenta. Nunca troca um valor que já existe no oficial.

| Chave | Regra |
|---|---|
| `espia:nickname`, `espia:golive`, `espia:volume:*`, `espia:owner:*`, `espia:session:*`, `espia:appinvite` | Se o oficial já tem, fica o do oficial. Se não tem, entra o do pacote. |
| `espia:rooms` | União por código, ordenada por `lastUsed` do mais recente para o mais antigo, cortada em 5 (`SAVED_ROOMS_MAX`). Com o mesmo código dos dois lados, fica a entrada de maior `lastUsed`, com o nome dela. |
| chave de dono | Nunca descartada, mesmo de sala que ficou fora das 5. |


### Idempotência

Repetir a ida não duplica nem apaga. Segunda visita ao endereço antigo: o pacote sai de novo, o oficial junta de novo, e como a junção só acrescenta e a lista é unida por código, nada muda (US2, cenário 6).

## Quando o site NÃO redireciona

O site só empacota e navega quando as três condições valem juntas:

1. o host da página é o antigo (`espia.espia.workers.dev`);
2. a página não está dentro do app de antes: `'__TAURI_INTERNALS__' in window` é falso;
3. a URL não tem `?mock`.

| Situação | Redireciona? | Por quê |
|---|---|---|
| App de antes (Tauri) no endereço antigo | Não | Ele só abre o endereço antigo. Redirecionar tiraria as Suas salas de dentro dele (R13, FR-006). |
| `?mock` no endereço antigo | Não | As telas de dados falsos continuam abrindo onde estão. |
| Host oficial | Não | Já é o destino. Só importa `#mv=` se vier. |
| `localhost`, `127.0.0.1` ou outro host | Não | Desenvolvimento e teste. |

A ordem no `src/main.ts` é: no antigo, empacotar e navegar com `location.replace` para `https://espia.diuliano.com` + caminho + busca + `#mv=…`, antes de montar a aplicação; no oficial, importar e limpar o fragmento, antes de montar a aplicação.
Uma sala rápida em andamento volta como numa queda: a sessão (`espia:session:*`) vai no pacote enquanto couber no teto, e o `resume` a retoma (edge case da spec).

## Privacidade

- O fragmento (tudo depois do `#`) nunca é enviado ao servidor pelo navegador. A chave de dono e o resto do pacote não aparecem em registro do servidor (FR-005, constituição III).
- O servidor não redireciona nada. Os apps instalados dependem do endereço antigo, e o mesmo Worker responde nos dois endereços (FR-006). A ida acontece só no navegador, no código do site.
- O pacote vive só durante a navegação. O oficial limpa o fragmento antes de montar a aplicação, para a chave não ficar no histórico nem em link copiado da barra de endereço.
