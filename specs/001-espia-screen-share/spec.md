# Feature Specification: Espia — compartilhamento de tela entre amigos

**Feature Branch**: `001-espia-screen-share`

**Created**: 2026-10-28

**Status**: Draft

**Input**: User description: "Com base no DEFINICOES.md (e no handoff de design `design_handoff_espia/`), especificar o app de compartilhamento de tela estilo 'Go Live' do Discord, quebrar as telas e começar por uma tela mockada acessível via parâmetro."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ver as telas com dados falsos (Priority: P1)

Quem desenvolve ou desenha abre o app com `?mock=<tela>&scene=<estado>` e vê qualquer tela/estado
do design com dados falsos, sem servidor, sem rede e sem permissão de captura.

**Why this priority**: É a base para validar o visual com o design antes de existir backend, e é a
forma de revisar todos os estados (inclusive erros) sem precisar reproduzi-los.

**Independent Test**: Rodar `npm run dev`, abrir `/?mock=room&scene=grid` e conferir que a sala
aparece igual ao design D5; trocar `scene` para cada valor da lista e conferir cada estado.

**Acceptance Scenarios**:

1. **Given** o app rodando sem backend, **When** abro `/?mock=room&scene=single`, **Then** vejo a sala com Lu ao vivo ocupando o palco (D4).
2. **Given** o modo mock, **When** troco `scene` pelo seletor de cenas, **Then** a URL é atualizada e a tela muda sem recarregar.
3. **Given** `?mock=room` sem `scene`, **When** abro, **Then** vejo a cena padrão (`single`).
4. **Given** um valor desconhecido de `mock` ou `scene`, **When** abro, **Then** vejo a lista de telas/cenas mockadas disponíveis.

---

### User Story 2 - Criar sala e convidar (Priority: P1) 🎯 MVP

Na página inicial a pessoa digita um apelido, clica **Criar sala**, cai na sala vazia e copia o
convite para mandar aos amigos.

**Why this priority**: Sem sala não existe nada; é o primeiro passo do fluxo real.

**Independent Test**: Criar uma sala, copiar o link e abrir em outra aba; as duas pessoas aparecem na lista "Na sala".

**Acceptance Scenarios**:

1. **Given** a página inicial, **When** digito "Rafa" e clico Criar sala, **Then** sou levado para `/s/<id>` e vejo "Ninguém ao vivo ainda." (D3).
2. **Given** já usei o app antes, **When** abro a página inicial, **Then** o apelido vem preenchido.
3. **Given** estou na sala, **When** clico Copiar convite, **Then** o link vai para a área de transferência, o botão mostra "Copiado!" por 2s e aparece o toast "Link copiado — é só mandar pra galera".
4. **Given** a página inicial, **When** colo um link/código de sala em "Tem um link?" e clico Entrar, **Then** vou para a tela de entrada daquela sala; se o texto não for de uma sala vejo "Esse link não parece de uma sala.".

---

### User Story 3 - Entrar por link (Priority: P1)

Quem recebe o link vê quem já está na sala, digita o apelido e entra.

**Why this priority**: É o caminho de todo convidado.

**Independent Test**: Abrir um link de sala existente em outra janela e entrar.

**Acceptance Scenarios**:

1. **Given** um link válido de sala com Lu, Bia e Tomás, **When** abro, **Then** vejo "Te chamaram pra uma sala." com os avatares e quem está ao vivo (D2).
2. **Given** a tela de entrada, **When** clico Entrar, **Then** vejo "Entrando na sala…" (E1) e depois a sala.
3. **Given** um link de sala que não existe mais, **When** abro, **Then** vejo "Essa sala não existe mais." com "Criar uma sala nova" e "Colar outro link" (E2).
4. **Given** uma sala com 10 pessoas, **When** abro o link, **Then** vejo "Essa sala está cheia (10 de 10)." e o botão Entrar desabilitado.

---

### User Story 4 - Transmitir a tela (Go Live) (Priority: P1)

Qualquer pessoa na sala clica **Go Live**, escolhe resolução (720p/1080p), fps (30/60) e se
compartilha áudio, escolhe tela/janela/aba e passa a transmitir para todos.

**Why this priority**: É a funcionalidade central do produto.

**Independent Test**: Duas abas na mesma sala; uma faz Go Live e a outra vê a transmissão.

**Acceptance Scenarios**:

1. **Given** estou na sala, **When** clico Go Live, **Then** abre o modal com as últimas escolhas que usei (padrão 1080p / 30 fps / com áudio) (D6).
2. **Given** o modal, **When** clico "Escolher tela e começar" e escolho uma tela, **Then** fico ao vivo: card "Você está ao vivo" na sidebar, botão "Parar transmissão", contador de tempo e toast "Você está ao vivo pra sala" (D7).
3. **Given** o seletor do navegador, **When** cancelo ou nego, **Then** vejo o alerta "O navegador não deixou compartilhar" com "Tentar de novo" e "Agora não" (E3).
4. **Given** estou ao vivo, **When** clico Parar transmissão ou uso o botão nativo do navegador, **Then** volto ao estado normal e os outros deixam de ver minha tela.
5. **Given** estou ao vivo, **Then** minha própria tela não aparece no palco, só como prévia na sidebar.

---

### User Story 5 - Assistir e escolher qual tela ver (Priority: P1)

Quem assiste vê uma transmissão em palco cheio ou várias em grade, foca numa, ajusta volume,
entra em tela cheia ou picture-in-picture.

**Why this priority**: Junto com Go Live, fecha o ciclo de valor.

**Independent Test**: Três abas transmitindo; uma quarta alterna entre grade e foco, muda volume e usa PiP.

**Acceptance Scenarios**:

1. **Given** só Lu está ao vivo, **Then** a tela de Lu ocupa o palco e os controles aparecem no hover (D4).
2. **Given** 2+ transmissões, **Then** vejo a grade; **When** clico num tile, **Then** ele vira foco e os outros vão para a faixa de miniaturas (D5/D5b).
3. **Given** o foco, **When** aperto Esc, **Then** volto para a grade; **When** clico numa miniatura, **Then** o foco troca.
4. **Given** uma transmissão nova, **Then** ela começa muda (autoplay) e o volume escolhido é lembrado por pessoa.
5. **Given** a conexão com uma transmissão falha, **Then** o tile mostra "Não deu pra conectar com {nome}" e "Tentar de novo" (E4).
6. **Given** a pessoa focada para de transmitir, **Then** volto para a grade (ou palco cheio se sobrar uma).

---

### User Story 6 - Avisos da sala (Priority: P2)

Toasts avisam quando alguém entra, sai, começa a transmitir (com ação "Assistir"), quando o link é
copiado e quando há erro.

**Independent Test**: Entrar/sair com outra aba e ver os toasts.

**Acceptance Scenarios**:

1. **Given** estou na sala, **When** Bia entra, **Then** vejo "Bia entrou na sala" por 4s.
2. **Given** Tomás começa a transmitir, **When** clico "Assistir" no toast, **Then** o tile de Tomás é focado.
3. **Given** há mais de 3 toasts, **Then** só os 3 mais recentes aparecem; toasts de erro ficam até eu clicar.

---

### User Story 7 - Assistir pelo celular (Priority: P3)

No celular a pessoa entra e assiste; o Go Live aparece desabilitado com a explicação.

**Independent Test**: Abrir o link no celular (ou viewport 390×844 sem `getDisplayMedia`).

**Acceptance Scenarios**:

1. **Given** um navegador sem `getDisplayMedia`, **When** abro o link, **Then** vejo "Entrar pra assistir" e a nota "No celular dá pra assistir. Transmitir só no computador." (M1).
2. **Given** estou assistindo no celular, **Then** vejo o palco 16:9, chips das transmissões ao vivo, lista de participantes e Go Live desabilitado (M2/M3).

---

### Edge Cases

- Sala cheia (10/10) ao entrar → aviso e Entrar desabilitado.
- Último participante sai → sala deixa de existir; o link passa a mostrar E2.
- Pessoa transmite e fecha a aba → os outros recebem "saiu da sala" e o tile some.
- `getDisplayMedia` sem áudio (Firefox/Safari, ou janela em vez de aba) → transmite só vídeo, sem erro.
- Conexão direta falha (CGNAT/4G) → usa TURN; se nem assim conectar, tile em `failed`.
- Duas pessoas com o mesmo apelido → ambas entram; a lista diferencia por tom de avatar.
- Apelido vazio → botão Criar/Entrar desabilitado; apelido limitado a 24 caracteres.
- Perda do WebSocket de sinalização → tenta reconectar; transmissões P2P já abertas continuam.

## Requirements *(mandatory)*

### Functional Requirements

**Mock**
- **FR-001**: O app MUST renderizar qualquer tela/estado do design com dados falsos quando a URL tiver `?mock=<tela>` (e opcionalmente `&scene=<estado>`), sem chamar rede, WebSocket ou APIs de captura.
- **FR-002**: Em modo mock, o app MUST exibir um seletor discreto de telas/cenas que atualiza a URL.

**Salas e acesso**
- **FR-003**: Usuários MUST poder criar uma sala informando só um apelido; a sala recebe um código curto aleatório no formato `xxx-xxx` (ex.: `k7f-q2x`) e a URL `/s/<código>`.
- **FR-004**: O sistema MUST permitir entrar em uma sala existente por link ou código colado, pedindo só o apelido.
- **FR-005**: O sistema MUST limitar cada sala a 10 participantes e recusar o 11º com aviso.
- **FR-006**: A sala MUST deixar de existir quando o último participante sair; links para ela mostram "sala não encontrada".
- **FR-007**: O apelido MUST ser lembrado no navegador e vir preenchido.
- **FR-008**: Antes de entrar, a tela de entrada SHOULD mostrar quem está na sala e quem está ao vivo.

**Transmissão**
- **FR-009**: Qualquer participante MUST poder transmitir; várias transmissões simultâneas por sala MUST ser suportadas.
- **FR-010**: Antes de transmitir, o usuário MUST escolher resolução (720p/1080p), fps (30/60) e áudio (sim/não); a última escolha é lembrada. *Regras de som e seletor: ver specs/002 e specs/004.*
- **FR-011**: Ao negar/cancelar a permissão de captura, o sistema MUST mostrar o alerta E3 com opção de tentar de novo.
- **FR-012**: O sistema MUST detectar o fim da captura pelo navegador e encerrar a transmissão.
- **FR-013**: A própria transmissão MUST aparecer só como prévia na sidebar, com estatísticas (assistindo · resolução · fps · áudio) e tempo ao vivo. *Texto do som: specs/002 FR-207.*

**Assistir**
- **FR-014**: Com 1 transmissão o palco mostra só ela; com 2+ mostra grade; clicar foca; Esc volta para a grade.
- **FR-015**: Cada tile MUST ter mudo/volume, picture-in-picture e tela cheia; os controles aparecem no hover e somem 2s após o mouse parar.
- **FR-016**: Transmissões novas MUST começar mudas; o volume MUST ser lembrado por pessoa.
- **FR-017**: Falha de conexão com uma transmissão MUST mostrar o estado `failed` com "Tentar de novo".
- **FR-018**: A lista "Na sala" MUST mostrar todos os participantes (você primeiro), quem está ao vivo e quem você está assistindo.

**Avisos**
- **FR-019**: O sistema MUST mostrar toasts para entrou, saiu, começou a transmitir (ação Assistir), link copiado e erro; máx. 3; somem em 4s, exceto erro.
- **FR-020**: "Copiar convite" MUST copiar a URL da sala e mostrar "Copiado!" por 2s.

**Celular**
- **FR-021**: Em navegadores sem `getDisplayMedia` o app MUST entrar em modo "só assistir" (Go Live desabilitado com explicação).
- **FR-022**: Abaixo de 1024px a sidebar vira faixa horizontal; abaixo de 640px usa o layout mobile.

**Segurança**
- **FR-023**: Credenciais TURN MUST ser temporárias, entregues só a quem está numa sala e revogadas quando a pessoa sai; a chave nunca vai ao navegador.
- **FR-024**: Vídeo e áudio MUST trafegar criptografados de navegador a navegador e MUST NOT ser gravados em lugar nenhum.
- **FR-025**: O código da sala MUST ser gerado pelo servidor; consultas e entradas MUST ter limite por IP para impedir varredura de códigos.
- **FR-026**: A identidade de quem envia cada mensagem MUST ser definida pelo servidor (ninguém se passa por outra pessoa).
- **FR-027**: Apelidos MUST ser exibidos como texto puro e higienizados (1–24 caracteres, sem caracteres de controle ou de direção).
- **FR-028**: O servidor MUST rejeitar mensagens grandes demais ou em excesso e conexões de outros sites (`Origin`).
- **FR-029**: As respostas MUST incluir os cabeçalhos de segurança de [security.md §5](./security.md), incluindo `Referrer-Policy: no-referrer`.
- **FR-030**: O servidor MUST NOT registrar apelidos, códigos de sala ou IPs em logs; o site MUST NOT carregar recursos de terceiros (fontes auto-hospedadas, sem analytics).
- **FR-031**: A conexão MUST ser direta por padrão (TURN só quando o direto falha); participantes veem o IP público uns dos outros, o que é aceito para o grupo de amigos (security.md §4.1, D-1).

**Conexão**
- **FR-032**: Uma queda curta da conexão com o servidor (até 20 s) MUST NOT remover a pessoa da sala nem interromper transmissões em andamento.
- **FR-033**: Quem transmite MUST reduzir automaticamente o envio para espectadores que estão vendo sua tela só em miniatura.
- **FR-034**: A qualidade de quem transmite MUST baixar automaticamente conforme o número de espectadores (teto por faixa em architecture.md §4.1b), nunca acima do escolhido; a qualidade efetiva MUST aparecer para quem transmite e para quem assiste.
- **FR-035**: Com mais de 5 pessoas na sala, o modal Go Live MUST avisar qual qualidade será usada.
- **FR-036**: A grade MUST comportar até 9 transmissões (2×1, 2×2, 3×2, 3×3).

### Key Entities

- **Sala**: ID aleatório, participantes (máx. 10), existe enquanto houver alguém.
- **Participante**: ID de sessão, apelido, tom do avatar (por ordem de entrada, em rodízio), se está ao vivo.
- **Transmissão**: pertence a um participante; configuração (resolução, fps, áudio), início; para cada espectador tem estado (carregando/ok/falhou), mudo e volume.
- **Preferências locais**: apelido, última configuração de Go Live, volume por pessoa.
- **Toast**: tipo, texto, ação opcional.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% das telas e estados do handoff (D1–D7, E1–E5, M1–M3) são abertos via `?mock=` sem backend.
- **SC-002**: Uma pessoa nova cria uma sala e copia o convite em menos de 15 segundos.
- **SC-003**: Um convidado vai do link aberto até assistir uma transmissão em menos de 20 segundos, com no máximo 2 interações (apelido + Entrar).
- **SC-004**: Com 5 pessoas na sala e 2 transmitindo em 1080p30 (uso normal), e com 10 pessoas e 1 transmitindo em qualidade reduzida (limite), todos assistem sem que o servidor trafegue vídeo (custo de mídia no servidor = 0, exceto relay TURN).
- **SC-005**: Uma sala vazia não deixa nenhum dado no servidor.

## Assumptions

- Rota da sala é `/s/<código>` com código curto `xxx-xxx` (design); o código mostrado na interface é o próprio código.
- Nome do produto na interface: "Espia" (handoff de design).
- Tema escuro é o padrão; tema claro é opcional e fica para depois.
- Mobile: só assistir, nunca transmitir.
- Público: um grupo fechado de até 10 amigos; uso normal de 3–5 pessoas por sala, 10 é o limite. Todos têm boa conexão.
- Acesso: só o código da sala, sem senha do grupo (confirmado pelo usuário).
- Sem chat de voz/texto, contas ou senha (DEFINICOES — fora do escopo).
- Requer conta gratuita na Cloudflare para deploy; o mock não requer nada.
- Como funciona por dentro: [architecture.md](./architecture.md). Segurança e privacidade: [security.md](./security.md).
