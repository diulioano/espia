# Feature Specification: Reduzir atraso da imagem

**Feature Branch**: `main` (branch atual; sem hook de criação de branch)

**Created**: 2026-11-04

**Status**: Draft

**Input**: User description: "cara, por algum motivo está dando muito atraso da imagem comparado ao discord, cerca de 10seg para alguns casos"

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Assistir perto do momento atual (Priority: P1)

Como pessoa assistindo à transmissão de um amigo, quero acompanhar a imagem com pouco atraso para conversar sobre o que está acontecendo no Discord.

**Why this priority**: o atraso relatado de cerca de 10 segundos prejudica o uso principal do Espia.

**Independent Test**: transmitir um contador visível e comparar a origem com a imagem recebida durante 30 minutos. Repetir usando Discord nas mesmas condições, sem executar as duas transmissões simultaneamente.

**Acceptance Scenarios**:

1. **Given** rede estável e aparelhos capazes de sustentar a qualidade em vigor, **When** a transmissão dura 30 minutos, **Then** pelo menos 95% das medições mostram atraso de até 1 segundo em relação à origem, sem nenhuma medição de 5 segundos ou mais.
2. **Given** a mesma origem, aparelhos, qualidade equivalente e condições de rede, **When** são comparados Espia e Discord, **Then** o percentil 95 do atraso do Espia fica no máximo 500 ms acima do Discord.
3. **Given** três espectadores com condições estáveis, **When** um deles entra ou alterna entre miniatura e destaque, **Then** os demais continuam atendendo aos limites do cenário 1.

---

### User Story 2 — Recuperar a imagem depois de um engasgo (Priority: P1)

Como pessoa assistindo, quero voltar automaticamente ao momento atual depois de uma oscilação de rede ou de um PC ocupado, sem continuar vendo vários segundos do passado.

**Why this priority**: melhorar somente o começo não resolve o atraso que aparece durante a transmissão.

**Independent Test**: provocar por 10 segundos uma limitação de rede ou sobrecarga que gere atraso, restaurar as condições e medir a recuperação com o contador visível.

**Acceptance Scenarios**:

1. **Given** um engasgo deixou a imagem atrasada, **When** as condições estáveis são restabelecidas, **Then** em até 5 segundos a imagem volta a ter atraso de até 1 segundo, sem recarregar ou reentrar na sala.
2. **Given** há imagens antigas pendentes, **When** a transmissão se recupera, **Then** o Espia prioriza a imagem atual; saltos de imagem são aceitáveis para recuperar a proximidade do momento ao vivo.
3. **Given** a transmissão tem som, **When** se recupera, **Then** som e imagem da própria transmissão voltam a ter diferença de até 200 ms no mesmo prazo de 5 segundos.

---

### User Story 3 — Entrar e trocar a visualização sem carregar atraso (Priority: P2)

Como pessoa na sala, quero começar a assistir e mudar a visualização vendo o momento atual da transmissão.

**Why this priority**: a melhora precisa permanecer nos caminhos normais de uso.

**Independent Test**: entrar numa transmissão em andamento, alternar a visualização, mudar a qualidade, ocultar e restaurar a janela e provocar uma queda seguida de retorno. Comparar o contador após cada transição.

**Acceptance Scenarios**:

1. **Given** condições estáveis e transmissão em andamento, **When** alguém começa a assistir ou volta após uma queda, **Then** em até 5 segundos após a primeira imagem recebida o atraso fica em até 1 segundo.
2. **Given** a pessoa está assistindo em condições estáveis, **When** alterna entre miniatura e destaque, o transmissor muda a qualidade ou a janela volta a ficar visível, **Then** em até 5 segundos após a transição o atraso fica em até 1 segundo.

### Edge Cases

- **Rede insuficiente ou aparelho sobrecarregado por tempo prolongado**: o limite de 1 segundo não é exigido durante essas condições; a recuperação deve ocorrer no prazo definido assim que forem restabelecidas as condições estáveis.
- **Sem som**: os mesmos critérios de atraso da imagem continuam valendo.
- **Conteúdo parado**: ausência de mudanças não significa atraso; usar contador ou eventos visuais identificáveis para medir.
- **Espectadores com redes diferentes**: a dificuldade de um não pode retirar os demais dos limites de rede estável. Quem estava com dificuldade recupera a imagem quando sua conexão melhora.
- **Transmissão encerrada durante o engasgo**: o encerramento prevalece; a recuperação não reativa imagem nem som.
- **Tela, janela e câmera; app e site**: verificar os caminhos existentes sem presumir que o relato esteja restrito a uma origem ou plataforma.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O Espia MUST entregar imagem próxima do momento em que ocorre na origem, atendendo aos limites absolutos e comparativos da história 1.
- **FR-002**: O atraso MUST NOT acumular durante uma sessão de 30 minutos em condições estáveis, conforme o cenário 1 da história 1.
- **FR-003**: Após um engasgo, o Espia MUST recuperar automaticamente o atraso de até 1 segundo em até 5 segundos após a normalização das condições, sem ação da pessoa.
- **FR-004**: A recuperação MUST priorizar a imagem atual sobre a reprodução integral das imagens antigas.
- **FR-005**: Nas transmissões com som, a recuperação MUST restabelecer a diferença de até 200 ms entre som e imagem da própria transmissão no prazo da história 2.
- **FR-006**: Início de visualização, retorno após queda, retorno de janela oculta, troca de miniatura para destaque e mudança de qualidade MUST atender aos prazos da história 3.
- **FR-007**: A entrada ou dificuldade de um espectador MUST NOT fazer os demais deixarem de atender aos limites de atraso quando as condições deles permanecem estáveis.
- **FR-008**: A correção MUST preservar os controles e passos existentes para assistir, sem exigir configuração manual de atraso ou instalação obrigatória.
- **FR-009**: A recuperação MUST respeitar o encerramento da transmissão e MUST NOT reativar imagem ou som de uma transmissão encerrada.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Em sessões de 30 minutos, com 1 e com 3 espectadores, pelo menos 95% das amostras em condições estáveis apresentam atraso de até 1 segundo e nenhuma atinge 5 segundos.
- **SC-002**: Nas comparações equivalentes, o percentil 95 do atraso do Espia supera o do Discord em no máximo 500 ms.
- **SC-003**: Em 5 repetições de cada perturbação (rede insuficiente, transmissor ocupado e espectador ocupado), a imagem volta a até 1 segundo de atraso em até 5 segundos após a normalização, sem intervenção manual.
- **SC-004**: Em 5 repetições de cada transição da história 3, a imagem atende ao prazo de recuperação. Os demais espectadores continuam atendendo a SC-001.
- **SC-005**: Nas repetições com som, a diferença entre um evento visual e seu som correspondente fica em até 200 ms após a recuperação. Em todas as repetições, a pessoa consegue continuar assistindo pelos controles existentes, sem recarregar ou reentrar na sala.
- **SC-006**: Em 5 encerramentos durante um engasgo, nenhuma transmissão encerrada volta a apresentar imagem ou som após a recuperação da conexão.

## Assumptions

- Os cerca de 10 segundos são um relato de atraso percebido, ainda sem reprodução ou causa confirmada. A comparação com Discord é interpretada como atraso de transmissão de imagem; a conversa de voz permanece no Discord.
- Os limites de 1 segundo, 500 ms comparativos, 200 ms entre som e imagem e 5 segundos para recuperação são metas iniciais assumidas para tornar o pedido verificável, não medições já feitas nem valores exigidos explicitamente pela pessoa usuária.
- Condições estáveis significam rede sem limitação artificial ou tráfego concorrente no ensaio e aparelhos sem sobrecarga artificial, capazes de sustentar a qualidade em vigor. Registrar aparelhos, versões, origem, qualidade, espectadores e condições de rede para permitir repetição.
- Medir com referência visual comum entre origem e destino, coletando amostras a cada segundo, sem depender da sincronização dos relógios dos aparelhos. Na comparação, alternar a ordem de Espia e Discord, mantendo conteúdo e condições equivalentes.
- Priorizar os casos relatados quando forem reproduzidos. Tela/janela no app Windows e navegador, câmera e visualização em navegador moderno continuam no escopo existente; nenhuma nova plataforma é adicionada.
- A [feature 010](../010-transmissao-leve/spec.md) já define adaptação de qualidade sem avisos e controle de recursos. A correção preserva esses comportamentos e os limites de sala existentes.
- A constituição permanece como restrição: sem servidor de mídia, sem contas e sem gravação de conteúdo para medir atraso. Esta especificação não escolhe a causa nem a solução técnica.
