# Feature Specification: Transmissão sob demanda com baixo atraso e custo controlado

**Feature Branch**: `main` — nenhuma branch criada nesta etapa; diretório independente da branch.

**Created**: 2026-11-05

**Status**: Draft

**Input**: User description: "Melhorar qualidade e performance por etapas, mantendo site e app. Duas transmissões, até sete espectadores assistindo às duas, uso superior a 60 horas mensais. Priorizar baixo atraso com adaptação ao conteúdo, buscando até 500 ms na maior parte do tempo. Conexão direta por padrão; ficar na franquia gratuita de mídia e bloquear retransmissão sem saldo. Começar a assistir somente após clicar em Assistir. Avaliar distribuição por servidor antes de migrar o motor."

**Relacionada a**: [014-assistir-e-remover](../014-assistir-e-remover/spec.md), [016-reduzir-atraso-imagem](../016-reduzir-atraso-imagem/spec.md), [discussão](../../docs/design/transmissao-motor-transporte.md) e [decisão de transporte e custo](../../docs/adr/0001-conexao-direta-e-franquia-de-midia.md).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Escolher quando começar a assistir (Priority: P1)

Entro na sala e vejo quem transmite. Cada transmissão aparece com nome, descrição disponível e Assistir. Só recebo imagem e som das transmissões que escolho. Posso transmitir sem começar a assistir às outras.

**Why this priority**: evita consumir mídia que ninguém escolheu, reduzindo tráfego e trabalho dos aparelhos.

**Independent Test**: entrar numa sala com duas transmissões, aguardar sem clicar, escolher uma, parar e escolher a outra. Conferir mídia recebida e contagem de espectadores.

**Acceptance Scenarios**:

1. **Given** duas transmissões em andamento, **When** entro na sala, **Then** vejo cartões Assistir, sem receber vídeo nem som de nenhuma.
2. **Given** estou na sala, **When** alguém começa a transmitir, **Then** aparece um cartão e a recepção aguarda meu clique.
3. **Given** duas transmissões disponíveis, **When** clico em Assistir em uma, **Then** somente a escolhida começa a chegar em até 3 segundos em condições estáveis.
4. **Given** estou assistindo, **When** clico em Parar de assistir, **Then** imagem e som deixam de chegar e tocar em até 2 segundos, preservando volume e mudo para uma escolha posterior.
5. **Given** ainda não escolhi transmissões alheias, **When** começo a transmitir ou restauro a janela, **Then** continuo sem recebê-las e minha prévia ao vivo continua dependendo de ação explícita.
6. **Given** uma transmissão termina e recomeça, **When** ela reaparece, **Then** é necessário um novo clique, mesmo que eu assistisse à anterior.
7. **Given** uma queda temporária da sala, **When** retorno à mesma transmissão, **Then** retomo somente as já escolhidas; novas transmissões aguardam clique.

---

### User Story 2 - Assistir junto sem acumular atraso (Priority: P1)

Até sete pessoas assistem a duas transmissões simultâneas. Se a rede ou o aparelho não sustentarem a qualidade escolhida, a imagem se adapta para preservar proximidade com o momento da origem. A adaptação considera legibilidade em texto e fluidez em movimento.

**Why this priority**: comentar junto depende de acompanhar o mesmo momento, sem ficar vários segundos atrás.

**Independent Test**: comparar contador e eventos sonoros na origem e nos receptores, com uma e duas transmissões e com um, quatro e sete espectadores. Incluir texto, movimento, redução temporária de capacidade e recuperação.

**Acceptance Scenarios**:

1. **Given** condições estáveis documentadas, **When** sete espectadores assistem às duas transmissões por 30 minutos, **Then** cada recepção atende a até 500 ms em pelo menos 95% das amostras estáveis, sem amostra de 5 segundos ou mais.
2. **Given** redução temporária de capacidade, **When** a qualidade escolhida causaria acúmulo de atraso, **Then** a imagem se adapta sem exigir ajuste manual de atraso e sem acumular reprodução de conteúdo antigo.
3. **Given** condições normais restabelecidas, **When** ocorre recuperação, **Then** o atraso volta a até 500 ms em até 5 segundos e a diferença entre som e imagem fica em até 200 ms no mesmo prazo.
4. **Given** início de visualização, restauração de janela ou mudança de qualidade em condições estáveis, **When** ocorre a transição, **Then** a imagem aparece em até 3 segundos e atende à meta de atraso em até 5 segundos após a transição.
5. **Given** dificuldade na rede de um espectador, **When** os demais continuam com capacidade suficiente, **Then** a dificuldade individual não os tira dos limites de atraso.

---

### User Story 3 - Usar sem exceder o saldo gratuito (Priority: P1)

Quero conexão direta por padrão. A retransmissão fica limitada ao saldo gratuito seguro. Sem saldo, continuo diretamente quando possível; se a rede exigir retransmissão, recebo uma explicação clara.

**Why this priority**: uso acima de 60 horas mensais torna arriscado distribuir tudo por servidor sem controle de consumo.

**Independent Test**: variar saldo disponível, reservado, esgotado e desconhecido, com várias salas ativas. Conferir admissões, consumo ativo e interrupção de retransmissões.

**Acceptance Scenarios**:

1. **Given** conexão direta viável, **When** começo a assistir, **Then** ela é o caminho padrão, sem exigir retransmissão.
2. **Given** conexão direta inviável e saldo seguro disponível, **When** preciso de retransmissão, **Then** o uso só é permitido dentro do orçamento gratuito reservado.
3. **Given** saldo esgotado ou não confiável, **When** tento iniciar ou manter retransmissão, **Then** novas alocações são recusadas e o consumo ativo cessa antes de exceder o orçamento reservado, tentando conexão direta.
4. **Given** saldo indisponível e rede que exige retransmissão, **When** tento assistir, **Then** vejo a razão da indisponibilidade, sem interromper quem consegue conexão direta.
5. **Given** duas salas disputando o saldo restante, **When** pedem retransmissão simultaneamente, **Then** o total autorizado não ultrapassa o saldo seguro compartilhado.
6. **Given** franquia renovada e saldo confirmado, **When** clico em Tentar novamente, **Then** a transmissão escolhida pode voltar; cartões não escolhidos continuam aguardando Assistir.

---

### User Story 4 - Comparar antes de mudar a distribuição (Priority: P2)

Como responsável pelo Espia, quero comparar a transmissão atual com uma alternativa em ensaio controlado. Quero decidir por atraso, qualidade, carga dos aparelhos e consumo, sem comprometer salas normais ou assumir cobrança.

**Why this priority**: mudar distribuição só se justifica com melhora observável dentro do limite de custo.

**Independent Test**: executar o mesmo roteiro com aparelhos, cenas e condições equivalentes, produzindo um relatório por recepção com qualidade efetiva e consumo.

**Acceptance Scenarios**:

1. **Given** medição inicial reproduzível, **When** comparo uma alternativa, **Then** o relatório distingue melhora real de mudanças de qualidade, cena ou número de espectadores.
2. **Given** falta de aparelhos ou evidência de custo e desempenho, **When** consolido o relatório, **Then** a aceitação permanece pendente; estimativas e simulações não substituem medições reais.
3. **Given** alternativa sem melhora comprovada ou fora do orçamento, **When** termino o ensaio, **Then** conexão direta continua padrão e a alternativa não é ativada nas salas normais.

### Edge Cases

- Sem espectadores: a transmissão continua anunciada, sem enviar mídia para quem não escolheu assistir. Captura e prévia local não equivalem a recepção alheia.
- Nenhuma tela escolhida: cartões sem recebimento, decodificação ou reprodução oculta de imagem e som, inclusive miniaturas ao vivo.
- Clique seguido de Parar, saída ou encerramento: uma abertura atrasada não reativa mídia.
- Reconexão não transfere a escolha para nova transmissão ou outra pessoa. Nova entrada após saída explícita começa aguardando clique.
- Janela escondida: só transmissões já escolhidas podem manter som. Restaurar janela, mudar grade, selecionar destaque ou volume não equivale a Assistir.
- Visualização fora da janela mantém o comportamento existente somente para conteúdo escolhido.
- Internet ou aparelho insuficientes: informar limitação; a meta estável não se aplica durante a perturbação, mas a recuperação é avaliada após normalização. Tela parada ou sem imagem útil não demonstra sucesso.
- Saldo desconhecido, várias salas, credenciais antigas e consumo ainda não contabilizado devem ser considerados no bloqueio. Fim de transmissão ou sala prevalece sobre qualquer retomada.
- Celular: nenhum conteúdo começa automaticamente; pode permanecer o limite atual de uma transmissão ativa por vez.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Cada transmissão alheia MUST começar disponível, sem vídeo nem som recebidos até Assistir. A regra vale ao entrar na sala, em novas transmissões e após nova entrada explícita.
- **FR-002**: O cartão MUST identificar pessoa e descrição disponível e oferecer Assistir, sem exigir instalação, conta ou configuração de transporte.
- **FR-003**: Assistir MUST iniciar somente a transmissão escolhida. Parar MUST interromper imagem e som em até 2 segundos, preservando volume e mudo.
- **FR-004**: A escolha MUST valer para aquela permanência na sala e transmissão específica. Queda temporária preserva a escolha; encerramento e reinício exigem novo clique.
- **FR-005**: Transmitir, restaurar janela, navegar ou selecionar destaque MUST NOT iniciar recepção não escolhida. A prévia própria ao vivo MUST continuar sob ação explícita.
- **FR-006**: O número de espectadores mostrado ao transmissor MUST contar somente pessoas efetivamente recebendo vídeo. Cartões disponíveis e recepção só de som não contam.
- **FR-007**: Esconder uma transmissão escolhida MUST suspender vídeo e manter som conforme a regra atual; MUST NOT ativar som não escolhido. Parar corta ambos.
- **FR-008**: O Espia MUST suportar duas transmissões com até sete espectadores assistindo às duas, dentro do limite atual de dez pessoas por sala.
- **FR-009**: Cada recepção MUST atender às metas de atraso, recuperação e transições da história 2 nas condições estáveis documentadas, priorizando conteúdo atual.
- **FR-010**: A adaptação MUST considerar o conteúdo, mostrar qualidade efetiva e ajuste ocorrido e respeitar o teto escolhido. Capacidade insuficiente não implica promessa de resolução ou fluidez fixas.
- **FR-011**: Encerramento ou pedido de parar MUST prevalecer sobre operações pendentes e recuperação. O servidor MUST rejeitar início, atualização ou encerramento atrasados de uma transmissão substituída. Repetir o pedido de uma transmissão encerrada MUST NOT anunciá-la novamente, inclusive após retomada ou reinício do servidor.
- **FR-012**: Dificuldade individual de um espectador MUST NOT tirar os demais dos limites quando a capacidade destes continua suficiente.
- **FR-013**: Conexão direta MUST ser padrão. Retransmissão MUST depender de saldo gratuito seguro disponível.
- **FR-014**: O consumo autorizado MUST respeitar orçamento global dentro da franquia vigente, descontando consumo compartilhado, em curso, reservas e margem para atraso de contabilização e encerramento.
- **FR-015**: Saldo esgotado ou não confiável MUST impedir novas retransmissões e cessar consumo ativo sob controle do Espia antes do limite reservado, tentando conexão direta e informando eventual indisponibilidade.
- **FR-016**: O orçamento MUST ser respeitado entre salas simultâneas e após falha ou reinício. Liberação de saldo MUST depender de confirmação suficiente, sem duplicar capacidade disponível.
- **FR-017**: Se controle efetivo de consumo e encerramento não puder ser demonstrado, a retransmissão MUST permanecer desabilitada. Alertas ou estimativas isoladas MUST NOT equivaler a garantia de custo zero.
- **FR-018**: A comparação MUST manter condições equivalentes e distribuição padrão durante o ensaio, registrando atraso, qualidade recebida, carga dos aparelhos, consumo e limitações da evidência.
- **FR-019**: Diagnósticos MUST NOT gravar conteúdo de mídia nem histórico identificável de participantes. Registros duráveis de consumo MUST conter apenas agregados necessários à proteção da franquia.
- **FR-020**: A mudança MUST preservar site, app opcional, câmera, tela, controles de som, visualização fora da janela e permissões existentes, exceto pela nova exigência de Assistir.

### Key Entities

- **Transmissão**: envio de tela ou câmera por alguém da sala, com início e fim próprios. Reiniciar cria nova transmissão para a escolha de assistir.
- **Escolha de assistir**: decisão local de receber uma transmissão específica. Pode estar aguardando clique, ativa, parada ou temporariamente indisponível; uma queda não equivale a nova escolha.
- **Visualização**: modo de uma recepção escolhida, como grade, destaque, escondida ou fora da janela; não substitui a escolha de assistir.
- **Orçamento gratuito de mídia**: saldo global seguro destinado a retransmissões, considerando franquia, consumo compartilhado, uso em curso e reservas.
- **Ensaio de transmissão**: comparação registrada de condições, aparelhos, cenas, espectadores, qualidade, atraso e consumo, com resultado aprovado, reprovado ou pendente.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Em todas as entradas e inícios do roteiro de aceitação, durante 60 segundos sem clique, cada receptor recebe zero conteúdo de vídeo e som das transmissões não escolhidas. Sinalização e identificação textual dos cartões não contam como mídia.
- **SC-002**: Em 100% dos casos do roteiro, escolher uma entre duas transmissões inicia apenas a escolhida em até 3 segundos em condições estáveis. Parar encerra recebimento, reprodução e atualiza a contagem de espectadores em até 2 segundos.
- **SC-003**: Em sessões de 30 minutos com uma e duas transmissões e com um, quatro e sete espectadores, cada recepção apresenta pelo menos 95% dos 1.800 instantes estáveis previstos com atraso ponta a ponta de até 500 ms. Não pode haver atraso de 5 segundos ou mais nem ausência de imagem por 5 segundos consecutivos. Exigir cobertura do instrumento de pelo menos 99% dos instantes previstos e no máximo 2 instantes consecutivos sem medição; abaixo disso, o ensaio é inconclusivo e deve ser repetido. Lacunas não contam como atraso zero e não saem do denominador de 1.800. Ausência de vídeo com instrumento funcionando conta como falha de entrega, não como lacuna do instrumento. Não agregar espectadores para ocultar falhas individuais.
- **SC-004**: Em cinco repetições de cada perturbação e transição da história 2, todas as recepções restabelecem atraso de até 500 ms em até 5 segundos após normalização ou transição. Havendo som, a diferença entre som e imagem fica em até 200 ms no mesmo prazo.
- **SC-005**: Ensaios de saldo esgotado, desconhecido, concorrência e reinício não autorizam consumo acima do saldo seguro, incluindo sessões em andamento. Sem prova de encerramento efetivo, o resultado aceito é retransmissão desabilitada.
- **SC-006**: Em todos os casos de reentrada, reconexão, reinício de transmissão, janela escondida, cliques rápidos e encerramento, a recepção corresponde à escolha descrita nas histórias, sem reativação por operação atrasada.
- **SC-007**: O relatório compara medição inicial e cenário-alvo em condições equivalentes, com qualidade efetiva e consumo. Para texto, exige pelo menos 19 acertos numa lista fixa de 20 itens, com fonte, tamanho, resolução de origem e janela definidos antes do ensaio. Para movimento em visualização completa, com origem gerando pelo menos 30 fps, exige média de pelo menos 27 quadros distintos decodificados por segundo em cada recepção, interrupções somadas de no máximo 1% do trecho estável e nenhuma pausa superior a 1 segundo. Os trechos estáveis, incluindo falhas de vídeo, entram no cálculo. A mediana dos FPS do jogo/tarefa concorrente não pode cair mais de 5% frente à baseline equivalente; manter a mesma cena, configurações, aparelhos e limite de FPS nas duas rodadas. Se não houver tarefa mensurável equivalente, essa parte do aceite permanece pendente. Registrar também CPU/GPU e memória; não declarar ganho somente por cumprir o atraso. Valores de referência e observações não podem ser reduzidos a posteriori para fazer o ensaio passar.

## Assumptions

- O adendo foi interpretado como clique obrigatório para espectadores. A prévia própria já é manual e permanece assim. Não se presume nova prévia automática.
- "Na maior parte do tempo" significa pelo menos 95% das amostras estáveis por recepção. O planejamento define antes dos ensaios os aparelhos, cenas e capacidades de referência. Ausência de evidência mantém aceitação pendente.
- O cenário máximo tem dois transmissores e sete espectadores, nove pessoas ao todo. Mais de 60 horas mensais de uso não significa que o máximo permaneça ativo durante todo o período.
- A qualidade deve se adaptar ao conteúdo; não há garantia permanente de 1080p ou 60 fps. Sem ganho medido, não se declara melhora de performance.
- O orçamento protegido é o de infraestrutura de mídia. Demais custos existentes não são redefinidos. Uso externo na mesma franquia precisa ser contabilizado ou isolado de forma verificável antes de habilitar retransmissão.
- Saldo confiável, encerramento efetivo do consumo e condições de medição são dependências de aceitação. Sem controle comprovado, somente conexão direta é permitida; a indisponibilidade em redes restritivas precisa ser demonstrada e informada.
- Esta feature substitui a decisão de assistir automaticamente da 014 e torna mais estrita a meta estável da 016; os demais comportamentos permanecem como referência.
- A constituição 3.0.0, emendada em 2026-11-05, exige até dois passos para entrar na sala e clique em Assistir por transmissão alheia. A emenda resolveu a divergência do princípio I. Qualquer prova de SFU continua dependendo de emenda ao princípio II e revisão da promessa de criptografia; esta feature não publica serviços.
- Ficam fora do escopo migração ampla do motor para Rust, substituição global do transporte por SFU, servidor próprio, chat e cobrança excedente. A alternativa de distribuição fica limitada a avaliação controlada após resolver suas dependências.
