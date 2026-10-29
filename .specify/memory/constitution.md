<!--
Sync Impact Report
- Versão: 1.0.1 → 1.1.0 (I: app desktop opcional permitido; restrição técnica do app desktop) — feature 003
- Anterior: 1.0.0 → 1.0.1 (VI: código curto de sala + limite de consultas)
- Anterior: (template) → 1.0.0
- Princípios definidos: I. Simplicidade para quem usa; II. Sem servidor de mídia; III. Efêmero e sem contas;
  IV. Camada visual separada; V. Telas mockáveis; VI. Segredos fora do navegador
- Seções adicionadas: Restrições técnicas; Fluxo de desenvolvimento
- Templates: plan-template.md ✅ (Constitution Check usa os gates abaixo) · spec/tasks ✅ sem mudança
-->

# Espia (Screen Share) Constitution

## Core Principles

### I. Simplicidade para quem usa
Do link aberto até assistir uma tela deve haver no máximo dois passos: digitar apelido e entrar.
Nenhuma funcionalidade pode exigir instalação, conta, login ou senha. Textos da interface são em
português informal (o tom do design "Espia"). Funcionalidades fora do escopo (voz, chat de texto,
contas, amigos) MUST NOT ser adicionadas sem emenda desta constituição. Um app desktop **opcional**
que abre o mesmo site é permitido (1.1.0); o site MUST continuar funcionando sem instalar nada.

### II. Sem servidor de mídia
Vídeo e áudio trafegam navegador ↔ navegador via WebRTC P2P em malha. O servidor só faz
sinalização e entrega credenciais de TURN. Salas são limitadas a 10 pessoas (o grupo de amigos), com qualidade reduzida automaticamente
conforme o número de espectadores, para manter o custo de
upload P2P viável. Qualquer proposta de SFU/MCU é uma mudança de arquitetura e precisa de emenda.

### III. Efêmero e sem contas
Uma sala existe enquanto houver alguém nela. Nada é persistido no servidor além do estado em
memória do Durable Object da sala. No navegador só guardamos preferências locais (apelido,
qualidade do Go Live, volume por pessoa) em `localStorage`.

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
- Navegadores alvo para transmitir: Chrome/Edge desktop. Assistir: qualquer navegador moderno,
  inclusive celular.
- App desktop (opcional): Tauri 2 + WebView2, só Windows, carrega o site publicado; nenhuma API
  nativa exposta à página. Código em `src-tauri/`.
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

**Version**: 1.1.0 | **Ratified**: 2026-10-28 | **Last Amended**: 2026-10-28
