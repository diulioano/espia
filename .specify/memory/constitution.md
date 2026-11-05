<!--
Sync Impact Report
- Versão: 2.0.0 → 3.0.0 (MAJOR: I redefine o caminho obrigatório: dois passos até entrar na sala e clique explícito para começar a receber cada transmissão). Pedido de 2026-11-05, confirmado na remediação da análise da feature 017.
- Princípio modificado: I. Simplicidade para quem usa (título preservado). Princípios II–VI preservados; SFU não autorizada por esta emenda.
- Seções adicionadas/removidas: nenhuma. Placeholders e TODOs pendentes: nenhum.
- Revisar este relatório temporário e removê-lo antes de um eventual commit; esta execução não cria commit.
- Anterior: 1.5.0 → 2.0.0 (MAJOR: III redefinido — "Sem contas e nada sobre as pessoas": sala fixa guarda nome/último uso/hash da chave do dono no servidor; Suas salas e chaves de dono no navegador) — feature 011, pedido de 2026-10-29/30 (salas fixas e chamada que sobrevive a quedas)
- Anterior: 1.4.0 → 1.5.0 (restrição técnica do app desktop: + aviso de sala, `app_set_room`) — feature 009, pedido de 2026-10-29 (soltar o motor de página escondido fora de sala, Q1: A)
- Anterior: 1.3.0 → 1.4.0 (restrição técnica do app desktop: + aviso de ao vivo para a bandeja, `app_set_live`) — feature 007, pedido de 2026-10-29 (fechar para a bandeja sem perder a transmissão)
- Anterior: 1.2.0 → 1.3.0 (I: som de uma câmera transmitida permitido; restrição: celular transmite câmera) — feature 006, pedido de 2026-10-29 (Q1: C, Q3: B)
- Anterior: 1.1.0 → 1.2.0 (restrição técnica do app desktop: ponte de captura nativa, só para a origem do site) — feature 005, pedido de 2026-10-29
- Anterior: 1.0.1 → 1.1.0 (I: app desktop opcional permitido; restrição técnica do app desktop) — feature 003
- Anterior: 1.0.0 → 1.0.1 (VI: código curto de sala + limite de consultas)
- Anterior: (template) → 1.0.0
- Princípios definidos: I. Simplicidade para quem usa; II. Sem servidor de mídia; III. Efêmero e sem contas (2.0.0: Sem contas e nada sobre as pessoas);
  IV. Camada visual separada; V. Telas mockáveis; VI. Segredos fora do navegador
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
que abre o mesmo site é permitido (1.1.0); o site MUST continuar funcionando sem instalar nada.
Voz como recurso próprio (chamada, push-to-talk, transmitir só som) continua fora do escopo; o som
de uma **câmera transmitida** (inclusive o microfone) é permitido, só enquanto a câmera estiver no ar (1.3.0).

### II. Sem servidor de mídia
Vídeo e áudio trafegam navegador ↔ navegador via WebRTC P2P em malha. O servidor só faz
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

### IV. Camada visual separada
Componentes de tela recebem **estado pronto** (view-models) e emitem eventos; não falam com
WebRTC/WebSocket diretamente. A lógica (stores, sinalização, peers) vive em `src/lib/` fora de
`src/ui/`. Assim o design pode ser trocado sem tocar na lógica, e a lógica testada sem UI.
Cores, fontes, raios e sombras vêm exclusivamente de `src/ui/tokens.css`.

### V. Telas mockáveis
Toda tela e todo estado de tela desenhado MUST ser acessível com dados falsos via parâmetro de URL
(`?mock=<tela>&scene=<estado>`), sem servidor, sem câmera/tela e sem rede. Uma tela nova só está
"pronta para o visual" quando tem mock; só está "pronta" quando também funciona com dados reais.

### VI. Segredos fora do navegador
A chave do Cloudflare Realtime TURN fica só no Worker (secret do wrangler). O navegador recebe
apenas credenciais temporárias. Códigos de sala são aleatórios (CSPRNG) e o Worker limita consultas por IP para impedir varredura.

## Restrições técnicas

- Front-end: Svelte 5 + Vite + TypeScript (SPA, sem SvelteKit). Ícones Lucide (`@lucide/svelte`).
- Back-end: um Cloudflare Worker (serve os assets estáticos + API) e um Durable Object por sala
  (WebSocket de sinalização). Deploy com `wrangler` em `*.workers.dev`.
- Navegadores alvo para transmitir: Chrome/Edge desktop; **câmera** também no celular (1.3.0).
  Assistir: qualquer navegador moderno, inclusive celular.
- App desktop (opcional): Tauri 2 + WebView2, só Windows, carrega o site publicado. Expõe à página
  **apenas** a ponte de captura (`specs/005-seletor-nativo-desktop/contracts/native-capture.md`:
  listar fontes, miniaturas, iniciar/parar captura de vídeo e som) e o aviso de ao vivo para a bandeja
  (`specs/007-fechar-para-bandeja/contracts/tray.md`: um comando, `app_set_live(live)`, sem retorno) e o
  aviso de sala (`specs/009-memoria-do-app/contracts/engine.md`: um comando, `app_set_room(inRoom)`, sem retorno),
  **apenas** para a origem do site,
  na janela principal. Qualquer outra API nativa exige emenda. Código em `src-tauri/`.
- Sem dependências pesadas de UI (sem frameworks de componentes). CSS próprio sobre os tokens.

## Fluxo de desenvolvimento

- Spec Kit: `spec.md` → `plan.md` → `tasks.md` → implementação, em `specs/<nnn>-<nome>/`.
- Ordem de entrega: (1) tela mockada → (2) lógica real atrás da mesma tela.
- Gate mínimo antes de considerar uma tarefa concluída: `npm run check` (svelte-check + tsc) e
  `npm run build` sem erros; mock da tela abre e mostra todos os estados desenhados.
- Mudanças em decisões do `DEFINICOES.md` são registradas lá e em `research.md` da feature.

## Governance

Esta constituição prevalece sobre preferências de implementação. Emendas: editar este arquivo,
subir a versão (MAJOR remove/redefine princípio; MINOR adiciona princípio/seção; PATCH esclarece)
e registrar o motivo no Sync Impact Report. Todo `plan.md` passa pelo Constitution Check.

**Version**: 3.0.0 | **Ratified**: 2026-10-28 | **Last Amended**: 2026-11-05
