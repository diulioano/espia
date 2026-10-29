# Research — Espia

Decisões fechadas em `DEFINICOES.md` não são reabertas aqui. Este arquivo registra as escolhas técnicas
que faltavam, com motivo e alternativa rejeitada. Detalhes em [architecture.md](./architecture.md) e
[security.md](./security.md).

## R1. SPA Svelte + Vite em vez de SvelteKit
- **Decisão**: Svelte 5 + Vite + TS, roteador próprio mínimo (`/` e `/s/:código`).
- **Motivo**: só 2 rotas; o Worker serve os assets com fallback de SPA. SvelteKit traria adapter e SSR sem ganho.
- **Rejeitado**: SvelteKit + `adapter-cloudflare`.

## R2. Código da sala curto
- **Decisão (confirmada pelo usuário em 2026-10-28)**: `/s/<código>`, formato `xxx-xxx`, 31 símbolos sem ambíguos (`abcdefghjkmnpqrstuvwxyz23456789`), ~887 milhões de combinações, `crypto.getRandomValues` com rejeição para não enviesar.
- **Por que basta**: salas são temporárias e poucas; rate limit por IP e entrada visível para todos (security.md A1). Entrada colada é normalizada (link inteiro, maiúsculas, sem hífen).
- **Rejeitado**: ID de 10 caracteres base62 — mais seguro contra chute, mas ruim de ditar/digitar.

## R3. Modo mock por parâmetro
- `App.svelte` checa `?mock` antes do roteador e carrega `MockApp` por `import()` dinâmico. Cena de celular abre num iframe 390×844 para os media queries valerem.

## R4. View-models entre UI e lógica
- Páginas recebem `RoomView`/`HomeView`/`JoinView` + ações. Mocks e stores reais produzem o mesmo formato.

## R5. Código gerado no servidor (`POST /api/rooms`)
- **Decisão**: o Worker gera o código e o DO o reserva por 60 s.
- **Motivo**: sem colisão, sem o cliente escolher código, rate limit de criação num ponto só. Distingue "criar" de "entrar numa sala que não existe" (E2).
- **Rejeitado**: cliente gera e o primeiro `join` cria — qualquer link inválido criaria uma sala em vez de mostrar E2.

## R6. Sinalização em Durable Object com Hibernation API
- Um DO por código (`idFromName`). Identidade de cada socket em `serializeAttachment` (sobrevive à hibernação); estado da sala reconstruído de `ctx.getWebSockets()`. `ping`/`pong` com `setWebSocketAutoResponse` (não acorda o DO).
- Plano grátis exige classe SQLite (`new_sqlite_classes`); **não usamos o armazenamento** — tudo em memória (Constituição III).
- **Rejeitado**: Worker sem DO + KV (KV não é tempo real); PartyKit (camada extra sem necessidade).

## R7. Conexão P2P criada ao entrar, com canal `ctl`
- **Decisão**: cada par conecta assim que alguém entra, usando um `RTCDataChannel` para a conexão nascer sem vídeo.
- **Motivo**: Go Live aparece para os outros em ~1 s (só renegocia); o canal serve para o controle de qualidade por espectador (R10).
- **Rejeitado**: conectar só quando alguém vai ao vivo — atraso de vários segundos a cada Go Live.

## R8. Perfect negotiation, sem biblioteca
- Padrão oficial (W3C/MDN) com `polite = meuId < idDoOutro`; trickle ICE pelo DO.
- **Rejeitado**: simple-peer / PeerJS — abstraem justamente o que precisamos controlar (renegociação, sender params) e estão pouco mantidos.

## R9. TURN da Cloudflare entregue pelo WebSocket
- DO chama `POST https://rtc.live.cloudflare.com/v1/turn/keys/{TURN_KEY_ID}/credentials/generate-ice-servers` (Bearer `TURN_KEY_API_TOKEN`, `{ ttl: 21600 }`) e envia `iceServers` no `welcome`. Renovação via `turn-refresh` + `pc.setConfiguration()`. Revogação em `.../credentials/{username}/revoke` quando a pessoa sai.
- Preço: US$ 0,05/GB, 1.000 GB/mês grátis (compartilhado com o SFU).
- **Rejeitado**: endpoint público `/api/turn` — qualquer um poderia pegar credenciais e usar nosso relay.

## R10. Qualidade por espectador
- `RTCRtpSender.setParameters` por conexão: cheia na grade/foco; ~360p·15fps·0,5 Mbps em miniatura; `active=false` se oculto. Pedido via canal `ctl`.
- Bitrates máximos: 720p30 2,5 · 720p60 4 · 1080p30 4,5 · 1080p60 6 Mbps.
- `contentHint` `motion` (60 fps) / `detail` (30 fps); `degradationPreference` correspondente.

## R11. Codec — **spike pendente (T046)**
- Hipótese: H.264 (encoder por hardware) aguenta melhor 4 codificações 1080p60 que VP8/VP9 em software. Medir CPU, qualidade de texto e fps em Chrome/Edge Windows com 4 espectadores antes de fixar `setCodecPreferences`.

## R12. Reconexão com `resumeToken`
- Queda do WebSocket não derruba as conexões P2P. DO segura a vaga por 20 s; cliente reconecta com backoff (1, 2, 4, 8, 10 s) e `resume`. Evita toasts falsos de saiu/entrou.

## R13. Fontes auto-hospedadas
- `@fontsource/caprasimo` e `@fontsource/figtree` em vez de Google Fonts: privacidade (IP não vai ao Google) e CSP `font-src 'self'`.

## R15. Até 10 por sala, P2P com teto de qualidade
- **Decisão (usuário, 2026-10-28)**: capacidade 10 (o grupo inteiro), uso normal 3–5. Mantém P2P; a qualidade cai em faixas conforme o nº de espectadores (architecture.md §4.1b).
- **Rejeitado**: manter 5 (o grupo não caberia junto); Cloudflare Realtime SFU (resolve 10 em 1080p60, mas adiciona servidor de mídia e complexidade). O SFU fica como **plano B** se o spike T046 mostrar que 9 codificações não são viáveis.

## R16. Acesso só pelo código
- **Decisão (usuário)**: sem senha do grupo e sem Cloudflare Access. As proteções de security.md bastam para o uso entre amigos.

## R14. Testes
- Vitest para `src/lib` puro; `@cloudflare/vitest-pool-workers` para o DO (join, cheia, `from`, flood, resume); Playwright com Chromium e captura falsa para o fluxo ponta a ponta com 2–3 contextos.
