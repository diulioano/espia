# Feature Specification: Memória do app (soltar o motor de página quando não precisa)

**Feature Branch**: `009-memoria-do-app`

**Created**: 2026-10-29

**Status**: Draft

**Relacionada a**: [003-espia-desktop](../003-espia-desktop/spec.md) (meta de RAM, SC-002), [005-seletor-nativo-desktop](../005-seletor-nativo-desktop/spec.md) (captura nativa), [006-transmitir-camera](../006-transmitir-camera/spec.md) (câmera) e [007-fechar-para-bandeja](../007-fechar-para-bandeja/spec.md) (bandeja).

**Input**: User description: "cara, temos um problema. ao fechar a transmissão ele continua rodando o webview2. e consome muita ram. inclusive seria interessante empacotar todo o gasto dentro do espia, pq no gerenciador tem o Webview2 e os Node.js que o espia abre"

## Contexto

### O que foi medido (2026-10-29, Espia v0.1.5 instalado)

Gerenciador de Tarefas, aba Processos, coluna Memória. Prints da pessoa usuária: janela do Espia **minimizada**, numa sala, ninguém mais transmitindo.

| Processo (grupo "WebView2 Manager", ícone do castor) | Transmitindo | 2 min depois de parar |
|---|---|---|
| WebView2: Espia (a página) | 103,6 MB | 94,4 MB |
| WebView2 GPU Process | 88,7 MB | 83,6 MB |
| WebView2 Manager (processo principal do motor) | 27,0 MB | 26,4 MB |
| Utilitários (rede, V8, som, armazenamento) + Crashpad | 24,0 MB | 23,9 MB |
| **Total** | **243,3 MB** | **228,3 MB** |

- **Parar de transmitir já devolve a memória da transmissão** (−15 MB). O que pesa é o **custo fixo do motor** (~228 MB) enquanto ele estiver carregado, mesmo sem ninguém transmitindo e com a janela minimizada.
- **Os "Node.js" não são do Espia.** São servidores de linguagem do Zed (eslint e tailwind), filhos do `Zed.exe`. O Espia não abre Node.js.
- **O agrupamento já existe em parte**: o Windows junta os 8 processos do motor num grupo com o ícone do castor, mas dá a ele o nome do processo principal, "WebView2 Manager". O Windows escolhe esse nome e não há como trocar.
- Uma medida anterior (conjunto de trabalho, ~490 MB) contava memória compartilhada. A régua desta feature é a coluna Memória do Gerenciador (conjunto de trabalho privado).

### O problema relatado

1. Depois de parar a transmissão, com a janela minimizada ou fechada no X (bandeja, 007), o motor continua rodando e segurando ~228 MB.
2. No Gerenciador, o gasto aparece como "WebView2", e não como "Espia".

## Clarifications

### Session 2026-10-29

- Q1: O que acontece escondido na bandeja sem transmitir? → **A: descarregar o motor de página só quando estiver fora de sala.** Numa sala, o motor continua (é ele que mantém a conexão), mas solta o que não usa.
- Preocupação principal da pessoa: **quem estava transmitindo e fecha** não pode ficar com o motor pesado rodando.
- Medição com prints (acima): parar já devolve a memória da transmissão. A prioridade passa a ser o custo fixo do motor (US1, US2). A antiga "pedir memória ao motor depois de parar" foi removida (ganho de ~15 MB, e conflitava com a regra da bandeja).
- Sair da sala com a janela escondida: usa **a mesma espera de 30 s** antes de descarregar (uma regra só).
- Minimizada também conta: não dá para descarregar (o botão na barra de tarefas precisa continuar funcionando), mas o motor deve soltar o que não usa.
- Gerenciador: aceitar o nome "WebView2 Manager" que o Windows dá ao grupo. O que importa é que tudo do Espia esteja junto, com o castor, e que nada sobre depois de sair.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Escondido fora de sala, o motor sai (Priority: P1) 🎯

Paro de transmitir, saio da sala e fecho no X (ou fecho no X e depois a sala acaba). O Espia fica só com o ícone na bandeja, sem nenhum "WebView2" dele rodando. Quando clico no ícone, ele volta em segundos.

**Why this priority**: é o pedido: "ao fechar a transmissão ele continua rodando o webview2". Tira os ~228 MB inteiros no caso mais comum, o app parado na bandeja o dia todo.

**Independent Test**: na tela inicial, fechar no X, esperar 1 min e olhar o Gerenciador. Nenhum processo do motor do Espia, total ≤ 40 MB.

**Acceptance Scenarios**:

1. **Given** o Espia escondido na bandeja, **fora de sala**, **When** passam 30 s, **Then** o motor de página é descarregado. Em 1 min, o Espia usa no máximo 40 MB no total, sem nenhum processo "WebView2" dele.
2. **Given** eu estava transmitindo, paro, saio da sala e fecho no X (em qualquer ordem, ou a sala acaba com a janela escondida), **Then** acontece o mesmo do cenário 1, contando 30 s a partir do momento em que fiquei escondido **e** fora de sala.
3. **Given** escondi e reabro antes dos 30 s, **Then** nada é descarregado e a janela volta como estava.
4. **Given** o motor foi descarregado, **When** abro pela bandeja, pelo atalho ou por um link `espia://`, **Then** a janela volta em até 2 s, na tela inicial (ou na sala do link).
5. **Given** estou transmitindo e escondo a janela, **Then** nada é descarregado e a transmissão continua igual (007, SC-701).

---

### User Story 2 — Minimizado ou escondido numa sala, o motor solta o que não usa (Priority: P2)

Fico numa sala ouvindo a galera, com o Espia minimizado ou na bandeja. Continuo na sala e ouvindo, mas o Espia segura menos memória do que com a janela aberta.

**Why this priority**: é o caso do print (minimizado, ~228 MB). O ganho é menor que o da US1 (o motor continua), mas vale para quem fica numa sala.

**Independent Test**: numa sala, sem transmitir, minimizar e esperar 1 min. Comparar com a janela aberta na mesma sala.

**Acceptance Scenarios**:

1. **Given** estou numa sala sem transmitir, **When** minimizo ou escondo na bandeja e passa 1 min, **Then** continuo na sala (a galera não me vê sair), o som de quem transmite continua, e a memória fica menor ou igual à da janela aberta.
2. **Given** estou minimizado **fora de sala**, **Then** vale o mesmo (o motor não é descarregado, porque a janela continua na barra de tarefas).
3. **Given** estou transmitindo, minimizado ou escondido, **Then** nada muda (sem economia que derrube quem assiste).
4. **Given** restauro a janela, **Then** ela volta ao normal na hora, sem recarregar.

---

### User Story 3 — Parar de transmitir solta tudo da transmissão (Priority: P3)

Paro de transmitir (pelo botão, pela bandeja, porque a sala acabou ou porque saí da sala) e a captura, o som, a câmera e as conexões de envio são encerrados. A memória volta para perto de antes.

**Why this priority**: a medição mostra que isso já funciona no caminho do botão (−15 MB). Fica para confirmar os outros caminhos e evitar vazamento em ciclos.

**Independent Test**: anotar a memória numa sala, transmitir 10 min, parar e esperar 30 s. Repetir saindo da sala no meio da transmissão.

**Acceptance Scenarios**:

1. **Given** estou transmitindo (tela, janela ou câmera), **When** paro por qualquer caminho, **Then** em até 30 s a memória fica no máximo 50 MB acima do que era antes de transmitir, e o aviso do Windows de câmera/microfone some.
2. **Given** transmito e paro 5 vezes seguidas, **Then** a memória não sobe a cada vez.

---

### User Story 4 — Tudo junto e nada sobra (Priority: P3)

No Gerenciador, todos os processos que o Espia usa ficam juntos, com o ícone do castor. Quando saio do Espia, ou ele é fechado à força, nenhum desses processos fica rodando.

**Why this priority**: transparência e limpeza. O agrupamento já existe (print). Falta garantir que nada sobra.

**Independent Test**: "Sair" e "Finalizar tarefa" no Espia → em 5 s nenhum processo do Espia no Gerenciador.

**Acceptance Scenarios**:

1. **Given** o Espia aberto, **Then** os processos do motor dele aparecem juntos num grupo com o ícone do castor (o nome do grupo é o que o Windows der).
2. **Given** outros apps que usam o mesmo motor (WhatsApp, Pesquisa do Windows), **Then** os processos deles não entram nesse grupo.
3. **Given** saio do Espia (Sair, 007) ou ele é fechado à força, **Then** em até 5 s não sobra nenhum processo dele, e nenhum programa que eu abri a partir dele (navegador aberto por um link) é fechado.

### Edge Cases

- **Assistindo (sem transmitir)**: ao sair da sala, a memória do vídeo dos outros volta. Escondido fora de sala, o motor sai de qualquer forma (US1).
- **Transmitir de novo logo depois de parar**: funciona normalmente, no mesmo tempo de antes.
- **Câmera/placa de captura**: ao parar, o dispositivo fica livre para outro app (OBS, Discord) na hora.
- **Motor compartilhado**: o Espia nunca encerra nem mexe em processos do motor de outros apps.
- **Minimizar e restaurar rápido**: sem piscar nem recarregar.
- **Site no navegador**: nada muda. Isto é só do app desktop.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-901**: Ao parar uma transmissão (por qualquer caminho: botão, bandeja, fim da sala, saída da sala, queda), o app MUST encerrar a captura de vídeo, a captura de som e as conexões de envio daquela transmissão.
- **FR-902**: *(removido em 2026-10-29: a medição mostrou que parar já devolve a memória da transmissão.)*
- **FR-903**: Transmitir e parar repetidas vezes MUST NOT fazer a memória crescer a cada ciclo.
- **FR-904**: Com a janela escondida na bandeja e **fora de sala** por 30 s seguidos, o app MUST descarregar o motor de página por completo (Q1: A), e MUST carregá-lo de novo ao reabrir a janela.
- **FR-904a**: Com a janela escondida **numa sala** e sem transmitir, o app MUST manter a pessoa na sala (inclusive o som) e MUST pedir ao motor que solte a memória que não usa.
- **FR-904b**: Sair da sala com a janela já escondida (inclusive quando a sala acaba) MUST começar a mesma espera de 30 s do FR-904.
- **FR-904c**: Com a janela **minimizada** e sem transmitir, o app MUST pedir ao motor que solte a memória que não usa, sem descarregá-lo.
- **FR-905**: Transmitindo, o app MUST NOT aplicar nenhuma economia (nem soltar memória, nem descarregar), esteja a janela visível, minimizada ou escondida.
- **FR-906**: Todos os processos do motor do Espia MUST ser do Espia (abertos por ele) e aparecer juntos no Gerenciador com o ícone do castor. O nome do grupo é o que o Windows der ("WebView2 Manager").
- **FR-907**: Ao sair (ou ser fechado à força), nenhum processo do motor do Espia MUST continuar rodando.
- **FR-908**: O app MUST NOT encerrar, agrupar ou afetar processos que não são do motor dele: motor de outros apps e programas abertos a partir do Espia (navegador aberto por um link).
- **FR-909**: O site no navegador MUST continuar igual.

### Key Entities

- **Estado de uso do app**: janela visível, minimizada ou escondida; numa sala ou não; transmitindo ou não. Decide quanto o app pode economizar.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-901**: Até 30 s depois de parar uma transmissão de 10 min (por qualquer caminho), a memória total do Espia fica no máximo 50 MB acima do valor medido antes de transmitir. (Base: 243 → 228 MB, já atende pelo botão.)
- **SC-902**: Depois de 5 ciclos de transmitir e parar, a memória total varia no máximo 50 MB em relação ao primeiro ciclo.
- **SC-903**: Escondido na bandeja, fora de sala, por 1 min: no máximo 40 MB no total e 0 processos de motor do Espia.
- **SC-903a**: Minimizado ou escondido numa sala, sem transmitir, por 1 min: memória menor ou igual à da mesma situação com a janela aberta, com o som funcionando.
- **SC-903b**: Reabrir com o motor descarregado: janela pronta em até 2 s.
- **SC-904**: Aberto numa sala, sem transmitir, com a janela visível: memória igual ou menor que a de hoje (~228 MB). Registrar o valor alcançado (meta da 003: ~150 MB).
- **SC-905**: 0 processos do Espia 5 s depois de "Sair" ou de "Finalizar tarefa"; um navegador aberto a partir do Espia continua aberto.
- **SC-906**: Escondendo ou minimizando durante uma transmissão, quem assiste não percebe interrupção (0 quedas em 10 tentativas, como na 007).

## Assumptions

- Os "Node.js" vistos no Gerenciador são do Zed (editor), não do Espia. Ficam fora.
- O app continua usando o motor de página do Windows (não embutir um navegador próprio, que pesaria mais).
- "Fechar a transmissão" cobre parar a própria transmissão, minimizar e fechar no X (bandeja).
- Medida de memória = coluna Memória da aba Processos do Gerenciador (conjunto de trabalho privado), somando os processos do Espia.
- Só Windows (como 003, 005 e 007).
