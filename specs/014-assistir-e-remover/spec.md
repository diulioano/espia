# Feature Specification: Assistir quando quiser, telas que dormem e remover pessoas

**Feature Branch**: `014-assistir-e-remover`

**Created**: 2026-10-31

**Status**: Draft

**Relacionada a**: [010-transmissao-leve](../010-transmissao-leve/spec.md) (como cada pessoa vê a sua tela: cheia, miniatura ou escondida; escondida pausa o envio), [007-fechar-para-bandeja](../007-fechar-para-bandeja/spec.md) (janela na bandeja), [011-salas-fixas](../011-salas-fixas/spec.md) (dono, queda não é sair).

**Input**: User description: "Clicar para assistir ou deixar de assistir a tela; esconder a tela ao tirar o foco do aplicativo; poder remover pessoas da sala (só o dono). Pelo que investigamos, quem está transmitindo continua carregando a prévia e a tela que está vendo, e isso derruba o FPS."

## Contexto

- Quem transmite e joga ao mesmo tempo perde FPS. Além de codificar a própria tela, o Espia dele continua **desenhando** duas coisas: a prévia ao vivo "Você está ao vivo", na barra do lado, e as telas dos outros que ele está assistindo, mesmo com o Espia minimizado ou na bandeja.
- Hoje não dá pra deixar de assistir alguém: se a pessoa está ao vivo, a tela dela chega e é desenhada.
- Quando uma tela fica **escondida** para alguém, quem transmite já para de mandar o vídeo pra essa pessoa e o som continua (010). Esta feature usa isso para que deixar de assistir e esconder a janela economizem dos **dois** lados. Parar de assistir corta também o som (correção do usuário, 2026-10-31).
- Em sala fixa o dono só pode renomear e apagar; não pode tirar ninguém.

## Clarifications

### Session 2026-10-31

- Q1: Quando alguém começa a transmitir, você assiste sozinho ou só depois de clicar? → **Assiste sozinho, como hoje.** Cada tela ganha "Parar de assistir" / "Assistir". ~~Sem assistir, o som continua.~~ **Corrigido pelo usuário (2026-10-31): parar de assistir corta vídeo E som; o som só continua com o Espia em segundo plano (US2).**
- Q2: Esconder as telas quando o Espia perde o foco? → **Só quando a janela está minimizada, na bandeja ou numa aba de trás.** Perder o foco com o Espia ainda visível (ex.: segundo monitor) não esconde.
- Q3: A sua prévia "Você está ao vivo"? → **Vira imagem parada**, e dá pra focar a própria tela pra vê-la grande no palco, ao vivo, quando quiser.
- Q4: Quem é removido pelo dono? → **Sai da sala com aviso e pode voltar pelo link** (sem bloqueio, porque não há contas).

## User Stories

### User Story 1 — Parar de assistir e voltar a assistir (Priority: P1) 🎯

Estou numa sala com duas pessoas transmitindo e só quero ver uma delas. Na tela da outra clico em "Parar de assistir". O vídeo e o som dela somem pra mim, e a pessoa para de mandá-los pra mim. Quando eu quiser, clico em "Assistir" e a tela volta.

**Why this priority**: é o pedido direto e o jeito mais simples de cortar custo dos dois lados: eu não desenho e quem transmite não manda o vídeo.

**Independent Test**: duas pessoas, uma transmitindo. Quem assiste clica em "Parar de assistir": o vídeo e o som somem e o "N assistindo" de quem transmite cai em 1. Clica em "Assistir": o vídeo volta em poucos segundos.

**Acceptance Scenarios**:

1. **Given** alguém começa a transmitir, **When** estou na sala, **Then** passo a assistir sozinho (como hoje).
2. **Given** estou assistindo uma tela, **When** clico em "Parar de assistir" nela, **Then** o vídeo some e no lugar fica um cartão com o nome, a descrição da transmissão (012) e o botão "Assistir". O som dessa pessoa também para. Ao voltar a assistir, volta com o mesmo volume e mudo de antes.
3. **Given** parei de assistir alguém, **When** olho a transmissão dessa pessoa, **Then** ela me conta como quem **não** está assistindo e para de me mandar o vídeo.
4. **Given** parei de assistir alguém, **When** clico em "Assistir" no cartão, **Then** o vídeo volta em até 3 segundos.
5. **Given** parei de assistir alguém, **When** essa pessoa para de transmitir e depois transmite de novo, **Then** volto a assistir sozinho (a escolha vale para aquela transmissão).
6. **Given** parei de assistir todo mundo que está ao vivo, **When** olho o palco, **Then** vejo só os cartões "Assistir", sem vídeo.
7. **Given** estou no celular, **When** abro a tela de alguém, **Then** também tenho "Parar de assistir" / "Assistir".
8. **Given** a sala cai e volta sozinha (011), **When** reconecto, **Then** quem eu tinha parado de assistir continua sem assistir.

---

### User Story 2 — As telas dormem quando o Espia não está à vista (Priority: P1)

Estou assistindo um amigo e minimizo o Espia (ou fecho para a bandeja, ou troco de aba no navegador) pra jogar. As telas param de chegar e de ser desenhadas, mas continuo ouvindo. Quando abro o Espia de novo, elas voltam sozinhas.

**Why this priority**: é o caso do jogo. Ninguém está olhando, mas hoje o Espia continua desenhando e recebendo tudo.

**Independent Test**: assistir uma transmissão, minimizar o Espia. Quem transmite vê "N assistindo" cair. Restaurar a janela: o vídeo volta em até 3 segundos e o número sobe.

**Acceptance Scenarios**:

1. **Given** estou assistindo uma ou mais telas, **When** minimizo a janela, fecho para a bandeja (app) ou troco para outra aba (site), **Then** todas as telas ficam escondidas: param de chegar e de ser desenhadas. O som continua.
2. **Given** as telas estão escondidas porque o Espia não está à vista, **When** volto para o Espia, **Then** elas voltam sozinhas em até 3 segundos, do jeito que estavam (as que eu tinha parado de assistir continuam paradas).
3. **Given** o Espia está visível num monitor e clico no jogo no outro monitor, **When** o Espia perde o foco mas continua à vista, **Then** nada é escondido.
4. **Given** uma tela está em picture-in-picture, **When** minimizo o Espia, **Then** essa tela continua chegando e aparecendo no picture-in-picture (a pessoa está olhando pra ela).
5. **Given** a tela de alguém está em tela cheia, **When** troco para outro programa com Alt+Tab, **Then** vale a mesma regra: se a janela ficou escondida atrás de outra, a tela dorme; se continua à vista, não.

---

### User Story 3 — Minha prévia não pesa (Priority: P1)

Estou transmitindo. Na barra do lado, "Você está ao vivo" mostra uma imagem parada da minha tela, que se atualiza de tempos em tempos, em vez de um vídeo ao vivo. Se eu quiser conferir como está ficando, clico nela e a minha tela aparece grande no palco, ao vivo. Quando saio disso, ela volta a ser só a imagem parada.

**Why this priority**: é a outra metade do custo de quem transmite, achada na investigação.

**Independent Test**: transmitir; a prévia da barra muda só de poucos em poucos segundos. Clicar nela: a própria tela aparece grande e ao vivo no palco. Esc ou clicar em outra tela: volta a imagem parada.

**Acceptance Scenarios**:

1. **Given** estou transmitindo, **When** olho "Você está ao vivo", **Then** vejo uma imagem parada da minha tela, trocada a cada 5 segundos, mais ou menos.
2. **Given** estou transmitindo, **When** clico na prévia, **Then** a minha tela aparece grande no palco, ao vivo, com o nome "Você" e sem som (pra não ouvir o próprio som de volta), junto das telas dos outros, como um foco a mais.
3. **Given** estou vendo a minha tela grande, **When** aperto Esc, clico em outra tela ou clico de novo na prévia, **Then** a minha tela sai do palco e a barra volta à imagem parada.
4. **Given** estou vendo a minha tela grande, **When** o Espia sai de vista (US2), **Then** ela também para de ser desenhada e volta ao reaparecer.
5. **Given** estou vendo a minha tela grande e escolhi transmitir a tela inteira do monitor em que o Espia está, **When** olho o palco, **Then** vejo o "espelho infinito", que é esperado; ele só aparece porque eu pedi.
6. **Given** o Espia está fora de vista, **When** estou transmitindo, **Then** a imagem parada também para de se atualizar.
7. **Given** transmito câmera, **When** olho a prévia, **Then** vale o mesmo: imagem parada, e grande ao vivo só se eu clicar.
8. **Given** estou no celular, **When** transmito câmera, **Then** não existe "ver grande" (não há barra lateral). Só a prévia parada vale (research R8).

---

### User Story 4 — O dono tira alguém da sala (Priority: P2)

Sou o dono de uma sala fixa e entrou alguém que não devia, ou um amigo deixou o Espia aberto e foi dormir. Na lista "Na sala", escolho "Remover da sala" ao lado da pessoa. Ela sai na hora, vê "O dono te tirou da sala" e volta pra tela inicial.

**Why this priority**: útil, mas mais raro que as outras e exige mudança no servidor.

**Independent Test**: o dono de uma sala fixa remove alguém. A pessoa some da lista de todos, as telas dela param e ela vê o aviso na tela inicial. Ela consegue entrar de novo pelo link.

**Acceptance Scenarios**:

1. **Given** sou o dono de uma sala fixa e estou nela, **When** abro as ações de alguém na lista "Na sala", **Then** vejo "Remover da sala". Ao meu lado não tem essa opção.
2. **Given** não sou o dono, ou a sala é rápida, **When** olho a lista, **Then** não existe "Remover da sala".
3. **Given** escolho "Remover da sala", **When** confirmo ("Tirar Lu da sala? Ela pode voltar pelo link."), **Then** a pessoa sai na hora pra todos, como se tivesse clicado em "Sair da sala", e a transmissão dela, se houver, termina.
4. **Given** fui removido, **When** isso acontece, **Then** volto pra tela inicial com o aviso "O dono te tirou da sala" e o Espia **não** tenta reconectar sozinho, como faria numa queda.
5. **Given** fui removido, **When** abro o link da sala de novo, **Then** entro normalmente.
6. **Given** a pessoa estava na sala em duas abas do mesmo navegador, **When** é removida, **Then** sai, porque as abas dividem o mesmo lugar e só uma está de fato na sala. App e navegador contam como duas pessoas na lista, e o dono remove cada uma (research R7).
7. **Given** alguém que não é o dono tenta remover outra pessoa (ex.: mexendo no site), **When** o pedido chega ao servidor, **Then** é recusado e nada acontece.

### Edge Cases

- **Parei de assistir a única tela ao vivo** e o palco fica só com o cartão "Assistir": o layout de "uma tela só" mostra o cartão grande, sem vídeo.
- **Parar de assistir a tela que está em foco**: o foco continua nela (o cartão fica grande) até eu escolher outra.
- **Picture-in-picture de uma tela que parei de assistir**: o picture-in-picture fecha.
- **Quem transmite fica com 0 assistindo** porque todos pararam ou esconderam: a transmissão continua (o som ainda vai pra quem só escondeu a janela), mostrando "0 assistindo".
- **Teto de qualidade** (010): quem parou de assistir ou escondeu a janela não conta como espectador em tela cheia, então pode liberar qualidade para os outros.
- **Celular com a tela apagada ou o app do navegador em segundo plano**: conta como "fora de vista" (US2).
- **O dono se remove**: não é possível; ele usa "Sair da sala".
- **O dono remove alguém e essa pessoa estava no meio de uma queda** (reconectando): ela não consegue retomar o lugar e vê o aviso quando a ligação volta.
- **Duas abas do dono** (mesmo aparelho): ambas podem remover.
- **Remover alguém de uma sala fixa que acabou de ser apagada**: vale o comportamento da 011 (todos saem com "a sala foi apagada").

## Requirements *(mandatory)*

### Functional Requirements

**Assistir (US1)**

- **FR-001**: Toda transmissão nova MUST começar sendo assistida por todos na sala (como hoje).
- **FR-002**: Cada tela ao vivo, no computador e no celular, MUST ter um controle "Parar de assistir". Sem assistir, ela MUST virar um cartão com nome, descrição (012) e "Assistir".
- **FR-003**: Parar de assistir MUST fazer quem transmite deixar de enviar o vídeo para essa pessoa e deixar de contá-la em "N assistindo". O som também MUST parar, e voltar com o volume e o mudo de antes ao assistir de novo.
- **FR-004**: "Assistir" MUST trazer o vídeo de volta em até 3 segundos.
- **FR-005**: A escolha de não assistir MUST valer até aquela transmissão acabar e MUST sobreviver a uma queda com volta automática.

**Fora de vista (US2)**

- **FR-006**: Quando o Espia está fora de vista (janela minimizada, escondida na bandeja, aba de trás ou tela do celular apagada), todas as telas MUST ficar escondidas: não chegam e não são desenhadas. O som MUST continuar.
- **FR-007**: Só perder o foco, com a janela ainda visível, MUST NOT esconder nada.
- **FR-008**: Ao voltar a ficar à vista, as telas MUST voltar sozinhas em até 3 segundos, respeitando as escolhas da US1.
- **FR-009**: Uma tela em picture-in-picture MUST continuar chegando enquanto estiver no picture-in-picture, mesmo com o Espia fora de vista.

**Sua prévia (US3)**

- **FR-010**: A prévia "Você está ao vivo" MUST mostrar uma imagem parada da própria transmissão, atualizada a cada 5 segundos ±2, e MUST parar de atualizar com o Espia fora de vista.
- **FR-011**: Clicar na prévia MUST mostrar a própria transmissão grande e ao vivo no palco, sempre muda, marcada como "Você". Esc, outra tela ou outro clique na prévia MUST tirá-la do palco.
- **FR-012**: A própria tela no palco MUST NOT mudar o que os outros recebem nem o "N assistindo".

**Remover (US4)**

- **FR-013**: Só o dono de uma sala fixa, estando nela, MUST ver "Remover da sala" ao lado das outras pessoas da lista "Na sala", com confirmação antes.
- **FR-014**: O servidor MUST conferir que o pedido vem do dono antes de remover, e MUST recusar qualquer outro.
- **FR-015**: A pessoa removida MUST sair na hora para todos (igual a "Sair da sala") de todos os lugares em que está naquela sala, e a transmissão dela MUST terminar.
- **FR-016**: A pessoa removida MUST voltar à tela inicial com o aviso "O dono te tirou da sala", e o Espia MUST NOT reconectar sozinho nem retomar o lugar.
- **FR-017**: A remoção MUST NOT bloquear a volta: abrir o link de novo entra normalmente. O servidor MUST NOT guardar quem foi removido (princípio III).

**Geral**

- **FR-018**: Todas as telas e estados novos MUST ter mock (`?mock=room&scene=...`): tela sem assistir, palco só com cartões, própria tela no palco, prévia parada, menu do dono com "Remover da sala", confirmação e aviso de removido na tela inicial. No celular também.

### Key Entities

- **Escolha de assistir**: por pessoa, por transmissão (de quem, se assiste). Fica só naquele aparelho, na memória da sala aberta.
- **Visibilidade do Espia**: à vista / fora de vista. Calculada no aparelho e nunca enviada a ninguém, a não ser pelo efeito de "escondida" que já existe.
- **Remoção**: pedido do dono (com a prova de que é o dono, igual a renomear/apagar da 011) e um aviso para quem saiu. Nada fica guardado depois.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Quem transmite e joga, com o Espia minimizado ou na bandeja, tem pelo menos a mesma taxa de quadros no jogo que teria transmitindo sem ninguém mais ao vivo na sala (medido no mesmo jogo e cenário, antes e depois).
- **SC-002**: Com o Espia fora de vista, o Espia de quem assiste não desenha nenhum quadro de vídeo (exceto o picture-in-picture), e quem transmite deixa de enviar vídeo para essa pessoa em até 3 segundos.
- **SC-003**: Voltar a assistir (pelo botão ou ao reabrir a janela) mostra imagem em até 3 segundos em 9 de cada 10 vezes.
- **SC-004**: Esconder a janela (segundo plano) nunca corta o som, e parar de assistir sempre corta (conferido no site e no app).
- **SC-005**: A prévia de quem transmite troca de imagem no máximo uma vez a cada 3 segundos, salvo quando a pessoa pediu a própria tela grande.
- **SC-006**: O dono remove alguém em 2 cliques (escolher e confirmar), e a pessoa some da lista de todos em até 2 segundos.

## Assumptions

- "Fora de vista" é o que o navegador e o app já informam como página escondida (minimizada, bandeja, aba de trás). Se uma janela fica totalmente coberta por outra sem minimizar, pode ou não contar como escondida, conforme o sistema; isso é aceito.
- Com o Espia em segundo plano o som continua, porque o grupo usa o som pra acompanhar o jogo do amigo. Parar de assistir é uma escolha explícita e corta tudo.
- A miniatura (faixa de baixo no foco) continua como hoje (360p, 15 fps). Esta feature não muda isso.
- Remover reaproveita a prova de dono da 011 (chave guardada no aparelho que criou a sala). Sala rápida não tem dono, então não tem remoção.
- Sem constituição nova: não há API nativa nova no app, e a remoção não guarda nada sobre as pessoas. O app desktop não precisa de versão nova se a visibilidade da página já refletir minimizar e bandeja; se não refletir, o plano decide.
