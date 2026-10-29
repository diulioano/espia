# Feature Specification: Seletor de tela nativo no app desktop

**Feature Branch**: `005-seletor-nativo-desktop`

**Created**: 2026-10-29

**Status**: Draft

**Relacionada a**: [004-seletor-de-tela](../004-seletor-de-tela/spec.md) (layout do seletor e regras de som), [002-audio-sem-discord](../002-audio-sem-discord/spec.md) (som sem vazar o Discord), [003-espia-desktop](../003-espia-desktop/spec.md) (app desktop) e ao design "Espia" v1.2, seção **4b · App desktop · seletor de tela** (`SourcePicker`). Referências visuais: `tela-esperada.png` (design) × `TELA-atual.png` (v0.1.2).

**Input**: User description: "estou testando manualmente e peguei alguns bugs, como da @tela-esperada.png x @TELA-atual.png. O que eu havia pensado é que a tela de selecionar o que compartilhar não seria a que usamos no navegador padrão, seria uma interna pelo app no desktop, e no navegador via website poderia ser o padrão do Chrome mesmo. Então, há alguma maneira de fazer ficar nativo a parte de selecionar qual tela compartilhar pelo nosso app igual no design da tela esperada?"

## Contexto

- Na v0.1.2 (004), o seletor do Espia só escolhe a **aba** (Janelas / Telas inteiras) e o som. A janela em si é escolhida **depois**, no seletor do Windows/navegador: a pessoa escolhe duas vezes e o visual não é o do design.
- O design (`tela-esperada.png`) mostra o seletor como **única** etapa: grade de miniaturas das janelas abertas (nome da janela, nome do programa, ícone, selo "tocando som"), contagem por aba ("Janelas 6", "Telas inteiras 2"), "Prévias atualizam a cada 2s", e o texto do som citando a janela escolhida ("Só o som de Hollow Knight — nada de notificação vazando.").
- A 004 descartou a grade porque a página não pode listar janelas. Isso continua verdade **no site**. No **app desktop**, o próprio app pode listar e capturar as janelas, então a grade passa a ser possível **só lá**.
- Hoje a constituição (1.1.0) diz que o app desktop "não expõe nenhuma API nativa à página". Esta feature precisa de uma emenda para permitir isso, limitada à escolha e à captura da tela.

## Clarifications

### Session 2026-10-29

- Q: Som da tela inteira no app? → A: B. Vai o som do PC inteiro menos o Discord (US3, FR-509).
- Q: "Seletor igual para todos"? → A: todos devem ter o seletor da `tela-esperada.png`. A primeira mensagem citou `TELA-atual.png` por engano nos nomes. No app desktop, isso é a grade real (US1–US3). No site, o navegador não deixa a página listar janelas, então lá vale a US4 (layout atual + seletor do Chrome), como no pedido original ("no navegador via website pode ser padrão do Chrome").

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Escolher a janela direto no seletor do app (Priority: P1)

No app desktop, clico em **Transmitir**. O seletor do Espia abre na aba **Janelas** com a grade de miniaturas das janelas abertas. Clico no Hollow Knight (fica com borda e ✓), deixo o som ligado e clico em **Transmitir**. A transmissão começa na hora, sem outra janela de escolha do Windows.

**Why this priority**: é o pedido principal. Tira a escolha dupla e deixa o app igual ao design.

**Independent Test**: no app desktop, com 3 janelas abertas, abrir o seletor, escolher uma e transmitir. A sala mostra aquela janela, e nenhum seletor do Windows aparece.

**Acceptance Scenarios**:

1. **Given** o app desktop com várias janelas abertas, **When** abro o seletor, **Then** vejo um cartão por janela com miniatura, título da janela, nome do programa e ícone, e a aba mostra a contagem ("Janelas 6").
2. **Given** a grade aberta, **When** clico num cartão, **Then** ele fica marcado (borda e ✓) e o botão Transmitir é liberado. Sem nenhuma escolha, o botão fica desabilitado.
3. **Given** uma janela escolhida, **When** clico em Transmitir, **Then** a transmissão começa com aquela janela e **nenhum** outro seletor aparece.
4. **Given** o seletor aberto, **When** passam 2 s, **Then** as miniaturas se atualizam. Janelas abertas ou fechadas nesse meio-tempo aparecem ou somem da grade.
5. **Given** a janela escolhida foi fechada antes de eu clicar em Transmitir, **Then** a marcação some e o botão volta a ficar desabilitado.
6. **Given** a lista de janelas, **Then** o próprio Espia não aparece nela.

---

### User Story 2 — Som só da janela escolhida, com selo "tocando som" (Priority: P1)

Na grade, as janelas que estão tocando som mostram o selo **"tocando som"**. Com "Compartilhar áudio" ligado, o texto diz "Só o som de *<janela>* — nada de notificação vazando." e a galera ouve só aquela janela: nada do Discord, nada de notificações.

**Why this priority**: sem isso, a grade nova quebraria a regra mais importante da 002 (não vazar o Discord).

**Independent Test**: com o Discord numa chamada e um vídeo tocando, transmitir a janela do vídeo com som. Quem assiste ouve o vídeo e nenhuma voz da chamada.

**Acceptance Scenarios**:

1. **Given** uma janela tocando som, **Then** o cartão dela mostra o selo "tocando som". Os cartões das janelas em silêncio não mostram o selo.
2. **Given** uma janela escolhida e o som ligado, **Then** o texto sob "Compartilhar áudio" cita o título dela.
3. **Given** transmissão com som, **Then** quem assiste ouve só o som do programa daquela janela.
4. **Given** som desligado, **Then** nenhum som é enviado e o texto diz "Só a imagem vai pra sala."

---

### User Story 3 — Escolher uma tela inteira no seletor do app (Priority: P2)

Troco para a aba **Telas inteiras** ("Telas inteiras 2"). Vejo uma miniatura por monitor ("Tela 1", "Tela 2", com o principal indicado), escolho uma e transmito, sem o seletor do Windows.

**Why this priority**: menos usado que a janela de um jogo, mas faz parte do design e de quem tem dois monitores.

**Independent Test**: num PC com 2 monitores, escolher a tela 2 no seletor do app e confirmar que a sala mostra aquele monitor.

**Acceptance Scenarios**:

1. **Given** a aba Telas inteiras, **Then** há um cartão por monitor com miniatura e nome.
2. **Given** uma tela escolhida e o som ligado, **Then** vai o som do PC inteiro **menos o Discord**, e o texto diz "Todo o som do computador vai junto — menos o Discord." (decisão de 2026-10-29, Q1: B).
3. **Given** uma tela escolhida e o som desligado, **Then** nenhum som é enviado.
4. **Given** o site (fora do app), **Then** a tela inteira continua sempre sem som (regra da 004).

---

### User Story 4 — Site continua com o seletor do navegador (Priority: P2)

No site (Chrome/Edge, sem o app), nada muda em relação à v0.1.2: o seletor do Espia escolhe aba, som e qualidade, e o seletor do navegador escolhe a janela.

**Why this priority**: a pessoa pediu isso explicitamente, e a constituição exige que o site funcione sem instalar nada.

**Independent Test**: no Chrome, abrir o seletor e ver o cartão "Você escolhe a janela no próximo passo". Depois, transmitir pelo seletor do Chrome, como na 004.

**Acceptance Scenarios**:

1. **Given** o site aberto no navegador, **When** abro o seletor, **Then** vejo o layout atual da 004, sem grade de miniaturas.
2. **Given** o site, **Then** todas as regras de som da 002 e da 004 continuam valendo.

---

### User Story 5 — Escolha lembrada (Priority: P3)

Ao abrir o seletor de novo no app, ele volta na última aba e, se a janela da última vez ainda estiver aberta, ela já vem marcada.

**Why this priority**: conforto. A pessoa costuma transmitir sempre o mesmo jogo.

**Independent Test**: transmitir uma janela, parar e abrir o seletor de novo. A mesma janela vem marcada.

**Acceptance Scenarios**:

1. **Given** transmiti a janela X e ela continua aberta, **When** abro o seletor, **Then** X já vem marcada.
2. **Given** X foi fechada, **Then** nada vem marcado.

### Edge Cases

- **App desktop sem a parte nativa** (versão antiga do app ou falha ao listar/capturar): o seletor cai para o comportamento da v0.1.2, com o cartão "próximo passo" e o seletor do Windows, e não fica travado.
- **Janela minimizada**: aparece na grade com a última miniatura disponível ou um ícone no lugar dela. Ao transmitir, a galera vê a janela assim que ela for restaurada.
- **Janela fechada durante a transmissão**: a transmissão para sozinha, como acontece hoje quando a fonte acaba.
- **Janelas sem título ou invisíveis** (ferramentas, bandeja, janelas de fundo): não aparecem na grade.
- **Jogo em tela cheia exclusiva**: o cartão aparece. Se a captura vier preta, o Espia avisa e sugere o modo janela/sem bordas.
- **Monitor conectado ou desconectado com o seletor aberto**: a aba Telas inteiras e a contagem se atualizam.
- **Muitas janelas (20+)**: a grade rola na vertical, sem quebrar o layout e sem rolagem horizontal.
- **Programa que toca som em vários processos** (por exemplo, navegadores): o som enviado é o do programa da janela, não o de outros programas.
- **Tela inteira com som e Discord aberto depois do início**: o Discord continua fora do som, mesmo tendo sido aberto ou reiniciado durante a transmissão.
- **Tela inteira com som**: notificações de outros programas (não do Discord) vão junto. É o custo aceito da opção B, e o texto do seletor não promete o contrário.
- **Privacidade**: as miniaturas só aparecem com o seletor aberto, ficam só no PC de quem transmite e nunca vão para a sala nem para o servidor.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-501**: No app desktop, o seletor MUST listar as janelas abertas e visíveis (sem o próprio Espia) em grade, cada uma com miniatura, título, nome do programa e ícone, conforme `tela-esperada.png`.
- **FR-502**: No app desktop, a aba Telas inteiras MUST listar os monitores com miniatura e nome.
- **FR-503**: As abas MUST mostrar a contagem de itens ("Janelas 6", "Telas inteiras 2"), e o canto superior direito MUST mostrar "Prévias atualizam a cada 2s".
- **FR-504**: As miniaturas e a lista MUST se atualizar a cada 2 s enquanto o seletor estiver aberto e MUST parar de atualizar quando ele fechar.
- **FR-505**: Uma escolha MUST ser obrigatória para liberar o botão principal ("Transmitir", decisão da 004). O item escolhido MUST ficar marcado com borda e ✓.
- **FR-506**: Transmitir no app desktop MUST começar a captura da fonte escolhida **sem** abrir nenhum seletor do Windows ou do navegador.
- **FR-507**: Cartões de janelas que estão tocando som MUST mostrar o selo "tocando som".
- **FR-508**: Com uma janela escolhida e o som ligado, MUST ser enviado **só** o som do programa daquela janela. O texto MUST ser "Só o som de *<título>* — nada de notificação vazando."
- **FR-509**: No app desktop, tela inteira com som ligado MUST enviar o som do PC inteiro **excluindo todo o som do Discord** (todos os seus processos). Se o Discord não puder ser excluído com segurança, o app MUST mandar a tela sem som e avisar quem transmite, nunca o som com o Discord junto. Na aba Telas inteiras, o interruptor de som MUST ficar destravado no app e continuar travado no site.
- **FR-510**: No site (fora do app), o seletor MUST continuar como na v0.1.2 (004), com o seletor do navegador escolhendo a fonte.
- **FR-511**: Se o app não conseguir listar ou capturar as fontes, o seletor MUST cair para o comportamento da v0.1.2, sem mensagem de erro bloqueante.
- **FR-512**: A última aba e a última fonte escolhida (se ainda existir) MUST ser lembradas, junto com as preferências de som e qualidade.
- **FR-513**: As miniaturas MUST ficar só no PC de quem transmite e não podem ser enviadas à sala nem ao servidor.
- **FR-514**: As regras da 002 (FR-201..FR-209) e da 004 (FR-401..FR-408) MUST continuar valendo, com duas exceções no app desktop: a FR-402 (lá a fonte é escolhida na grade, não por dica ao seletor nativo) e a FR-403 (lá a tela inteira pode ter som, conforme a FR-509).
- **FR-515**: Os estados novos do seletor (grade de janelas, grade de telas, janela escolhida com som, lista vazia, fallback) MUST ter mock por URL (princípio V).
- **FR-516**: Resolução, FPS e a dica do rodapé MUST continuar como na 004, e a qualidade escolhida MUST valer para a captura nativa.

### Key Entities

- **Fonte de captura**: janela ou monitor que pode ser transmitido. Tem tipo (janela/tela), título, nome do programa, ícone, miniatura recente, se está tocando som e se é o monitor principal. Existe só no PC de quem transmite e só enquanto o seletor está aberto ou a transmissão está ativa.
- **Escolha do seletor**: aba, fonte escolhida, som ligado/desligado, resolução e FPS. As preferências são lembradas localmente.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-501**: No app desktop, transmitir uma janela exige **uma** escolha só: do clique em Transmitir até a galera ver a tela, a pessoa não vê nenhum seletor do Windows/navegador (0 em 10 tentativas).
- **SC-502**: Com o seletor aberto, a grade aparece em até 1 s e as miniaturas se atualizam a cada 2 s (±0,5 s).
- **SC-503**: Transmitindo a janela de um jogo com som durante uma chamada do Discord, quem assiste ouve o jogo e **nenhuma** voz do Discord (roteiro manual da 002).
- **SC-503b**: Transmitindo a tela inteira com som pelo app, durante uma chamada do Discord, quem assiste ouve o som do PC (jogo, vídeo) e **nenhuma** voz do Discord.
- **SC-504**: O selo "tocando som" está certo para pelo menos 9 de 10 janelas testadas (tocando × em silêncio).
- **SC-505**: No site, o fluxo de transmitir fica idêntico ao da v0.1.2 (os testes atuais continuam passando).
- **SC-506**: O app desktop continua leve: a referência é crescer até ~5 MB em relação à v0.1.2 (1,13 MB), mas não é um limite rígido (decisão de 2026-10-29). O tamanho real é medido e registrado no relatório.
- **SC-507**: Comparando lado a lado com `tela-esperada.png`, o seletor do app tem os mesmos elementos: abas com contagem, grade, selo de som, ✓ de seleção, faixa de som/resolução/FPS e rodapé.

## Assumptions

- O app desktop continua só Windows (10/11), sem macOS.
- O botão principal continua "Transmitir", não "Go Live" (decisão de 2026-10-28).
- "Bugs" no pedido significa a diferença entre o design e a v0.1.2 no seletor. Nenhuma outra falha foi descrita.
- A constituição vai ganhar uma emenda (MINOR) que permite ao app desktop dar à página acesso nativo restrito à **listagem e captura de telas/janelas e do som por programa**, e a nada mais.
- Quem tiver uma versão antiga do app instalada continua com o fluxo da v0.1.2 até atualizar (o site é o mesmo para todos).
- A transmissão continua P2P (princípio II): só muda **de onde vem** a imagem e o som de quem transmite.
