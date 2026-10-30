# Feature Specification: Salas fixas e chamada que sobrevive a quedas

**Feature Branch**: `011-salas-fixas`

**Created**: 2026-10-30

**Status**: Draft

**Relacionada a**: [001-espia-screen-share](../001-espia-screen-share/spec.md) (salas, entrada, tolerância de queda de 20 s) e [003-espia-desktop](../003-espia-desktop/spec.md) (o app tem a própria lista).

**Input**: User description: "continuamos com a parte das salas, eu criei o design baseado no que tu falaste". Design "Espia" v2, seção 7 "Salas fixas e chamada que sobrevive a quedas" (Claude Design, projeto `dc89098d`): `Espia.dc.html` (F1–F8, S1–S5, Q1–Q3, N1–N4, M4–M5), `Home.dc.html`, `SavedRoom.dc.html`, `Room.dc.html`, `NotFound.dc.html`. Decisões do grilling de 2026-10-29 (glossário em `CONTEXT.md`).

## Contexto

Em 2026-10-29 a chamada da galera caiu, provavelmente por uma publicação nova do site, que derruba a ligação de todo mundo ao mesmo tempo. A sala sumiu, porque hoje ela só espera 20 s por quem caiu. Foi preciso criar outra sala e mandar outro link. O grupo quer duas coisas:

1. **Um link que não muda**, pra fixar no grupo e usar sempre (sala fixa).
2. **Que uma queda não seja sair**: a pessoa volta sozinha pra mesma sala, sem clicar em nada.

Decisões já tomadas (não reperguntar):

- Continua P2P, sem servidor de mídia.
- **Criar sala** continua fazendo a **sala rápida** e segue sendo o caminho principal. **Criar sala fixa** é um botão separado que pede um nome.
- O link da sala fixa usa o mesmo código sorteado de hoje (`/s/k7f-q2x`). O nome nunca entra no link.
- **Dono** é o aparelho que criou a sala (uma chave guardada nele), não uma pessoa nem uma conta. Só o dono renomeia ou apaga.
- A sala fixa some depois de 30 dias sem ninguém entrar, ou quando o dono apaga.
- **Suas salas** é a lista das salas fixas em que se entrou naquele aparelho. Mostra nome e números ("3 pessoas agora · 1 transmitindo"), **nunca apelidos**. O navegador e o app têm listas separadas. "Tirar da lista" vale só pra aquele aparelho.
- Sala rápida em que todos caíram espera 5 min antes de deixar de existir.
- Constituição: o princípio III ("Efêmero e sem contas") vira "Sem contas e nada sobre as pessoas" (2.0.0). O servidor pode guardar a sala (nome, código, último uso, chave do dono), nunca apelidos nem histórico de quem entrou.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Queda não é sair (Priority: P1) 🎯

Estou na sala, assistindo ou transmitindo, e a internet oscila ou o site é publicado de novo. A sala continua na tela e aparece um aviso discreto, "Reconectando… Você continua na sala". O que eu estava assistindo continua no palco enquanto der, e se eu estava transmitindo continuo transmitindo. Quando a ligação volta, aparece "De volta!" e tudo segue sem eu clicar em nada. Se em 1 minuto não voltar, aparece "Não deu pra voltar pra sala." com "Tentar de novo" e "Sair da sala".

**Why this priority**: é a causa do pedido (a chamada caiu e a sala sumiu). Vale para sala rápida e fixa, e sozinha já resolve a queda por publicação.

**Independent Test**: com 3 pessoas numa sala rápida, uma transmitindo, publicar o site (ou derrubar a ligação de todos). Em até 30 s todas voltam sozinhas pra mesma sala, a transmissão aparece de novo pra quem assiste e ninguém precisou recarregar a página.

**Acceptance Scenarios**:

1. **Given** estou numa sala e minha ligação com o servidor cai, **When** a queda começa, **Then** continuo na tela da sala, o aviso "Reconectando… Você continua na sala" aparece no topo do palco, a lista "Na sala" esmaece e o palco segue mostrando o que eu assistia (Q1).
2. **Given** eu estava transmitindo quando caiu, **When** estou reconectando, **Then** minha transmissão não é interrompida: o cartão "Você está ao vivo" mostra "Sua transmissão continua · volta sozinha" e o rodapé mostra "Ao vivo · reconectando…" (Q1).
3. **Given** estou reconectando, **When** a ligação volta, **Then** aparece o aviso "De volta!", a lista volta ao normal e quem estava transmitindo continua aparecendo como ao vivo pra todos (Q2).
4. **Given** estou reconectando há 60 s, **When** a ligação não volta, **Then** aparece a tela "Não deu pra voltar pra sala." com "Tentar de novo" e "Sair da sala" (Q3). "Tentar de novo" recomeça a tentativa; "Sair da sala" volta pra tela inicial.
5. **Given** todas as pessoas de uma **sala rápida** caem ao mesmo tempo, **When** elas voltam em até 5 min, **Then** encontram a mesma sala, com o mesmo link.
6. **Given** todas as pessoas de uma sala rápida caem, **When** ninguém volta em 5 min, **Then** a sala deixa de existir e o link mostra "Essa sala não existe mais." (N4).
7. **Given** uma pessoa clica em "Sair da sala", **When** ela sai, **Then** os outros a veem sair na hora. Sair não é queda e não espera nada.
8. **Given** estou assistindo no celular e caiu, **When** reconecto, **Then** vejo o aviso "Reconectando…" sobre o palco e a lista esmaecida, com "Transmitir" desabilitado enquanto isso (M5).

---

### User Story 2 — Criar uma sala fixa e entrar nela (Priority: P2)

Na tela inicial clico em "Criar sala fixa". Abre um modal que pede o nome ("CS da galera"). Clico em "Criar e entrar" e caio na sala vazia, com o nome no cabeçalho e o link já copiado ("Sala criada — fixe o link no grupo"). Mando o link no grupo. Quem abre vê a tela de entrada com o selo "Sala fixa", o nome em destaque e quem já está lá, digita o apelido e entra.

**Why this priority**: é o "link que não muda". Depende só de criar e entrar; Suas salas e o menu do dono vêm depois.

**Independent Test**: criar a sala fixa "CS da galera", sair, esperar 10 min com ela vazia e abrir o mesmo link: a sala existe, mostra o nome e deixa entrar.

**Acceptance Scenarios**:

1. **Given** estou na tela inicial, **When** olho as ações, **Then** vejo "Criar sala" (principal) e, abaixo, "Criar sala fixa" com "Com nome e um link que não muda." (F1).
2. **Given** clico em "Criar sala fixa", **When** o modal abre, **Then** ele explica "O link não muda. Fixe no grupo e use sempre o mesmo. Some se ficar 30 dias sem uso.", tem o campo "Nome da sala" com contador (até 40), a dica "Aparece só na tela, nunca no link." e os botões "Cancelar" e "Criar e entrar" (F7).
3. **Given** o nome está vazio ou só com espaços, **When** tento criar, **Then** "Criar e entrar" fica desabilitado.
4. **Given** criei a sala, **When** entro, **Then** o cabeçalho mostra o nome da sala no lugar do título, o convite já está copiado e aparece "Sala criada — fixe o link no grupo" (S1).
5. **Given** alguém abre o link de uma sala fixa, **When** a tela de entrada carrega, **Then** mostra o selo "Sala fixa", o nome, quem já está lá e o aviso "Ao entrar, ela fica em Suas salas neste aparelho." (F8).
6. **Given** a sala fixa ficou vazia, **When** alguém abre o link dias depois (menos de 30), **Then** a sala existe e dá pra entrar.
7. **Given** uma sala fixa ficou 30 dias sem ninguém entrar, **When** alguém abre o link, **Then** vê "Esta sala ficou 30 dias sem uso e deixou de existir." com o nome da sala acima (N3).
8. **Given** estou no celular, **When** abro a tela inicial, **Then** também posso criar sala fixa pelo botão "Sala fixa" ao lado de "Criar sala" (M4).
9. **Given** estou dentro de uma sala fixa, **When** olho o cabeçalho, **Then** o subtítulo diz o que está acontecendo ("Ninguém transmitindo agora", "Lu está ao vivo · 1080p · 60 fps").

---

### User Story 3 — Suas salas na tela inicial (Priority: P3)

Abro o Espia e, no lugar da ilustração, vejo o painel "Suas salas · neste aparelho" com as salas fixas em que já entrei. Cada uma mostra o nome, quantas pessoas estão lá agora e quantas transmitem, ou "Vazia · usada ontem". As que têm gente aparecem primeiro, com o selo "AO VIVO" quando alguém transmite. Clico em "Entrar" e vou direto pra sala.

**Why this priority**: tira a necessidade de procurar o link no grupo. Precisa da US2 pra ter o que listar.

**Independent Test**: entrar em 3 salas fixas, voltar à tela inicial: as 3 aparecem, com os números certos de quem está em cada uma, e "Entrar" leva pra sala.

**Acceptance Scenarios**:

1. **Given** nunca entrei numa sala fixa neste aparelho, **When** abro a tela inicial, **Then** vejo a ilustração de sempre e nenhum painel (F1).
2. **Given** entrei em salas fixas, **When** abro a tela inicial, **Then** o painel "Suas salas" substitui a ilustração e cada linha mostra a inicial colorida, o nome, "Sua" se fui eu que criei e o resumo (F2):
   - com gente: "3 pessoas agora · 1 transmitindo" (ou "1 pessoa agora");
   - vazia: "Vazia · usada ontem" ("há 3 dias", "há 2 semanas");
   - com alguém transmitindo: selo "AO VIVO" e anel na inicial.
3. **Given** a lista abre, **When** os números ainda não chegaram, **Then** cada linha mostra o nome e uma barrinha no lugar do resumo (F5), e o "Entrar" já funciona.
4. **Given** tenho 4 ou 5 salas, com gente em umas e vazias em outras, **When** abro a tela inicial, **Then** elas aparecem em dois grupos, "Com gente agora" e "Usadas recentemente", com "N salas neste aparelho" ao lado do título (F4, com no máximo 5 salas).
5. **Given** uma sala da lista foi apagada ou expirou, **When** a lista carrega, **Then** a linha mostra "Esta sala não existe mais", esmaecida, sem "Entrar", com o botão "Tirar da lista" (F6).
6. **Given** clico em "⋯" numa linha, **When** o menu abre, **Then** vejo "Tirar da lista" com "Some só deste aparelho. A sala continua pros outros." Se sou o dono, vejo também "Renomear sala" e "Apagar sala" (F3).
7. **Given** clico em "Tirar da lista", **When** confirmo, **Then** a sala some da lista deste aparelho e continua existindo pros outros.
8. **Given** estou no celular, **When** vejo Suas salas, **Then** as linhas não têm o botão "Entrar": tocar na linha entra (M4).
9. **Given** uso o site no navegador e também o app, **When** entro numa sala fixa pelo app, **Then** ela aparece em Suas salas do app, não na do navegador (listas separadas).
10. **Given** a lista está aberta, **When** os números são consultados, **Then** nenhum apelido é mostrado nem recebido pela tela inicial: só contagens.

---

### User Story 4 — O dono renomeia e apaga (Priority: P4)

Fui eu que criei "CS da galera". Ao lado do nome, no cabeçalho da sala, tenho um "⋯" que ninguém mais tem. Por ele (ou pelo "⋯" em Suas salas) renomeio a sala, e o link continua o mesmo. Se apago a sala, o link para de funcionar pra todo mundo. Se tem gente dentro, elas saem e veem um aviso.

**Why this priority**: manutenção. Sem isso a sala ainda funciona e some sozinha em 30 dias.

**Independent Test**: no aparelho do dono, renomear a sala com outra pessoa dentro: o nome muda na tela dela. Apagar: ela vai pra "Esta sala foi apagada." e o link deixa de funcionar.

**Acceptance Scenarios**:

1. **Given** sou o dono e estou na sala, **When** olho o cabeçalho, **Then** vejo "⋯" ao lado do nome. O menu tem "Renomear sala", "Apagar sala" e "Só aparece pra você — a sala foi criada neste aparelho." (S2).
2. **Given** não sou o dono, **When** estou na sala fixa, **Then** não há "⋯" nem nenhuma forma de renomear ou apagar (S5).
3. **Given** escolho "Renomear sala", **When** o modal abre, **Then** ele mostra "O link continua o mesmo.", o nome atual editável com contador até 40 e "Cancelar"/"Salvar" (S3). Ao salvar, o novo nome aparece pra todos que estão na sala, sem recarregar.
4. **Given** escolho "Apagar sala", **When** o modal abre, **Then** pergunta "Apagar "CS da galera"?", avisa "O link para de funcionar para todo mundo. Não tem como desfazer." e, se tem gente, "Tem 3 pessoas aqui agora. Elas saem da sala e veem um aviso." (S4).
5. **Given** apaguei a sala com gente dentro, **When** a exclusão acontece, **Then** cada pessoa sai da sala e vê "Esta sala foi apagada." / "Quem criou a sala apagou agora há pouco, e todo mundo saiu junto." com o nome da sala acima (N2). As transmissões param.
6. **Given** a sala foi apagada, **When** alguém abre o link depois, **Then** vê "Esta sala foi apagada." / "Quem criou a sala apagou. O link não funciona mais." (N1).
7. **Given** a sala que deixou de existir estava em Suas salas, **When** vejo a tela de "não encontrada", **Then** a segunda ação é "Tirar de Suas salas" em vez de "Colar outro link" (N1–N3).
8. **Given** alguém que não é o dono tenta renomear ou apagar sem a chave do dono, **When** o pedido chega ao servidor, **Then** ele é recusado e nada muda.

---

### Edge Cases

- **Queda e publicação ao mesmo tempo para todos**: todos voltam pra mesma sala, inclusive a rápida (até 5 min). A sala fixa nunca some por queda.
- **Voltou depois da tolerância individual**: se a pessoa demorou demais pra voltar e o servidor já a tirou da sala, ela entra de novo com o mesmo apelido, sem tela de entrada. Pros outros, aparece saindo e entrando.
- **Transmissão durante a queda**: as ligações diretas entre as pessoas podem continuar de pé mesmo sem o servidor. Quem transmitia continua ao vivo. Se ao voltar alguma ligação tiver se perdido, ela é refeita sozinha.
- **Sala cheia na volta**: se, durante a queda, outra pessoa entrou e a sala ficou com 10, quem volta continua tendo o lugar reservado enquanto a tolerância vale. Passou dela, vê "sala cheia" como hoje.
- **Sala apagada durante a minha queda**: ao voltar, vou pra "Esta sala foi apagada." (N2).
- **Dono perdeu a chave** (limpou os dados do navegador, reinstalou o app): ninguém mais renomeia ou apaga. A sala some sozinha em 30 dias sem uso. A sala continua aparecendo como de outra pessoa (sem "Sua").
- **Mesmo dono em dois aparelhos**: o dono é só o aparelho que criou. No outro aparelho a sala aparece sem "Sua" e sem renomear/apagar.
- **Nome**: espaços no começo e no fim são tirados, espaços repetidos viram um, mínimo 1 e máximo 40 caracteres. Caracteres invisíveis ou de controle são recusados. O nome aparece só como texto, nunca como formatação.
- **Nome repetido**: pode haver duas salas fixas com o mesmo nome (o link é que identifica). Em Suas salas aparecem as duas.
- **Limite de Suas salas**: a lista guarda no máximo 5 (pedido de 2026-10-30). Ao entrar numa 6ª, a usada há mais tempo sai da lista. Sair da lista não tira a prova de dono do aparelho: se ele abrir o link de novo, a sala volta como "Sua".
- **Suas salas sem rede**: a lista aparece com os nomes guardados e a barrinha de carregando. Se não der pra perguntar, o resumo some, mas "Entrar" continua.
- **Sala fixa vazia**: não gasta nada enquanto vazia além de ficar guardada. Abrir só a lista (sem entrar) não conta como uso para os 30 dias.
- **Abrir pelo link no app** (`espia://s/<código>`) uma sala fixa: funciona igual ao site e entra em Suas salas do app.
- **Celular**: assiste e cria sala fixa. Renomear e apagar também funcionam, se o celular for o dono.

## Requirements *(mandatory)*

### Functional Requirements

**Queda e volta (US1)**

- **FR-001**: Uma queda da ligação com o servidor MUST NOT tirar a pessoa da tela da sala. A tela continua, com o aviso "Reconectando… Você continua na sala" e a lista "Na sala" esmaecida.
- **FR-002**: Durante a queda, o palco MUST continuar mostrando a transmissão que a pessoa assistia enquanto a ligação direta com quem transmite estiver de pé, e a transmissão de quem transmite MUST continuar.
- **FR-003**: O sistema MUST tentar voltar sozinho, sem nenhuma ação da pessoa, e na volta MUST mostrar "De volta!". Ligações diretas perdidas MUST ser refeitas sozinhas.
- **FR-004**: Depois de 60 s sem conseguir voltar, o sistema MUST mostrar "Não deu pra voltar pra sala." com "Tentar de novo" (recomeça as tentativas) e "Sair da sala".
- **FR-005**: Uma sala rápida em que todos caíram MUST continuar existindo por 5 min. Se alguém voltar nesse tempo, é a mesma sala. Passado isso sem ninguém, ela deixa de existir.
- **FR-006**: "Sair da sala" MUST continuar tirando a pessoa na hora, sem a espera da queda.
- **FR-007**: Durante a queda, ações que dependem do servidor (Transmitir, renomear, apagar) MUST ficar desabilitadas.

**Sala fixa (US2)**

- **FR-008**: A tela inicial MUST oferecer "Criar sala fixa", separado de "Criar sala", no computador e no celular.
- **FR-009**: Criar sala fixa MUST pedir um nome de 1 a 40 caracteres (depois de tirar espaços das pontas e juntar espaços repetidos), sem caracteres de controle ou invisíveis.
- **FR-010**: O link da sala fixa MUST usar o mesmo formato de código sorteado das salas rápidas. O nome MUST NOT fazer parte do link.
- **FR-011**: Depois de criar, a pessoa MUST entrar direto na sala, com o convite já copiado e o aviso "Sala criada — fixe o link no grupo".
- **FR-012**: A sala fixa MUST continuar existindo vazia e só deixar de existir quando (a) ficar 30 dias seguidos sem ninguém entrar ou (b) o dono apagar.
- **FR-013**: Dentro de uma sala fixa, o cabeçalho MUST mostrar o nome da sala no lugar do título, e o subtítulo MUST resumir quem está ao vivo.
- **FR-014**: A tela de entrada por link de uma sala fixa MUST mostrar o selo "Sala fixa", o nome, quem já está lá e "Ao entrar, ela fica em Suas salas neste aparelho."
- **FR-015**: O aparelho que criou a sala MUST guardar a prova de que é o dono, sem conta e sem login.

**Suas salas (US3)**

- **FR-016**: Ao entrar numa sala fixa, o aparelho MUST guardá-la em Suas salas (código, nome, se é dono, último uso). O navegador e o app MUST ter listas independentes.
- **FR-017**: A tela inicial MUST mostrar o painel "Suas salas" no lugar da ilustração quando a lista tiver pelo menos uma sala, e a ilustração quando estiver vazia.
- **FR-018**: Cada linha MUST mostrar nome, "Sua" se o aparelho é o dono, e o resumo: "N pessoa(s) agora · M transmitindo", ou "Vazia · <há quanto tempo foi usada>", ou "Esta sala não existe mais".
- **FR-019**: A consulta de quem está em cada sala MUST devolver só contagens (pessoas, transmitindo) e se a sala existe, nunca apelidos.
- **FR-020**: A ordem MUST ser: com gente agora primeiro, depois as usadas mais recentemente. Com 4 ou mais salas e os dois tipos presentes, MUST agrupar em "Com gente agora" e "Usadas recentemente".
- **FR-021**: Cada linha MUST ter "⋯" com "Tirar da lista" (só deste aparelho). Para o dono, MUST também ter "Renomear sala" e "Apagar sala".
- **FR-022**: Uma sala que deixou de existir MUST aparecer como "Esta sala não existe mais", sem "Entrar", com "Tirar da lista".
- **FR-023**: No celular, a linha inteira MUST ser o toque para entrar (sem botão "Entrar").
- **FR-024**: A lista MUST guardar no máximo 5 salas; ao passar, a usada há mais tempo sai. A prova de dono MUST continuar guardada no aparelho mesmo quando a sala sai da lista (automaticamente ou por "Tirar da lista").
- **FR-025**: Se o nome de uma sala mudou, Suas salas MUST mostrar o nome novo na próxima vez que a lista carregar.

**Dono (US4)**

- **FR-026**: Só o aparelho dono MUST ver as opções de renomear e apagar (cabeçalho da sala e Suas salas). O servidor MUST recusar renomear ou apagar sem a prova de dono.
- **FR-027**: Renomear MUST manter o link e MUST mostrar o novo nome a todos que estão na sala, sem recarregar.
- **FR-028**: Apagar MUST pedir confirmação, avisando quantas pessoas estão na sala agora. Ao confirmar, a sala deixa de existir na hora, todos que estavam dentro saem com o aviso N2 e o link passa a mostrar N1.

**Sala não encontrada**

- **FR-029**: A tela de sala não encontrada MUST distinguir: sala rápida que acabou ou link errado (N4), sala fixa apagada (N1), apagada com você dentro (N2) e expirada após 30 dias (N3). Nas variações de sala fixa, MUST mostrar o nome dela acima do título quando o aparelho o conhecer.
- **FR-030**: Se a sala estava em Suas salas, a ação secundária MUST ser "Tirar de Suas salas"; senão, "Colar outro link".

**Privacidade e limites**

- **FR-031**: O servidor MUST guardar de uma sala fixa só: código, nome, último uso e a prova de dono. MUST NOT guardar apelidos, quem entrou nem quando cada pessoa entrou depois que a sala esvazia.
- **FR-032**: Criar salas fixas e consultar Suas salas MUST ter limite por origem, como hoje para salas rápidas e prévias, de modo que uma lista de até 5 salas carregue sem bater no limite.
- **FR-033**: Uma sala apagada ou expirada MUST ter seus dados apagados do servidor.

**Mocks**

- **FR-034**: Todas as cenas do design MUST abrir com dados falsos via `?mock=<tela>&scene=<cena>`: `home` (first, few, menu, many, loading, gone, create), `join` (fixed), `room` (fixed, created, ownermenu, rename, delete, guest, reconnecting, back, reconnectfail), `notfound` (quick, deleted, kicked, expired, com `&saved=1`) e `mobile` (home, reconnecting).

### Key Entities

- **Sala fixa**: código (o mesmo do link), nome (1–40 caracteres), último uso (quando alguém entrou pela última vez), prova de dono. Guardada no servidor até 30 dias sem uso ou até ser apagada.
- **Sala rápida**: igual a hoje, sem nome e sem dono. Existe enquanto há alguém ou por até 5 min depois que todos caíram.
- **Prova de dono**: segredo criado junto com a sala e guardado só no aparelho que criou. O servidor guarda apenas o suficiente para conferir, não o segredo em si.
- **Item de Suas salas** (no aparelho): código, nome conhecido, se o aparelho é o dono (junto com a prova), último uso neste aparelho.
- **Resumo da sala** (consultado, nunca guardado): existe ou não, motivo se não existe (apagada/expirada), nome atual, quantas pessoas, quantas transmitindo.
- **Estado da ligação** (na tela da sala): conectada, reconectando (desde quando) ou falhou.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Numa publicação do site com 3+ pessoas na sala, 100% voltam sozinhas pra mesma sala em até 30 s, sem recarregar a página e sem digitar nada.
- **SC-002**: Uma transmissão em andamento continua visível pra quem assiste durante uma queda curta (até 10 s) da ligação com o servidor, sem tela preta.
- **SC-003**: O link de uma sala fixa continua funcionando depois de 7 dias vazia.
- **SC-004**: Da tela inicial, entrar numa sala de Suas salas leva no máximo 2 cliques ("Entrar" e, se ainda não houver apelido, confirmar o apelido).
- **SC-005**: Os números de Suas salas aparecem em até 2 s com uma lista de até 5 salas.
- **SC-006**: Nenhuma resposta usada pela tela inicial para Suas salas contém apelidos (verificável inspecionando as respostas).
- **SC-007**: Pedidos de renomear/apagar sem a prova de dono são recusados em 100% dos casos testados.
- **SC-008**: Criar uma sala fixa, do clique em "Criar sala fixa" até estar dentro com o link copiado, leva menos de 15 s.
- **SC-009**: Todas as 23 cenas do design (com as variações `saved` da tela não encontrada) abrem pelo mock sem servidor, sem câmera e sem rede.

## Assumptions

- "30 dias sem uso" conta a partir da última vez que alguém **entrou** na sala. Abrir a prévia ou consultar Suas salas não conta.
- Tolerância individual de queda: o lugar da pessoa fica guardado até 60 s (hoje 20 s), alinhado com o tempo até a tela "Não deu pra voltar". Passou disso, ela volta como entrada nova, com o mesmo apelido e sem tela de entrada.
- A sala fixa reaproveita a sala de hoje (mesmo limite de 10 pessoas, mesma qualidade, mesmo convite). A diferença é o nome, o dono e a vida longa.
- O dono pode apagar ou renomear também pelo "⋯" de Suas salas, sem entrar na sala (F3).
- "Tirar da lista" não pede confirmação: some na hora (é local e dá pra voltar abrindo o link).
- No celular, renomear/apagar ficam no "⋯" da linha (o design M4 mostra só o resumo; mesmas opções do computador).
- O app desktop não precisa de nenhuma API nativa nova: a lista e a prova de dono ficam no armazenamento da própria página do app.
- Os textos da interface seguem o design. "Go Live" continua sendo "Transmitir".
- Emenda da constituição 2.0.0 (princípio III) acompanha esta feature e é feita antes de implementar.
