# Quanto custa migrar o app para Rust

**Feature**: [spec.md](./spec.md) · **Levantado em**: 2026-11-11 · **Estado**: estimativa, antes do plano

Este arquivo responde "o que gastaríamos para migrar". Ele alimenta a escolha de alcance da spec (FR-010) e o `research.md` do plano.

Como foi levantado: três varreduras do repositório contaram o código e acharam onde o convite é montado. Duas pesquisas na web olharam o estado das bibliotecas de vídeo e de interface em Rust. Duas revisões tentaram derrubar a estimativa, uma contra o código e outra contra a viabilidade técnica. As correções delas já estão aqui. O que veio de fora e não foi confirmado está marcado.

## Resumo

- **O app já é Rust na parte nativa.** São 3.252 linhas: captura de tela, som por janela, miniaturas, bandeja, memória, links. O que não é Rust é a tela e a ligação de vídeo, porque o app abre o site publicado dentro do motor de página do Windows.
- **"Migrar para Rust" então significa tirar trabalho de dentro da página.** Dá para tirar só o envio da transmissão, ou tirar a página inteira.
- **O convite sai com o endereço antigo porque o app instalado ainda abre o endereço antigo.** O instalador 0.1.10 publicado foi montado em 2026-11-05, dois dias antes da troca de endereço no código. Consertar isso custa pouco e não depende da migração.
- **Dinheiro**: nenhuma opção exige licença paga nem muda a conta da Cloudflare. O gasto é trabalho, teste com o grupo e manutenção depois.

| Opção | O que muda | Código novo (estimado) | Prazo no ritmo do projeto | Risco | O que a pessoa ganha |
|---|---|---|---|---|---|
| Só o link | Convite sempre em `espia.diuliano.com` | 100 a 200 linhas no site, mais um instalador novo | 1 dia | Baixo | Convite com o endereço certo |
| A. Envio nativo | O app codifica e envia a transmissão sem passar pela página. A tela continua sendo o site | 4 a 6 mil linhas de Rust, mais 800 a 1.500 no site | 2 a 4 semanas, com teste nas placas de vídeo do grupo | Alto | Talvez menos peso no jogo ao transmitir. Memória não muda |
| B. App inteiro nativo | Sai o motor de página. Tela, sala, envio e recepção em Rust | 14 a 22 mil linhas de Rust | 2 a 5 meses, com estabilização | Alto | O mesmo da A, e memória talvez menor |
| C. Tudo em Rust | B, mais o site e o servidor reescritos | B, mais cerca de 11 mil linhas refeitas | B, mais 1 a 2 meses | Alto | Nada além da B |

O ritmo de referência é o do próprio projeto: 18 features e 43 commits entre 2026-10-28 e 2026-11-11. O prazo das opções A e B é dominado por teste manual em várias máquinas, que não acelera com mais código pronto.

**Recomendação do levantamento**: fazer o link agora. Não fazer B nem C. Fazer a A só depois de uma medição curta mostrar que o ganho existe (ver "Antes de decidir pela A").

**Decisão (2026-11-11)**: opção B, o app inteiro nativo. O site continua como está. O servidor também continua: ele só apresenta as pessoas umas às outras, e imagem e som vão direto de uma pessoa para a outra, então reescrevê-lo não muda o envio. A spec foi reescrita para esse alcance.

## O que existe hoje

| Parte | Linguagem | Tamanho | Observação |
|---|---|---|---|
| Parte nativa do app (`src-tauri/src`) | Rust | 3.252 linhas, 14 arquivos | Captura (`capture/`, 2.286 linhas), bandeja, memória, links, barra de compartilhamento |
| Lógica da sala (`src/lib`) | TypeScript | 4.629 linhas, 30 arquivos | Só o `stores/room.svelte.ts` tem 1.395 |
| Telas (`src/ui` e `src/routes`) | Svelte | 4.399 linhas, 36 arquivos | 27 componentes, 5 páginas |
| Telas com dados falsos (`src/mocks`) | Svelte e TypeScript | 1.224 linhas | 5 telas, cerca de 100 cenas |
| Protocolo compartilhado (`src/shared`) | TypeScript | 204 linhas | 6 mensagens do cliente, 15 do servidor |
| Servidor (`src/worker`) | TypeScript | 1.068 linhas | Worker e uma sala por Durable Object |
| Testes | TypeScript | 2.931 linhas, 268 casos | Lógica, servidor e roteiro de aceite |

Fatos que pesam na conta:

- **O app pega as mudanças do site na hora.** Como a tela é o site publicado, uma publicação do site atualiza o app de todo mundo. O app não se atualiza sozinho: a feature 015 tem 0 de 36 tarefas feitas. Cada instalador novo depende de cada pessoa baixar de novo.
- **A ligação de vídeo inteira vem do motor de página**: controle de banda, reenvio de pacotes perdidos, fila de recepção, sincronia de som e imagem, codificação pela placa de vídeo. Nada disso é código do projeto hoje.
- **Caminho de um quadro hoje**: o Rust captura, converte na placa de vídeo, copia para a memória e entrega à página. A página monta o quadro e o navegador codifica e envia. O som segue o mesmo caminho.
- **Há uma ligação por par de pessoas**, usada nos dois sentidos, e o navegador codifica uma vez por espectador. Cada espectador recebe conforme o que está fazendo: imagem cheia, miniatura (360p, 15 fps) ou nada (antes de clicar em Assistir, ou com a página escondida).
- **Só conexão direta.** A retransmissão está desligada no servidor e no site desde a 017.
- **O instalador tem 1,7 MB** porque o motor de página já vem no Windows.

Medidas anteriores, e o quanto valem:

| Medida | Valor | De onde vem | Cuidado |
|---|---|---|---|
| Memória numa sala | cerca de 228 MB | `specs/009-memoria-do-app/relatorio.md` | Minimizado, 2 min depois de parar de transmitir |
| Memória na bandeja, fora de sala | cerca de 30 MB | `specs/009-memoria-do-app/relatorio.md` | — |
| Processador da página em 1080p60 para 3 pessoas | 27 a 30% | `specs/010-transmissao-leve/spec.md` | Medido na 0.1.6, antes das melhorias da 010. Não há medição depois |
| FPS do jogo transmitindo pelo app | de 200 para 160 a 180 | Relato do grupo em 2026-10-29, fora do repositório | Um jogo, uma máquina, depois da troca para codificação pela placa de vídeo |

Nenhuma dessas medidas separa quanto do peso vem da captura e quanto vem da página. Essa é a pergunta que decide a opção A.

## Só o link de convite

O convite é montado em um lugar só, com o endereço em que a página foi aberta (`src/lib/stores/room.svelte.ts:518`). O botão Copiar convite, o Convidar do celular e a cópia automática ao criar sala fixa usam esse mesmo valor.

Por que sai o endereço antigo:

- O código do app já aponta para `espia.diuliano.com` desde 2026-11-07.
- O instalador 0.1.10 que está para baixar foi montado em 2026-11-05. O executável dele contém `espia.espia.workers.dev` e não contém `espia.diuliano.com`.
- Então todo app instalado hoje abre o endereço antigo e copia convite com ele. O mesmo vale para quem abre o site pelo endereço antigo no navegador.

O que entra:

- O convite passa a usar sempre o endereço oficial. Como é mudança no site, vale na hora para os apps já instalados.
- Um instalador novo que abre o endereço oficial.
- Suas salas, chaves de dono, apelido, volumes e escolhas de transmissão ficam guardados por endereço, no navegador e dentro do app. Sem cuidado, a pessoa chega ao endereço novo com a lista vazia e sem poder apagar as salas que criou. Isso vale para o navegador e para o app novo instalado por cima do antigo. A troca precisa levar esses dados junto.
- Apps já instalados só aceitam o endereço antigo. Se o endereço antigo passar a redirecionar tudo, eles ficam presos na tela de carregamento e abrem uma aba do navegador. O endereço antigo precisa continuar servindo esses apps.
- O servidor não liga para o endereço: a mesma sala é alcançada pelos dois. Links antigos colados no site já funcionam.
- Textos que ainda citam o endereço antigo como endereço do site: constituição (linha 70) e contratos das specs 003, 005, 007, 011 e 015.

Custo: 100 a 200 linhas no site, mais testes, uma publicação do site e um instalador novo.

## A. Envio nativo (a tela continua sendo o site)

O app codifica com a placa de vídeo e envia direto aos espectadores. A página fica com a sala, os botões e a recepção do que os outros transmitem.

**Ganho possível**

- Uma codificação para todos os espectadores, em vez de uma por espectador.
- Os quadros deixam de ser copiados para a página 60 vezes por segundo.
- Quem assiste pelo navegador não muda nada.

**Por que o ganho é incerto**

- A captura, a conversão e a codificação continuam disputando a placa de vídeo com o jogo. O envio nativo não tira isso.
- Se o navegador já estiver codificando pela placa de vídeo, cada espectador a mais custa pouco. O ganho só é grande se ele estiver caindo para codificação por software.
- A memória não muda. O motor de página continua carregado, perto dos 228 MB em sala.

**O que precisa ser feito**

- **Codificação H.264 pela placa de vídeo.** Não foi achada biblioteca Rust mantida para o caminho do Windows. O comportamento muda por fabricante: há relato de placa NVIDIA sem o codificador registrado, e os controles de qualidade e de quadro-chave se comportam diferente em AMD e Intel (não confirmado nesta pesquisa). Exige teste em NVIDIA, AMD e Intel e volta automática ao caminho atual. De 800 a 1.500 linhas por um caminho só, ou cerca de 3.000 com um caminho por fabricante.
- **Envio pela rede em Rust.** A biblioteca candidata é a `str0m` 0.25 (2026-11-03). Ela tem controle de banda e reenvio. Ela não descobre o endereço público sozinha, não tem retransmissão e foi mais testada em servidor do que entre duas pessoas. O laço de rede fica por nossa conta.
- **iPhone.** Há registro de falha de negociação de H.264 entre a `str0m` e o Safari em níveis altos. O nível seguro limita a imagem a perto de 720p30.
- **Regra do Assistir e miniaturas.** Hoje cada espectador recebe cheio, miniatura ou nada. Com uma codificação só, isso vira trabalho novo: parar e retomar o envio por espectador, e decidir se a miniatura recebe a imagem cheia ou ganha uma segunda codificação.
- **Quadro-chave para todos.** Quando um espectador entra ou perde pacotes, todos recebem um quadro grande. Precisa de limite.
- **Som.** Codificado em Rust (Opus) e amarrado ao relógio do vídeo, para o som não descolar da imagem.
- **Ajuste automático de qualidade.** Hoje ele lê os números do navegador. Passa a precisar de um sinal vindo do app.
- **Site.** A ligação de envio passa a ser outra, separada da ligação por par. O protocolo, a malha e a sala (o arquivo mais complicado do projeto) precisam aceitar isso. O servidor rejeita campos extras nas mensagens de negociação, então também muda. Clientes antigos precisam continuar funcionando.
- **Firewall do Windows.** Uma conexão de rede nova feita pelo próprio app pode abrir o aviso do firewall na primeira vez (não confirmado).
- **Compatibilidade de versões.** O site se atualiza sozinho e o app não. Site e app precisam continuar se entendendo entre versões.
- **Emenda da constituição**: a ponte do app ganha comandos novos.

**O que se perde**

- Qualidade ajustada por espectador conforme a rede de cada um. Todos seguem o pior, ou o app mantém mais de uma codificação e perde parte do ganho.
- Instalador maior. Estimativa sem medição: 2 a 6 MB a mais.

**Tamanho**: 4 a 6 mil linhas de Rust e 800 a 1.500 no site. O módulo de captura inteiro tem 2.286.

**Risco**: alto. Os três maiores são o codificador por fabricante, o iPhone e a imagem ficar pior do que hoje em rede ruim, porque o controle de banda passa a ser nosso.

### Antes de decidir pela A

Duas medições de poucas horas dizem se a A pode ajudar:

1. **Qual codificador o navegador está usando.** O site já lê essa informação. Se já for a placa de vídeo, o ganho da A é pequeno.
2. **Captura sem envio.** Transmitir com a página descartando os quadros e medir o FPS do jogo. Se o jogo já perder quase o mesmo FPS só com a captura, a A não recupera nada.

Complemento: medir com 1, 3 e 6 espectadores e olhar o uso da placa de vídeo por processo no Gerenciador de Tarefas.

## B. App inteiro nativo (sem motor de página)

Tudo da A, e mais:

- **Recepção**: fila de recepção, decodificação, sincronia de som e imagem, som com volume por pessoa. É a parte que o navegador faz melhor e que mais demora para ficar boa.
- **Telas**: 36 arquivos e 4.399 linhas refeitos numa ferramenta de interface nativa, mais as cerca de 100 cenas com dados falsos que a constituição exige.
- **Lógica da sala**: 4.629 linhas refeitas em Rust. Cerca de metade é lógica pura e porta fácil. O resto conversa com a ligação de vídeo.
- **Câmera, tela cheia, janela flutuante, link `espia://`, instância única, posição da janela**: refeitos sem as peças prontas de hoje. Bandeja, links e captura são reaproveitados. O código de memória e de esconder a barra de compartilhamento deixa de existir (347 linhas).

**Ligação de vídeo**: duas saídas, nenhuma confortável.

- Usar a mesma do Chrome por uma ponte para Rust (`libwebrtc` 0.4.0 da LiveKit, 2026-11-04). É a mais completa. A codificação pela placa de vídeo no Windows só foi achada em versões modificadas por terceiros, a montagem no Windows é pouco documentada e o executável cresce dezenas de MB (inferido).
- Usar a `str0m` e construir a recepção por conta própria.

**Ferramenta de interface**: nenhuma se destaca.

- `egui` e `iced`: 125 a 190 MB com a janela escondida num comparativo de 2026-10-29, e 72 a 143 MB em outro. O ganho de memória sobre os 228 MB de hoje é incerto.
- `Slint` (estável, 1.17): memória menor em relato não confirmado. A exibição de vídeo usa uma parte marcada como instável.
- `GPUI`: 19 a 25 MB em versões de terceiros, sem publicação oficial utilizável.

**Custos permanentes**

- **Duas interfaces para sempre.** O site tem de continuar funcionando sem instalar nada (princípio I). Cada feature futura seria feita duas vezes, com duas telas de dados falsos.
- **O app deixa de acompanhar o site.** Cada mudança de tela vira instalador novo, que hoje cada pessoa baixa na mão.
- **Instalador** de 1,7 MB para algo entre 15 e 40 MB (inferido).
- **Fidelidade do design**: sombras, fontes e o mascote animado precisam ser refeitos, com perda provável.
- **Emenda maior da constituição**: a restrição "Tauri 2 + WebView2, carrega o site publicado" cai, e os princípios IV e V passam a valer para uma segunda interface.

**Tamanho**: 14 a 22 mil linhas de Rust. É de 4 a 7 vezes o Rust que existe hoje e parecido com o projeto inteiro em TypeScript (14.886 linhas com testes).

**Referências de fora**: o Hopp, um app do mesmo tipo, moveu para nativo só a janela de vídeo e manteve o resto em página. Reportagens de novembro de 2026 dizem que o Discord, ao reescrever o cliente em Rust, mantém o vídeo em página. A informação sobre o Discord não foi confirmada na fonte original.

## C. Tudo em Rust, com site e servidor

Reescreve também o site e o servidor (11.558 linhas) e os 268 testes. O site em Rust roda no navegador e usa a mesma ligação de vídeo do navegador, então não fica mais rápido. O servidor só faz sinalização e não é gargalo. Nenhum ganho para quem usa.

## Dinheiro

- **Licenças**: nenhuma. Todas as bibliotecas citadas são abertas.
- **Cloudflare**: sem mudança. O servidor continua só com sinalização.
- **H.264**: codificar pela placa de vídeo ou pelo Windows não exige distribuir codificador. Um plano B por software (OpenH264) exige baixar o binário da Cisco e mostrar o crédito dela.
- **Sessões de IA**: proporcionais ao volume. A opção A fica acima do que custaram as features 005 e 010 juntas. A B fica perto de refazer o projeto inteiro uma vez, ou uma vez e meia.

## O que não foi confirmado

- Quanto do peso de transmitir vem da captura e quanto vem da página. As duas medições acima respondem.
- Se o navegador dentro do app está de fato codificando pela placa de vídeo em todas as máquinas do grupo.
- Se a ponte `libwebrtc` oferece codificação e decodificação pela placa de vídeo no Windows na versão oficial.
- O tamanho real do executável em cada opção.
- A memória real de um app nativo com vídeo ao vivo. Os comparativos medem janela parada.
- O comportamento do codificador do Windows em cada fabricante e o aviso do firewall. Vieram de conhecimento do revisor, sem fonte conferida.
