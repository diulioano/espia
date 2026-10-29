# Feature Specification: Espia Desktop — app leve para Windows

**Feature Branch**: `003-espia-desktop`

**Created**: 2026-10-28

**Status**: Draft

**Relacionada a**: [001-espia-screen-share](../001-espia-screen-share/spec.md). O site continua sendo o
jeito principal de usar o Espia; o app desktop é uma porta de entrada **opcional** para a mesma sala.

**Input**: User description: "App desktop do Espia (opcional, leve) com Tauri 2. O site continua sendo o principal; o app desktop é um jeito alternativo de abrir o Espia sem navegador, focado em ser leve (instalador pequeno, pouca RAM, usa o WebView2 do Windows em vez de embutir um Chromium). O app abre o Espia publicado (https://espia.espia.workers.dev) numa janela própria, com ícone do castor, e permite transmitir a tela (getDisplayMedia com seletor de tela/janela/aba e áudio) e assistir igual ao site. Links de sala (espia://s/xxx-xxx e colar link https) abrem a sala no app. Uma única instância: abrir um link com o app aberto foca a janela e navega para a sala. Alvo: Windows 10/11 (o grupo usa Windows); macOS/Linux fora do escopo por enquanto. Distribuição: instalador gerado por GitHub Actions e publicado como Release no GitHub (repositório espia). Também: CI no GitHub (check + test + build) e workflow de deploy do Worker na Cloudflare. Renomear a pasta do projeto de screen-share para espia. Fora de escopo: atualização automática, assinatura de código, voz/chat."

## Contexto

Hoje o Espia roda no navegador. Parte da galera prefere ter um ícone na área de trabalho/barra de tarefas
que abre direto o Espia, sem abas misturadas com o resto do navegador, e que não pese no PC enquanto
jogam. O app desktop precisa ser **leve**: instalador pequeno e pouca memória, reaproveitando o motor web
que já vem no Windows em vez de carregar um navegador inteiro junto.

O app é só outra "janela" para o mesmo Espia: mesma sala, mesmo link, mesmas telas. Quem está no site e
quem está no app se veem e transmitem um para o outro normalmente.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Instalar e abrir o Espia como app (Priority: P1)

Rafa baixa o instalador do Espia na página de releases do projeto, instala em poucos cliques e passa a ter
o Espia no menu Iniciar com o ícone do castor. Ao abrir, vê a tela inicial do Espia numa janela própria e
cria uma sala igual faria no site.

**Why this priority**: Sem instalar e abrir, nada mais existe. É o MVP do app.

**Independent Test**: Baixar o instalador da última release, instalar num Windows 10/11 sem nada do
projeto, abrir pelo menu Iniciar e criar uma sala.

**Acceptance Scenarios**:

1. **Given** um Windows 10/11 sem o Espia, **When** baixo e rodo o instalador, **Then** o Espia aparece no menu Iniciar com o ícone do castor, sem pedir conta nem configuração.
2. **Given** o app instalado, **When** abro, **Then** vejo a tela inicial do Espia (apelido + Criar sala) numa janela com título "Espia".
3. **Given** o PC sem internet, **When** abro o app, **Then** vejo uma mensagem amigável dizendo que não foi possível conectar e um botão para tentar de novo, em vez de uma página de erro técnica.
4. **Given** o app instalado, **When** desinstalo pelo Windows, **Then** ele é removido sem deixar atalhos para trás.

---

### User Story 2 - Transmitir e assistir pelo app (Priority: P1)

Dentro do app, Bia entra numa sala, clica em **Transmitir**, escolhe a tela/janela/aba e a qualidade e
começa a transmitir. Os amigos (no site ou no app) assistem normalmente, com tela cheia, volume e lista de
participantes. Picture-in-picture funciona quando o motor web do Windows suportar.

**Why this priority**: Transmitir e assistir é o motivo de o Espia existir; um app que não transmite não
serve.

**Independent Test**: Duas máquinas: uma no app, outra no site, na mesma sala. A do app transmite uma
janela com áudio; a do site assiste com imagem e som. Depois invertem.

**Acceptance Scenarios**:

1. **Given** estou numa sala no app, **When** clico em Transmitir e confirmo, **Then** aparece o seletor do Windows/motor web para escolher tela ou janela, e ao escolher a transmissão começa.
2. **Given** estou transmitindo pelo app, **When** um amigo no site assiste, **Then** ele vê a imagem e ouve o som conforme as mesmas regras de áudio do site.
3. **Given** um amigo transmite, **When** assisto pelo app, **Then** consigo tela cheia, ajustar o volume e ver a lista de participantes, igual ao site.
4. **Given** cancelo o seletor de tela, **When** volto para a sala, **Then** nada é transmitido e a sala continua normal.
5. **Given** comecei a transmitir pelo app, **When** olho a tela, **Then** não aparece a barra flutuante "… está compartilhando …" do navegador; parar é pelo botão Parar do Espia (pedido da pessoa usuária em 2026-10-28).

---

### User Story 3 - Abrir um link de sala direto no app (Priority: P2)

Lu manda no Discord o link da sala. Rafa, que tem o app instalado, cola o link (https ou `espia://`) na
tela inicial do app — ou clica num link `espia://s/k7f-q2x` — e cai direto na sala, dentro do app. Se o
app já estava aberto, a mesma janela vem para a frente e vai para a sala, sem abrir uma segunda cópia.

**Why this priority**: Facilita muito entrar nas salas, mas o app já é útil sem isso (dá para digitar o
código na tela inicial).

**Independent Test**: Com o app fechado, abrir `espia://s/k7f-q2x` pelo Executar do Windows (Win+R): o app
abre na tela de entrada dessa sala. Repetir com o app aberto: a mesma janela vem para frente e navega.

**Acceptance Scenarios**:

1. **Given** o app fechado, **When** abro `espia://s/k7f-q2x`, **Then** o app abre já na tela de entrada da sala `k7f-q2x`.
2. **Given** o app aberto em qualquer tela, **When** abro `espia://s/k7f-q2x`, **Then** a janela existente vem para frente e vai para a sala; nenhuma segunda janela é aberta.
3. **Given** o app aberto, **When** abro o atalho do Espia de novo, **Then** a janela existente vem para frente.
4. **Given** um link malformado (`espia://qualquer-coisa`), **When** abro, **Then** o app abre (ou foca) na tela inicial, sem erro.
5. **Given** estou transmitindo, **When** chega um link de outra sala, **Then** o app pede confirmação antes de sair da sala atual.

---

### User Story 4 - Conseguir o instalador e manter o projeto publicado (Priority: P2)

Quem cuida do projeto publica uma nova versão marcando uma versão no repositório; o instalador do Windows
é gerado sozinho e aparece na página de releases. Toda mudança enviada passa por verificação automática
(tipos, testes, build), e o site/servidor pode ser publicado na Cloudflare automaticamente.

**Why this priority**: Sem isso a galera não tem de onde baixar, mas é infraestrutura, não uso direto.

**Independent Test**: Criar a tag `v0.1.0`; em até ~20 min a release `v0.1.0` existe com o instalador
anexado; baixar e instalar.

**Acceptance Scenarios**:

1. **Given** o repositório no GitHub, **When** envio uma mudança, **Then** a verificação automática roda e mostra se passou.
2. **Given** uma tag de versão `vX.Y.Z`, **When** é enviada, **Then** uma release com o instalador do Windows é publicada.
3. **Given** o token da Cloudflare configurado no repositório, **When** envio para a branch principal, **Then** o site/servidor é publicado; sem o token, o passo é pulado sem quebrar a verificação.
4. **Given** o repositório público, **When** alguém lê o código, **Then** nenhum segredo (token da Cloudflare/TURN) está no repositório.

---

### Edge Cases

- O motor web do Windows (WebView2) não está instalado (Windows 10 antigo): o instalador instala-o sozinho.
- O Windows mostra o aviso "editor desconhecido" (SmartScreen) porque o app não é assinado: a página de download explica como seguir ("Mais informações → Executar assim mesmo").
- A conexão cai no meio da sala: o app se comporta igual ao site (reconexão/aviso da 001).
- O site publicado muda (nova versão): o app mostra a nova versão na próxima abertura, sem reinstalar.
- O app tenta navegar para fora do Espia (ex.: link externo): abre no navegador padrão, não dentro do app.
- Duas telas com DPI diferente: a janela se ajusta normalmente.
- Permissão de captura negada/cancelada: mesmo comportamento do site (toast de cancelado).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O app MUST abrir o Espia publicado numa janela própria com título "Espia" e o ícone do castor (janela, barra de tarefas, menu Iniciar, instalador).
- **FR-002**: O app MUST permitir transmitir (escolher tela/janela/aba, qualidade e áudio) e assistir com as mesmas funcionalidades do site em Chrome/Edge.
- **FR-003**: O app MUST ser interoperável com o site: pessoas no app e no site na mesma sala se veem e transmitem entre si.
- **FR-004**: O app MUST registrar o esquema de link `espia://` e abrir `espia://s/<código>` na tela de entrada dessa sala.
- **FR-005**: O app MUST rodar em instância única: uma segunda abertura (atalho ou link) foca a janela existente e, se trouxer um link de sala válido, navega para ela.
- **FR-006**: Se houver transmissão ativa, trocar de sala por link MUST pedir confirmação.
- **FR-007**: Links que saem do Espia MUST abrir no navegador padrão do sistema.
- **FR-008**: Sem conexão, o app MUST mostrar uma tela própria amigável (no tom do Espia) com "Tentar de novo".
- **FR-009**: O app MUST lembrar tamanho e posição da janela entre aberturas.
- **FR-010**: O instalador MUST ser para Windows 10/11 x64, sem exigir permissão de administrador, e garantir o motor web necessário.
- **FR-011**: A tela inicial (site e app) MUST aceitar colar um link completo da sala (https ou `espia://`) além do código.
- **FR-012**: O repositório MUST ter verificação automática (checagem de tipos, testes e build) em todo envio e pull request.
- **FR-013**: Uma tag `v*` MUST gerar uma release no GitHub com o instalador anexado.
- **FR-014**: O site/servidor MUST poder ser publicado automaticamente na Cloudflare a partir da branch principal, usando um token guardado como segredo do repositório; sem o token, o passo é pulado.
- **FR-015**: Nenhum segredo MUST ir para o repositório; o app não MUST conter chaves.
- **FR-016**: O app MUST NOT dar ao site acesso a recursos do sistema além dos necessários (sem acesso a arquivos, shell ou comandos nativos expostos à página).
- **FR-017**: A pasta do projeto MUST se chamar `espia`, e os documentos MUST usar o nome Espia.
- **FR-018**: Ao transmitir pelo app, a barra flutuante de aviso "… está compartilhando …" do motor web MUST NOT ficar visível; se não for possível escondê-la (motor web diferente), a transmissão MUST continuar funcionando normalmente.

### Key Entities

- **Link de sala**: `https://espia.espia.workers.dev/s/<código>` ou `espia://s/<código>`, código `xxx-xxx`.
- **Release**: versão `vX.Y.Z` com o instalador do Windows anexado.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Instalador com no máximo 10 MB.
- **SC-002**: App aberto numa sala, sem transmitir, usa no máximo ~150 MB de RAM no total (app + motor web), contra 300 MB+ de um navegador com a mesma aba.
- **SC-003**: Do clique no atalho até a tela inicial do Espia visível: até 3 s numa conexão comum.
- **SC-004**: Uma pessoa sem conhecimento técnico instala e entra numa sala em até 2 minutos seguindo a página de download.
- **SC-005**: 100% dos links `espia://s/<código>` válidos abrem a sala certa, com o app aberto ou fechado, sem abrir segunda janela.
- **SC-006**: Transmissão app → site e site → app funciona com imagem e som no teste com 2 PCs.
- **SC-007**: A release é publicada em até 30 min após a tag, sem passo manual.
- **SC-008**: Em 100% das transmissões no teste manual (janela, aba, tela inteira), a barra "está compartilhando" não fica visível por mais de 1 s depois do início.

## Assumptions

- O app carrega o site publicado (não uma cópia embutida): assim fica sempre igual ao site e não precisa de atualização automática para receber melhorias de interface.
- Sem internet não há Espia (é tempo real), então não carregar nada offline não é perda.
- O grupo usa Windows 10/11 x64; macOS/Linux ficam para depois.
- O app não é assinado digitalmente (custo); o aviso do SmartScreen é aceito e explicado.
- Atualização automática do app (o "casco") está fora do escopo; como a interface vem do site, raramente será preciso reinstalar.
- O repositório `espia` no GitHub é público, para a galera baixar o instalador sem conta no GitHub.
- O publicar automático na Cloudflare depende de quem cuida do projeto criar um token e guardá-lo como segredo do repositório.
- Regras de áudio da transmissão seguem as do site (incluindo a feature 002, quando implementada).

## Out of Scope

- Atualização automática do app, assinatura de código, loja da Microsoft.
- macOS, Linux, celular.
- Voz, chat de texto, bandeja do sistema, iniciar com o Windows.
- Recursos nativos exclusivos do app (atalhos globais, overlay em jogos).
