# Feature Specification: Tela cheia e sala responsiva no celular

**Feature Branch**: `main` — nenhuma branch criada nesta etapa; diretório independente da branch.

**Created**: 2026-11-07

**Status**: Draft

**Input**: User description: "precisa poder deixar ver a tela inteira e ser responsivo pelo celular"

**Relacionada a**: [001-espia-screen-share](../001-espia-screen-share/spec.md) (sala no celular, M2/M3), [014-assistir-e-remover](../014-assistir-e-remover/spec.md) e [017-transmissao-sob-demanda](../017-transmissao-sob-demanda/spec.md) (Assistir explícito; celular com uma transmissão ativa por vez).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ver a transmissão em tela cheia no celular (Priority: P1)

Estou assistindo pelo celular. Toco em tela cheia e a transmissão passa a ocupar a tela toda, sem cabeçalho, lista de pessoas nem barra de botões. Deito o celular e a imagem aproveita a tela deitada. A imagem aparece inteira, sem cortar as bordas da tela de quem transmite. Saio da tela cheia com um toque ou com o gesto de voltar do celular.

**Why this priority**: hoje a transmissão fica numa faixa pequena no topo da sala do celular. Uma tela de computador cheia de texto fica ilegível nesse tamanho. Esse é o principal motivo de assistir pelo celular não funcionar bem.

**Independent Test**: num celular Android e num iPhone, entrar numa sala com uma transmissão de tela, clicar em Assistir, entrar em tela cheia em pé e deitado e sair. Conferir área ocupada, imagem sem corte, som e controles.

**Acceptance Scenarios**:

1. **Given** estou assistindo a uma transmissão no celular, **When** toco em tela cheia, **Then** a imagem ocupa a maior área possível da tela, sem elementos da sala por cima nem em volta, e aparece inteira, sem corte nas bordas.
2. **Given** estou em tela cheia com o celular em pé, **When** deito o celular, **Then** a imagem se reorganiza para a tela deitada em até 1 segundo, sem parar o vídeo nem o som.
3. **Given** estou em tela cheia, **When** toco na imagem, **Then** os controles aparecem (som, volume, sair da tela cheia) e somem sozinhos depois de alguns segundos sem toque.
4. **Given** estou em tela cheia, **When** toco em sair da tela cheia ou uso o gesto ou botão de voltar do celular, **Then** volto para a sala no mesmo ponto, com a mesma transmissão assistida e sem sair da sala.
5. **Given** estou em tela cheia, **When** a transmissão termina ou eu paro de assistir, **Then** saio da tela cheia e vejo o estado normal da sala, com o aviso correspondente.
6. **Given** estou em tela cheia, **When** a conexão cai e volta sozinha, **Then** a tela cheia continua e a imagem volta assim que a recepção retoma.
7. **Given** um celular que não permite tela cheia da página, como o iPhone, **When** toco em tela cheia, **Then** a transmissão ocupa a página toda, com imagem, som, controles e aproximação, sem erro nem tela preta.

---

### User Story 2 - Sala que se adapta ao celular em pé e deitado (Priority: P1)

Abro o link no celular, em pé ou deitado, e a sala cabe na tela. Não preciso dar zoom nem arrastar para os lados. Botões e textos têm tamanho de toque. Ao deitar o celular dentro da sala, sem tela cheia, a transmissão ganha a maior parte da tela. Não aparece a versão de computador espremida.

**Why this priority**: celulares deitados e telas grandes de celular hoje podem cair no layout de computador, que não foi feito para toque. Quem entra pelo link mandado no grupo não pode cair numa tela quebrada.

**Independent Test**: abrir a sala, a entrada e a tela inicial em larguras de 320 a 932 pontos, em pé e deitado. Conferir rolagem lateral, áreas de toque, cortes de texto e acesso a todas as ações. Testar também com zoom de texto do sistema ativado.

**Acceptance Scenarios**:

1. **Given** um celular em pé com largura de 320 a 480 pontos, **When** abro a tela inicial, a entrada ou a sala, **Then** todo o conteúdo cabe na largura, sem rolagem lateral, e todas as ações ficam alcançáveis.
2. **Given** um celular deitado, **When** estou na sala, **Then** vejo a versão de celular da sala, com a transmissão ocupando a maior parte da altura e o resto das informações acessível sem cobrir a imagem.
3. **Given** a sala aberta, **When** giro o celular, **Then** o layout se reorganiza sem recarregar, sem sair da sala e sem interromper o que estou assistindo.
4. **Given** celulares com entalhe, ilha ou barra de gestos, **When** uso a sala em pé ou deitado, **Then** nenhum botão ou texto importante fica escondido sob essas áreas.
5. **Given** o teclado do celular aberto para digitar o apelido ou o nome da sala, **When** digito, **Then** o campo e o botão de confirmar continuam visíveis.
6. **Given** um nome, apelido ou descrição de transmissão longo, **When** aparece no celular, **Then** é abreviado sem empurrar outros elementos para fora da tela.

---

### User Story 3 - Aproximar detalhes da tela transmitida (Priority: P2)

Na tela cheia do celular, uma tela de computador inteira ainda fica pequena. Afasto dois dedos para aproximar um trecho, como um texto ou um canto do jogo, e arrasto para olhar outra parte. Um toque duplo volta para a imagem inteira.

**Why this priority**: tela cheia já resolve a maior parte. Aproximar é o que torna legível o texto miúdo de uma tela de computador num celular. Depende da história 1.

**Independent Test**: em tela cheia, com uma transmissão de tela com texto pequeno, aproximar, arrastar e voltar ao tamanho inteiro. Conferir que a página da sala não se mexe e que a transmissão continua ao vivo.

**Acceptance Scenarios**:

1. **Given** estou em tela cheia, **When** afasto dois dedos sobre a imagem, **Then** a imagem aproxima até pelo menos 4 vezes, sem a página se mover.
2. **Given** a imagem aproximada, **When** arrasto com um dedo, **Then** vejo outra parte da imagem, sem passar das bordas.
3. **Given** a imagem aproximada, **When** dou um toque duplo, saio da tela cheia ou giro o celular, **Then** a imagem volta a aparecer inteira.
4. **Given** a imagem aproximada, **When** a transmissão continua, **Then** continuo vendo ao vivo a parte aproximada, sem congelar.

### Edge Cases

- Transmissão de câmera em pé (celular de outra pessoa): a imagem aparece inteira, com faixas nas laterais quando a proporção não bate com a tela, sem corte.
- Celular com giro de tela travado: a tela cheia funciona em pé. Nada força o giro nem pede para destravar.
- Várias transmissões: em tela cheia vejo só a escolhida. Trocar de transmissão é feito na sala, fora da tela cheia, e segue a regra de Assistir da 017.
- Transmissão sem Assistir: não há tela cheia de um cartão que ainda não recebe imagem. O cartão continua oferecendo Assistir.
- Picture-in-picture ativado a partir da tela cheia: sai da tela cheia e mantém a imagem flutuante, como hoje.
- Aviso da sala com tela cheia aberta (pessoa entrou, queda, tirado da sala): avisos que exigem ação, como ser removido ou a sala ser apagada, tiram da tela cheia. Avisos informativos não atrapalham a imagem.
- Tablet: larguras maiores que a de celular seguem a versão de computador, desde que caibam e sejam tocáveis.
- Computador com janela estreita: segue a mesma regra de largura. Não pode haver uma faixa em que nenhuma das duas versões caiba.
- Transmitindo a câmera pelo celular: a sala deitada e em pé mantém Parar câmera sempre alcançável.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Quem assiste pelo celular MUST poder colocar a transmissão assistida em tela cheia com um toque, a partir da sala.
- **FR-002**: Em tela cheia, a imagem MUST ocupar a maior área possível da tela e aparecer inteira, sem corte, com faixas quando as proporções diferem. Elementos da sala MUST NOT aparecer por cima ou em volta da imagem, exceto controles temporários e avisos que exigem ação.
- **FR-003**: Em tela cheia, controles de som, volume e saída MUST aparecer ao tocar na imagem e sumir sozinhos depois de alguns segundos sem toque.
- **FR-004**: Sair da tela cheia MUST ser possível por um controle visível e pelo gesto ou botão de voltar do celular. Isso MUST devolver a pessoa à sala, sem sair dela nem parar de assistir.
- **FR-005**: Entrar e sair da tela cheia e girar o celular MUST NOT interromper vídeo nem som, MUST NOT reiniciar a recepção e MUST NOT contar como novo Assistir.
- **FR-006**: Quando o aparelho não permite tela cheia da página, o Espia MUST ocupar a página toda com imagem, som, controles e aproximação, sem erro visível. O player nativo do aparelho MUST NOT ser usado, porque ele não controla o som separado da transmissão (ver research R2).
- **FR-007**: Fim da transmissão, Parar de assistir, remoção da sala ou sala apagada MUST encerrar a tela cheia e mostrar o estado e o aviso correspondentes na sala.
- **FR-008**: Queda e volta automática da conexão MUST manter a tela cheia aberta e retomar a imagem nela.
- **FR-009**: Em tela cheia no celular, a pessoa MUST poder aproximar a imagem com dois dedos até pelo menos 4 vezes, arrastar dentro das bordas e voltar à imagem inteira com toque duplo. Sair da tela cheia ou girar o celular também MUST voltar à imagem inteira.
- **FR-010**: O celular MUST mostrar a versão de celular da sala em pé e deitado. O critério MUST ser o tipo de uso (tela pequena ou toque), não só a largura.
- **FR-011**: Com o celular deitado e sem tela cheia, a transmissão MUST ocupar a maior parte da altura. Participantes, troca de transmissão e ações MUST continuar acessíveis sem cobrir a imagem.
- **FR-012**: Tela inicial, entrada, sala, seletor de transmissão e diálogos MUST caber em larguras a partir de 320 pontos, sem rolagem lateral, com todas as ações alcançáveis.
- **FR-013**: Elementos tocáveis no celular MUST ter área de toque de pelo menos 44 × 44 pontos.
- **FR-014**: Nenhum controle ou texto essencial MUST ficar sob entalhe, ilha ou barra de gestos, em pé ou deitado.
- **FR-015**: Campos de texto MUST continuar visíveis, junto com o botão de confirmar, com o teclado do celular aberto.
- **FR-016**: O layout MUST se reorganizar ao girar o celular ou mudar o tamanho da janela, sem recarregar a página nem sair da sala.
- **FR-017**: Toda tela e estado novos (tela cheia, controles visíveis, imagem aproximada, sala deitada) MUST ter mock acessível por parâmetro de URL, sem servidor, câmera ou rede.
- **FR-018**: Textos novos MUST seguir o tom informal em português do Espia.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Em tela cheia num celular deitado, a imagem de uma transmissão 16:9 ocupa pelo menos 90% da largura ou da altura útil da tela.
- **SC-002**: Entrar em tela cheia leva 1 toque a partir da sala, e sair leva 1 toque ou 1 gesto de voltar.
- **SC-003**: Girar o celular reorganiza a tela em até 1 segundo, sem nenhuma interrupção perceptível de som em 10 giros seguidos.
- **SC-004**: Em larguras de 320 a 932 pontos, em pé e deitado, nenhuma tela do Espia tem rolagem lateral e 100% das ações ficam alcançáveis.
- **SC-005**: 100% dos elementos tocáveis da sala no celular têm pelo menos 44 × 44 pontos.
- **SC-006**: Com a transmissão de uma tela de computador em 1080p e imagem aproximada 3 vezes, uma pessoa lê um texto de tamanho padrão de interface na transmissão, num celular de 6 polegadas.
- **SC-007**: Em teste com Android e iPhone, quem nunca usou o Espia entra em tela cheia, aproxima, volta ao tamanho inteiro e sai sem ajuda na primeira tentativa.

## Assumptions

- "Ver a tela inteira" foi lido como o lado de quem assiste: a transmissão ocupando a tela do celular e aparecendo inteira, sem corte. Compartilhar a tela inteira do computador já existe no seletor de transmissão (specs/004 e 005) e não muda aqui.
- Transmitir pelo celular continua limitado à câmera (constituição 1.3.0). Esta feature não muda o que o celular transmite, só como ele mostra a sala e a transmissão.
- O limite de uma transmissão ativa por vez no celular (017) permanece. A tela cheia mostra só essa.
- A tela cheia não força o giro da tela. Ela segue a orientação do aparelho e respeita a trava de giro.
- A aproximação com dois dedos vale para a tela cheia no celular. A sala normal mantém o comportamento atual.
- A tela cheia e a aproximação não pedem mais qualidade à origem. A qualidade continua seguindo as regras de adaptação existentes.
- Navegadores alvo para assistir: Chrome no Android e Safari no iPhone, nas duas versões principais mais recentes.
- Tablets e computadores continuam com a versão atual de computador, que já tem tela cheia.
