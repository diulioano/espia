# Espia — Definições do projeto

Compartilhamento de tela entre amigos, no estilo "Go Live" do Discord, focado em ser simples de usar.

## Como funciona para quem usa

1. Abre o site, digita um apelido e clica em **Criar sala**. É gerado um link (ex.: `/s/k7f-q2x`).
2. Manda o link para os amigos. Quem abre digita um apelido e entra.
3. Qualquer pessoa clica em **Go Live**, escolhe tela/janela/aba, qualidade e áudio.
4. Quem assiste escolhe qual transmissão ver, com tela cheia, volume por transmissão e picture-in-picture.

## Decisões

| Tema | Decisão | Motivo |
|---|---|---|
| Plataforma | App web no navegador | Zero instalação; o navegador já faz captura de tela e WebRTC |
| Tamanho das salas | Até 10 pessoas (o grupo inteiro); uso normal de 3–5 | P2P sem servidor de mídia; acima de 4 espectadores a qualidade baixa sozinha |
| Transporte | WebRTC P2P em malha (mesh) | Sem custo de servidor de vídeo |
| Acesso | Sala por link com código curto aleatório, sem contas | Fácil de ditar/digitar; salas são temporárias e o servidor limita tentativas |
| Senha | Não tem | O link já é o "segredo"; menos atrito |
| Vida da sala | Temporária, deixa de existir quando todos saem | Nada para guardar ou limpar |
| Transmissões | Várias ao mesmo tempo por sala | Igual ao Discord |
| Qualidade | Seletor 720p/1080p + 30/60fps | Controle sobre o upload usado |
| Áudio | Só o som da janela ou aba escolhida; tela inteira vai sem som (specs/002) | Som de jogos e vídeos sem vazar o Discord |
| Espectador | Tela cheia, volume, lista de participantes, picture-in-picture | — |
| Front-end | Svelte + Vite + TypeScript | Leve e fácil de reestilizar |
| Design | Design final "Espia" em `design_handoff_espia/` (alta fidelidade, tema escuro) | Feito no Claude Design; a camada visual fica separada para ser trocada |
| App desktop | Opcional, Tauri 2 + WebView2, só Windows; abre o site publicado, links `espia://s/<código>`, instância única, esconde a barra "está compartilhando" | Leve (instalador de poucos MB); o site continua sendo o principal (feature 003) |
| Código | GitHub `DiulianoVargas/espia` (público), CI + release do instalador por tag + deploy do Worker | A galera baixa o app sem conta; nenhum segredo no repositório |
| Rota da sala | `/s/<código>`, código `xxx-xxx` (ex.: `k7f-q2x`) | Igual ao design; curto o bastante pra mandar por voz ou digitar |

### Fora do escopo (por enquanto)
- Chat de voz
- Chat de texto
- Contas, login, lista de amigos, servidores/canais

## Arquitetura (tudo na Cloudflare)

- **Navegador**: captura a tela (`getDisplayMedia`) e envia o vídeo direto para cada participante via WebRTC.
- **Cloudflare Worker**: serve o site, gera o código das salas e aplica limites contra abuso.
- **Durable Object (um por sala)**: sinalização via WebSocket. Apresenta os navegadores uns aos outros, mantém a lista de participantes e entrega credenciais TURN temporárias só para quem está na sala (a chave secreta nunca vai para o navegador).
- **Cloudflare Realtime TURN**: retransmite o vídeo (cifrado) quando a conexão direta falha (4G, CGNAT, redes corporativas). US$ 0,05/GB, 1.000 GB/mês grátis.
- **Vídeo**: WebRTC nativo, uma conexão por par de pessoas, criptografia fim a fim (DTLS-SRTP). Nada é gravado.
- **Deploy**: `wrangler`, publicado em `*.workers.dev`, sem domínio próprio.
- Detalhes: `specs/001-espia-screen-share/architecture.md` (como funciona) e `security.md` (segurança e privacidade).

## Limitações conhecidas
- Quem transmite envia uma cópia para cada espectador: 1080p60 para 4 pessoas usa ~15–25 Mbps de upload.
- No celular dá para assistir, mas não compartilhar a tela.
- Som da janela precisa de Chrome/Edge 141+ no Windows; Firefox/Safari têm limitações. Som do sistema inteiro nunca vai (vazaria o Discord).
- Na conexão direta, quem está na sala vê o IP público dos outros participantes — aceito, já que o uso é entre amigos (D-1 em `security.md`).

## Pré-requisitos
- [ ] Conta grátis na Cloudflare
- [x] Node.js instalado (v24)
- [x] Design final (feito no Claude Design → `design_handoff_espia/`)

## Documentação e desenvolvimento
- Spec Kit: `specs/001-espia-screen-share/` (site), `specs/002-audio-sem-discord/` (áudio), `specs/003-espia-desktop/` (app desktop + GitHub), `specs/004-seletor-de-tela/` (seletor janela/tela + som) e `.specify/memory/constitution.md`.
- App desktop: `src-tauri/` (`npm run desktop:dev`, `npm run desktop:build`).
- Telas mockadas: `npm run dev` e abrir `/?mock` (ex.: `/?mock=room&scene=grid`).

## Design (Claude Design)
- Projeto: https://claude.ai/design/p/dc89098d-c77d-4d62-9ecf-8d4335dffa7d?file=Espia.dc.html
- Cópia local (handoff): `design_handoff_espia/`. O `README.md` de lá descreve telas, componentes e tokens; `tokens.css` já foi copiado para `src/ui/tokens.css`.
- Arquivo principal: `Espia.dc.html` (quadro com todas as telas). Ele importa `_ds/organic-a1d220e8-9438-4bb5-a7ba-867248d96bf3/_ds_bundle.js`, `_ds/organic-a1d220e8-9438-4bb5-a7ba-867248d96bf3/styles.css` e `support.js`.
- Para reimportar a versão mais nova: MCP `claude_design` (https://api.anthropic.com/v1/design/mcp, login com `/design-login`).
