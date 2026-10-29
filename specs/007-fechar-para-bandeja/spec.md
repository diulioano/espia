# Feature Specification: Fechar o app sem sair (e a pergunta sobre Linux)

**Feature Branch**: `007-fechar-para-bandeja`

**Created**: 2026-10-29

**Status**: Draft

**Relacionada a**: [003-espia-desktop](../003-espia-desktop/spec.md) (app desktop) e [005-seletor-nativo-desktop](../005-seletor-nativo-desktop/spec.md) (captura nativa, só Windows).

**Input**: User description: "o Tauri já tem suporte para linux? e também se é possível que ao 'fechar', não feche o app. Que ele seja movido para a barra de tarefas"

## Contexto

### Linux: resposta curta

O Tauri roda no Linux, mas o app do Espia não funcionaria bem lá. Os motivos:

| Ponto | Situação no Linux |
|---|---|
| A base (janela, links `espia://`, uma instância só) | ✅ Funciona. |
| **Assistir e transmitir** | ❌ O motor de página que o Tauri usa no Linux (WebKitGTK) **não traz chamada de vídeo P2P ligada** na maioria das distribuições. Sem isso, o Espia não conecta com ninguém, nem para assistir. |
| Seletor nativo com miniaturas e som sem o Discord (005) | ❌ Foi feito com APIs do Windows. No Linux, teria que ser refeito do zero (PipeWire). |
| Esconder a barra "está compartilhando" (003) | ❌ É coisa do motor do Windows. |

**Conclusão**: no Linux, o caminho bom é usar o **site no Chrome, Edge ou Firefox**. Lá tudo funciona, menos o que é exclusivo do app (grade de janelas nativa). Um app Linux "de verdade" exigiria trocar o motor (por exemplo, embutir o Chromium, com +100 MB) ou esperar o WebKitGTK ligar o WebRTC por padrão. Por isso, Linux fica **fora** (Q2: decisão de não seguir agora).

### Fechar sem sair

Hoje, o X da janela fecha o app: quem estava numa sala sai, e uma transmissão para. A pessoa quer que o X **não encerre o app** e que ele continue rodando na barra do Windows, como o Discord.

## Clarifications

### Session 2026-10-29

- Q1: Para onde vai ao clicar no X? → **A: bandeja** (ícone perto do relógio; some da barra de tarefas).
- Q2: Linux? → **Fora**: não seguiremos com Linux agora (nem documentação nem feature futura nesta spec).

## User Scenarios & Testing *(mandatory)*

### User Story 1 — O X não fecha o app (Priority: P1)

Estou na sala, ouvindo a galera ou transmitindo o jogo, e clico no X da janela do Espia. A janela some, mas eu continuo na sala e a transmissão continua. O Espia fica na **bandeja** do Windows (ícone perto do relógio), sem botão na barra de tarefas (Q1: A).

**Why this priority**: é o pedido. Evita sair da sala sem querer e deixa transmitir o jogo sem a janela do Espia aberta.

**Independent Test**: transmitir uma janela, clicar no X. Outra pessoa continua vendo a transmissão, e o Espia continua rodando (visível onde a Q1 decidir).

**Acceptance Scenarios**:

1. **Given** o app aberto (em qualquer tela), **When** clico no X, **Then** a janela some e o app continua rodando, sem sair da sala e sem parar a transmissão.
2. **Given** a primeira vez que isso acontece, **Then** aparece um aviso do Windows: "O Espia continua aberto aqui. Pra sair de vez, clique com o botão direito e escolha Sair."
3. **Given** o app escondido, **When** clico no ícone dele ou abro o Espia de novo pelo atalho ou por um link `espia://`, **Then** a janela volta onde estava.

---

### User Story 2 — Sair de verdade (Priority: P1)

Quero fechar o Espia de vez. Clico com o botão direito no ícone e escolho **Sair**. Se eu estiver transmitindo, ele pergunta antes.

**Why this priority**: sem isso, não haveria jeito claro de fechar o app.

**Independent Test**: com o app escondido e transmitindo, escolher Sair. Aparece a confirmação. Depois de confirmar, o app fecha e a sala vê que eu saí.

**Acceptance Scenarios**:

1. **Given** o menu do ícone, **Then** ele tem: "Abrir o Espia", "Parar transmissão" (só se estiver transmitindo) e "Sair".
2. **Given** estou transmitindo, **When** escolho Sair, **Then** o app pergunta "Você está transmitindo. Sair mesmo assim?" antes de fechar.
3. **Given** o Windows está desligando ou reiniciando, **Then** o app fecha sem perguntar.

---

### User Story 3 — Escolher o comportamento do X (Priority: P3)

Nas configurações do app (menu do ícone), há a opção "Fechar no X" ligada/desligada, para quem prefere que o X feche de verdade.

**Why this priority**: conforto. O padrão já atende o pedido.

**Independent Test**: desligar a opção e clicar no X. O app fecha (com a mesma confirmação se estiver transmitindo).

**Acceptance Scenarios**:

1. **Given** "Fechar no X" desligado (padrão), **Then** o X esconde a janela.
2. **Given** "Fechar no X" ligado, **Then** o X fecha o app.
3. **Given** reabro o app, **Then** a escolha continua a mesma.

### Edge Cases

- **Abrir o app com ele já escondido** (atalho, menu Iniciar, link `espia://s/...`): mostra a janela existente e, se for link de sala, segue o fluxo atual da 003 (pergunta se está transmitindo).
- **Sem conexão / tela de carregamento**: o X também esconde. Não há sala, então nada muda.
- **Transmitindo com a janela escondida**: quem transmite precisa saber que continua ao vivo. O ícone muda para a versão "ao vivo" e a dica dele diz "Espia · ao vivo".
- **Escondido e alguém começa a transmitir**: nesta versão, sem notificação (pode virar feature).
- **Windows com o ícone escondido na "seta" da bandeja**: o aviso da primeira vez explica onde achar.
- **Atualização do app** (instalador novo com o app escondido): o instalador fecha o app como hoje.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-701**: Clicar no X da janela MUST esconder a janela sem encerrar o app, sem sair da sala e sem parar uma transmissão (padrão).
- **FR-702**: Com a janela escondida, o app MUST continuar acessível conforme a Q1, e um clique (ou abrir o app de novo) MUST trazer a janela de volta, no mesmo estado.
- **FR-703**: Na primeira vez que a janela for escondida pelo X, o app MUST mostrar um aviso explicando que continua aberto e como sair. Esse aviso só aparece uma vez.
- **FR-704**: MUST haver um menu com "Abrir o Espia", "Parar transmissão" (só ao vivo), "Fechar no X" (liga/desliga) e "Sair".
- **FR-705**: "Sair" MUST pedir confirmação quando houver uma transmissão ativa, e MUST fechar sem perguntar quando o Windows estiver desligando.
- **FR-706**: Enquanto transmite, o ícone/dica do app MUST indicar "ao vivo".
- **FR-707**: A escolha "Fechar no X" MUST ser lembrada entre aberturas do app.
- **FR-708**: O site (no navegador) MUST continuar igual. Isso é só do app desktop.
- **FR-709**: Linux fica fora (Q2): nada muda para Linux nesta feature.

### Key Entities

- **Preferência "Fechar no X"**: liga/desliga, guardada no PC. Padrão: desligada (o X esconde).
- **Aviso da primeira vez**: já mostrado ou não, guardado no PC.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-701**: Clicando no X durante uma transmissão, quem assiste não percebe interrupção (0 quedas em 10 tentativas).
- **SC-702**: Trazer a janela de volta leva 1 clique e menos de 1 s.
- **SC-703**: 100% das saídas com uma transmissão ativa passam pela confirmação (exceto o desligamento do Windows).
- **SC-704**: Com o app escondido e parado numa sala, o consumo de memória não cresce além do que era com a janela aberta.

## Assumptions

- O app continua só Windows nesta feature.
- "Barra de tarefas" = bandeja do Windows (Q1: A).
- O app **não** abre sozinho com o Windows (isso seria outra feature).
- O app é opcional (princípio I): nada disso muda o site.
