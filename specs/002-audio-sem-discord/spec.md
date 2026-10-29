# Feature Specification: Som da transmissão sem vazar o Discord

**Feature Branch**: `002-audio-sem-discord`

**Created**: 2026-10-28

**Status**: Draft

**Relacionada a**: [001-espia-screen-share](../001-espia-screen-share/spec.md) (US4 Transmitir, FR-010, FR-013). Esta feature **substitui** a regra de áudio da 001: "áudio do sistema junto com a tela" deixa de existir.

**Input**: User description: "Áudio da transmissão sem vazar o Discord (nem outros apps). Problema relatado em teste real (2026-10-28): ao transmitir, o som do Discord (vozes da chamada de voz em que a galera está) vai junto pra quem assiste, gerando eco e expondo a conversa. Causa: ao compartilhar a TELA INTEIRA no Chrome/Edge do Windows, o app pede o som do sistema inteiro. Comportamento desejado: a transmissão leva só o som do que foi escolhido. Aba → som da aba. Janela → só o som daquela janela (ex.: o jogo). Tela inteira → sem som, com explicação na interface de que pra levar o som é preciso escolher a janela do jogo ou uma aba. Rede de segurança: se o navegador ignorar e entregar som do sistema com tela inteira, o app descarta esse som e avisa a pessoa. A interface (modal Transmitir, card 'Você está ao vivo', toasts) precisa deixar claro o que vai de som. Fora de escopo: filtrar um app específico do som do sistema, mixar microfone."

## Contexto

A galera usa o Espia **junto** com uma chamada de voz no Discord (o Espia não tem voz, por decisão de produto). No teste de 2026-10-28, quem transmitiu a tela inteira mandou junto todo o som do computador. As vozes da chamada do Discord voltaram para quem estava na chamada, com atraso. Isso causa eco, atrapalha a conversa e expõe sons que a pessoa não quis compartilhar (notificações, outra música, outra chamada).

A regra nova é simples: **o som que vai é o som da coisa que você escolheu mostrar, e nada mais.**

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Transmitir um jogo com o som do jogo e sem o Discord (Priority: P1)

Rafa está numa chamada do Discord com os amigos e quer mostrar o jogo dele. Clica em **Transmitir**, deixa "Compartilhar áudio" ligado e escolhe a **janela do jogo**. Os amigos ouvem o som do jogo, e as vozes do Discord não voltam pela transmissão.

**Why this priority**: É o uso principal (mostrar jogo para a galera na call) e é exatamente o problema relatado. Sem isso, transmitir com som fica inutilizável para o grupo.

**Independent Test**: Com uma chamada de voz do Discord tocando no PC de quem transmite e um jogo (ou vídeo) numa janela, transmitir essa janela com áudio. Do outro lado, tirar o mudo: deve tocar o som da janela e não as vozes do Discord.

**Acceptance Scenarios**:

1. **Given** estou numa chamada do Discord e com um jogo aberto, **When** transmito a janela do jogo com "Compartilhar áudio" ligado, **Then** quem assiste ouve o som do jogo e não ouve nenhuma voz do Discord.
2. **Given** estou transmitindo a janela do jogo, **When** chega uma notificação de outro app ou toca música em outro programa, **Then** esse som não vai para quem assiste.
3. **Given** transmito uma janela que não faz som, **When** os amigos tiram o mudo, **Then** não ouvem nada (sem eco, sem chiado), e a transmissão continua normal.

---

### User Story 2 - Tela inteira vai sem som, e a pessoa entende por quê (Priority: P1)

Bia escolhe **tela inteira** no seletor do navegador. A transmissão começa só com imagem. O app deixa claro, antes e durante, que tela inteira vai sem som e como levar o som (escolher a janela do jogo ou uma aba).

**Why this priority**: Tela inteira é exatamente o caminho que vazou o Discord. Fechar esse caminho é obrigatório. Explicar evita que a pessoa ache que o som "quebrou".

**Independent Test**: Transmitir a tela inteira com "Compartilhar áudio" ligado, com o Discord tocando. Quem assiste não recebe som nenhum. Quem transmite vê no card "sem som (tela inteira)" e recebe um aviso com a dica.

**Acceptance Scenarios**:

1. **Given** abro o modal Transmitir com "Compartilhar áudio" ligado, **When** leio o modal, **Then** ele explica que vai o som da janela ou aba escolhida, que tela inteira vai sem som, e que o Discord e outros apps ficam de fora.
2. **Given** escolho tela inteira no seletor, **When** a transmissão começa, **Then** nenhum som do computador é enviado, o card "Você está ao vivo" mostra "sem som (tela inteira)" e aparece um aviso: "Tela inteira vai sem som. Pra levar o som, transmita a janela do jogo ou uma aba."
3. **Given** o navegador entregou som junto com a tela inteira mesmo assim, **When** a transmissão começa, **Then** o app descarta esse som antes de enviar e mostra o mesmo aviso. Quem assiste não recebe som nenhum.

---

### User Story 3 - Saber o que está indo de som (Priority: P2)

Quem transmite e quem assiste sabem o que está saindo de som: "com som da janela", "com som da aba" ou "sem som".

**Why this priority**: Dá confiança. Quem transmite sabe que o Discord não vazou, e quem assiste sabe se vale a pena tirar o mudo. Não bloqueia o uso.

**Independent Test**: Transmitir uma aba, depois uma janela, depois a tela inteira. Em cada caso, conferir o texto no card de quem transmite e o subtítulo de quem assiste.

**Acceptance Scenarios**:

1. **Given** estou transmitindo uma aba com som, **When** olho o card "Você está ao vivo", **Then** ele mostra "com som da aba".
2. **Given** estou transmitindo uma janela com som, **When** olho o card, **Then** ele mostra "com som da janela".
3. **Given** estou transmitindo sem som (desliguei a opção, tela inteira, ou o que escolhi não deu som), **When** olho o card, **Then** ele mostra "sem som".
4. **Given** estou assistindo alguém, **When** olho o subtítulo da transmissão, **Then** aparece se ela vem com som ou não, para eu saber se tiro o mudo.

---

### Edge Cases

- **Navegador antigo, que não sabe separar o som da janela:** transmitir uma janela vai sem som (nunca com o som do sistema inteiro). O card mostra "sem som" e o aviso sugere transmitir uma aba ou atualizar o navegador.
- **A pessoa troca o que está compartilhando no meio** (o botão "compartilhar esta aba" do navegador): o rótulo de som acompanha a troca. Se a troca for para tela inteira e chegar som do sistema, esse som é descartado na hora.
- **"Compartilhar áudio" desligado no modal:** nada de som, em qualquer escolha. Rótulo "sem som".
- **A pessoa desmarca "compartilhar áudio" no seletor do navegador:** rótulo "sem som", sem aviso de erro.
- **Firefox/Safari** (não mandam som de captura): continua como hoje, só vídeo e rótulo "sem som".
- **Discord no navegador, numa aba:** transmitir **outra** aba não leva o som da aba do Discord. Transmitir a própria aba do Discord leva, porque foi isso que a pessoa escolheu.
- **O próprio Espia tocando som** (o áudio da transmissão de outra pessoa): nunca volta pela sua transmissão. O Espia já não aparece na lista de abas para compartilhar, e tela inteira vai sem som.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-201**: A transmissão MUST levar só o som da superfície escolhida: aba → som da aba; janela → som só daquela janela.
- **FR-202**: Transmitir a **tela inteira** MUST NOT enviar som, mesmo com "Compartilhar áudio" ligado. O seletor do navegador não deve oferecer "som do sistema" junto com a tela inteira.
- **FR-203**: Se, mesmo assim, o navegador entregar som junto com a tela inteira, o app MUST descartar esse som antes de enviar para qualquer pessoa e MUST avisar quem transmite.
- **FR-204**: Se o navegador não conseguir separar o som de uma janela, transmitir essa janela MUST ir sem som. Nunca com o som do sistema inteiro.
- **FR-205**: Se a pessoa trocar a superfície durante a transmissão, as regras FR-201 a FR-204 MUST valer de novo para a nova superfície, na hora.
- **FR-206**: O modal Transmitir MUST explicar, junto da opção "Compartilhar áudio", que vai o som da janela ou aba escolhida, que tela inteira vai sem som e que Discord e outros apps ficam de fora. Substitui o texto atual "O som do jogo ou vídeo vai junto. Funciona melhor com uma aba."
- **FR-207**: O card "Você está ao vivo" MUST mostrar o som atual: "com som da aba", "com som da janela" ou "sem som". No caso de tela inteira, "sem som (tela inteira)". Substitui "com áudio / sem áudio" da FR-013 da 001.
- **FR-208**: Ao começar (ou trocar para) tela inteira com "Compartilhar áudio" ligado, quem transmite MUST receber um aviso: "Tela inteira vai sem som. Pra levar o som, transmita a janela do jogo ou uma aba." Um aviso por transmissão, que some sozinho.
- **FR-209**: Quem assiste MUST ver no subtítulo da transmissão se ela vem com som ou não (ex.: "1080p · 60 fps · com som" / "sem som").
- **FR-210**: Os textos MUST seguir o tom informal em pt-BR do Espia, com o botão chamado "Transmitir".

### Key Entities

- **Superfície compartilhada**: o que a pessoa escolheu no seletor (aba, janela ou tela inteira). Determina se o som pode ir e qual.
- **Som da transmissão**: um de "som da aba", "som da janela" ou "sem som". Vem da superfície mais a opção do modal. É anunciado para a sala junto com a resolução e o fps.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-201**: Num teste com uma chamada de voz do Discord ativa no PC de quem transmite, **0 de 3** transmissões (aba, janela, tela inteira) deixam qualquer voz do Discord chegar a quem assiste.
- **SC-202**: Transmitindo a janela de um jogo ou vídeo com som, quem assiste ouve o som dessa janela em **3 de 3** tentativas, no Chrome/Edge atual do Windows.
- **SC-203**: Quem transmite a tela inteira entende, só pelo que o app mostra, por que foi sem som e como resolver. No teste com a galera, a pessoa consegue passar a transmitir com som em menos de 1 minuto, sem ajuda.
- **SC-204**: Em 100% das transmissões, o rótulo de som no card de quem transmite e no subtítulo de quem assiste bate com o que realmente está saindo.

## Assumptions

- Quem transmite usa Chrome ou Edge atual no Windows (alvo da constituição). Nessas versões o navegador consegue capturar o som só de uma janela. Versões antigas caem no FR-204 (janela sem som).
- Não existe forma de o site tirar **um app específico** (o Discord) do som do sistema. Por isso a regra é "só o som do que foi escolhido", e não "tudo menos o Discord".
- A galera aceita perder o som ao transmitir a tela inteira: o caso de uso com som (jogo, vídeo) funciona transmitindo a janela ou a aba.
- O som de aba já funciona hoje e não muda.
- Microfone (falar pela transmissão) continua fora de escopo. A voz é no Discord.
- O protocolo de "ao vivo" entre as pessoas da sala ganha a informação do tipo de som (aba/janela/nenhum). É uma mudança compatível com o resto da 001.
