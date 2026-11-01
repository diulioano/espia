# Feature Specification: Atualizar o app sozinho

**Feature Branch**: `015-atualizar-app`

**Created**: 2026-11-01

**Status**: Draft (clarificado)

**Relacionada a**: [003-espia-desktop](../003-espia-desktop/spec.md) (o app), [007-fechar-para-bandeja](../007-fechar-para-bandeja/spec.md) (o X esconde na bandeja), [008-instalar-app](../008-instalar-app/spec.md) (o instalador e o download pelo site), [009-memoria-do-app](../009-memoria-do-app/spec.md) (app escondido fora de sala).

**Input**: User description: "eu poderia ter alguma forma de quando lançar uma atualização no app, solicitar para atualizar ele ou atualizar qnd reinicializar o app novamente"

## Contexto

O app abre o site publicado. Então toda mudança de tela e de lógica do site já chega sozinha, sem instalar nada. O que **não** chega sozinho é a parte do app em si: o seletor de janelas, o som do PC sem o Discord, a bandeja, a economia de memória e o "dormir" minimizado (0.1.8).

Hoje, para pegar uma versão nova do app, cada amigo precisa saber que ela existe, entrar no site, clicar em "Baixar o app" e rodar o instalador de novo. Na prática, ninguém sabe que saiu versão nova. A 0.1.8, que corta o FPS perdido com o app minimizado, só ajuda quem reinstalou.

Já aconteceu de um app antigo não funcionar com o site novo (app 0.1.7 com site antigo ficou sem som e sem imagem). Hoje a única proteção é publicar na ordem certa e torcer para todo mundo atualizar.

Como o X esconde na bandeja (007), o app quase nunca é fechado de verdade. "Atualizar quando reiniciar" sozinho pode levar semanas.

## Clarifications

### Session 2026-11-01

- Q1: App abaixo da versão mínima → **A: o aviso não tem "Depois" e não some até atualizar, mas o app continua usável** (não trava sala nem transmissão).
- Q2: Atualizar sozinho escondido na bandeja → **A: sim**, na bandeja e fora de sala há 30 min, em silêncio, continuando escondido. Em sala, nunca.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — O app avisa que tem versão nova e atualiza com um clique (Priority: P1)

Saiu uma versão nova do app. Abro o Espia (ou ele já estava aberto) e aparece um aviso discreto: "Tem versão nova do app". Ele tem dois botões: "Atualizar agora" e "Depois". Clico em "Atualizar agora": o app fecha, instala e abre de novo sozinho, em poucos segundos, sem eu baixar nada nem passar pelo site. Se eu estava numa sala, ele volta pra mesma sala.

**Why this priority**: é o pedido principal. Sem isso, as melhorias do app não chegam aos amigos.

**Independent Test**: com a 0.1.8 instalada, publicar uma versão maior. Abrir o app: o aviso aparece. Clicar em "Atualizar agora": o app reabre na versão nova, sem nenhum download manual.

**Acceptance Scenarios**:

1. **Given** o app instalado é mais antigo que o publicado e já estou numa sala (ou a checagem da abertura demorou), **Then** aparece o aviso "Tem versão nova do app" com "Atualizar agora" e "Depois". Abrindo o app do zero com versão nova, ele já se atualiza na abertura (US2, research R6).
2. **Given** o app já estava aberto quando a versão nova foi publicada, **Then** o aviso aparece sozinho em até 1 h, sem eu reabrir nada.
3. **Given** clico em "Atualizar agora", **Then** o app fecha, instala e abre de novo na versão nova, sem eu clicar em mais nada (nada de assistente do instalador, nada de "Avançar").
4. **Given** eu estava numa sala ao clicar em "Atualizar agora", **Then** o app reabre e volta pra mesma sala com o mesmo apelido.
5. **Given** o app já está na versão mais nova, **Then** nenhum aviso aparece.
6. **Given** estou transmitindo, **Then** "Atualizar agora" não derruba a transmissão sem perguntar: o app pede pra confirmar ("Sua transmissão vai parar por alguns segundos").

---

### User Story 2 — "Depois": atualiza sozinho na próxima vez que o app abrir (Priority: P1)

Estou no meio de uma partida e clico em "Depois". O aviso some. Da próxima vez que o app abrir (depois que eu sair pela bandeja, ou quando o PC reiniciar), ele já abre atualizado, sem perguntar de novo.

**Why this priority**: é a segunda metade do pedido ("ou atualizar quando reinicializar"). Ninguém é interrompido numa hora ruim.

**Independent Test**: com versão nova publicada, clicar em "Depois", sair pela bandeja e abrir o app de novo: já abre na versão nova, sem aviso.

**Acceptance Scenarios**:

1. **Given** cliquei em "Depois", **Then** o aviso some e não volta nessa execução do app.
2. **Given** qualquer abertura nova do app (com ou sem "Depois" antes), ainda fora de sala, **Then** ele se atualiza na hora e abre na versão nova.
3. **Given** cliquei em "Depois" e depois saí do app (bandeja → Sair, ou desliguei o PC), **When** abro o app de novo, **Then** ele já abre na versão nova, sem aviso e sem clique.
4. **Given** a instalação na abertura falhou (sem internet, arquivo corrompido), **Then** o app abre normalmente na versão que já tinha, e o aviso volta a aparecer depois.
5. **Given** escolhi "Depois", **Then** a atualização na abertura não deixa o app mais de 15 s sem abrir, contando do clique no ícone.

---

### User Story 3 — App velho demais para o site (Priority: P2)

Publicamos um site que precisa de uma versão mínima do app (como o caso da 0.1.7). Quem está com um app mais antigo que esse mínimo vê o aviso de atualizar, e o aviso diz que sem atualizar o app pode não funcionar direito.

**Why this priority**: evita o "sem som e sem imagem" que já aconteceu. É menos comum que o caso normal (US1).

**Independent Test**: marcar no site uma versão mínima acima da instalada. Abrir o app: o aviso explica que precisa atualizar, não tem "Depois", e mesmo assim dá pra entrar numa sala.

**Acceptance Scenarios**:

1. **Given** o app instalado está abaixo da versão mínima, **Then** o aviso diz "Essa versão do app não funciona mais com o Espia. Atualize pra continuar" sem o botão "Depois". O aviso só some atualizando.
3. **Given** o aviso de versão mínima, **When** entro numa sala ou transmito, **Then** tudo funciona como antes: o aviso não bloqueia nada, só fica visível.
4. **Given** o aviso de versão mínima e estou transmitindo, **Then** "Atualizar agora" pede a mesma confirmação da US1.
2. **Given** o app está acima do mínimo mas abaixo da mais nova, **Then** vale o aviso normal (US1), com "Depois".

---

### User Story 4 — Atualizar sozinho quando ninguém está usando (Priority: P3)

O app fica dias escondido na bandeja, fora de sala. Nesse estado, ninguém está usando, então ele poderia se atualizar sozinho, sem aviso, e continuar escondido na bandeja.

**Why this priority**: resolve o caso "o app nunca é fechado de verdade" (X = bandeja). Complementa US1/US2, que já funcionam sem ela.

**Independent Test**: com o app na bandeja e fora de sala, publicar versão nova. Depois do tempo combinado, o app está na versão nova e continua na bandeja, sem janela aberta.

**Acceptance Scenarios**:

1. **Given** o app está escondido e fora de sala há pelo menos 30 min, e tem versão nova, **Then** ele se atualiza e volta pra bandeja sem abrir janela nem fazer barulho.
2. **Given** estou numa sala (mesmo com a janela escondida, assistindo ou transmitindo), **Then** ele **nunca** se atualiza sozinho.
3. **Given** ele está se atualizando sozinho e clico no ícone ou abro um link `espia://`, **Then** o app abre assim que terminar (até 15 s), já na versão nova.
4. **Given** a atualização silenciosa falhou, **Then** nada aparece; o app continua na bandeja na versão atual e tenta de novo mais tarde.

---

### Edge Cases

- **Sem internet ao procurar versão nova**: nada acontece, nenhum erro aparece. O app tenta de novo mais tarde.
- **Download cai no meio**: não instala nada pela metade. O app continua na versão atual e tenta de novo depois.
- **Arquivo adulterado ou que não veio do Espia**: o app **não** instala e continua na versão atual. Ninguém consegue empurrar um programa estranho fingindo ser atualização.
- **Duas versões novas seguidas** (a pessoa ficou sem abrir o app): instala direto a mais nova, sem passar pelas do meio.
- **Versão publicada mais velha que a instalada** (uma volta atrás): o app não "atualiza pra trás" sozinho.
- **O Windows avisa que protegeu o computador** (SmartScreen): a atualização pelo app não deve parar nesse aviso. Se não der pra evitar, o aviso de atualizar explica o que clicar, como na 008.
- **Antivírus bloqueia ou a instalação falha**: o app continua funcionando na versão atual e mostra "Não deu pra atualizar. Tente baixar pelo site", com o link pro download.
- **Duas janelas/instâncias do app**: não existe (instância única). Um link `espia://` aberto durante a atualização abre a sala quando o app voltar.
- **Site aberto no navegador** (sem app): nada disso aparece. O site já está sempre na versão mais nova.
- **Ninguém atualizou ainda e o site mudou**: o site continua funcionando com apps acima do mínimo (US3), como hoje.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O app MUST descobrir sozinho que existe uma versão mais nova: ao abrir e, enquanto aberto, pelo menos de hora em hora.
- **FR-002**: Com versão nova, o app MUST mostrar o aviso "Tem versão nova do app" com "Atualizar agora" e "Depois". O aviso é discreto: não cobre a tela da sala nem rouba o foco de um jogo.
- **FR-003**: "Atualizar agora" MUST fechar, instalar e reabrir o app sem nenhum outro clique e sem passar pelo site.
- **FR-004**: Depois de atualizar, o app MUST voltar para onde estava: a mesma sala (com o mesmo apelido), ou a tela inicial.
- **FR-005**: Atualizar MUST NOT derrubar uma transmissão sem confirmação explícita.
- **FR-006**: "Depois" MUST esconder o aviso nessa execução e fazer a versão nova ser instalada na próxima abertura do app, sem perguntar de novo.
- **FR-007**: A instalação na abertura MUST NOT segurar o app mais de 15 s. Se falhar, o app abre na versão antiga.
- **FR-008**: O app MUST instalar só atualizações comprovadamente publicadas pelo Espia. Arquivo adulterado, incompleto ou de outra origem é descartado sem instalar.
- **FR-009**: O app MUST NOT voltar para uma versão mais antiga sozinho.
- **FR-010**: O site MUST poder dizer qual é a versão mínima do app que funciona com ele. Abaixo dela, o aviso muda de texto, perde o "Depois" e não some até atualizar, sem bloquear nada do app (US3).
- **FR-011**: Falha de rede ao procurar versão nova MUST NOT mostrar erro. Falha ao instalar mostra "Não deu pra atualizar" com o link de download do site.
- **FR-012**: O botão "Baixar o app" do site (008) MUST continuar baixando sempre a versão mais nova, a mesma que o app oferece.
- **FR-013**: Publicar uma versão nova MUST continuar sendo um só comando de lançamento + o deploy do site de sempre. Não pode precisar de GitHub (decisão de 2026-10-29) nem de nenhum serviço novo pago.
- **FR-014**: Procurar versão nova MUST NOT mandar nada sobre a pessoa: nem apelido, nem sala, nem identificador do aparelho. No máximo a versão instalada.
- **FR-015**: Nada disso aparece no site aberto no navegador.
- **FR-017**: Com o app escondido e fora de sala há 30 min, e versão nova disponível, o app MUST se atualizar sozinho em silêncio e continuar escondido. Em sala (mesmo escondido) MUST NOT (US4).
- **FR-016**: O aviso (normal, versão mínima e falha) MUST ter mock (`?mock=…&scene=…`), como toda tela.

### Key Entities

- **Versão publicada**: número da versão mais nova do app, o instalador dela e a prova de que veio do Espia. Fica junto com o site.
- **Versão mínima**: a menor versão do app que funciona com o site no ar. Muda raramente.
- **Atualização pendente**: marca local de que a pessoa clicou em "Depois" e que a próxima abertura deve instalar.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Do clique em "Atualizar agora" até o app aberto na versão nova: até 30 s numa internet comum.
- **SC-002**: Uma semana depois de publicar uma versão, todos do grupo que usam o app estão nela, sem ninguém precisar avisar no grupo.
- **SC-003**: Zero cliques além de "Atualizar agora" (ou zero cliques no caminho do "Depois").
- **SC-004**: Ninguém perde uma transmissão por causa de uma atualização sem ter confirmado.
- **SC-005**: Um instalador adulterado nunca é instalado (testado com um arquivo modificado).
- **SC-006**: Abrir o app com atualização pendente leva no máximo 15 s a mais que uma abertura normal.

## Assumptions

- Só o app Windows (o único que existe). O site no navegador já atualiza sozinho e não muda.
- O instalador continua sem pedir administrador (instalação só do usuário, como hoje), então atualizar não abre a janela de permissão do Windows.
- A versão publicada e o instalador continuam servidos pelo próprio site (como o "Baixar o app" da 008). Nada de GitHub nem loja.
- Quem está com 0.1.8 ou mais antigo **precisa baixar pelo site uma última vez**: versões antigas não sabem procurar atualização. Da próxima em diante, é sozinho.
- O aviso segue o visual do Espia (castor, tom informal) e fica num canto, como o aviso "O dono te tirou da sala".
- Isso exige uma nova capacidade nativa do app (procurar, baixar, conferir e instalar a atualização). Pela constituição, toda API nativa nova exposta à página precisa de emenda: o plano decide se a página participa (mostra o aviso) ou se o app faz tudo sozinho.
- Sem prova de origem paga (certificado de assinatura de código do Windows): a prova de que a atualização veio do Espia é uma chave própria do projeto, guardada só no PC de quem publica, fora do repositório.
