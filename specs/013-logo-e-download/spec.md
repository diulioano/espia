# Feature Specification: Logo leva pro início e botão de baixar o app

**Feature Branch**: `013-logo-e-download`

**Created**: 2026-10-30

**Status**: Draft

**Relacionada a**: [008-instalar-app](../008-instalar-app/spec.md) (convite do app na tela inicial, que some para sempre com "Agora não") e [011-salas-fixas](../011-salas-fixas/spec.md) (queda não é sair; sair da sala).

**Input**: User description: "ao clicar no icone volte para home e também adicione um botão de download na versão web"

## Contexto

- O logo (castor + "Espia") aparece no topo de todas as telas, mas não faz nada ao clicar.
- O convite do app (008) aparece só na tela inicial, só no Windows, e some **para sempre** depois de "Agora não". Quem dispensou, ou quem quer o app depois, não tem mais onde baixar.

## Clarifications

### Session 2026-10-30

- Q1: Clicar no logo dentro da sala? → **A: sai da sala e vai pro início**, como "Sair da sala", com confirmação só se você estiver transmitindo.
- Q2: Onde fica o botão "Baixar o app"? → **A: só na tela inicial.**

## User Stories

### User Story 1 — Clicar no logo volta pro início (Priority: P1) 🎯

Estou na tela de entrada de uma sala, na "sala não encontrada" ou dentro de uma sala, e clico no castor/"Espia" no topo. Vou para a tela inicial.

**Why this priority**: é o comportamento esperado de qualquer site. Hoje não há um jeito óbvio de voltar ao início fora da sala.

**Independent Test**: abrir um link de sala (tela de entrada), clicar no logo e cair na tela inicial.

**Acceptance Scenarios**:

1. **Given** estou na tela de entrada, na "sala não encontrada" ou na "sala cheia", **When** clico no logo, **Then** vou para a tela inicial.
2. **Given** já estou na tela inicial, **When** clico no logo, **Then** nada muda (ou a página volta ao topo).
3. **Given** estou dentro de uma sala e não estou transmitindo, **When** clico no logo, **Then** saio da sala (os outros me veem sair na hora, como "Sair da sala") e vou pro início.
4. **Given** estou dentro de uma sala **transmitindo**, **When** clico no logo, **Then** aparece "Você está transmitindo. Parar e sair da sala?". Confirmando, paro, saio e vou pro início; cancelando, nada muda.
5. **Given** uso teclado, **When** o foco chega no logo, **Then** ele tem nome acessível ("Espia — início") e Enter funciona.

---

### User Story 2 — Botão de baixar o app sempre à mão no site (Priority: P2)

Estou no site pelo navegador, no Windows. No topo da tela inicial há um botão "Baixar o app" que baixa o instalador, mesmo que eu já tenha clicado "Agora não" no convite antes.

**Why this priority**: hoje, depois de dispensar o convite, não há como baixar o app pelo site.

**Independent Test**: dispensar o convite ("Agora não"), recarregar a tela inicial e ver o botão "Baixar o app" no topo; clicar baixa o instalador.

**Acceptance Scenarios**:

1. **Given** estou no site, num computador com Windows, **When** abro a tela inicial, **Then** vejo "Baixar o app" no topo, à direita (só na tela inicial).
2. **Given** clico em "Baixar o app", **When** o download começa, **Then** baixa o mesmo instalador do convite (a versão mais nova), sem sair da página.
3. **Given** já dispensei o convite com "Agora não", **When** abro a tela inicial, **Then** o botão continua lá. O convite grande continua dispensado.
4. **Given** estou **no app** desktop, **When** abro qualquer tela, **Then** o botão não aparece.
5. **Given** estou no celular, num Mac ou num Linux, **When** abro a tela inicial, **Then** o botão não aparece, porque o app só existe para Windows.
6. **Given** o convite grande está aparecendo na tela inicial, **When** olho o topo, **Then** o botão pequeno também aparece; os dois baixam o mesmo arquivo.

### Edge Cases

- **Não dá pra saber o sistema** (navegador que esconde o sistema): na dúvida, não mostra, como o convite (008).
- **Clicar no logo no meio de um modal** (Criar sala fixa, Transmitir): o modal fecha junto com a troca de tela.
- **Clicar no logo enquanto cria uma sala** ("Criar e entrar" aguardando): vai pro início; a sala criada continua em Suas salas (011).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O logo MUST ser clicável em todas as telas fora da sala (inicial, entrada, sala não encontrada, sala cheia) e levar à tela inicial.
- **FR-002**: Dentro da sala, o logo MUST sair da sala e ir para o início, igual a "Sair da sala" (saída na hora pros outros). Se a pessoa estiver transmitindo, MUST pedir confirmação antes.
- **FR-003**: O logo MUST ter nome acessível ("Espia — início"), foco visível e mão de link ao passar o mouse, sem mudar a aparência de hoje.
- **FR-004**: No site, num computador com Windows, MUST haver um botão "Baixar o app" (ícone de download) no topo da **tela inicial**, que baixa o instalador da versão mais nova sem sair da página.
- **FR-005**: O botão MUST aparecer mesmo depois de "Agora não" no convite, e MUST NOT aparecer no app desktop, no celular nem fora do Windows (mesma regra de sistema do convite, sem a parte do "dispensado").
- **FR-006**: As telas com o botão MUST ter mock (`?mock=home&scene=default` mostrando o botão; uma cena sem ele, como no app).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: De qualquer tela, voltar ao início leva 1 clique (2 se estiver transmitindo, pela confirmação).
- **SC-002**: Quem dispensou o convite consegue baixar o app em 1 clique a partir da tela inicial.
- **SC-003**: O botão nunca aparece no app, no celular nem fora do Windows (conferido nos três casos).

## Assumptions

- O instalador é o mesmo que o convite já usa, servido pelo próprio site (008).
- O botão é discreto (secundário, no canto de cima), para não competir com "Criar sala".
- Sem mudança no app desktop nem no servidor.
