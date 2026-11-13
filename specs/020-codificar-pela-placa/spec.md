# Feature Specification: Transmitir sem pesar no jogo, codificando uma vez pela placa de vídeo

**Feature Branch**: `main` — nenhuma branch criada nesta etapa; diretório independente da branch.

**Created**: 2026-11-12

**Status**: Draft

**Input**: User description: "investigue oq ele falou e podes dividir em subtrabalhos se precisar, e verifique se é interressante a ideia, visto que a maioria tem placas de video boas"

**Relacionada a**: [019-app-rust-convite](../019-app-rust-convite/spec.md) (app nativo; esta feature decide como a história 3 dela, "Transmitir pelo app nativo", codifica o vídeo), [010-transmissao-leve](../010-transmissao-leve/spec.md) (qualidade por número de espectadores e pelo processador), [017-transmissao-sob-demanda](../017-transmissao-sob-demanda/spec.md) (regra do Assistir), [014-assistir-e-remover](../014-assistir-e-remover/spec.md) (miniatura e tela escondida).

**Investigação**: [investigacao.md](./investigacao.md) traz o que foi conferido, o que se ganha, o que se perde, o que ainda não foi confirmado e os subtrabalhos.

## Contexto

A prova do app nativo (gate G1 da 019) mostrou que, do jeito escolhido, o vídeo é comprimido pelo processador, uma vez para cada espectador. Só para uma pessoa assistindo, isso já gastou o mesmo processador que o app de hoje gasta para três. O app nativo ficaria mais pesado que o de hoje, o contrário do que a migração promete.

A ideia investigada: comprimir o vídeo **uma vez só**, usando a **placa de vídeo**, e mandar o mesmo vídeo a todos que assistem, sempre pela conexão direta entre as pessoas. A maioria do grupo tem placa de vídeo boa, então o caso comum é o favorecido. A investigação concluiu que a ideia vale a pena e que o caminho existe, mas nada foi testado. Por isso a primeira história é a prova.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Saber se funciona antes de construir (Priority: P1)

Sou o dono do Espia. Antes de gastar semanas, quero uma prova pequena que mostre, com números da minha máquina, que comprimir o vídeo uma vez pela placa de vídeo funciona com quem assiste pelo navegador e gasta menos processador que o app de hoje. Se a prova falhar, quero saber cedo e decidir outro caminho.

**Why this priority**: tudo o mais depende disso. A ideia está confirmada só no papel. Uma prova barata evita construir em cima de algo que não funciona.

**Independent Test**: rodar a prova contra o site local, com vídeo de teste, primeiro com uma pessoa assistindo e depois com três. Conferir que a imagem chega e anotar o processador usado em cada caso.

**Acceptance Scenarios**:

1. **Given** a prova transmitindo vídeo de teste 1080p60, **When** uma pessoa clica em Assistir no navegador, **Then** ela vê a imagem em movimento e ouve o som.
2. **Given** a mesma prova, **When** três pessoas assistem ao mesmo tempo, **Then** as três veem a imagem, e o processador usado fica perto do usado com uma pessoa só.
3. **Given** duas pessoas já assistindo, **When** uma terceira clica em Assistir no meio da transmissão, **Then** ela passa a ver a imagem em poucos segundos, sem a imagem das outras travar por mais de um instante.
4. **Given** a prova terminou, **When** leio o registro, **Then** ele traz o processador usado, a taxa de quadros que chegou a quem assiste e um "passou" ou "falhou" para cada item.
5. **Given** a prova falhou em algum item, **Then** o trabalho para ali, o motivo fica registrado e a decisão de como o app nativo transmite volta a ser discutida.

---

### User Story 2 - Transmitir sem derrubar o jogo (Priority: P1)

Estou jogando e transmito a tela do jogo em 1080p60 para os amigos pelo app. O jogo continua liso: perde menos quadros do que perde hoje com o app instalado, e o app quase não aparece no uso do processador.

**Why this priority**: é a razão de existir do app nativo. Sem isso, a migração inteira perde o sentido.

**Independent Test**: na mesma máquina e na mesma sessão, com o mesmo jogo, transmitir 1080p60 para 3 pessoas primeiro pelo app de hoje e depois pelo app nativo. Comparar o FPS do jogo e o processador usado pelo app.

**Acceptance Scenarios**:

1. **Given** estou jogando e transmitindo 1080p60 para 3 pessoas, **When** olho o contador de FPS do jogo, **Then** a perda em relação a jogar sem transmitir não é maior que a do app de hoje.
2. **Given** a mesma transmissão, **When** olho o uso de processador do app, **Then** ele é no máximo a metade do que o app de hoje usa na mesma situação.
3. **Given** estou transmitindo, **When** o jogo muda de uma cena parada para uma cena de muito movimento, **Then** a imagem de quem assiste continua fluida, sem congelar.
4. **Given** escolhi a qualidade ao começar a transmitir, **When** começo, **Then** os passos são os mesmos de hoje, sem pergunta nova e sem nenhuma opção sobre placa de vídeo.

---

### User Story 3 - Mais gente assistindo não pesa mais (Priority: P1)

Estou transmitindo para uma pessoa. Entram mais seis para assistir. O jogo continua igual: não sinto diferença entre transmitir para uma pessoa e para a sala cheia.

**Why this priority**: hoje cada pessoa a mais custa uma compressão a mais. É o que faz a transmissão pesar justamente nas noites em que o grupo todo está.

**Independent Test**: transmitir e medir o processador do app com 1, 3 e 7 pessoas assistindo em tela cheia.

**Acceptance Scenarios**:

1. **Given** estou transmitindo para 1 pessoa, **When** passam a ser 7 assistindo, **Then** o processador usado pelo app quase não muda.
2. **Given** várias pessoas assistindo, **When** uma delas sai, **Then** nada muda para as que ficaram.
3. **Given** a sala tem muita gente assistindo, **Then** a regra que já existe de baixar a qualidade conforme o número de espectadores continua valendo, porque o envio pela internet continua crescendo com a plateia.

---

### User Story 4 - Quem assiste não percebe diferença (Priority: P1)

Assisto à transmissão de um amigo pelo navegador no computador, pelo app ou pelo celular. A imagem chega nítida e fluida, com o som junto, como hoje. Se eu deixo a tela dele em miniatura, ela continua atualizando. Se eu paro de assistir, não recebo mais nada.

**Why this priority**: a mudança é do lado de quem transmite. Quem assiste não pode perder nada, em nenhum aparelho.

**Independent Test**: com uma pessoa transmitindo pelo app nativo, assistir pelo Chrome no computador, pelo app, pelo Chrome no celular e pelo Safari no iPhone. Conferir imagem, som, miniatura, parar e voltar a assistir.

**Acceptance Scenarios**:

1. **Given** alguém transmite pelo app nativo, **When** clico em Assistir no navegador, no app ou no celular, **Then** vejo a imagem em poucos segundos e ouço o som.
2. **Given** estou assistindo, **When** a transmissão dura dez minutos, **Then** o som e a imagem continuam juntos.
3. **Given** a tela de alguém está em miniatura para mim, **Then** ela continua atualizando, em qualidade menor, e eu gasto menos internet do que em tela cheia.
4. **Given** estou assistindo, **When** clico para parar de assistir ou escondo a janela, **Then** paro de receber a imagem, como hoje.
5. **Given** parei de assistir, **When** volto a assistir, **Then** a imagem volta em poucos segundos.
6. **Given** não cliquei em Assistir, **Then** não recebo imagem nem som de ninguém.

---

### User Story 5 - Internet fraca de um não estraga a dos outros (Priority: P2)

Somos quatro assistindo. A internet de um de nós está ruim. Ele passa a ver em qualidade menor, mas nós três continuamos vendo em qualidade cheia.

**Why this priority**: com um vídeo só para todos, o risco é o mais fraco puxar todo mundo para baixo. Não é o caso mais comum, mas quando acontece é o que mais irrita.

**Independent Test**: com três pessoas assistindo, limitar a internet de uma delas e observar a qualidade que cada uma recebe.

**Acceptance Scenarios**:

1. **Given** três pessoas assistindo em qualidade cheia, **When** a internet de uma fica fraca, **Then** só ela passa a receber qualidade menor.
2. **Given** essa pessoa recebe qualidade menor, **When** a internet dela melhora, **Then** ela volta sozinha à qualidade cheia.
3. **Given** a internet de quem **transmite** fica fraca, **Then** a qualidade baixa para todos, como hoje, e o indicador de qualidade que já existe mostra a mudança.

---

### User Story 6 - Computador sem placa boa continua transmitindo (Priority: P2)

Meu computador não tem placa de vídeo capaz, ou ela está ocupada com outra coisa. Clico em Transmitir e funciona do mesmo jeito, sem pergunta nova. A qualidade pode ser menor, e o indicador que já existe mostra qual é.

**Why this priority**: a maioria do grupo tem placa boa, mas o Espia não pode deixar de funcionar para quem não tem.

**Independent Test**: num computador sem placa de vídeo capaz (ou com o uso dela desligado para o teste), transmitir e conferir que alguém consegue assistir.

**Acceptance Scenarios**:

1. **Given** o computador não consegue comprimir pela placa de vídeo, **When** clico em Transmitir, **Then** a transmissão começa com os mesmos passos, sem aviso técnico e sem escolha nova.
2. **Given** essa transmissão, **Then** a qualidade é ajustada sozinha para o que o computador aguenta, e o indicador de qualidade mostra a qualidade efetiva.
3. **Given** a placa de vídeo para de funcionar no meio da transmissão, **Then** a transmissão continua pelo outro caminho, com no máximo uma pausa curta, sem eu precisar transmitir de novo.

---

### Edge Cases

- **Pessoa entra para assistir no meio**: ela precisa de um quadro inteiro para começar. Todos recebem esse quadro. A imagem dos outros não pode travar por isso.
- **Muitas pessoas entrando em sequência**: os pedidos de quadro inteiro não podem se acumular a ponto de derrubar a qualidade de todos.
- **Jogo que usa a placa de vídeo no limite**: a compressão passa a disputar a placa com o jogo. O custo em FPS precisa ser medido antes de prometer melhora nesse tipo de jogo.
- **Outro programa gravando ou transmitindo ao mesmo tempo** (gravador de tela, outro app de transmissão): a placa pode recusar mais uma compressão. A transmissão do Espia cai para o outro caminho, sem falhar.
- **Duas placas de vídeo no computador** (notebook com placa integrada e dedicada): a transmissão funciona com a placa em que a janela capturada está.
- **Todos em miniatura, ninguém em tela cheia**: a compressão cheia não precisa rodar.
- **Ninguém assistindo**: nada é comprimido nem enviado, como hoje.
- **Transmissão de câmera**: segue as mesmas regras da transmissão de tela.
- **Quem assiste por um navegador que não aceita o formato**: o caso precisa ser detectado, e a pessoa recebe o vídeo por um caminho compatível ou um aviso claro. Não pode ficar numa tela preta sem explicação.
- **Janela capturada muda de tamanho no meio**: a imagem continua, com no máximo uma pausa curta.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Antes de construir, uma prova MUST mostrar que o vídeo comprimido uma vez pela placa de vídeo chega a quem assiste pelo navegador, com uma e com três pessoas, e MUST registrar o processador usado em cada caso.
- **FR-002**: Se a prova falhar em qualquer item, o trabalho MUST parar e a decisão sobre como o app nativo transmite MUST voltar a ser tomada, com o resultado registrado.
- **FR-003**: Ao transmitir pelo app nativo num computador com placa de vídeo capaz, a compressão do vídeo MUST ser feita pela placa de vídeo, e não pelo processador.
- **FR-004**: O vídeo MUST ser comprimido uma vez e o mesmo resultado MUST ser enviado a todas as pessoas que assistem na mesma qualidade.
- **FR-005**: O envio MUST continuar direto entre as pessoas. Nenhum servidor recebe, guarda ou repassa imagem ou som.
- **FR-006**: MUST existir no máximo duas qualidades ao mesmo tempo para uma transmissão: a cheia e a reduzida. Quem vê em miniatura ou está com internet fraca recebe a reduzida.
- **FR-007**: A qualidade reduzida MUST ser comprimida só enquanto alguém precisa dela, e a cheia só enquanto alguém assiste em tela cheia.
- **FR-008**: A regra do Assistir MUST continuar valendo nos dois sentidos: nada é enviado a quem não clicou em Assistir, a quem parou de assistir ou a quem está com a janela escondida.
- **FR-009**: Uma pessoa que começa a assistir no meio da transmissão MUST receber imagem em poucos segundos, sem que a imagem das outras trave de forma perceptível.
- **FR-010**: Quando a internet de uma pessoa que assiste fica fraca, só ela MUST passar à qualidade reduzida, e MUST voltar sozinha à cheia quando a internet melhorar.
- **FR-011**: A regra de qualidade pelo número de espectadores MUST continuar valendo, e a qualidade efetiva MUST continuar aparecendo no indicador que já existe.
- **FR-012**: Num computador que não consegue comprimir pela placa de vídeo, a transmissão MUST funcionar pelo processador, com os mesmos passos, sem pergunta nova e com a qualidade ajustada sozinha.
- **FR-013**: Se a compressão pela placa falhar no meio da transmissão, a transmissão MUST continuar pelo outro caminho sem a pessoa precisar transmitir de novo.
- **FR-014**: Quem transmite MUST NOT ver nenhuma opção, pergunta ou termo técnico sobre placa de vídeo ou compressão. A escolha é sempre automática.
- **FR-015**: Quem assiste pelo navegador no computador, pelo app, pelo navegador no celular e pelo Safari no iPhone MUST receber imagem e som, sem instalar nada.
- **FR-016**: O som da transmissão e a sincronia entre som e imagem MUST continuar como hoje.
- **FR-017**: O ganho MUST ser medido na mesma máquina, com o mesmo jogo e na mesma sessão, comparando o app de hoje e o app nativo, e os números MUST ficar registrados.
- **FR-018**: O Diagnóstico de mídia MUST mostrar se a transmissão em andamento está sendo comprimida pela placa de vídeo ou pelo processador.

### Key Entities

- **Transmissão**: a tela, janela ou câmera que uma pessoa está mostrando. Tem uma qualidade escolhida e uma qualidade efetiva.
- **Qualidade cheia**: a versão na qualidade efetiva da transmissão. Vai para quem assiste em tela cheia e tem internet para isso.
- **Qualidade reduzida**: a versão menor da mesma transmissão. Vai para quem vê em miniatura ou está com internet fraca.
- **Espectador**: pessoa que clicou em Assistir. A cada momento recebe a cheia, a reduzida ou nada.
- **Caminho de compressão**: pela placa de vídeo ou pelo processador. Escolhido sozinho pelo app, por transmissão.
- **Registro da prova e das medições**: os números e o "passou/falhou" de cada item, guardados junto da feature.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A prova termina com resultado registrado para uma e para três pessoas assistindo, antes de qualquer outra parte desta feature começar.
- **SC-002**: Transmitindo 1080p60 para 3 pessoas pelo app nativo, o app usa no máximo a metade do processador que o app de hoje usa na mesma situação (medido em 2026-11-12: cerca de meio núcleo no app de hoje).
- **SC-003**: Na mesma medição, a perda de FPS do jogo não é maior que a do app de hoje (medido em 2026-11-12: de 300 para 200 a 220).
- **SC-004**: Entre 1 e 7 pessoas assistindo em tela cheia, com a regra de qualidade por número de espectadores valendo, o processador usado pelo app para transmitir cresce no máximo 25%.
- **SC-005**: Quando a qualidade efetiva é 1080p60 (até 4 pessoas em tela cheia, pela regra que já existe), quem assiste em tela cheia recebe pelo menos 53 quadros por segundo.
- **SC-006**: Quem clica em Assistir vê a primeira imagem em até 3 segundos, inclusive quando entra no meio da transmissão.
- **SC-007**: Quando uma pessoa entra para assistir, a imagem das que já assistiam não congela por mais de meio segundo.
- **SC-008**: Som e imagem não se afastam mais de 80 milésimos de segundo ao longo de 10 minutos.
- **SC-009**: Com a internet de um espectador limitada e a qualidade efetiva em 1080p60, os outros continuam recebendo pelo menos 53 quadros por segundo.
- **SC-010**: Num computador sem placa de vídeo capaz, a transmissão começa em 100% das tentativas, com os mesmos passos, em pelo menos 720p30.
- **SC-011**: Chrome no computador, o app, Chrome no celular e Safari no iPhone recebem imagem e som da mesma transmissão.
- **SC-012**: A memória do app ao transmitir não passa da meta que a 019 já fixou para o app numa sala.

## Assumptions

- Esta feature não substitui a 019. Ela decide como a história "Transmitir pelo app nativo" da 019 comprime o vídeo, e as histórias de transmitir da 019 passam a depender do resultado da prova daqui.
- O app de hoje e o site não mudam. Quem transmite pelo navegador continua como está.
- A conexão continua direta entre as pessoas, sem servidor de mídia (constituição, princípio II; ADR 0001). A parte "um servidor repassa o vídeo" da ideia original fica fora.
- "Placa de vídeo capaz" quer dizer uma placa que comprime vídeo no formato que os navegadores já recebem hoje. Assume-se que a maioria do grupo tem. Quais placas e marcas o grupo usa não foi levantado; a prova roda primeiro na máquina do dono.
- Com um vídeo para todos, a qualidade deixa de ser sob medida por espectador. Duas qualidades (cheia e reduzida) são consideradas suficientes para cobrir miniatura e internet fraca.
- O envio pela internet continua crescendo com o número de espectadores, porque o mesmo vídeo sobe uma vez para cada pessoa. Por isso a regra de qualidade por número de espectadores continua.
- As metas de 3 segundos para a primeira imagem e de meio segundo de congelamento, e a troca da qualidade sob medida por duas qualidades (cheia e reduzida), foram aceitas pelo dono em 2026-11-12 e são confirmadas na prova. O limite de 53 quadros por segundo foi definido pelo dono em 2026-11-12.
- Os números do app de hoje usados como base (meio núcleo, 300 para 200 a 220 FPS) vêm de uma leitura de um instante, com LoL, na máquina do dono. A comparação que vale é a feita lado a lado, na mesma sessão. Nessa leitura, parte das 3 pessoas podia não estar assistindo; por isso a medição lado a lado confere no app quantas pessoas estão assistindo no momento.
- A melhora é esperada principalmente em jogos que disputam o processador. Em jogos que usam a placa de vídeo no limite, o ganho não está garantido e será medido.
- O teste no iPhone depende de alguém do grupo com o aparelho. Na prova da 019 ele não foi feito e o risco foi aceito pelo dono; aqui ele volta a ser pedido porque o formato do vídeo muda.
- Qual compressão o app de hoje usa não foi lido. A diferença de processador sugere que ele já usa a placa de vídeo; confirmar no Diagnóstico de mídia é o primeiro subtrabalho.
