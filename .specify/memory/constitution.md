<!--
Sync Impact Report
- Versão: 3.0.0 → 4.0.0 (MAJOR: o app desktop deixa de ser "Tauri 2 + WebView2 que carrega o site" e passa a ser nativo, com interface própria; IV e V passam a valer também para o app nativo; VI renomeado).
  Pedido de 2026-11-11, feature 019 (specs/019-app-rust-convite).
- Princípios modificados: I (app desktop nativo, mesmo caminho e mesmas regras do site), II (vale entre navegador e app nativo), III (onde o app nativo guarda os dados do aparelho), IV e V (valem para as duas interfaces), VI. Segredos fora do navegador → VI. Segredos fora do cliente.
- Seções adicionadas/removidas: nenhuma. Restrições técnicas e Fluxo de desenvolvimento reescritos nos itens do app desktop, do endereço público e das duas interfaces.
- Follow-up TODOs: (1) a pasta do código do app nativo, a ferramenta de interface e a biblioteca de ligação de vídeo são decididas no plano da 019 e registradas em `DEFINICOES.md`; (2) `DEFINICOES.md` (linhas 29 e 62), `README.md` (seção do app), `docs/design/transmissao-motor-transporte.md`, `.github/workflows/release.yml` e os contratos das specs 003, 005, 007, 009 e 015 ainda descrevem o app anterior; (3) revisar a spec 015 (atualização) para o app nativo.
- Revisar este relatório temporário e removê-lo antes de um eventual commit; esta execução não cria commit.
- Anterior: 2.0.0 → 3.0.0 (MAJOR: I redefine o caminho obrigatório: dois passos até entrar na sala e clique explícito para começar a receber cada transmissão). Pedido de 2026-11-05, confirmado na remediação da análise da feature 017. Princípio modificado: I. Simplicidade para quem usa (título preservado); II–VI preservados; SFU não autorizada por esta emenda.
- Anterior: 1.5.0 → 2.0.0 (MAJOR: III redefinido — "Sem contas e nada sobre as pessoas": sala fixa guarda nome/último uso/hash da chave do dono no servidor; Suas salas e chaves de dono no navegador) — feature 011, pedido de 2026-10-29/30 (salas fixas e chamada que sobrevive a quedas)
- Anterior: 1.4.0 → 1.5.0 (restrição técnica do app desktop: + aviso de sala, `app_set_room`) — feature 009, pedido de 2026-10-29 (soltar o motor de página escondido fora de sala, Q1: A)
- Anterior: 1.3.0 → 1.4.0 (restrição técnica do app desktop: + aviso de ao vivo para a bandeja, `app_set_live`) — feature 007, pedido de 2026-10-29 (fechar para a bandeja sem perder a transmissão)
- Anterior: 1.2.0 → 1.3.0 (I: som de uma câmera transmitida permitido; restrição: celular transmite câmera) — feature 006, pedido de 2026-10-29 (Q1: C, Q3: B)
- Anterior: 1.1.0 → 1.2.0 (restrição técnica do app desktop: ponte de captura nativa, só para a origem do site) — feature 005, pedido de 2026-10-29
- Anterior: 1.0.1 → 1.1.0 (I: app desktop opcional permitido; restrição técnica do app desktop) — feature 003
- Anterior: 1.0.0 → 1.0.1 (VI: código curto de sala + limite de consultas)
- Anterior: (template) → 1.0.0
- Princípios definidos: I. Simplicidade para quem usa; II. Sem servidor de mídia; III. Efêmero e sem contas (2.0.0: Sem contas e nada sobre as pessoas);
  IV. Camada visual separada; V. Telas mockáveis; VI. Segredos fora do navegador (4.0.0: Segredos fora do cliente)
- Seções adicionadas: Restrições técnicas; Fluxo de desenvolvimento
- Templates: plan-template.md ✅ (Constitution Check usa os gates abaixo) · spec/tasks ✅ sem mudança
-->

# Espia (Screen Share) Constitution

## Core Principles

### I. Simplicidade para quem usa
Do link aberto até entrar na sala MUST haver no máximo dois passos: digitar apelido e entrar.
Começar a receber vídeo e som de cada transmissão alheia MUST depender de clicar em **Assistir**.
Entrar na sala, iniciar transmissão própria, mudar destaque ou restaurar a janela MUST NOT
iniciar recepção não escolhida. Essa escolha evita consumo de mídia que a pessoa não pediu.
Nenhuma funcionalidade pode exigir instalação, conta, login ou senha. Textos da interface são em
português informal (o tom do design "Espia"). Funcionalidades fora do escopo (voz, chat de texto,
contas, amigos) MUST NOT ser adicionadas sem emenda desta constituição. Um app desktop **opcional**
para Windows é permitido (1.1.0). Desde a 4.0.0 ele é nativo, com interface própria. Ele MUST
oferecer o mesmo caminho e obedecer às mesmas regras do site: os dois passos até entrar na sala, o
clique explícito em **Assistir**, os mesmos limites de escopo e os mesmos textos em português
informal. O site MUST continuar funcionando sem instalar nada: entrar numa sala, assistir e
transmitir MUST NOT depender do app.
Voz como recurso próprio (chamada, push-to-talk, transmitir só som) continua fora do escopo; o som
de uma **câmera transmitida** (inclusive o microfone) é permitido, só enquanto a câmera estiver no ar (1.3.0).

### II. Sem servidor de mídia
Vídeo e áudio trafegam direto entre os aparelhos das pessoas, navegador ou app nativo em qualquer
combinação, via WebRTC P2P em malha. O app nativo MUST interoperar com os navegadores alvo do site;
uma sala MUST NOT depender de todos usarem o app. O servidor só faz
sinalização e entrega credenciais de TURN. Salas são limitadas a 10 pessoas (o grupo de amigos), com qualidade reduzida automaticamente
conforme o número de espectadores, para manter o custo de
upload P2P viável. Qualquer proposta de SFU/MCU é uma mudança de arquitetura e precisa de emenda.

### III. Sem contas e nada sobre as pessoas
Não existem contas, login nem perfis. O servidor pode guardar **a sala**: código, nome de uma sala
fixa, último uso e o hash da chave do dono. Nada disso identifica uma pessoa. O servidor MUST NOT guardar apelidos, quem
entrou nem quando, depois que a sala esvazia. Uma sala rápida existe enquanto houver alguém nela, ou
por até 5 min depois que todos caíram. Uma sala fixa some depois de 30 dias sem uso ou quando o dono apaga.
No navegador guardamos só preferências locais (apelido, qualidade, volume por pessoa), as salas fixas
em que se entrou naquele aparelho e as chaves de dono das que ele criou, em `localStorage` (2.0.0).
No app nativo os mesmos dados (apelido, qualidade, volume por pessoa, salas fixas daquele aparelho e
chaves de dono) ficam numa pasta do app no aparelho e MUST NOT ser enviados ao servidor (4.0.0).

### IV. Camada visual separada
Componentes de tela recebem **estado pronto** (view-models) e emitem eventos; não falam com
WebRTC/WebSocket diretamente. A lógica (stores, sinalização, peers) vive em `src/lib/` fora de
`src/ui/`. Cores, fontes, raios e sombras vêm exclusivamente de `src/ui/tokens.css`.
No app nativo (4.0.0), as telas também recebem estado pronto e emitem eventos, e MUST NOT falar
direto com rede, captura ou mídia. Telas e lógica vivem em módulos separados. Cores, fontes, raios e
sombras do app nativo MUST ter os mesmos valores de `src/ui/tokens.css`, que continua sendo a fonte
única do design. Assim, em cada interface, o design pode ser trocado sem tocar na lógica, e a lógica
testada sem UI.

### V. Telas mockáveis
Toda tela e todo estado de tela desenhado MUST ser acessível com dados falsos via parâmetro de URL
(`?mock=<tela>&scene=<estado>`), sem servidor, sem câmera/tela e sem rede.
No app nativo (4.0.0), toda tela e todo estado desenhado MUST ser acessível com dados falsos por uma
opção ao abrir o app, sem servidor, sem câmera, sem captura e sem rede. Uma tela nova só está
"pronta para o visual" quando tem mock; só está "pronta" quando também funciona com dados reais, em
cada interface em que ela existe.

### VI. Segredos fora do cliente
A chave do Cloudflare Realtime TURN fica só no Worker (secret do wrangler). Nem o navegador nem o app
nativo a recebem: recebem apenas credenciais temporárias. O executável do app nativo MUST NOT conter
nenhum segredo. Códigos de sala são aleatórios (CSPRNG) e o Worker limita consultas por IP para impedir varredura.
Uma atualização do app MUST ter a origem verificada antes de ser instalada (4.0.0); a chave que assina
as atualizações fica fora do repositório.

## Restrições técnicas

- Site (front-end): Svelte 5 + Vite + TypeScript (SPA, sem SvelteKit). Ícones Lucide (`@lucide/svelte`).
- Back-end: um Cloudflare Worker (serve os assets estáticos + API) e um Durable Object por sala
  (WebSocket de sinalização). Deploy com `wrangler`. O endereço público oficial é
  `espia.diuliano.com`; `*.workers.dev` continua respondendo para links já compartilhados e para apps
  já instalados. Convites MUST usar só o endereço oficial.
- Navegadores alvo para transmitir: Chrome/Edge desktop; **câmera** também no celular (1.3.0).
  Assistir: qualquer navegador moderno, inclusive celular.
- App desktop (opcional, 4.0.0): nativo em Rust, só Windows, com interface própria, sem motor de
  página embutido e sem carregar o site. Fala com o mesmo servidor pelo mesmo protocolo de sinalização
  do site (`src/shared/protocol.ts` é a referência) e troca mídia com os navegadores via WebRTC. Seu
  código fica em pasta própria, separada de `src/`.
- O que o app nativo pode acessar no computador é só o que transmitir exige: lista de telas e janelas,
  captura de imagem e som, entradas de câmera e de som, bandeja, notificações, registro do link
  `espia://`, pasta de dados própria, e rede até o servidor e até as pessoas da sala. Qualquer outro
  acesso (arquivos da pessoa, teclado ou mouse, controle remoto) exige emenda.
- A ferramenta de interface e a biblioteca de ligação de vídeo do app nativo são escolhidas no plano
  da feature 019, contra as metas de memória e de processador da sua spec, e registradas em
  `DEFINICOES.md`.
- Atualização: o app nativo MUST avisar quando houver versão nova e quando a versão instalada for
  velha demais para a sala, e MUST NOT interromper uma transmissão em andamento para atualizar.
- App anterior (Tauri 2 + WebView2, que carrega o site): continua com suporte do site enquanto o app
  nativo é testado com o grupo. Sua ponte com a página
  (`specs/005-seletor-nativo-desktop/contracts/native-capture.md`, `app_set_live` em
  `specs/007-fechar-para-bandeja/contracts/tray.md`, `app_set_room` em
  `specs/009-memoria-do-app/contracts/engine.md`) fica **apenas** para a origem do site, na janela
  principal, e MUST NOT ganhar comandos novos. Código em `src-tauri/`.
- No site, sem dependências pesadas de UI (sem frameworks de componentes). CSS próprio sobre os tokens.

## Fluxo de desenvolvimento

- Spec Kit: `spec.md` → `plan.md` → `tasks.md` → implementação, em `specs/<nnn>-<nome>/`.
- Ordem de entrega, em cada interface: (1) tela mockada → (2) lógica real atrás da mesma tela.
- Duas interfaces (4.0.0): uma feature que vale para o site e para o app nativo é especificada uma
  vez e só está pronta quando funciona nos dois; quando vale para só um, a spec diz qual.
- Gate mínimo antes de considerar uma tarefa concluída: `npm run check` (svelte-check + tsc) e
  `npm run build` sem erros; mock da tela abre e mostra todos os estados desenhados. Quando a
  tarefa mexe no app: `cargo check` e `cargo test` sem erros na pasta do app, e o mock da tela abre
  no app.
- O app nativo não atualiza junto com o site (4.0.0). Por isso, uma mudança no protocolo de
  sinalização MUST manter funcionando a versão do app publicada antes, ou fazê-la avisar que precisa
  atualizar.
- Mudanças em decisões do `DEFINICOES.md` são registradas lá e em `research.md` da feature.

## Governance

Esta constituição prevalece sobre preferências de implementação. Emendas: editar este arquivo,
subir a versão (MAJOR remove/redefine princípio; MINOR adiciona princípio/seção; PATCH esclarece)
e registrar o motivo no Sync Impact Report. Todo `plan.md` passa pelo Constitution Check.

**Version**: 4.0.0 | **Ratified**: 2026-10-28 | **Last Amended**: 2026-11-11
