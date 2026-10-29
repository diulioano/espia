# Feature Specification: Transmissão leve (memória estável, upload no teto, menos CPU)

**Feature Branch**: `010-transmissao-leve`

**Created**: 2026-10-29

**Status**: Draft

**Relacionada a**: [005-seletor-nativo-desktop](../005-seletor-nativo-desktop/spec.md) (captura nativa e som), [009-memoria-do-app](../009-memoria-do-app/spec.md) (medição e régua `npm run memoria`), [001](../001-espia-screen-share/spec.md) (qualidade por número de espectadores).

**Input**: User description: "Transmissão leve no app desktop. Medido (2026-10-29, v0.1.6, captura nativa 1080p60, 3 pessoas assistindo): (1) com som a página do motor cresce ~12-14 MB/min sem parar (sem som ~2-3 MB/min), e tudo é solto ao parar; (2) a página usa ~27-30% de CPU (compressão por pessoa + cópias de quadros da captura nativa); (3) o upload chegou a 45 Mbps, acima do teto de 6 Mbps por pessoa (esperado ~19 Mbps, que aparece em outro momento). Objetivo: memória estável durante a transmissão, upload sempre dentro do teto, e menos CPU (menos cópias de quadros; baixar a qualidade sozinho com aviso se a CPU ficar alta)."

## Contexto

### O que foi medido (2026-10-29, app 0.1.6, seletor nativo, 1080p a 60 fps, 3 pessoas assistindo)

Fonte: `npm run memoria` a cada 5 s e prints do Gerenciador de Tarefas ([relatório da 009](../009-memoria-do-app/relatorio.md)).

| Medida | Com som | Sem som |
|---|---|---|
| Memória da página | **+12 a 14 MB por minuto, sem parar** (~750 MB por hora) | +2 a 3 MB por minuto, sobe e desce |
| Ao parar a transmissão | volta (~400 → ~270 MB) | volta |
| CPU da página | **27–30 %** do PC | (não medido) |
| Upload | **45 Mbps** num print, 19 Mbps em outro | — |

- O teto de upload em 1080p60 é **6 Mbps por pessoa**: 3 pessoas ≈ 18–19 Mbps. Os 45 Mbps passam de 2× esse teto.
- A CPU vem da compressão, uma vez por pessoa (modelo sem servidor de mídia, princípio II), e das cópias de cada quadro da captura nativa até a compressão.
- Em transmissões longas, a memória sozinha já chega a centenas de MB a mais. E a CPU de ~30 % compete com o jogo.

## Clarifications

### Session 2026-10-29

- Qualidade usada na medição: 1080p a 60 fps (pessoa usuária).
- Baixar a qualidade sozinho quando o PC não dá conta: **sim, automático e sem aviso** (a pessoa usuária achou o aviso chato). Volta sozinho quando o PC folga. A qualidade atual aparece só no indicador que já existe.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Transmitir por horas sem a memória subir (Priority: P1) 🎯

Transmito o jogo com som, pelo app, por uma noite inteira. A memória do Espia fica no mesmo patamar do começo ao fim, em vez de crescer sem parar.

**Why this priority**: é um defeito (vazamento) e piora com o tempo. Em 1 hora seriam ~750 MB a mais, o bastante para atrapalhar o jogo.

**Independent Test**: transmitir uma janela com som, pelo seletor nativo, para 1 pessoa por 15 min, medindo a cada minuto. A curva fica estável.

**Acceptance Scenarios**:

1. **Given** estou transmitindo com som pelo seletor do app, **When** passam 15 min, **Then** a memória do Espia cresce no máximo 3 MB por minuto em média (o mesmo de sem som), medida do minuto 2 ao 15.
2. **Given** transmito com som por 15 min, **Then** quem assiste continua ouvindo o som sincronizado com a imagem, sem cortes perceptíveis nem atraso que cresça com o tempo.
3. **Given** o PC fica ocupado por um instante (o jogo carregando, por exemplo), **When** o som atrasa, **Then** o Espia descarta o atraso em vez de acumular. Quem assiste pode ouvir um corte curto, mas o atraso não se acumula.

---

### User Story 2 — O upload nunca passa do teto (Priority: P2)

Transmito para 3 pessoas. O meu upload fica sempre perto do teto combinado (6 Mbps por pessoa em 1080p60), inclusive quando alguém entra, sai ou troca a minha tela de miniatura para tela cheia.

**Why this priority**: 45 Mbps de upload trava a internet de quem joga (o jogo online sente) e gasta CPU à toa.

**Independent Test**: transmitir para 3 pessoas em 1080p60 por 5 min, com as pessoas entrando, saindo e trocando a visualização, e registrar o pico de upload do Espia.

**Acceptance Scenarios**:

1. **Given** transmito para N pessoas em tela cheia, **Then** o upload médio de cada intervalo de 5 s fica no máximo 10 % acima de N × o teto por pessoa (mais o som).
2. **Given** alguém acaba de entrar ou de trocar para tela cheia, **Then** o teto já vale para essa pessoa desde o primeiro segundo do vídeo dela.
3. **Given** alguém vê a minha tela em miniatura ou escondida, **Then** essa pessoa conta como miniatura (pouco) ou zero no upload, como hoje.

---

### User Story 3 — Menos CPU para transmitir, e o Espia se ajusta sozinho (Priority: P3)

Transmito em 1080p60. O Espia usa menos CPU que hoje para levar cada quadro da tela até a compressão. E se mesmo assim o PC não der conta, ele baixa a qualidade sozinho, sem me interromper, em vez de travar o jogo ou deixar a transmissão engasgando.

**Why this priority**: ~30 % de CPU com 3 pessoas compete com o jogo. A redução de cópias ajuda sempre; o ajuste automático é a rede de segurança.

**Independent Test**: (a) mesma transmissão da medição (1080p60, 3 pessoas): CPU da página antes e depois. (b) Forçar o PC a ficar ocupado (outro programa pesado) e ver a troca de qualidade no indicador.

**Acceptance Scenarios**:

1. **Given** transmito 1080p60 pelo seletor do app para 3 pessoas, **Then** a CPU da página do Espia fica pelo menos 20 % menor que na medição de base (27–30 % → no máximo ~22 %).
2. **Given** o PC não consegue comprimir na qualidade escolhida por mais de 10 s seguidos, **Then** o Espia baixa um degrau (1080p60 → 1080p30 → 720p30) **sem aviso**: nenhum toast nem janela. Só o indicador de qualidade mostra o valor novo.
3. **Given** a qualidade foi baixada sozinha, **When** o PC fica folgado por 60 s seguidos, **Then** o Espia volta um degrau (até a qualidade escolhida), também sem aviso.
4. **Given** a qualidade baixou sozinha, **Then** quem transmite e quem assiste veem a qualidade nova no indicador que já existe (como no ajuste por número de espectadores).

### Edge Cases

- **Transmissão sem som** (ou só vídeo): nada muda no som. As outras histórias valem igual.
- **Câmera e seletor do navegador** (sem a ponte nativa): a US1 é da ponte nativa. A US2 e a US3 (teto de upload e ajuste automático) valem para todas as fontes.
- **Qualidade já no mínimo (720p30) e o PC ainda não dá conta**: mantém 720p30, sem aviso.
- **Ajuste por número de espectadores (001) e por CPU ao mesmo tempo**: vale o mais baixo dos dois.
- **Pico de CPU curto** (menos de 10 s): não baixa a qualidade.
- **Quem transmite troca a qualidade na mão** durante um ajuste automático: a escolha nova vira o teto, e o ajuste automático recomeça a partir dela.
- **Site no navegador**: a US2 e a US3 também valem no site, já que a compressão é igual. A US1 é só do app.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-1001**: Durante uma transmissão com som pela ponte nativa, a memória do app MUST NOT crescer de forma contínua. O som atrasado MUST ser descartado, em vez de acumulado.
- **FR-1002**: O som transmitido MUST continuar sincronizado com a imagem, com atraso que não cresce com o tempo.
- **FR-1003**: O upload para cada pessoa MUST respeitar o teto da qualidade em vigor desde o primeiro quadro enviado a ela, inclusive ao entrar, reconectar ou trocar de miniatura para tela cheia.
- **FR-1004**: O caminho do quadro, da captura nativa até a compressão, MUST usar menos trabalho de CPU que hoje (menos cópias por quadro), sem perder qualidade de imagem.
- **FR-1005**: Quando o PC não conseguir comprimir na qualidade em vigor por mais de 10 s seguidos, o Espia MUST baixar um degrau (primeiro 60 → 30 fps; depois 1080p → 720p; mínimo 720p30) sem aviso (nenhum toast ou janela).
- **FR-1006**: Depois de 60 s seguidos de folga, o Espia MUST subir um degrau, até a qualidade escolhida na mão.
- **FR-1007**: O ajuste automático MUST combinar com o ajuste por número de espectadores: vale o mais baixo dos dois.
- **FR-1008**: Quem transmite e quem assiste MUST ver a qualidade efetiva atual só pelo indicador que já existe.
- **FR-1009**: Nada disso pode exigir servidor de mídia (princípio II) nem nova ação de quem assiste.

### Key Entities

- **Degrau de qualidade**: 1080p60 → 1080p30 → 720p30 (quem escolheu 720p60 desce para 720p30). Tem a escolhida na mão (teto), a do número de espectadores e a do ajuste por CPU. A efetiva é a mais baixa das três.
- **Estado de folga do PC**: "no limite" ou "folgado", com o tempo seguido em cada estado (10 s para baixar, 60 s para subir).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-1001**: Transmitindo com som pelo app por 15 min (1 pessoa), a memória do Espia cresce no máximo 3 MB por minuto em média (hoje: 12–14).
- **SC-1002**: Na mesma transmissão, quem assiste não percebe o som ficando atrasado em relação à imagem no fim dos 15 min (comparação do começo com o fim).
- **SC-1003**: Com 3 pessoas em 1080p60, em 5 min com entradas, saídas e trocas de visualização, nenhum intervalo de 5 s passa de 21 Mbps de upload (3 × 6 Mbps + som + 10 %). Hoje: pico de 45.
- **SC-1004**: Com 3 pessoas em 1080p60 pelo seletor do app, a CPU da página do Espia fica no máximo em ~22 % (hoje: 27–30 %).
- **SC-1005**: Com o PC forçado ao limite, a qualidade baixa em até 15 s (visível no indicador, sem aviso). Com o PC folgado de novo, ela volta em até 75 s.

## Assumptions

- Medições com a mesma régua da 009 (`npm run memoria` e a aba Processos do Gerenciador).
- O modelo sem servidor de mídia continua (princípio II): a compressão uma vez por pessoa é aceita. A US3 reduz o custo de cada uma, mas não muda o modelo.
- Os degraus automáticos (1080p60 → 1080p30 → 720p30) seguem as qualidades que o Espia já oferece.
- Sem aviso do ajuste automático (decisão da pessoa usuária): o indicador de qualidade que já existe basta.
- Só o app tem a ponte nativa (US1). O resto vale também no site.
