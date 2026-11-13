# Quickstart: validar o app nativo e o convite no endereço oficial (019)

Referências: [spec.md](./spec.md) · [data-model.md](./data-model.md) · [contracts/convite-e-endereco.md](./contracts/convite-e-endereco.md) · [contracts/app-nativo.md](./contracts/app-nativo.md) · [contracts/sinalizacao-nativa.md](./contracts/sinalizacao-nativa.md) · [contracts/atualizacao.md](./contracts/atualizacao.md) · [contracts/dados-falsos.md](./contracts/dados-falsos.md)

Este guia diz como validar, não como construir. Anote todo resultado em `specs/019-app-rust-convite/relatorio.md`, com data (AAAA-MM-DD), máquina e versão testada.

## 1. Antes de começar

### Pré-requisitos

- Node e as dependências do site instaladas (`npm install`), como nas features anteriores.
- Toolchain do Rust (Rust 1.90, edição 2024) para compilar `app/`.
- Windows 10 ou 11, x64, para o app nativo.
- Um segundo aparelho com Chrome, para ser o "amigo no navegador".
- Um iPhone com Safari, para o item (h) do G1 e para a sala mista.
- O jogo de referência e a máquina das medições anteriores (a mesma do T008). Sem a mesma máquina e o mesmo jogo, SC-004 e SC-005 não valem.
- O app de antes (0.1.10) instalado, para as comparações lado a lado e para a sala mista.

### Rodar o site localmente

```powershell
npm run dev:worker     # vite build + wrangler dev: site e servidor locais
npm run dev            # só o site, para abrir os mocks
```

Para testar com um celular na mesma rede, `npm run dev:lan` (como na 018).

### Abrir o app

Os comandos abaixo são criados pelas tarefas T003 (scripts) e T006 (`-Nativo`); antes delas não existem.

| Modo | Comando | Para quê |
|------|---------|----------|
| Mock | `npm run app:mock` (índice) ou `Espia.exe --mock <tela> --scene <cena>` | Ver tela e estado sem servidor, câmera, captura ou rede. Telas: `home`, `join`, `room`, `notfound`, `offline`. Ver [dados-falsos](./contracts/dados-falsos.md). |
| Real, contra o site local | `npm run app:dev`, com `npm run dev:worker` rodando | Entrar em sala do site local. Com `ESPIA_SERVER=http://127.0.0.1:8787 npm run app:dev` (e `npm run dev:worker` rodando), o app usa o servidor local e manda `Origin: http://127.0.0.1:8787`. A variável só vale em build de depuração; release usa sempre `https://espia.diuliano.com`. Ver [sinalizacao-nativa](./contracts/sinalizacao-nativa.md). |
| Gates | `npm run app:check` e `npm run app:test` | `cargo check` e `cargo test` em `app/`. |
| Instalador | `npm run app:release` (T094) | Montar, assinar e escrever `latest.json`. |
| Memória | `npm run memoria -- -Nativo` (modo criado pelo T006) | Medir um processo só. |

### Regra de segurança

Teste automático nunca captura a tela de verdade. Captura real (tela, janela, câmera, som) só é testada à mão, por uma pessoa, com a janela de teste escolhida por ela. O que o teste automático cobre: lógica pura, fixtures do protocolo, mocks e cenas de dados falsos.

### Gates automáticos (rodar antes de qualquer roteiro)

```powershell
npm run check
npm test
npm run build
npm run app:check      # T003
npm run app:test       # T003
```

**Esperado**: tudo verde. Inclui `test/lib/site-origin.test.ts`, `test/lib/origin-move.test.ts`, `test/shared/protocol-fixtures.test.ts` e o teste de paridade de cenas do app (T033).

## 2. Gates

Regra: **gate vermelho para a trilha**. Se um gate falhar, parar, registrar em `relatorio.md` e replanejar as fases afetadas com o plano B da decisão. Não seguir adiante com gate vermelho. Resultado de cada gate vai para `relatorio.md`, uma linha por item.

### G0, linha de base (T008)

No app de hoje (0.1.10), na máquina e no jogo de referência:

1. Medir o FPS do jogo sem transmitir.
2. Transmitir 1080p60 para 3 pessoas no navegador e medir o FPS do jogo.
3. Anotar o uso de processador do app durante essa transmissão.
4. Medir a memória do app numa sala e na bandeja fora de sala (`npm run memoria`).
5. Anotar qual codificador o motor usa (Diagnóstico de mídia do site).

**Passa**: os cinco números estão registrados. É a base de SC-004, SC-005 e SC-006.

### G1, ligação (T009, T010; `app/spikes/webrtc-probe/`)

1. Compilar com `cargo build --release` no Windows. (a)
2. Entrar numa sala do site local e conferir que o Chrome recebe vídeo 1080p60 H.264 e som. (b)
3. Conferir que o app recebe vídeo e som do Chrome. (c)
4. Conferir a troca de `ViewMessage` com o site pelo canal `ctl`. (d)
5. Mudar por ligação bitrate, fps, redução de resolução e ativo/inativo, e ver o efeito no Chrome. (e)
6. Conferir que a placa de vídeo codifica, ou que o processador do processo fica em até metade do G0. (f)
7. Deixar 10 minutos e conferir que som e imagem não se afastam mais de 80 ms. (g)
8. Conferir que o Safari no iPhone recebe. (h)
9. Anotar a memória do processo e o tamanho do executável. (i)
10. Anotar se o Windows mostrou aviso de firewall. (j)

**Passa**: (a) a (h) cumpridos; (i) e (j) anotados (em (i), memória do processo e tamanho do executável). Falhou: plano B (versão modificada com codificador do Windows) ou plano C (`str0m`), conforme R2, e replanejar as fases de mídia.

### G2, interface (T011, T012; `app/spikes/ui-probe/`)

1. Abrir a tela da sala com dados falsos e comparar lado a lado com o site.
2. Mostrar um vídeo 1080p60 e duas miniaturas por alguns minutos.
3. Medir a memória privada do processo com o motor do G1 carregado.

**Passa**: tela igual ao site na comparação; sem engasgo; memória em até 150 MB. Falhou: plano B, `iced` (R3).

### G3, dados do app de antes (T013, T014; `app/spikes/webview-data/`)

1. Fazer uma cópia do perfil real do app instalado (`%LOCALAPPDATA%\dev.espia.app\EBWebView\`), com o app fechado.
2. Rodar a prova sobre a cópia, nunca sobre a pasta original.
3. Conferir que ela lê as salas fixas, as chaves de dono e o apelido.

**Passa**: as três coisas lidas da cópia do perfil real. Falhou: plano B, exportação manual pelo site (R10), e replanejar.

## 3. Histórias

### História 1: convite sempre com o endereço oficial (MVP, só site)

**Precisa de**: site local (`npm run dev:worker`), os mocks, o site publicado, o app de antes instalado. Ver [convite-e-endereco](./contracts/convite-e-endereco.md).

**Passos**

1. Rodar `npm test` e conferir a tabela de hosts de `test/lib/site-origin.test.ts`: host oficial e host antigo devolvem `https://espia.diuliano.com`; qualquer outro host (localhost, LAN) devolve a origem da própria página. A regra é por nome do host, então o teste de unidade é a prova local do endereço antigo.
2. Abrir o mock da sala (`/?mock=room`, cena do convite) e conferir o texto mostrado.
3. Com o site local, criar uma sala, copiar o convite e colar num editor.
4. Criar uma sala fixa e colar o que foi copiado sozinho.
5. Depois do deploy (`npm run deploy`, feito pela pessoa): repetir 3 e 4 pelo endereço oficial e pelo endereço antigo publicado.
6. Abrir o app de antes instalado, entrar numa sala e copiar o convite.
7. Abrir um link com o endereço antigo (`https://espia.espia.workers.dev/s/<código>`) e um com o oficial para a mesma sala.

**Esperado**

- Convite copiado e mostrado começa com `https://espia.diuliano.com` pelo endereço oficial, no site e no app (US1-1, SC-001).
- Pelo endereço antigo publicado, o convite também usa o oficial (US1-2, SC-001).
- No app de antes instalado, sem instalar nada, o convite usa o oficial, porque o app carrega o site (US1-3, SC-001).
- O link antigo cai na mesma sala que o oficial, com no máximo apelido e entrar (US1-4, SC-002). Testar 100% dos links antigos disponíveis.
- O convite copiado sozinho ao criar sala fixa usa o oficial (US1-5, SC-001).
- Caso de borda: em `localhost` ou na LAN, o convite usa o endereço da própria página (FR-007).

### História 2: chegar pelo endereço antigo sem perder nada

**Precisa de**: um navegador com dados no endereço antigo (5 salas fixas, 2 próprias, apelido, volume, escolhas de transmissão), uma cópia do perfil real do app de antes, o app nativo instalado por cima (depois de T084 e T085). Ver [convite-e-endereco](./contracts/convite-e-endereco.md) para o `#mv=` e as regras de junção, e [app-nativo](./contracts/app-nativo.md) para a importação.

**Parte do site (T080 a T083)**

1. Rodar `npm test` e conferir `test/lib/origin-move.test.ts`: empacotar, desempacotar, juntar, validar, teto de 16.384 caracteres com a ordem de descarte, e idempotência. Localmente a regra de ida só vale no host antigo, então a prova antes do deploy é o teste de unidade e os mocks (com `?mock` o site não é levado a lugar nenhum).
2. Depois do deploy, no navegador com os dados no endereço antigo, abrir `https://espia.espia.workers.dev/`.
3. Conferir o endereço final, a lista Suas salas, as ações de dono (renomear e apagar nas 2 salas próprias), o apelido e o volume.
4. Abrir um link de sala com o endereço antigo.
5. Já com dados nos dois endereços, repetir o passo 2.
6. Abrir de novo o endereço antigo, depois de já ter sido levado uma vez.
7. Abrir o endereço antigo em janela anônima ou com dados bloqueados.
8. Estando numa sala rápida aberta pelo endereço antigo, recarregar a página.
9. Abrir o app de antes instalado.

**Esperado (site)**

- Chega ao endereço oficial com salas, chaves de dono, apelido e preferências, sem digitar nem confirmar nada (US2-1, SC-003).
- Link de sala antigo leva à entrada da mesma sala no oficial, com no máximo apelido e entrar (US2-2, SC-002).
- Com listas nos dois endereços: união sem sala repetida, ordenada pelo uso mais recente, cortada em 5, nenhuma chave de dono perdida, mesmo de sala fora da lista (US2-3, FR-004).
- Repetir não duplica nem apaga (US2-6).
- O `#mv=` some da barra de endereço depois da importação e não aparece em nenhum registro do servidor (FR-005).
- Navegador que bloqueia o que a página guarda: chega ao oficial e entra na sala, só sem o que carregar (borda).
- A sala rápida em andamento volta como numa queda (borda).
- O app de antes continua abrindo e entrando em salas, sem ser levado ao endereço oficial (US2-4, FR-006). O servidor não redireciona nada.

**Parte do app (T084 a T086)**

1. Com o app de antes fechado e com os dados do passo anterior, instalar o app nativo por cima (ver história 7) e abrir.
2. Conferir Suas salas, as ações de dono, o apelido, o volume, a preferência de fechar para a bandeja e a posição da janela.
3. Copiar um convite.

**Esperado (app)**

- Na primeira abertura, as 5 salas e as 2 ações de dono aparecem, sem ação da pessoa (US2-5, SC-003); `importedFromWebview: true` no `device.json`.
- A pasta do motor de página não foi apagada, para o app de antes continuar funcionando se a pessoa voltar.
- Convite com o endereço oficial (US2-5, SC-001).

### História 3: transmitir pelo app nativo

**Precisa de**: gates G0 e G1 verdes, o app nativo, o jogo e a máquina de referência, 3 pessoas ou aparelhos no navegador (Chrome no computador, Chrome no celular, Safari no iPhone). Captura real à mão. Ver [sinalizacao-nativa](./contracts/sinalizacao-nativa.md).

**Passos**

1. Abrir o app nativo, criar uma sala rápida, clicar em Transmitir.
2. Conferir o seletor: telas e janelas com miniaturas, selo "tocando", qualidade, opções de som.
3. Escolher a janela do jogo, 1080p60, com som da janela, e começar.
4. Nos três navegadores, entrar e clicar em Assistir.
5. Com uma pessoa na sala que não clicou em Assistir, conferir que nada chega a ela.
6. **Medição lado a lado (SC-004 e SC-005)**: na mesma máquina e na mesma sessão, com o jogo de referência, transmitir 1080p60 para 3 pessoas primeiro com o app de antes e depois com o app nativo, e medir em cada um o FPS do jogo e o uso de processador do app. Repetir cada um pelo menos 3 vezes e usar a mediana. Comparar com o G0.
7. Piorar a rede de um espectador (limitar a banda no navegador dele) e voltar ao normal.
8. Trocar a qualidade ao vivo, trocar de janela, parar e transmitir de novo.
9. Fechar a janela para a bandeja, transmitindo, e parar pela bandeja.
10. Numa máquina sem placa de vídeo capaz (ou forçando a falha da codificação pela placa, se a tarefa T056 permitir), clicar em Transmitir.
11. Escolher uma transmissão com o nome do programa e conferir o rótulo.

**Esperado**

- A transmissão começa e aparece para todos na sala (US3-1).
- Quem está no navegador, no computador ou no celular, clica em Assistir e vê imagem e som sem passo a mais (US3-2, FR-011).
- A perda de FPS no jogo não é maior que a do app de antes (US3-3, SC-004).
- O processador do app ao transmitir cai pelo menos pela metade em relação ao app de antes (US3-3, SC-005).
- Placa que não ajuda: a transmissão começa em qualidade menor e o app diz isso em uma frase; nunca tela preta sem explicação (US3-4, FR-013).
- Rede ruim de um espectador: a imagem dele se ajusta e se recupera sozinha, sem travar mais que hoje, e a das outras pessoas não cai (US3-5, FR-017; bordas de rede pior e de entrada no meio).
- Quem não clicou em Assistir não recebe imagem nem som (US3-6, FR-021).
- Troca de qualidade, parar e bandeja se comportam como hoje: sem processo sobrando, som duplicado nem imagem congelada em duas transmissões seguidas; escondido na bandeja continua transmitindo (US3-7, bordas).
- O rótulo mostra o nome do programa, nunca o título da janela.
- Primeira transmissão: se o Windows mostrar aviso de rede, anotar (G1 j). Negar a permissão resulta em aviso claro, não em sala muda (borda).
- Anotar em `relatorio.md`: FPS sem transmitir, FPS com o app de antes, FPS com o app nativo, processador nos dois, razão entre eles.

### História 4: assistir pelo app nativo

**Precisa de**: o app nativo, uma transmissão 1080p60 vinda do navegador (Chrome) e outra vinda do app de antes, rede boa, 30 minutos livres. Ver [sinalizacao-nativa](./contracts/sinalizacao-nativa.md).

**Passos**

1. Entrar numa sala com transmissões, sem clicar em Assistir, e conferir o tráfego.
2. Clicar em Assistir numa transmissão.
3. **Lado a lado (SC-008)**: no mesmo computador, abrir a mesma transmissão no Chrome e no app nativo. Medir, nos dois, o tempo entre clicar em Assistir e ver a imagem, e o atraso da imagem (por exemplo, com um cronômetro na tela de quem transmite, filmando as duas janelas juntas). Repetir 5 vezes.
4. **Sincronia (SC-009)**: assistir 30 minutos a uma transmissão com som e imagem com batida visível (vídeo de teste com palmas). Medir o desvio no início, aos 10, 20 e 30 minutos.
5. Com duas transmissões na sala, assistir às duas, ajustar o volume de cada uma e destacar qualquer uma.
6. Cortar a rede do app por cerca de 10 s e religar.
7. Usar tela cheia, janela flutuante e Parar de assistir.
8. Com a janela minimizada ou escondida, assistindo, conferir o som e depois restaurar.

**Esperado**

- Antes do Assistir, nenhuma imagem nem som chega (US4-6, FR-021).
- Depois do Assistir, imagem e som com atraso e fluidez não piores que os do Chrome na mesma rede (US4-1, SC-008).
- Em 30 minutos, som e imagem não se afastam mais de 80 ms (US4-2, SC-009).
- Cada transmissão tem seu volume e pode ser destacada (US4-3).
- Na queda, a imagem se recupera sozinha e a pessoa volta à sala assistindo ao mesmo, sem refazer nada (US4-4).
- Tela cheia, janela flutuante e Parar de assistir funcionam como hoje (US4-5).
- Escondida ou minimizada, para de desenhar e pede `hidden`; o som continua (T067).
- Em rede ruim, a imagem baixa de qualidade e se recupera; o iPhone ou navegador com limite recebe uma imagem que consegue mostrar, mesmo menor (bordas).
- Dez pessoas e vários espectadores: a qualidade continua baixando conforme o número de espectadores (borda).

### História 5: o mesmo Espia, sem perder função

**Precisa de**: o app nativo, o site aberto no Chrome, o app de antes instalado em outra conta ou máquina (só um app fica instalado por computador; usar outro computador para a sala mista), e [checklists/paridade.md](./checklists/paridade.md) (criado pela T077, com um item por linha do FR-012).

**Passos**

1. Percorrer `checklists/paridade.md` no app nativo, item por item, marcando "igual", "melhor" ou "pior" em relação ao app de antes. Anotar o resultado em `relatorio.md` (T079).
2. **Comparação lado a lado (SC-010)**: abrir `npm run app:mock` e o `/?mock=...` do site, lado a lado, para cada tela e cena (home, join, room, notfound, sem conexão, atualização). Uma pessoa do grupo aponta qualquer diferença de cor, fonte, texto, ícone, castor ou posição de ação.
3. Abrir o app no modo de dados falsos para cada tela e estado, sem servidor, câmera, captura ou rede, e rodar o teste de paridade de cenas (`npm run app:test`).
4. Conferir o Gerenciador de Tarefas e a tela durante a transmissão: nenhuma barra "está compartilhando".
5. Testar com monitor de 125%, 150% e 200% de escala e com mais de um monitor.
6. Testar nomes, apelidos e nomes de sala com acento, emoji e muito longos.
7. Abrir um link `espia://s/<código>` e um link oficial colado, com o app aberto e fechado; abrir uma segunda instância.
8. **Sala mista (FR-014)**: uma sala com o site, o app de antes e o app nativo. Para cada combinação (cada um transmitindo para cada outro, e todos assistindo a todos), conferir imagem e som.
9. Usar só o site, sem instalar nada, depois que o app nativo estiver publicado.

**Esperado**

- Todo item da lista de paridade é igual ou melhor (US5-1, SC-007: 100%).
- Em 100% das telas comparadas, ninguém do grupo aponta diferença (US5-2, SC-010, FR-019).
- Todos se veem e se ouvem na sala mista, em qualquer combinação (US5-3, FR-014).
- Cada tela e estado abre com dados falsos e sem rede, câmera ou captura (US5-4, FR-018).
- O site funciona como antes, sem instalar nada (US5-5, FR-011).
- Escala e vários monitores: telas nítidas e do tamanho certo (borda).
- Textos longos ou com emoji aparecem como no site (borda).
- Link `espia://` e link colado abrem a sala no app; com o app aberto, a instância existente recebe o link e uma segunda janela não abre; o app de antes e o nativo não ficam instalados juntos (bordas).
- Trocar de sala transmitindo pede confirmação, como hoje.

### História 6: app leve

**Precisa de**: o app nativo, `npm run memoria -- -Nativo` (modo criado pelo T006), a linha de base do G0, a máquina de referência. Ver [app-nativo](./contracts/app-nativo.md).

**Passos**

1. Numa sala assistindo a uma transmissão 1080p, esperar 2 minutos e medir a memória.
2. Numa sala transmitindo, medir a memória após 10 minutos.
3. Numa sala, parado, sem transmitir nem assistir, medir.
4. Fora de sala e escondido na bandeja, esperar 1 minuto e medir.
5. Abrir o Gerenciador de Tarefas e procurar o Espia, com a janela aberta e escondida.
6. Com o app fechado, abrir e cronometrar até a tela inicial aparecer (5 vezes, a primeira a frio).
7. Medir o app de antes nas mesmas quatro situações, na mesma máquina, para a comparação.

**Esperado**

- Assistindo 1080p: no máximo 150 MB e abaixo do app de antes (US6-1, SC-006).
- Na bandeja fora de sala: no máximo 40 MB e não maior que o de hoje (US6-2, SC-006).
- O Gerenciador mostra o Espia com nome e ícone, um processo só, sem grupo de processos de navegador (US6-3, FR-023).
- A tela inicial aparece em até 2 segundos depois de aberto (US6-4, SC-011).
- Se passar das metas: perfil de memória e correções (T091).

### História 7: trocar de versão sem susto

**Precisa de**: um par de chaves de atualização gerado pela pessoa (T092; privada em `%USERPROFILE%\.espia\update.key`, fora do repositório), o instalador do app nativo 0.2.0 e uma versão de teste mais nova, o app de antes (0.1.10) instalado, internet de 10 Mbps para o SC-013. Ver [atualizacao](./contracts/atualizacao.md).

**Passos**

1. **Instalar por cima**: com o app de antes instalado e com dados, baixar o instalador do site e instalar o app nativo por cima. Cronometrar do clique em baixar até o app aberto.
2. Conferir os dados do aparelho, a posição da janela e a preferência de fechar para a bandeja.
3. **Atualizar de uma versão de teste**: instalar 0.2.0, publicar localmente uma versão mais nova com `npm run app:release` (T094) e um `latest.json` de teste, abrir o app e contar os cliques até estar atualizado.
4. Repetir com `minimum` acima da versão instalada.
5. Entrar numa sala com uma versão velha demais para ela (ou simular o fechamento 4426 do servidor).
6. Com o app transmitindo, publicar a versão nova.
7. Com a atualização feita em sala, conferir se volta à sala.
8. Testar assinatura inválida no `latest.json` de teste.
9. **Voltar para o app de antes**: instalar `Espia-setup-0.1.10.exe` por cima do nativo, ou abrir o site no navegador, e transmitir e assistir.
10. Conferir que o app de antes instalado hoje continua transmitindo e assistindo com o site já publicado.

**Esperado**

- A instalação por cima leva menos de 2 minutos em 10 Mbps e mantém os dados do aparelho, a posição da janela e a preferência da bandeja (SC-013, FR-015, US2-5).
- A versão nova é avisada com "Atualizar agora" e "Depois"; a pessoa atualiza em até 3 cliques, sem abrir o site, e volta para onde estava (US7-1, SC-014, FR-022).
- "Depois" fecha o aviso até a próxima abertura do app, e na abertura seguinte o aviso volta.
- Abaixo de `minimum`, o aviso não tem "Depois".
- Versão velha demais para a sala: uma frase, "Atualize o Espia", com o botão de atualizar, em vez de falha em silêncio (US7-2).
- Assinatura inválida: não instala, e o app diz que não foi possível atualizar.
- Transmitindo: nada é interrompido e a atualização só começa quando a pessoa decidir (US7-5, FR-022).
- Voltar ao app de antes ou ao site: transmite e assiste, com as salas e preferências (US7-3).
- O app de antes continua funcionando com o site publicado (US7-4).
- Só um app fica instalado e só uma janela abre; `espia://` abre o que estiver instalado (borda).

## 4. Tabela de critérios

| SC | Bloco que prova | Como se mede |
|----|-----------------|--------------|
| SC-001 | História 1 (e passo final da história 2, app) | Texto copiado e mostrado em oficial, antigo publicado, app de antes e app nativo; 100% com `espia.diuliano.com`. |
| SC-002 | Histórias 1 e 2 (site) | Links antigos testados abrem a mesma sala em no máximo 2 passos. |
| SC-003 | História 2 | Aparelho com 5 salas fixas (2 próprias): 5 salas e 2 ações de dono visíveis na primeira visita pelo navegador e na primeira abertura do app nativo. |
| SC-004 | História 3, passo 6 | FPS do jogo de referência no app nativo contra o app de antes, mesma máquina e sessão, mediana de 3 repetições. |
| SC-005 | História 3, passo 6 | Processador do app na mesma medição; razão nativo / de antes de no máximo 0,5. |
| SC-006 | História 6 | `npm run memoria -- -Nativo`: até 150 MB assistindo 1080p; até 40 MB na bandeja fora de sala. |
| SC-007 | História 5, passo 1 | Lista `checklists/paridade.md`: 100% igual ou melhor. |
| SC-008 | História 4, passo 3 | Tempo até a imagem e atraso, app nativo contra Chrome, 5 repetições, mesma rede. |
| SC-009 | História 4, passo 4 | Desvio entre som e imagem em 30 minutos, no máximo 80 ms. |
| SC-010 | História 5, passo 2 | Comparação lado a lado de cada tela e cena; ninguém do grupo aponta diferença. |
| SC-011 | História 6, passo 6 | Cronômetro da abertura até a tela inicial, até 2 s. |
| SC-012 | Uma semana de uso do grupo (T108) | Ninguém volta ao app de antes ou ao site por travamento, imagem pior ou som faltando; registrar cada caso em `relatorio.md`. |
| SC-013 | História 7, passo 1 | Cronômetro de baixar e instalar por cima, em 10 Mbps; menos de 2 minutos. |
| SC-014 | História 7, passo 3 | Cliques da pessoa até estar atualizada, no máximo 3, sem abrir o site. |

## 5. Ordem de publicação

1. **Site primeiro** (história 1 e a parte de site da história 2, T039 a T044 e T080 a T083). A publicação é feita pela pessoa com `npm run deploy`, depois de `npm run check`, `npm test` e `npm run build` verdes. Conferir pelo endereço oficial, pelo antigo e pelo app de antes instalado. Nenhuma das duas ordens de publicação quebra quem está numa sala.
2. **App nativo depois**. Ele só substitui o download quando as histórias 3, 4 e 5 passam (T058, T068, T079) e depois de uma semana de uso do grupo sem voltar ao app de antes (SC-012, T108).
3. **Na troca** (T109): `public/download/Espia-setup.exe` passa a ser o nativo; o app de antes continua baixável como `/download/Espia-setup-0.1.10.exe`. Enquanto isso, o instalador publicado continua sendo o de 2026-11-05.
4. Não publicar instalador novo do app de antes. A remoção de `src-tauri/` fica para uma feature futura (T110).
