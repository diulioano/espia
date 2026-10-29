# Segurança e privacidade — Espia

Sem contas e sem senha, o modelo é: **quem tem o código da sala entra**. A segurança aqui é
garantir que (1) só quem tem o código entra, (2) ninguém consegue achar códigos chutando,
(3) o vídeo só é visto por quem está na sala, (4) ninguém usa nossa infraestrutura de graça
e (5) guardamos o mínimo possível sobre as pessoas.

## 1. O que protegemos

| Ativo | Por quê |
|---|---|
| Conteúdo da tela e do áudio | Pode ter mensagens, senhas na tela, coisas pessoais |
| Código da sala | É a "chave" de entrada |
| IP de cada pessoa | Revela localização aproximada |
| Apelidos | Dado pessoal leve |
| Cota TURN / Workers | Abuso vira custo |

## 2. Criptografia — quem consegue ver o quê

| Trecho | Proteção | Quem pode ler |
|---|---|---|
| Site, API, WebSocket | HTTPS / WSS (TLS) | Cloudflare (nosso servidor) |
| Vídeo e áudio | **DTLS-SRTP, obrigatório no WebRTC** — fim a fim entre os dois navegadores | **Só os dois navegadores.** O TURN retransmite pacotes cifrados e não consegue ver |
| Canal `ctl` | DTLS (SCTP) | Só os dois navegadores |

Limite honesto: as "impressões digitais" DTLS passam pela sinalização. Um servidor de sinalização
**malicioso** poderia se colocar no meio. Como o servidor é nosso e não guarda nada, aceitamos esse
risco (é o mesmo modelo do Google Meet, Discord etc. em P2P).

## 3. Ameaças e mitigações

| # | Ameaça | Mitigação |
|---|---|---|
| A1 | **Chutar códigos** para cair em salas alheias | Código de 6 caracteres em 31 símbolos (~887 milhões). Rate limit por IP: `GET /api/rooms/:c` e `/ws/:c` em **20/min**; com ~100 salas ativas, cada tentativa tem ~1 chance em 9 milhões; no limite de 20/min, um IP insistindo o dia todo tem ~0,3% de cair em *alguma* sala (aleatória, não numa escolhida). Quem entra aparece na lista e gera toast "X entrou na sala" — intrusão é visível. Sala cheia (10) recusa. |
| A2 | **Link vazar** (print, Referer, histórico) | `Referrer-Policy: no-referrer` (o código na URL não vaza para outros sites ao clicar links). Salas morrem quando esvaziam, então um link velho não vale nada. |
| A3 | **Se passar por outra pessoa** nas mensagens | O ID de cada pessoa é gerado **pelo servidor**; o campo `from` das mensagens é preenchido **pelo DO**, nunca aceito do cliente. `to` precisa ser alguém da mesma sala. |
| A4 | **Apelido malicioso** (HTML, script, texto invertido) | Svelte escapa texto por padrão; proibido `{@html}`. Servidor normaliza (NFC), remove caracteres de controle e de direção (U+202A–202E, U+2066–2069), corta espaços, 1–24 caracteres. |
| A5 | **Flood de mensagens / mensagens gigantes** no WebSocket | Máx. 64 KB por mensagem; máx. 200 mensagens / 10 s por conexão → fecha com código 1008. Esquema validado; mensagem inválida → `error` e, se repetir, fecha. |
| A6 | **Outro site usar nosso WebSocket/API** | Checagem do cabeçalho `Origin` no upgrade e nos `POST` (só o próprio domínio; `localhost` em dev). |
| A7 | **Usar nosso TURN como relay grátis** | Não existe endpoint público de TURN: credenciais vêm **só** dentro do `welcome`, para quem entrou numa sala. TTL de 6 h com renovação pelo WebSocket; **revogadas quando a pessoa sai** (`/credentials/{username}/revoke`). Criação de sala limitada a 10/min por IP. Alerta de uso no painel. |
|  | *Medido em 2026-10-28:* a revogação da Cloudflare responde 204, mas a credencial revogada **continuou abrindo relay por pelo menos 60 s**. Revogar é um "melhor esforço"; a proteção real é a credencial só existir para quem está numa sala e expirar sozinha. Se o uso do TURN fugir do esperado, baixar o TTL (ex.: 1 h, com `turn-refresh` antes de expirar). |
| A8 | **Criar salas em massa** | Rate limit em `POST /api/rooms`; reserva expira em 60 s se ninguém conectar; DO vazio não custa nada. |
| A9 | **Clickjacking / embutir o site** | `Content-Security-Policy: frame-ancestors 'self'` (o `'self'` é para o quadro de celular do modo mock). |
| A10 | **Script injetado / dependência comprometida** | CSP restritiva (abaixo), poucas dependências, `package-lock.json` versionado, `npm audit` no CI, sem CDNs de terceiros em produção. |
| A11 | **Captura da tela errada** | Seletor nativo do navegador (o site nunca escolhe sozinho); `selfBrowserSurface: 'exclude'`; card "Você está ao vivo" com prévia sempre visível e botão Parar a um clique. |
| A12 | **Segredos vazarem** | `TURN_KEY_API_TOKEN` só como *secret* do wrangler; `.dev.vars` no `.gitignore`; o navegador nunca recebe a chave, só credenciais temporárias. |

## 4. Privacidade

| Dado | Onde fica | Por quanto tempo |
|---|---|---|
| Apelido, tom, quem está ao vivo | Memória do DO da sala | Até a sala esvaziar (+20 s) |
| Apelido, preferências, volumes | `localStorage` do seu navegador | Até você limpar |
| Vídeo / áudio | Em trânsito, cifrado; **nunca gravado** | — |
| Logs | Logs do Worker **sem** apelidos, códigos de sala ou IPs | Retenção padrão da Cloudflare |

- **Fontes auto-hospedadas** (`@fontsource`): carregar do Google Fonts entregaria o IP de cada visitante ao Google.
- **Sem analytics, sem cookies.**

### 4.1 Exposição de IP entre participantes — **D-1 decidida: opção A** (2026-10-28)
Uso previsto: grupo fechado de até 10 amigos. Ver o IP público uns dos outros é aceitável.
No WebRTC direto, cada participante **descobre o IP público dos outros** da sala (é assim que a
conexão direta acontece). Endereços da rede local já são escondidos pelo navegador (mDNS).

| Opção | Privacidade | Custo / qualidade |
|---|---|---|
| **A. Direto (recomendado)** | Amigos da sala veem o IP uns dos outros | Grátis, menor latência; TURN só quando precisa |
| B. Sempre via TURN (`iceTransportPolicy: 'relay'`) | Ninguém vê o IP de ninguém (só a Cloudflare) | Todo o vídeo passa pelo TURN: 1 sala de 5 em 1080p60 gasta ~11 GB/h → a cota grátis acaba em ~90 h/mês |
| C. Direto por padrão + opção "modo privado" na entrada | Escolha de cada pessoa | Precisa de um controle novo na interface (não desenhado) |

Decisão: **A**. B e C ficam registradas caso o uso saia do grupo de amigos.

## 5. Cabeçalhos HTTP (todas as respostas)

```text
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
  font-src 'self'; img-src 'self' data:; media-src 'self' blob:; connect-src 'self';
  frame-ancestors 'self'; base-uri 'none'; form-action 'self'; object-src 'none'
Referrer-Policy: no-referrer
X-Content-Type-Options: nosniff
Permissions-Policy: display-capture=(self), camera=(self), microphone=(self), geolocation=(), picture-in-picture=(self), fullscreen=(self)
(camera/microphone liberados para o próprio site na 006: aba Câmeras.)
Cross-Origin-Opener-Policy: same-origin
Strict-Transport-Security: max-age=31536000; includeSubDomains
```
- `style-src 'unsafe-inline'` é necessário porque o Svelte injeta estilos das transições em tempo de execução; scripts continuam bloqueados.
- `connect-src 'self'` cobre `wss://` do mesmo domínio. STUN/TURN não passam por CSP.
- Assets estáticos recebem os mesmos cabeçalhos via arquivo `_headers`.

## 6. Checklist antes do deploy
- [ ] Nenhum `{@html}` no código
- [ ] `from` sempre definido pelo servidor (teste automatizado)
- [ ] 11ª pessoa recebe `room-full` (teste)
- [ ] Mensagem > 64 KB e flood fecham a conexão (teste)
- [ ] `Origin` estranho recebe 403 (teste)
- [ ] Credencial TURN revogada após `peer-left` (teste com mock da API)
- [ ] Cabeçalhos da §5 presentes no `/`, `/s/x` e `/assets/*`
- [ ] Logs sem apelido / código / IP
- [ ] `npm audit` sem vulnerabilidades altas
