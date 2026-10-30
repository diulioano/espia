# Feature Specification: Mostrar o que cada pessoa está transmitindo

**Feature Branch**: `012-mostrar-o-que-transmite`

**Created**: 2026-10-30

**Status**: Draft

**Relacionada a**: [004-seletor-de-tela](../004-seletor-de-tela/spec.md) (janela/tela no site), [005-seletor-nativo-desktop](../005-seletor-nativo-desktop/spec.md) (grade de janelas no app, com nome e ícone de cada uma) e [006-transmitir-camera](../006-transmitir-camera/spec.md) (câmera/placa de captura).

**Input**: User description: "vamos criar agora para quando a pessoa tiver compartilhando a tela mostrar qual tela ela está compartilhando"

## Contexto

Hoje quem assiste vê só "Lu está ao vivo" e a qualidade ("1080p · 60 fps · com som"). Não dá pra saber, sem abrir a transmissão, se a Lu está mostrando o jogo, o navegador ou a tela inteira. O design "Espia" já previa isso nas legendas ("tela de Lu · Hollow Knight", "aba de Bia · YouTube"), mas a sala real nunca recebeu essa informação.

O que cada lugar sabe hoje sobre o que está sendo transmitido:

| Onde transmite | O que se sabe |
|---|---|
| App desktop (seletor próprio, 005) | O programa (ex.: "Hollow Knight", "Google Chrome"), o título da janela (ex.: "YouTube — Google Chrome", às vezes com coisas pessoais) e, na tela inteira, "Tela 1"/"Tela 2" |
| Site no navegador (seletor do navegador, 004) | Só o **tipo**: tela inteira, janela ou aba. O navegador não conta o nome da janela nem da aba |
| Câmera (006) | O nome do aparelho (ex.: "HD Pro Webcam C920") ou "câmera frontal"/"traseira" no celular |

## Clarifications

### Session 2026-10-30

- Q1: O que mostrar de uma janela transmitida pelo app? → **A: só o nome do programa** ("Hollow Knight", "Google Chrome"), nunca o título da janela (pode ter coisas pessoais).
- Q2: No site, que só sabe o tipo? → **A: mostrar só o tipo** ("Tela inteira", "Janela", "Aba do navegador"), sem campo novo.
- Q3: Esconder o nome? → **A: não precisa**, já que só aparece o nome do programa.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Ver o que cada um está mostrando (Priority: P1) 🎯

Estou na sala e a Lu e o Tomás estão ao vivo. Na grade, na faixa de miniaturas e na lista "Na sala", vejo que a Lu está mostrando "Hollow Knight" e o Tomás "VS Code", sem precisar abrir cada transmissão.

**Why this priority**: é o pedido. Ajuda a escolher qual tela assistir quando há várias.

**Independent Test**: Lu transmite a janela do Hollow Knight pelo app. Na tela de quem assiste, o cartão da Lu e a linha dela em "Na sala" mostram "Hollow Knight" em até 2 s depois de ela começar.

**Acceptance Scenarios**:

1. **Given** Lu transmite a janela do Hollow Knight pelo app, **When** eu vejo a sala, **Then** o cartão dela mostra "Hollow Knight" (o programa, nunca o título da janela).
2. **Given** Lu transmite a tela inteira pelo app, **When** eu vejo a sala, **Then** aparece "Tela inteira" (ou "Tela 2" se ela tiver mais de um monitor).
3. **Given** Bia transmite pelo site, **When** eu vejo a sala, **Then** aparece o tipo: "Tela inteira", "Janela" ou "Aba do navegador".
4. **Given** Tomás transmite a câmera, **When** eu vejo a sala, **Then** aparece "Câmera" com o nome dela (ex.: "Câmera · HD Pro Webcam C920").
5. **Given** Lu troca o que está transmitindo (para de transmitir e começa outra janela), **When** a nova transmissão começa, **Then** o texto muda junto.
6. **Given** estou no celular, **When** vejo as transmissões, **Then** os chips "Ao vivo agora" e a lista mostram o mesmo texto, encurtado se não couber.
7. **Given** alguém entra na sala depois, **When** a sala carrega, **Then** já vê o que cada um está transmitindo, sem esperar nada mudar.
8. **Given** Lu transmite uma aba do YouTube pela janela do Chrome no app, **When** eu vejo a sala, **Then** aparece "Google Chrome", não o título da aba.

---

### User Story 2 — Eu vejo o que estou mostrando (Priority: P2)

Estou transmitindo. No cartão "Você está ao vivo" vejo exatamente o que a sala está vendo como nome da minha transmissão, para conferir que não escolhi a janela errada.

**Why this priority**: evita mostrar a coisa errada (constituição, segurança A11). Já existe em parte (o app mostra o nome da câmera).

**Independent Test**: transmitir uma janela pelo app; o cartão "Você está ao vivo" mostra o mesmo texto que aparece pra quem assiste.

**Acceptance Scenarios**:

1. **Given** estou transmitindo, **When** olho o cartão "Você está ao vivo", **Then** vejo o mesmo texto que os outros veem.

---

### Edge Cases

- **Nome comprido** (título de janela grande): o texto é cortado com "…" no espaço disponível. O servidor também recusa nomes acima de um limite.
- **Título com coisa pessoal** (assunto de e-mail, nome de arquivo, conversa): nunca aparece, porque só vai o nome do programa (FR-002).
- **Janela muda de título durante a transmissão** (ex.: o navegador troca de aba): não importa, porque só o programa aparece.
- **Programa sem nome conhecido**: cai no tipo ("Janela").
- **Quem está numa versão antiga do site ou do app**: continua vendo como hoje, só "ao vivo". Quem transmite de uma versão antiga aparece só com o tipo, ou sem texto.
- **Suas salas e a prévia da tela de entrada** não mostram o que está sendo transmitido. Lá só aparecem números e quem está ao vivo, como hoje (FR-019 da 011).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Quando alguém começa a transmitir, quem está na sala MUST ver, junto do nome da pessoa, uma descrição curta do que está sendo transmitido. Ela aparece no cartão da transmissão (palco, grade e miniaturas), na lista "Na sala" e nos chips do celular.
- **FR-002**: A descrição de uma **janela** transmitida pelo app MUST ser só o nome do programa ("Hollow Knight", "Google Chrome"), o mesmo que aparece no cartão do seletor. O título da janela MUST NOT sair do aparelho de quem transmite (Q1: A).
- **FR-003**: Tela inteira MUST aparecer como "Tela inteira". Com mais de um monitor, aparece o nome que a pessoa viu ao escolher ("Tela 2").
- **FR-004**: Pelo site, onde o navegador não informa o nome, a descrição MUST ser só o tipo: "Tela inteira", "Janela" ou "Aba do navegador". Sem campo novo para digitar um nome (Q2: A).
- **FR-005**: Câmera MUST aparecer como "Câmera" mais o nome do aparelho, ou "Câmera frontal"/"Câmera traseira" no celular.
- **FR-006**: A descrição MUST chegar a quem já está na sala e a quem entra depois, e MUST mudar quando a pessoa começa uma transmissão nova.
- **FR-007**: Quem transmite MUST ver no cartão "Você está ao vivo" exatamente a mesma descrição que os outros veem.
- **FR-008**: Não há opção de esconder a descrição: ela nunca vai além do nome do programa, do tipo ou do nome da câmera (Q3: A).
- **FR-009**: A descrição MUST ter no máximo 60 caracteres e aparecer só como texto. Caracteres invisíveis e de controle são removidos (mesma regra dos apelidos).
- **FR-010**: O servidor MUST guardar a descrição só enquanto a pessoa está ao vivo, como o resto do "ao vivo" de hoje, e nunca fora da sala (constituição III). Ela MUST NOT aparecer na prévia da tela de entrada nem em Suas salas.
- **FR-011**: Todas as cenas com transmissões MUST mostrar a descrição nos mocks (`?mock=room` e `?mock=mobile`), incluindo uma descrição comprida (corte) e uma transmissão de versão antiga, sem descrição.

### Key Entities

- **Descrição da transmissão**: texto curto (≤ 60) que acompanha o "ao vivo" de uma pessoa: tipo (tela inteira, janela, aba, câmera) e, quando houver, o nome. Vive só enquanto a transmissão dura.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Quem assiste identifica o que cada pessoa transmite sem abrir a transmissão: a descrição aparece em até 2 s depois do início, em 100% das transmissões feitas pelo app ou pelo site atual.
- **SC-002**: Com 3 transmissões na grade, dá pra escolher a certa na primeira tentativa, porque a descrição está visível em todas as miniaturas.
- **SC-003**: Nenhuma descrição contém o título de uma janela (só programa, tipo ou câmera), e nenhuma aparece fora da sala (prévia e Suas salas conferidas).
- **SC-004**: Descrição comprida nunca quebra o layout do cartão, da lista nem dos chips do celular.

## Assumptions

- No site, o navegador não informa o nome da janela/aba escolhida: só o tipo está disponível sem pedir nada à pessoa.
- A descrição é decidida no momento em que a transmissão começa e não acompanha mudanças de título da janela.
- Não muda nada no app desktop além do que a página já sabe pela ponte de captura (nome do programa e título já chegam hoje para a grade do seletor): **sem API nativa nova**.
- O tipo de transmissão (tela/câmera) já existe no "ao vivo". A descrição é um campo a mais, opcional, compatível com versões antigas.
- Textos em português informal; "Go Live" segue como "Transmitir".
