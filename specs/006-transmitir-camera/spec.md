# Feature Specification: Transmitir câmera ou placa de captura

**Feature Branch**: `006-transmitir-camera`

**Created**: 2026-10-29

**Status**: Draft

**Relacionada a**: [004-seletor-de-tela](../004-seletor-de-tela/spec.md) e [005-seletor-nativo-desktop](../005-seletor-nativo-desktop/spec.md) (o seletor "Escolha o que compartilhar"), [002-audio-sem-discord](../002-audio-sem-discord/spec.md) (regras de som).

**Input**: User description: "Eu gostaria de poder transmitir a minha câmera ou outras entradas, como uma placa de captura. Qual seria o problema de implementar ou os tradeoffs"

## Contexto e análise de tradeoffs

Hoje só dá para transmitir uma tela, janela ou aba. A ideia é também transmitir um **dispositivo de vídeo**: webcam, placa de captura (console, outro PC), câmera virtual etc.

| Ponto | Avaliação |
|---|---|
| Dá pra fazer no site e no app? | ✅ Sim, nos dois. O navegador e o motor do app já sabem listar e abrir câmeras/placas, com a permissão da pessoa. Placas de captura comuns aparecem para o PC como uma "câmera", então entram pelo mesmo caminho. |
| Escolher pela grade, sem segundo seletor? | ✅ Sim, **também no site**. Ao contrário das janelas, a página pode listar os dispositivos (depois da primeira permissão) e mostrar uma prévia ao vivo de cada um. |
| Custo de rede | ➖ O mesmo de transmitir uma tela: cada pessoa que assiste recebe uma cópia do upload de quem transmite. Uma webcam a 720p30 pesa menos que um jogo a 1080p60. Uma placa a 1080p60 pesa igual. |
| Privacidade | ⚠️ Câmera é mais sensível que tela. Precisa ficar óbvio que ela está ligada, e ela não pode ligar sozinha. |
| **Som** | ⚠️ **O ponto delicado.** O som de uma placa de captura (o console) é bom de ter. Mas, para o PC, a placa e um **microfone** são o mesmo tipo de entrada. Aceitar qualquer entrada de som vira, na prática, **chat de voz**, e a constituição deixa isso fora do escopo (princípio I). |
| Tela + câmera ao mesmo tempo ("facecam") | ⚠️ Hoje cada pessoa transmite **uma** coisa. Duas ao mesmo tempo dobram o upload de quem transmite e mudam o palco: quem assiste vê duas telas da mesma pessoa. |
| Celular transmitir a câmera | ⚠️ Possível (o celular tem câmera e o navegador dele permite), mas hoje o celular só assiste. O upload pelo 4G e a bateria pesam. |
| Placa de captura com som "por fora" (via cabo de áudio) | ℹ️ Algumas placas mandam o som por um dispositivo separado, ou só pelo programa do fabricante. Sem o som certo, vai só a imagem. |
| Atraso/qualidade da placa | ℹ️ A placa entrega a resolução que suporta. O Espia reduz para 720/1080 e 30/60 como hoje. A latência extra da própria placa não muda. |

## Clarifications

### Session 2026-10-29

- Q1: Que som pode ir junto? → **C: qualquer entrada de som, inclusive microfone.** A pessoa vê isso como "compartilhar a câmera" (com a própria voz), não como um chat de voz. Limites para não virar chat de voz: o som só existe **junto de uma transmissão de câmera** (não existe transmitir só som); cada pessoa liga o seu de propósito; e quem assiste controla o volume/mudo de cada um, como já acontece hoje. Exige emenda do princípio I (MINOR): "voz só como som de uma câmera transmitida".
- Q2: Tela e câmera ao mesmo tempo? → **C: fica para uma feature futura.** A 006 é uma coisa por vez.
- Q3: Celular transmite a câmera? → **B: sim, só a câmera** (tela continua só no PC).

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Transmitir a webcam (Priority: P1)

No seletor, abro a nova aba **Câmeras**. Vejo um cartão por câmera com a prévia ao vivo, escolho a minha e clico em **Transmitir**. A galera me vê no palco, como veria uma tela.

**Why this priority**: é o pedido mais direto e não depende das decisões de som.

**Independent Test**: com uma webcam ligada, transmitir pela aba Câmeras. Outra pessoa vê a imagem da câmera, com nome e qualidade no tile.

**Acceptance Scenarios**:

1. **Given** é a primeira vez, **When** abro a aba Câmeras, **Then** o Espia explica que vai pedir permissão e só pede depois de um clique meu ("Permitir câmeras").
2. **Given** a permissão dada, **Then** vejo um cartão por dispositivo de vídeo, com o nome dele e a prévia ao vivo. As prévias desligam quando fecho o seletor.
3. **Given** uma câmera escolhida, **When** clico em Transmitir, **Then** a transmissão começa direto, sem outro seletor, e quem assiste vê "câmera de <apelido>".
4. **Given** permissão negada, **Then** a aba explica como liberar, e as abas Janelas/Telas continuam funcionando.

---

### User Story 2 — Transmitir uma placa de captura com o som do console (Priority: P1)

Ligo o videogame na placa de captura. Na aba Câmeras aparece "Placa de captura (nome do dispositivo)". Escolho, deixo o som ligado e transmito. A galera vê e ouve o jogo do console.

**Why this priority**: é o segundo caso pedido e o que mais traz valor (jogar console com a galera assistindo).

**Independent Test**: com uma placa e um console ligados, transmitir com som. Quem assiste vê o jogo e ouve o som do console, e nada do microfone.

**Acceptance Scenarios**:

1. **Given** uma placa escolhida e o som ligado, **Then** o Espia sugere a entrada de som com o nome parecido com o da placa. Posso trocar por qualquer outra entrada.
2. **Given** a entrada é de jogo (placa), **Then** o som vai "cru": sem cancelamento de eco, sem redução de ruído e sem ajuste automático de volume, para o som do jogo não ser "tratado como voz".
3. **Given** a placa entrega 1080p60, **When** escolho 1080p e 60 fps, **Then** a transmissão sai assim, com a mesma redução automática de hoje quando há muita gente assistindo.

---

### User Story 3 — Câmera com a minha voz (Priority: P2)

Escolho a webcam, ligo "Compartilhar áudio" e escolho o microfone. A galera me vê e me ouve enquanto estou transmitindo a câmera.

**Why this priority**: pedido da pessoa (Q1: C), mas o caso principal é a imagem.

**Independent Test**: transmitir a webcam com o microfone. Quem assiste ouve a voz, sem eco, e pode deixar no mudo ou baixar o volume como hoje.

**Acceptance Scenarios**:

1. **Given** a entrada é de voz (microfone), **Then** o som vai **com** cancelamento de eco e redução de ruído, para não voltar o som de quem está nas caixas.
2. **Given** a transmissão começou com som, **Then** começa no mudo para quem assiste (regra atual de autoplay) e cada pessoa liga quando quiser.
3. **Given** paro a transmissão da câmera, **Then** o som para junto. Não existe transmitir só o microfone.
4. **Given** estou com o Discord na mesma chamada, **Then** o seletor avisa: "A galera pode te ouvir duas vezes (aqui e no Discord)."

---

### User Story 4 — Celular transmitindo a câmera (Priority: P3)

No celular, o botão **Transmitir** abre o seletor só com a aba **Câmeras** (frontal e traseira). Escolho uma e transmito.

**Why this priority**: amplia quem transmite (Q3: B), mas pesa no 4G/bateria e é menos usado que o PC.

**Independent Test**: num celular, transmitir a câmera traseira. Um PC na sala vê a imagem.

**Acceptance Scenarios**:

1. **Given** estou no celular, **Then** só aparece a aba Câmeras: tela continua só no PC.
2. **Given** a câmera do celular no ar, **Then** a qualidade começa em 720p30, com o aviso de que transmitir gasta internet e bateria.
3. **Given** a tela do celular apagou ou troquei de app, **Then** a transmissão para, e a galera vê como fonte encerrada.

---

### Fora de escopo (futuro)

- Tela **e** câmera ao mesmo tempo, com dois tiles da mesma pessoa (Q2: C).

### Edge Cases

- **Câmera em uso por outro programa** (OBS, Discord): aviso "Essa câmera está sendo usada por outro programa" e a câmera não é escolhida.
- **Câmera desconectada no meio** (cabo USB): a transmissão acaba sozinha, como hoje quando a fonte termina.
- **Nenhum dispositivo de vídeo**: a aba mostra "Nenhuma câmera ou placa encontrada" e as outras abas seguem normais.
- **Câmera virtual** (OBS Virtual Camera): aparece como um dispositivo comum. É um jeito de mandar cenas montadas no OBS.
- **Prévias de várias câmeras ao mesmo tempo**: no máximo 4 prévias ao vivo. As demais mostram só o nome, com "toque para ver".
- **Privacidade**: enquanto a câmera estiver no ar, o card "Você está ao vivo" mostra "câmera" e quem transmite vê a própria imagem. Nada é gravado.
- **Dispositivo plugado com o seletor aberto**: a lista se atualiza.
- **Eco entre duas pessoas com microfone**: as duas transmitem câmera + microfone, e as caixas de uma entram no microfone da outra. O cancelamento de eco das entradas de voz resolve. Fone de ouvido continua sendo o recomendado (dica no seletor).
- **Não dá pra saber se a entrada é microfone ou placa**: o Espia decide pelo nome (contém "mic"/"microfone" → voz; nome parecido com o da câmera escolhida → jogo). A pessoa pode trocar entre "Som de voz" e "Som de jogo".

## Requirements *(mandatory)*

### Functional Requirements

- **FR-601**: O seletor MUST ganhar a aba **"Câmeras"**, no site e no app, junto de Janelas e Telas inteiras.
- **FR-602**: A permissão de câmera MUST ser pedida **só** depois de um clique explícito na aba ("Permitir câmeras"), nunca ao abrir o seletor.
- **FR-603**: Com a permissão, a aba MUST listar todos os dispositivos de vídeo (webcams, placas, câmeras virtuais), com nome e prévia ao vivo, limitada a 4 prévias simultâneas.
- **FR-604**: As prévias MUST desligar ao fechar o seletor ou ao começar a transmitir. Só fica ligado o dispositivo transmitido.
- **FR-605**: Transmitir um dispositivo MUST seguir o mesmo caminho de uma tela: mesmo palco, mesmo tile, mesmo limite de qualidade por número de espectadores e mesmo "Parar".
- **FR-606**: Com "Compartilhar áudio" ligado na aba Câmeras, a pessoa MUST poder escolher **qualquer** entrada de som (Q1: C). O Espia sugere a de nome parecido com o da câmera/placa. Entradas de **jogo** MUST ir sem cancelamento de eco, sem supressão de ruído e sem ajuste automático de ganho. Entradas de **voz** MUST ir **com** cancelamento de eco e supressão de ruído. A pessoa pode trocar entre "Som de voz" e "Som de jogo".
- **FR-606b**: O som de uma entrada MUST existir só junto de uma transmissão de câmera. Não existe transmitir só som, e parar a câmera para o som.
- **FR-606c**: No celular, o seletor MUST mostrar só a aba Câmeras (frontal/traseira), começando em 720p30 e avisando do gasto de internet/bateria (Q3: B).
- **FR-607**: Quem assiste MUST ver que é uma câmera/placa ("câmera de Rafa"), além da qualidade e do com/sem som que já aparecem hoje.
- **FR-608**: Desconectar o dispositivo, ou ele ficar indisponível, MUST encerrar a transmissão com o aviso de hoje.
- **FR-609**: O dispositivo escolhido MUST ser lembrado (pelo nome) junto com as preferências do seletor.
- **FR-610**: As cenas novas (aba Câmeras sem permissão, com permissão, sem dispositivos, negada, com microfone escolhido, celular, ao vivo com câmera) MUST ter mock por URL (princípio V).

### Key Entities

- **Dispositivo de vídeo**: webcam, placa de captura ou câmera virtual. Tem nome, um identificador local e as resoluções/fps que suporta. Existe só no PC de quem transmite.
- **Dispositivo de som associado**: a entrada de som que acompanha uma placa de captura (mesmo nome ou nome parecido). Seu uso depende da Q1.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-601**: Uma pessoa transmite a webcam em até 3 cliques depois de abrir o seletor (aba → câmera → Transmitir), sem contar a permissão da primeira vez.
- **SC-602**: Com uma placa de captura a 1080p60 e até 4 pessoas assistindo, a imagem chega com a mesma fluidez de uma janela de jogo na mesma resolução.
- **SC-603**: A câmera nunca liga sem um clique explícito: 0 pedidos de permissão ou luz da câmera acesa ao apenas abrir o seletor.
- **SC-604**: A voz de quem transmite só chega à sala com a câmera dela no ar e com o som ligado de propósito: 0 casos de som sem câmera.
- **SC-605**: Com duas pessoas transmitindo câmera + microfone pelas caixas de som, nenhuma ouve o próprio eco de volta de forma perceptível.

## Assumptions

- Uma coisa por vez por pessoa (tela **ou** câmera). Tela + câmera juntas ficam para o futuro (Q2: C).
- A constituição ganha uma emenda MINOR no princípio I: voz continua fora como recurso próprio, mas o som de uma câmera transmitida (inclusive o microfone) é permitido.
- Placas de captura que aparecem para o Windows como câmera (USB/UVC, a maioria das Elgato/AVerMedia/genéricas) são suportadas. Placas que só funcionam pelo programa do fabricante ficam de fora, mas é possível usá-las via câmera virtual do OBS.
- Sem efeitos de câmera (fundo desfocado, espelhar para quem assiste, recortes), filtros ou gravação.
- A malha continua P2P (princípio II); só muda a origem da imagem.
