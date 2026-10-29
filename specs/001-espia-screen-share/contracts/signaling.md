# Contrato — Sinalização (WebSocket `/ws/:code`)

JSON, um objeto por mensagem, campo `t` = tipo. Máx. 64 KB por mensagem e 200 mensagens / 10 s
por conexão (excedeu → fecha com 1008). Ver [security.md](../security.md).

## Tipos compartilhados

```ts
type Tone = 'accent' | 'sage' | 'sand' | 'deep';
interface LiveInfo { resolution: 720 | 1080; fps: 30 | 60; audio: boolean; startedAt: number }
interface Peer { id: string; nickname: string; tone: Tone; live: LiveInfo | null }
```

## Cliente → Durable Object

| `t` | Campos | Efeito |
|---|---|---|
| `join` | `nickname` | Primeira mensagem obrigatória. Resposta `welcome`, `room-full` ou `room-not-found` |
| `resume` | `peerId`, `resumeToken` | Reconexão dentro da tolerância de 20 s: recupera a mesma identidade. Resposta `welcome` ou `resume-failed` (aí o cliente manda `join`) |
| `signal` | `to`, `data`: `{ description: RTCSessionDescriptionInit }` ou `{ candidate: RTCIceCandidateInit \| null }` | Repassado ao peer `to` com `from` preenchido pelo servidor |
| `live` | `resolution`, `fps`, `audio` | Marca você como ao vivo (`startedAt` definido pelo servidor) |
| `unlive` | — | Encerra sua transmissão |
| `turn-refresh` | — | Pede credenciais TURN novas (antes de expirar). Resposta `ice-servers` |
| `ping` | — | Respondido com `pong` automaticamente, sem acordar o DO (`setWebSocketAutoResponse`) |

## Durable Object → Cliente

| `t` | Campos | Quando |
|---|---|---|
| `welcome` | `you: Peer`, `peers: Peer[]`, `iceServers: RTCIceServer[]`, `iceExpiresAt`, `resumeToken`, `capacity` | Após `join`/`resume` aceito |
| `room-not-found` | — | Código sem sala ativa nem reservada; servidor fecha (4404) |
| `room-full` | `capacity` | 10 pessoas; servidor fecha (4409) |
| `resume-failed` | — | Token inválido ou tolerância expirou |
| `peer-joined` | `peer: Peer` | Alguém entrou |
| `peer-left` | `id` | Saiu (ou não voltou em 20 s) |
| `peer-live` | `id`, `live: LiveInfo \| null` | Começou (`live`) ou parou (`null`) |
| `signal` | `from`, `data` | Mensagem WebRTC de outro peer |
| `ice-servers` | `iceServers`, `iceExpiresAt` | Resposta a `turn-refresh` |
| `error` | `code`, `message` | Mensagem inválida (`bad-message`, `not-joined`, `unknown-peer`, `rate-limited`) |

## Regras

- Quem **acabou de entrar** cria a conexão WebRTC (com canal `ctl`) com cada peer da lista `peers`. Os outros esperam o primeiro `signal`.
- Papel na *perfect negotiation*: `polite = meuId < idDoOutro`.
- Fechar o socket inicia a tolerância de 20 s; sem `resume` → `peer-left` para todos e revogação da credencial TURN dessa pessoa.
- Fechar com código **4001** = saída voluntária ("Sair da sala"): `peer-left` na hora, sem tolerância.
- `resume` com o socket antigo ainda aberto: a nova conexão assume a identidade e a antiga é fechada com **4000**.
- `ping` precisa ser exatamente a string `{"t":"ping"}` (resposta automática do runtime).
- Abuso (> 64 KB, > 200 mensagens / 10 s, > 5 mensagens inválidas) fecha com **1008**.
- Sala com 0 sockets e sem tolerância pendente → estado descartado; o código volta a "não existe".
- Tons atribuídos por ordem de entrada em rodízio: `deep, accent, sage, sand`.

## Canal de dados `ctl` (P2P, entre dois navegadores)

| `t` | Campos | Efeito |
|---|---|---|
| `view` | `mode: 'full' \| 'thumb' \| 'hidden'` | Como eu estou vendo a SUA transmissão. Quem transmite ajusta o envio só para mim (thumb ≈ 360p·15fps·0,5 Mbps; hidden = pausa o envio com `encodings[0].active = false`) |
