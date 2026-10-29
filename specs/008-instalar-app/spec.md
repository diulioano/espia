# Feature Specification: Instalar o app pela tela inicial (e um instalador com a cara do Espia)

**Feature Branch**: `008-instalar-app`

**Created**: 2026-10-29

**Status**: Draft (clarificado)

**Relacionada a**: [003-espia-desktop](../003-espia-desktop/spec.md) (o app e a Release), [007-fechar-para-bandeja](../007-fechar-para-bandeja/spec.md).

**Input**: User description: "crie uma forma na tela de entrada de instalar o app ou adicione um instalador mais bonito também" + "no web também não tem o icone na aba de pesquisa"

## Contexto

Hoje, para instalar o app, a pessoa precisa receber o link da página de Releases do GitHub e achar o arquivo certo na lista. A tela inicial do site não fala do app: diz "Sem conta, sem instalar nada", o que continua verdade (o app é opcional, princípio I).

O instalador é o assistente padrão do Windows, cinza e genérico, sem nada do visual do Espia (cores, castor, fontes).

O site também não tem ícone na aba do navegador: aparece o ícone genérico (e `/favicon.ico` devolve a página do site).

## Clarifications

### Session 2026-10-29

- Q1: Quanto mais bonito fica o instalador? → **A: o assistente do Windows com a arte do Espia** (imagem lateral com o castor, faixa no topo, textos informais, "Abrir o Espia" no fim). Não haverá instalador feito à mão.
- Q2: Onde aparece o convite para instalar? → **A: só na tela inicial.**
- Pedido extra na mesma sessão: o site precisa do ícone do Espia na aba do navegador (US4).

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Baixar o app pela tela inicial (Priority: P1)

Abro o Espia no Chrome do meu PC com Windows. Na tela inicial, vejo um convite discreto: "Tem Windows? Baixe o app do Espia", com uma linha dizendo o que ele traz a mais (escolher a janela numa grade com prévias, som sem vazar o Discord, fechar e continuar na sala). Clico em "Baixar o app" e o download do instalador da versão mais nova começa na hora, sem passar pelo GitHub.

**Why this priority**: é o pedido principal; hoje não existe caminho para o app a partir do site.

**Independent Test**: abrir a tela inicial no Windows, clicar em "Baixar o app" e conferir que baixa o instalador da versão publicada mais nova.

**Acceptance Scenarios**:

1. **Given** a tela inicial num PC com Windows, num navegador, **Then** aparece o convite com o botão "Baixar o app" e o resumo do que o app traz.
2. **Given** clico em "Baixar o app", **Then** começa o download do instalador da versão mais nova, com o nome do arquivo mostrando que é o Espia.
3. **Given** o convite, **Then** ele não compete com "Criar sala" (fica abaixo, menor) e a frase "sem instalar nada" continua valendo: entrar numa sala pelo site não exige o app.
4. **Given** depois de clicar, **Then** aparece uma dica curta: "Se o Windows avisar que protegeu o computador, clique em Mais informações → Executar assim mesmo."

---

### User Story 2 — Instalador com a cara do Espia (Priority: P1)

Abro o instalador baixado e ele já parece o Espia: o castor, as cores e um texto informal ("Bora instalar o Espia"). Ele instala sem pedir administrador e, no fim, oferece "Abrir o Espia" já marcado.

**Why this priority**: é a segunda metade do pedido e a primeira impressão de quem baixa.

**Independent Test**: rodar o instalador num PC sem o app e conferir visual, textos e o "Abrir o Espia" no fim.

**Acceptance Scenarios**:

1. **Given** o instalador aberto, **Then** as telas mostram a arte do Espia (castor, cores do design) e textos em português informal.
2. **Given** o fim da instalação, **Then** a opção "Abrir o Espia" vem marcada e abre o app.
3. **Given** o app já instalado e aberto (talvez escondido na bandeja), **When** rodo o instalador de uma versão nova, **Then** ele fecha o app sozinho, atualiza e reabre se eu deixar marcado.

---

### User Story 3 — Não incomodar quem não pode ou já tem (Priority: P2)

Quem está no celular, no Mac, no Linux ou já dentro do app não vê o convite. Quem dispensou o convite ("Agora não") não vê de novo nesse navegador.

**Why this priority**: evita ruído; o app só existe para Windows.

**Independent Test**: abrir a tela inicial no celular, num Mac (ou trocando o sistema no navegador) e dentro do app: nada de convite. Dispensar no Windows e recarregar: não volta.

**Acceptance Scenarios**:

1. **Given** celular, Mac ou Linux, **Then** o convite não aparece.
2. **Given** a tela inicial aberta dentro do app desktop, **Then** o convite não aparece.
3. **Given** clico em "Agora não", **Then** o convite some e não volta nesse navegador.

---

### User Story 4 — Ícone do Espia na aba do navegador (Priority: P1)

Abro o Espia no navegador e a aba mostra o castor do Espia, não o ícone genérico. O mesmo ícone aparece nos favoritos e, no celular, ao adicionar à tela inicial.

**Why this priority**: pedido direto; é barato e faz o site parecer pronto.

**Independent Test**: abrir o site no Chrome, no Edge e no celular e olhar a aba e os favoritos.

**Acceptance Scenarios**:

1. **Given** qualquer página do site (inicial, sala, mocks), **Then** a aba mostra o ícone do Espia, nítido no tema claro e no escuro do navegador.
2. **Given** adiciono o site à tela inicial do celular, **Then** o atalho usa o ícone do Espia.

### Edge Cases

- **Nenhuma versão publicada ainda / GitHub fora do ar**: o botão leva para a página de Releases como reserva, sem quebrar a tela.
- **Windows em ARM**: o app é x64 e roda via emulação; o convite aparece igual (sem promessa de desempenho).
- **Navegador que esconde o sistema**: na dúvida, não mostra o convite.
- **Antivírus/SmartScreen**: a dica da US1 cobre o aviso do Windows; não há assinatura de código nesta feature.
- **Mocks** (princípio V): o convite tem cena de mock na tela inicial (com e sem convite).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-801**: A tela inicial MUST mostrar, só em navegador de PC com Windows, um convite para baixar o app, com o que ele traz a mais e o botão "Baixar o app".
- **FR-802**: "Baixar o app" MUST baixar direto o instalador da versão publicada mais nova, sem a pessoa escolher arquivo no GitHub. Sem versão disponível, MUST levar à página de Releases.
- **FR-803**: O convite MUST ser secundário a "Criar sala" e "Entrar", e o texto da tela MUST continuar deixando claro que o site funciona sem instalar nada.
- **FR-804**: O convite MUST NOT aparecer no celular, Mac, Linux nem dentro do app desktop.
- **FR-805**: "Agora não" MUST esconder o convite de vez nesse navegador (preferência local).
- **FR-806**: Depois de clicar em baixar, MUST aparecer a dica sobre o aviso do Windows (SmartScreen).
- **FR-807**: O instalador MUST ter a identidade visual do Espia (castor, cores) e textos em português informal, mantendo o assistente do Windows (Q1: A).
- **FR-808**: O instalador MUST oferecer "Abrir o Espia" no fim, marcado por padrão, e continuar sem pedir administrador.
- **FR-809**: O convite MUST ter mock (`?mock=home&scene=...`) com e sem o convite.
- **FR-810**: O convite aparece **só na tela inicial** (Q2: A).
- **FR-811**: Todas as páginas do site MUST ter o ícone do Espia na aba do navegador, em tamanhos para aba, favoritos e atalho no celular.

### Key Entities

- **Convite do app**: visível ou não (sistema, dentro do app, dispensado).
- **Preferência "convite dispensado"**: guardada no navegador.
- **Instalador mais novo**: o arquivo da última versão publicada, com um endereço fixo que sempre aponta para ele.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-801**: Do site aberto ao instalador baixado: 1 clique, sem passar por outra página.
- **SC-802**: 0 convites mostrados no celular, Mac, Linux e dentro do app (conferido nos mocks e em navegador real).
- **SC-803**: Do instalador aberto ao app aberto: no máximo 3 cliques (sem contar o aviso do Windows).
- **SC-804**: A tela inicial continua cabendo sem rolar num notebook 1366×768 com o convite visível.

## Assumptions

- O instalador continua vindo da Release do GitHub (003); não há loja nem site de download separado.
- Sem assinatura de código (custa dinheiro); a dica do SmartScreen fica.
- O site não sabe se o app já está instalado num navegador comum; por isso existe o "Agora não".
- O app só para Windows x64 (decisão da 003/007).
