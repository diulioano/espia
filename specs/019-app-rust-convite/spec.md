# Feature Specification: App nativo em Rust e convite no endereço oficial

**Feature Branch**: `main` — nenhuma branch criada nesta etapa; diretório independente da branch.

**Created**: 2026-11-11

**Status**: Draft

**Input**: User description: "vamos migrar o app para rust e mudar o link para utilizar o espia.diuliano.com para convite também, oq gastariamos para migrar pode utilizar subagentes quantos você quiser para validar, pode utilizar de modelos inferiores para vasculhar os arquivos"

**Relacionada a**: [003-espia-desktop](../003-espia-desktop/spec.md) (app que abre o site), [005-seletor-nativo-desktop](../005-seletor-nativo-desktop/spec.md) (captura nativa), [009-memoria-do-app](../009-memoria-do-app/spec.md), [010-transmissao-leve](../010-transmissao-leve/spec.md), [011-salas-fixas](../011-salas-fixas/spec.md) (Suas salas e chave de dono), [015-atualizar-app](../015-atualizar-app/spec.md) (atualização automática, não implementada).

**Custo da migração**: [custo-migracao.md](./custo-migracao.md) traz o levantamento pedido, com as opções de alcance, tamanho, prazo e risco.

## Clarifications

### Session 2026-11-11

- Q: Quanto do app passa para a parte nativa em Rust: só o envio da transmissão, o app inteiro sem motor de página, ou app, site e servidor? → A: O app inteiro nativo, sem motor de página. O site continua como está.
- Q: O servidor também em Rust melhoraria o envio? → A: Não. O servidor só apresenta as pessoas umas às outras; imagem e som vão direto de uma pessoa para a outra e nunca passam por ele. O servidor continua como está.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Convite sempre com o endereço oficial (Priority: P1)

Estou numa sala e copio o convite para mandar no grupo. O link sai sempre com `espia.diuliano.com`, não importa se abri o Espia pelo endereço antigo, pelo site ou pelo app. Quem recebe um link antigo que já estava circulando continua entrando na mesma sala.

**Why this priority**: o endereço oficial mudou, mas o convite ainda sai com o endereço em que a página foi aberta. O app que o grupo tem instalado ainda abre o endereço antigo, então todo convite copiado no app espalha o endereço antigo. É a mudança mais barata e não depende do app nativo.

**Independent Test**: abrir a mesma sala pelo endereço oficial, pelo endereço antigo e pelo app. Copiar o convite nos três e conferir o texto copiado e o texto mostrado na sala. Abrir um link antigo e conferir que cai na mesma sala.

**Acceptance Scenarios**:

1. **Given** estou numa sala aberta pelo endereço oficial, no site ou no app, **When** copio o convite, **Then** o link copiado e o link mostrado na sala começam com `espia.diuliano.com`.
2. **Given** abri o site pelo endereço antigo, **When** copio o convite, **Then** o link copiado e o mostrado usam `espia.diuliano.com`, nunca o endereço antigo.
3. **Given** uso o app que já está instalado, que só abre o endereço antigo, **When** copio o convite, **Then** o link usa o endereço oficial, sem eu instalar nada.
4. **Given** recebi um link de sala com o endereço antigo, **When** abro o link, **Then** entro na mesma sala em que entraria pelo link oficial, com os mesmos passos de hoje.
5. **Given** criei uma sala fixa e o convite é copiado sozinho, **When** colo o que foi copiado, **Then** o link usa o endereço oficial.

---

### User Story 2 - Chegar pelo endereço antigo sem perder nada (Priority: P2)

Tenho o endereço antigo salvo nos favoritos, ou uso o app que ainda abre o endereço antigo. Abro o site por ele, ou instalo o app nativo, e passo a usar o endereço oficial. Minhas salas fixas, as salas de que sou dono, meu apelido e minhas preferências aparecem lá, sem eu refazer nada.

**Why this priority**: Suas salas, as chaves de dono e as preferências ficam guardadas por endereço em cada aparelho, no navegador e dentro do app. Mandar a pessoa para o endereço novo sem levar isso junto deixa a lista vazia e tira dela o poder de renomear ou apagar as salas que criou.

**Independent Test**: num navegador e num app com salas fixas, uma sala própria e preferências guardadas no endereço antigo, abrir o endereço antigo no navegador e instalar o app nativo por cima. Conferir o endereço final, a lista Suas salas, as ações de dono e as preferências nos dois.

**Acceptance Scenarios**:

1. **Given** tenho salas fixas, chaves de dono, apelido e preferências guardados no endereço antigo, **When** abro o endereço antigo no navegador, **Then** chego ao endereço oficial com tudo isso presente, sem digitar nem confirmar nada.
2. **Given** abro um link de sala com o endereço antigo no navegador, **When** a página carrega, **Then** estou na entrada da mesma sala no endereço oficial, com no máximo os dois passos de sempre (apelido e entrar).
3. **Given** já tenho salas guardadas nos dois endereços, **When** chego ao oficial vindo do antigo, **Then** as listas são juntadas sem sala repetida e sem perder nenhuma chave de dono.
4. **Given** uso o app que já está instalado, que só abre o endereço antigo, **When** abro o app, **Then** ele continua funcionando e entrando em salas até eu instalar o app nativo.
5. **Given** tenho salas fixas, chaves de dono e preferências guardados no app instalado, **When** instalo o app nativo por cima e abro, **Then** tudo isso está presente e os convites saem com o endereço oficial.
6. **Given** já fui levado ao endereço oficial uma vez, **When** abro o endereço antigo de novo, **Then** sou levado ao oficial sem apagar nem duplicar o que já está lá.

---

### User Story 3 - Transmitir pelo app nativo (Priority: P1)

Abro o app nativo, entro na sala, clico em Transmitir e escolho uma janela no seletor com miniaturas. Meus amigos assistem pelo navegador, no computador ou no celular, sem instalar nem configurar nada. Enquanto transmito e jogo, o app usa menos processador do que usa hoje e o jogo não perde mais FPS do que perde hoje.

**Why this priority**: transmitir é a razão de existir do app. Hoje cada quadro capturado atravessa a página e a mesma tela é preparada uma vez para cada pessoa que assiste. O app nativo precisa fazer o mesmo trabalho pesando menos, e quem está no navegador não pode perceber a troca.

**Independent Test**: na mesma máquina e no mesmo jogo, transmitir 1080p60 para três pessoas no navegador com o app instalado hoje e com o app nativo. Comparar FPS do jogo, uso de processador do app e a imagem e o som recebidos.

**Acceptance Scenarios**:

1. **Given** estou numa sala pelo app nativo, **When** clico em Transmitir, escolho uma janela, a qualidade e o som, **Then** a transmissão começa e aparece para todos na sala, como hoje.
2. **Given** alguém assiste pelo navegador no computador ou no celular, **When** eu transmito pelo app nativo, **Then** a pessoa clica em Assistir e vê imagem e som como hoje, sem nenhum passo a mais.
3. **Given** transmito uma janela de jogo em 1080p60 para três pessoas, **When** comparo com o app instalado hoje na mesma máquina, **Then** o uso de processador do app e a perda de FPS no jogo ficam dentro das metas de Success Criteria.
4. **Given** minha placa de vídeo não ajuda a preparar o vídeo, **When** clico em Transmitir, **Then** a transmissão começa mesmo assim, em qualidade menor se preciso, sem aviso: o indicador de qualidade mostra a qualidade em uso.
5. **Given** estou transmitindo, **When** a rede de quem assiste piora, **Then** a imagem dessa pessoa se ajusta e se recupera sozinha, sem travar por mais tempo do que trava hoje.
6. **Given** estou transmitindo, **When** alguém que não clicou em Assistir está na sala, **Then** essa pessoa não recebe imagem nem som da minha transmissão.
7. **Given** estou transmitindo, **When** troco a qualidade, paro a transmissão ou fecho a janela para a bandeja, **Then** o comportamento é o mesmo de hoje.

---

### User Story 4 - Assistir pelo app nativo (Priority: P1)

Estou no app nativo e um amigo transmite, pelo navegador ou por outro app. Clico em Assistir e vejo a imagem e ouço o som com a mesma qualidade que teria no navegador. Ajusto o volume dele, coloco em tela cheia ou na janela flutuante, e paro de assistir quando quero.

**Why this priority**: o app nativo deixa de aproveitar o que o navegador faz sozinho ao receber vídeo. Se assistir ficar pior do que no site, o grupo volta para o site e o app perde o sentido.

**Independent Test**: com a mesma transmissão 1080p60, assistir lado a lado no navegador e no app nativo. Comparar o tempo até a imagem aparecer, o atraso, a fluidez, o som em sincronia e o comportamento em rede ruim.

**Acceptance Scenarios**:

1. **Given** há uma transmissão na sala, **When** clico em Assistir no app nativo, **Then** vejo a imagem e ouço o som, com atraso e fluidez não piores que os do navegador na mesma rede.
2. **Given** estou assistindo, **When** o som e a imagem tocam por 30 minutos, **Then** os dois continuam em sincronia.
3. **Given** há várias transmissões na sala, **When** assisto a mais de uma, **Then** cada uma tem seu volume e posso destacar qualquer uma.
4. **Given** estou assistindo, **When** minha rede oscila ou cai e volta, **Then** a imagem se recupera sozinha e eu volto para a sala sem refazer nada.
5. **Given** estou assistindo, **When** uso tela cheia, janela flutuante ou Parar de assistir, **Then** funcionam como hoje.
6. **Given** entro numa sala com transmissões, **When** ainda não cliquei em Assistir, **Then** não recebo imagem nem som de ninguém.

---

### User Story 5 - O mesmo Espia, sem perder função (Priority: P1)

Instalo o app nativo e uso como sempre. As telas têm a mesma cara do site: mesmas cores, fontes, textos e o castor. O seletor com miniaturas, o som da janela sem vazar o Discord, a câmera, a bandeja, o link `espia://`, Suas salas e as salas fixas continuam funcionando do mesmo jeito.

**Why this priority**: uma troca interna que tira função ou muda a cara do produto é uma piora para quem usa. O grupo só deve notar que ficou mais leve.

**Independent Test**: percorrer a lista de paridade (FR-012) no app nativo e marcar cada item como igual, melhor ou pior em relação ao app instalado hoje. Comparar cada tela com a tela equivalente do site, lado a lado.

**Acceptance Scenarios**:

1. **Given** o app nativo instalado, **When** percorro a lista de paridade, **Then** todo item funciona igual ou melhor do que no app instalado hoje.
2. **Given** uma tela do app nativo e a mesma tela do site lado a lado, **When** comparo, **Then** as cores, fontes, textos, ícones, o castor e a disposição são os mesmos, e nenhuma ação está em lugar diferente.
3. **Given** a sala tem gente no site, no app instalado hoje e no app nativo, **When** qualquer uma transmite ou assiste, **Then** todos se veem e se ouvem.
4. **Given** uma tela ou estado do app nativo, **When** abro o app no modo de dados falsos, **Then** vejo essa tela sem servidor, câmera, captura ou rede.
5. **Given** uso só o site, **When** o app nativo é publicado, **Then** o site funciona como antes, sem instalar nada.

---

### User Story 6 - App leve (Priority: P2)

Deixo o app aberto o dia inteiro, numa sala ou escondido na bandeja. Ele ocupa menos memória do que ocupa hoje e não aparece mais como vários processos de navegador no Gerenciador de Tarefas. Abre rápido.

**Why this priority**: é o ganho que só o app inteiro nativo pode dar. Vem depois de funcionar bem, porque um app leve que transmite ou mostra pior não serve.

**Independent Test**: medir a memória do app com o roteiro de medição do projeto, numa sala assistindo, numa sala transmitindo, parado numa sala e na bandeja fora de sala. Medir o tempo de abertura. Comparar com o app instalado hoje na mesma máquina.

**Acceptance Scenarios**:

1. **Given** estou numa sala assistindo a uma transmissão 1080p, **When** meço a memória do app, **Then** ela fica dentro da meta de Success Criteria e abaixo da do app instalado hoje.
2. **Given** o app está na bandeja e fora de sala, **When** meço a memória, **Then** ela não passa da de hoje.
3. **Given** abro o Gerenciador de Tarefas, **When** procuro o Espia, **Then** vejo o Espia com o nome e o ícone dele, sem grupo de processos de navegador.
4. **Given** o app está fechado, **When** abro, **Then** a tela inicial aparece dentro da meta de Success Criteria.

---

### User Story 7 - Trocar de versão sem susto (Priority: P2)

O app nativo me avisa quando tem versão nova e eu atualizo sem procurar o instalador. Se a versão nova der problema no meu computador, consigo voltar a usar o app de antes ou o site, sem perder minhas salas.

**Why this priority**: hoje o app mostra o site, então cada publicação do site atualiza o app de todo mundo. O app nativo perde isso: cada mudança de tela passa a ser uma versão nova do app. Sem um jeito fácil de atualizar, o grupo fica com versões diferentes que não se entendem.

**Independent Test**: publicar uma versão de teste mais nova e conferir o aviso e a atualização. Entrar numa sala com uma versão velha demais e conferir o aviso. Instalar o app de antes por cima do nativo e conferir que funciona.

**Acceptance Scenarios**:

1. **Given** existe uma versão mais nova do app nativo, **When** abro o app, **Then** ele avisa e me deixa atualizar sem eu procurar o instalador, e volto para onde estava.
2. **Given** minha versão é velha demais para a sala, **When** tento entrar, **Then** o app explica em uma frase e oferece atualizar, em vez de falhar em silêncio.
3. **Given** o app nativo não me serve, **When** instalo o app de antes pelo site ou abro o site no navegador, **Then** consigo transmitir e assistir, com minhas salas e preferências.
4. **Given** o site foi publicado com suporte ao app nativo, **When** alguém usa o app instalado hoje, **Then** ele continua transmitindo e assistindo.
5. **Given** estou transmitindo, **When** chega uma versão nova, **Then** nada é interrompido e a atualização espera eu decidir.

### Edge Cases

- Página aberta em endereço de desenvolvimento ou de teste: o convite usa o endereço daquela página, para o teste local continuar possível. A regra do endereço oficial vale para o que está publicado.
- Navegador que bloqueia o que a página guarda (janela anônima, dados bloqueados): a pessoa é levada ao endereço oficial e entra na sala. Só não há o que carregar junto.
- Limite de 5 salas fixas por aparelho ao juntar as listas dos dois endereços: ficam as 5 usadas mais recentemente. Uma sala de que a pessoa é dono nunca perde a chave, mesmo fora da lista.
- Sala rápida em andamento quando a pessoa é levada do endereço antigo ao oficial: ela volta para a mesma sala, como numa queda.
- Link oficial ou antigo colado no app, ou aberto com o app já aberto: abre a sala no app, como hoje.
- Computador sem placa de vídeo capaz, com driver antigo ou com Windows sem o recurso necessário: o app transmite e assiste em qualidade menor se preciso, sem aviso (o indicador de qualidade mostra). Nunca fica em tela preta sem explicação.
- Quem assiste pelo iPhone ou por um navegador com limite de qualidade: recebe uma imagem que consegue mostrar, mesmo que menor que a transmitida.
- Dez pessoas na sala e vários espectadores: a qualidade continua baixando sozinha conforme o número de espectadores.
- Quem assiste com rede muito pior que a dos outros: o problema de uma pessoa não derruba a imagem das demais além do que já acontece hoje.
- Alguém entra ou perde pacotes no meio da transmissão: os outros espectadores não veem a imagem travar por causa disso.
- Duas transmissões da mesma pessoa em sequência, troca de janela e troca de qualidade no meio: sem sobra de processo, som duplicado ou imagem congelada.
- App escondido na bandeja transmitindo: continua transmitindo.
- Primeira transmissão ou primeira vez assistindo depois de instalar: se o Windows pedir permissão de rede, o app explica o que é antes. Negar a permissão resulta num aviso claro, não numa sala muda.
- Sem internet ao abrir o app: aparece a tela de sem conexão, com tentar de novo, como hoje.
- Publicação do site antes do app nativo, e o contrário: nenhuma das duas ordens quebra quem está numa sala.
- App nativo e app de antes instalados no mesmo computador: só um fica instalado e só uma janela abre. O link `espia://` abre o que estiver instalado.
- Nomes, apelidos e nomes de sala com acento, emoji ou muito longos: aparecem como no site.
- Monitor com escala de 125%, 150% ou 200% e mais de um monitor: as telas ficam nítidas e do tamanho certo.

## Requirements *(mandatory)*

### Functional Requirements

**Convite e endereço**

- **FR-001**: O convite mostrado e copiado MUST usar sempre o endereço oficial `espia.diuliano.com`, no site, no app nativo e nos apps já instalados, qualquer que seja o endereço em que a página foi aberta.
- **FR-002**: Links de sala com o endereço antigo MUST continuar levando à mesma sala, no navegador e no app, com no máximo os dois passos de entrada de hoje.
- **FR-003**: Quem abre o endereço antigo no navegador MUST ser levado ao endereço oficial, na mesma tela ou sala.
- **FR-004**: Na ida do endereço antigo para o oficial no navegador, e na troca do app instalado pelo app nativo, os dados do aparelho (Suas salas, chaves de dono, apelido e preferências) MUST continuar presentes sem ação da pessoa. Listas dos dois endereços MUST ser juntadas sem repetição e sem perda de chave de dono.
- **FR-005**: As chaves de dono MUST NOT ser enviadas ao servidor nem ficar em registro do servidor durante essa ida. Elas continuam existindo só no aparelho.
- **FR-006**: Os apps já instalados, que só abrem o endereço antigo, MUST continuar abrindo e entrando em salas enquanto a pessoa não instala o app nativo. O app nativo MUST usar o endereço oficial.
- **FR-007**: Em endereço de desenvolvimento ou teste, o convite MUST usar o endereço da própria página.
- **FR-008**: Documentos do projeto que citam o endereço antigo como endereço do site MUST passar a citar o oficial. Os registros históricos de publicação ficam como estão.

**App nativo**

- **FR-009**: Transmitir tela ou janela pelo app nativo MUST pesar menos no computador de quem transmite do que pesa no app instalado hoje, nas metas de Success Criteria.
- **FR-010**: O app para Windows MUST ser inteiro nativo: telas, sala, transmissão e recepção funcionam sem motor de página embutido e sem carregar o site. O site e o servidor MUST continuar como estão, mudando só o necessário para o convite, o endereço e a convivência com o app nativo.
- **FR-011**: Quem usa o site, em computador ou celular, MUST continuar transmitindo para o app nativo e assistindo às transmissões dele sem instalar, atualizar ou configurar nada. O site MUST continuar funcionando sem o app.
- **FR-012**: O app nativo MUST manter, igual ou melhor, tudo o que o app faz hoje: tela inicial, criar sala rápida e sala fixa, entrar por código ou link, entrada com apelido, seletor com miniaturas de telas e janelas, som só da janela escolhida, tela inteira com som sem o Discord, aviso de som indisponível, câmera e placa de captura com som de voz ou de jogo, escolha e troca de qualidade, ajuste automático de qualidade, nome do programa transmitido, prévia parada da própria transmissão e ver a própria tela em destaque, Assistir e Parar de assistir, volume por pessoa, destaque, tela cheia, janela flutuante, lista de pessoas, Suas salas, renomear e apagar sala fixa, remover pessoa, avisos da sala, volta automática depois de queda, tela de sala não encontrada, tela de sem conexão, fechar para a bandeja, ícone de ao vivo, Parar transmissão pela bandeja, confirmação ao sair transmitindo, link `espia://`, instância única, links externos no navegador padrão, posição da janela lembrada e nenhuma barra "está compartilhando" na tela.
- **FR-013**: Quando a placa de vídeo ou o Windows não ajudam a preparar ou mostrar o vídeo, o app nativo MUST ainda transmitir e assistir, em qualidade menor se preciso. MUST NOT mostrar aviso técnico sobre placa de vídeo: a pessoa vê só o indicador de qualidade (decisão do dono em 2026-11-12; FR-014 da 020).
- **FR-014**: Salas com pessoas no site, no app instalado hoje e no app nativo MUST funcionar entre todas elas, em qualquer combinação de quem transmite e quem assiste.
- **FR-015**: O app nativo MUST poder ser instalado por cima do app de hoje sem perder os dados do aparelho, a posição da janela nem a preferência de fechar para a bandeja. O app de antes MUST continuar disponível para baixar e funcionando com o site enquanto o app nativo estiver em teste com o grupo.
- **FR-016**: O app nativo MUST manter a transmissão direta entre as pessoas, sem servidor de mídia, sem contas e sem guardar nada novo sobre as pessoas. Imagem e som MUST continuar protegidos de ponta a ponta entre as pessoas.
- **FR-017**: A qualidade enviada e recebida pelo app nativo MUST NOT ser pior que a do navegador em rede boa, e MUST se ajustar e se recuperar sozinha em rede ruim. Som e imagem MUST ficar em sincronia.
- **FR-018**: Toda tela e todo estado do app nativo MUST poder ser aberto com dados falsos, sem servidor, câmera, captura ou rede, cobrindo os mesmos estados que o site cobre.
- **FR-019**: As telas do app nativo MUST seguir o design Espia: mesmas cores, fontes, textos, ícones, castor e disposição do site. Os textos MUST seguir o tom informal em português e usar "Transmitir".
- **FR-020**: A mudança do app para inteiro nativo MUST entrar por emenda da constituição antes do plano, porque troca a restrição técnica atual do app desktop.
- **FR-021**: A regra do Assistir MUST valer no app nativo nos dois sentidos: ele não envia imagem nem som a quem não clicou em Assistir, e não recebe de ninguém antes do clique.
- **FR-022**: O app nativo MUST avisar quando existe versão nova e deixar atualizar sem procurar o instalador. Ele MUST avisar, em vez de falhar, quando a versão instalada é velha demais para a sala. A atualização MUST NOT interromper uma transmissão em andamento.
- **FR-023**: O app nativo MUST aparecer para o Windows como um programa só, com o nome e o ícone do Espia.
- **FR-024**: A lógica da sala e as telas do app nativo MUST ficar separadas, do mesmo jeito que no site, para que o design possa ser trocado sem tocar na lógica.

### Key Entities

- **Endereço oficial**: `espia.diuliano.com`. É o único que aparece em convites.
- **Endereço antigo**: `espia.espia.workers.dev`. Continua respondendo para links que já circulam e para os apps já instalados.
- **Convite**: link de sala que a pessoa copia e manda. Formado pelo endereço oficial e pelo código da sala.
- **Dados do aparelho**: Suas salas, chaves de dono, apelido e preferências (volume por pessoa, últimas escolhas de transmissão, aviso de baixar o app dispensado). Ficam só no aparelho. No navegador são separados por endereço.
- **App instalado hoje (app de antes)**: abre o site dentro de um motor de página, no endereço antigo, e deixa a página transmitir e receber.
- **App nativo**: resultado desta feature. Tem telas, sala, transmissão e recepção próprias, sem motor de página.
- **Site**: continua sendo o jeito de usar o Espia sem instalar nada, no computador e no celular.
- **Lista de paridade**: os itens do FR-012, usados para aceitar o app nativo.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% dos convites copiados ou mostrados no site publicado e no app usam o endereço oficial, em testes pelo endereço oficial, pelo antigo, pelo app já instalado e pelo app nativo.
- **SC-002**: 100% dos links antigos de sala testados levam à mesma sala, em no máximo dois passos.
- **SC-003**: Num aparelho com 5 salas fixas, 2 delas próprias, guardadas no endereço antigo, as 5 salas e as 2 ações de dono aparecem no endereço oficial na primeira visita pelo navegador e na primeira abertura do app nativo, sem nenhuma ação da pessoa.
- **SC-004**: Transmitindo 1080p60 para 3 pessoas pelo app nativo, a perda de FPS no jogo de referência não é maior que a do app instalado hoje, medida na mesma máquina e na mesma sessão.
- **SC-005**: Na mesma medição, o uso de processador do app ao transmitir cai pelo menos pela metade.
- **SC-006**: A memória do app nativo fica em no máximo 150 MB numa sala assistindo a uma transmissão 1080p, e em no máximo 40 MB na bandeja fora de sala.
- **SC-007**: 100% dos itens da lista de paridade são marcados como iguais ou melhores.
- **SC-008**: Em teste lado a lado em rede boa, o tempo entre clicar em Assistir e ver a imagem e o atraso da imagem no app nativo não ficam piores que os do navegador.
- **SC-009**: Em 30 minutos assistindo pelo app nativo, som e imagem não se afastam mais do que 80 milésimos de segundo.
- **SC-010**: Em 100% das telas comparadas lado a lado com o site, uma pessoa do grupo não aponta diferença de cor, fonte, texto ou posição de ação.
- **SC-011**: O app nativo mostra a tela inicial em até 2 segundos depois de aberto.
- **SC-012**: Em uma semana de uso do grupo, ninguém precisa voltar para o app de antes ou para o site por causa de travamento, imagem pior ou som faltando.
- **SC-013**: Baixar e instalar o app nativo por cima do app de hoje leva menos de 2 minutos numa internet de 10 Mbps.
- **SC-014**: Com uma versão nova publicada, uma pessoa atualiza o app nativo em até 3 cliques, sem abrir o site.

## Assumptions

- "O app" é o app desktop para Windows. O site continua como está, por decisão de 2026-11-11.
- O servidor continua como está. Ele só apresenta as pessoas umas às outras e guarda a sala; imagem e som nunca passam por ele. Reescrever o servidor não muda a qualidade nem o peso da transmissão.
- A parte nativa do app de hoje já é escrita em Rust (captura, som, bandeja, links). O que muda é que telas, sala, envio e recepção, que hoje acontecem numa página aberta pelo app, passam a ser do próprio app. O levantamento está em [custo-migracao.md](./custo-migracao.md).
- O Espia passa a ter duas interfaces: a do site e a do app nativo. Toda feature futura que valha para os dois é feita e testada nos dois.
- O app nativo deixa de receber as mudanças do site na hora. Por isso avisar e atualizar (FR-022) faz parte desta feature. A spec 015 é o ponto de partida e é revista para o app nativo.
- O instalador publicado hoje (0.1.10, montado em 2026-11-05) abre o endereço antigo. O código do app já aponta para o oficial, mas ainda não virou instalador.
- O endereço antigo continua existindo e respondendo. Esta feature não desliga o endereço antigo.
- O servidor é o mesmo nos dois endereços, então uma sala é a mesma sala em qualquer um deles.
- A transmissão continua direta entre as pessoas, só com conexão direta, como ficou na 017. Retransmissão e servidor de mídia continuam fora.
- O app continua só para Windows. No celular e nos outros sistemas, o Espia continua sendo o site.
- As metas SC-004 e SC-005 comparam com uma medição nova do app instalado hoje, feita na mesma máquina antes da troca. Os números antigos (FPS do jogo de 200 para 160 a 180 com o app transmitindo, 27 a 30% de processador da página antes da 010) servem só de ordem de grandeza.
- As metas de memória do SC-006 vêm das metas já usadas no projeto (150 MB na 003, 40 MB na 009). O levantamento achou ferramentas de interface nativa acima e abaixo disso, então a escolha da ferramenta no plano é feita contra essa meta.
- O instalador do app nativo fica maior que o de hoje (1,7 MB). O limite é o tempo do SC-013, não um tamanho fixo.
- A entrega é em etapas: primeiro o convite e o endereço (histórias 1 e 2 no site), que não dependem do app nativo. O app nativo só substitui o de hoje para o grupo quando as histórias 3, 4 e 5 passam.
