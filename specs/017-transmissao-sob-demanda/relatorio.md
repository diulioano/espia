# Relatório de implementação da 017

Data: 2026-11-05. Estado: implementação local com aceitação externa pendente.

## Revisões e ambiente

O commit pedido antes da implementação é c0e43da, contendo a constituição 3.0.0 e os documentos revisados. A revisão anterior do código 0.1.9 é 3cc5d65081f57711dfc9345b542b891e91087cde. A implementação desta execução permanece no working tree; nenhum serviço foi publicado e nenhuma credencial ou configuração remota foi alterada.

Um ambiente Windows local, Node 24.15.0 e navegador interno do Codex. Há WebRTC real para ensaio sintético local, mas não nove aparelhos independentes, jogo instrumentado, rede restritiva de ensaio ou instrumento A/V calibrado. Não inventariar hardware ou versões que não foram verificadas. A captura Rust e a configuração nativa não foram alteradas.

Baseline local antes das alterações: npm run check sem erros/avisos; npm test com 21 arquivos e 239 testes aprovados; npm run build aprovado. Isso não é baseline de desempenho ponta a ponta.

## Implementação

- Escolhas positivas por transmissão em watch-state.ts; ausência não recebe mídia, queda preserva a mesma escolha, nova captura pede novo clique e mobile mantém uma escolha.
- ctl v2 com chave de transmissão e revisão; senders permanecem sem tracks antes de grant válido. Perda de controle, Parar ou troca de captura invalida o envio; resultados assíncronos antigos não autorizam a nova captura.
- Protocolo de sala v2, live-ack, broadcastSeq e marca terminal preservados na retomada/hibernação/reinício. Cliente incompatível recebe atualização necessária, sem loop de reconexão.
- STUN-only aplicado no servidor antes de verificar segredos ou gerar credenciais. Filtros no cliente removem TURN, descartam credenciais antigas e limitam a reconstrução de rota incompatível. Sem ledger nem ativação por segredo configurado.
- Hints de conteúdo preferem resolução para texto/detalhe e fluidez para movimento. Tetos escolhidos, limites por espectadores e limiares CPU de 10/60 s foram preservados. Não houve medição de ganho nem alteração de encoder nativo.
- Cartões available/stopped distintos, toast Assistir escolhe a transmissão, timers só para escolhas e estados de erro/atualização mockáveis. O título mobile não diz Vendo antes do clique.

Os testes de protocolo reproduziram a aceitação do join sem v2 e a tentativa de emitir TURN antes da correção. Os testes locais agora cobrem essas fronteiras, grants/controle encerrados, stop concorrente com replaceTrack, sequências inválidas e pedidos encerrados depois de reinício do Durable Object. Testes legados de emissão TURN e áudio boolean foram migrados para o contrato atual, sem manter compatibilidade que reative mídia automática.

## Ensaio WebRTC sintético real no navegador

Roteiro reproduzível em scripts/watch-017.html, servido por npm run dev. O botão cria duas transmissões a partir de canvas e oscilador, sem capturar tela, câmera ou microfone. Esse roteiro usa PeerLink real em duas conexões locais; não representa a sala completa com nove aparelhos.

Resultado observado: PASS. Após conexão, durante 60 segundos sem escolha, ambas enviaram zero bytes de vídeo e áudio. Após escolher a primeira, foram observados 833.722 bytes de vídeo e 128.720 de áudio na primeira e zero na segunda. Após Parar e a janela de encerramento de 2 s, os contadores ficaram estáveis. Uma chave nova de transmissão não retomou o envio. Esses bytes demonstram recepção sob escolha, não velocidade da rede ou ganho de performance.

A evidência sanitizada está em .wrangler/acceptance-017/webrtc-local.json e o visual mobile em .wrangler/acceptance-017/mobile-framed.jpg, ambos locais e ignorados pelo Git. Não guardar frames, áudio ou dados pessoais como prova.

## Visual

As seis cenas watch-available, watch-stopped, watch-opening, watch-playing, watch-unavailable e watch-reconnecting foram abertas no navegador. Assistir e Parar alteraram o cartão corretamente. O mock update-required mostrou a atualização sem diálogo bloqueante. No componente mobile dentro do quadro de teste 390×844, o cartão começou sem recepção e o título mudou de Transmissão de Lu para Vendo Lu após o clique. O override de viewport foi removido ao terminar.

PiP real, captura de câmera/tela e saída sonora em aparelhos distintos não foram exercitados por esses mocks.

## Checks e aceitação

Rodada intermediária completa: npm run check sem erros/avisos; npm test com 27 arquivos e 249 testes; npm run build aprovado; node --test test/scripts/acceptance-017.test.mjs com seis testes aprovados. Foram acrescentados depois três casos de borda de protocolo/fechamento; a rodada final está registrada ao fim deste relatório.

| Critério | Evidência disponível | Estado integral |
| --- | --- | --- |
| SC-001 | 60 s sem vídeo/som no PeerLink real local; reducer e mocks | Parcial; falta sala completa nos caminhos reais suportados |
| SC-002 | Ação no mock e corte do PeerLink real; testes de revisões | Parcial; falta tempo de início e contagem da sala real |
| SC-003 | Ferramenta numérica valida cobertura ≥99%, denominador e falhas | Pendente: matriz de 30 min e atraso visual real |
| SC-004 | Correções da 016 preservadas; regressões locais passam | Pendente: perturbações e medição A/V calibrada |
| SC-005 | STUN-only, chaves presentes/ausentes, concorrência e recusa de legado testados | Ramo desabilitado local aprovado; conta/drenagem pendentes; saldo positivo não exercitado |
| SC-006 | Reducer, filas, ctl encerrado, reinício DO e mocks | Parcial; falta matriz real de câmera/tela/PiP/rede |
| SC-007 | Ferramenta aplica 27 fps, interrupções, texto e limite de regressão | Pendente: comparações reais equivalentes de imagem e jogo |

FR-001–007/011/020 estão implementados e têm testes locais; FR-008–012 dependem também da aceitação em escala e desempenho. FR-013–017 usam o fallback desabilitado, sem prometer controle de consumo ativo legado. FR-018/019 têm roteiro, ferramenta e registros sanitizados; a comparação sem dados permanece pendente. Pesquisa SFU está no documento próprio, com gates não atendidos e sem provisionamento.

## Sequenciamento e limitações

O desenvolvimento dos hints e da sua integração foi verificado por testes determinísticos sem retunar tetos ou governor. A baseline real T028 não estava disponível; a revisão anterior foi preservada para comparação posterior e a eficácia de T029/T030 não foi aprovada por isso. A ativação em ensaio comparativo/publicação continua condicionada ao roteiro real. Essa adaptação de sequência permite concluir trabalho local reversível sem fabricar baseline ou mexer na produção.

T025/T028/T033/T034/T042/T045 e as partes reais de T048/T049 permanecem abertas. T044 entrega a estrutura comparativa, com ambas as rodadas de desempenho pendentes. A feature não é declarada integralmente aceita. Checklist de qualidade foi apenas lida; seus marcadores não foram alterados durante implement.


## Rodada final

- npm run check: PASS, zero erros/avisos no Svelte e TypeScript do Worker.
- npm test: PASS, 27 arquivos e 252 testes.
- node --test test/scripts/acceptance-017.test.mjs: PASS, seis testes numéricos.
- npm run build: PASS.
- git diff --check: PASS depois de corrigir linhas vazias finais.
- Seis estados mobile também confirmados dentro do iframe de teste 390×844; atualização necessária mockável e screenshot local registrado.
- tasks.md: 44 concluídas, 8 pendentes, preservando os aceites reais acima. requirements.md não foi modificado durante implement.

Nenhum hook before_implement/after_implement configurado: .specify/extensions.yml não existe. Nenhuma rejeição automática ou pedido de permissão ocorreu. Não houve deploy, instalação de app, migração Rust ou mudança de versão/publicação. A implementação permanece no working tree depois do commit documental c0e43da.


## Publicação para teste com o grupo — 2026-11-05

Autorização explícita: "adicione o link de download igual antigamente e publique para testar com o pessoal". Esta seção sucede o estado anterior somente local; não converte os oito aceites externos pendentes em aprovação.

Versão 0.1.10 sincronizada em package.json/package-lock.json, Cargo.toml/Cargo.lock e tauri.conf.json. Nenhuma dependência ou código Rust foi alterado nesta etapa. npm run desktop:release gerou o instalador e o copiou para public/download/Espia-setup.exe; a versão de produto do executável foi conferida como 0.1.10. O botão existente no site Windows usa esse endereço, e README e resposta de entrega também oferecem o link direto.

Alvo conferido: Worker espia na conta já autenticada do projeto, configuração padrão wrangler.jsonc, origem https://espia.espia.workers.dev. Binding ROOM/Room e variáveis remotas conferidos antes da publicação; secrets e migrações não foram alterados. O deploy preservou variáveis remotas com --keep-vars.

- Versão Cloudflare anterior: 60d9b5e3-4064-454e-8dff-1887bf16ac71.
- Versão publicada: c2a655d3-7e78-4b09-b262-26bd738d8f85.
- Download: https://espia.espia.workers.dev/download/Espia-setup.exe.
- Instalador: 1.700.932 bytes; SHA-256 b3552f655d147fe4678521f3ebd7d55e29658be020259e99f79d68a7bbb81af6.

Checks: desktop:release aprovado; npm run check sem erros/avisos; npm run build aprovado; wrangler deploy --dry-run --keep-vars aprovado; npm run deploy -- --name espia --keep-vars aprovado.

O smoke em .wrangler/deploy-smoke-017.mjs confirmou HTTP 200 e hashes idênticos ao build para página, bundles e instalador; assinatura MZ do arquivo; welcome mediaProtocol:2/direct-only com apenas STUN; sala rápida exclusiva de teste com dois clientes sintéticos, entrada, ping/pong, repasse de sinalização, live-ack, unlive e rejeição de pedido de transmissão encerrada. Todos os sockets do teste foram fechados com código de saída explícita. Não foi usada sala existente nem capturado conteúdo de pessoas.

Resultado: publicação e smoke PASS. Meta de 500 ms, matriz de aparelhos e rede restritiva permanecem para o teste do grupo. A emissão de TURN está desabilitada no código novo; credenciais anteriores e outros emissores da conta não foram inventariados/revogados nesta etapa, portanto a drenagem financeira global continua pendente. Quem usa app 0.1.9 com o site remoto recebe as mudanças ao sair pela bandeja e abrir novamente; o instalador 0.1.10 também está disponível. No navegador, recarregar antes de entrar.

## Retorno do teste com o grupo — 2026-11-05

A pessoa usuária informou: "acredito que está bom fizemos os testes". Registrar como resultado prático satisfatório relatado para a versão publicada 0.1.10, sem falhas específicas informadas nesse retorno.

Não foram fornecidos cenário exato, número de participantes, duração, medições de atraso ou FPS. Portanto, esse retorno não comprova os limiares numéricos de SC-003/004/007, todos os cenários de lifecycle ou a drenagem financeira da conta. Os resultados locais anteriores permanecem válidos; não alterar tarefas de medição pendentes apenas a partir da avaliação geral.

## Perda de definição em movimento — 2026-11-05

Após o teste, a pessoa usuária esclareceu o sintoma: a imagem perde definição durante o movimento e recupera quando a cena para; o áudio continua normal. Cenário relatado: app 0.1.10, 1080p60 e três espectadores. Ainda não foi comparado com os demais espectadores.

A inspeção do Espia aberto confirmou recepção de uma fonte anunciada como Câmera/USB3 Video em 1080p60. O atalho de ferramentas de desenvolvimento não abriu diagnóstico no app instalado. A qualidade anunciada e uma captura de tela não permitem determinar o bitrate, a resolução efetivamente recebida nem a origem da perda de definição. O teto configurado de 6 Mbps por espectador foi conferido no código, sem afirmar que esse teto foi alcançado ou causou o sintoma.

Foi adicionado um painel local, fechado por padrão, acessível com Ctrl+Shift+D e fechado pelo botão Fechar, Escape ou pelo mesmo atalho. Usa a coleta já existente de getStats; mostra bitrate, FPS codificados/decodificados calculados pelo intervalo RTP, dimensões reais informadas, limitação declarada pelo encoder, perdas, congelamentos, buffer e decodificação. Não envia diagnóstico a servidores nem registra tela/som, nomes, endereços ou SDP. Não modifica os limites de bitrate, resolução, FPS, codecs ou transporte.

Validação: npm run check sem erros/avisos; 27 arquivos e 254 testes aprovados; build e deploy dry-run aprovados. O ensaio scripts/media-diagnostics.html confirmou contadores reais recebidos por WebRTC e abertura/fechamento do painel em navegador local com fonte sintética 640×360. Esse ensaio valida a instrumentação, não reproduz o problema da fonte USB. Screenshot de verificação local: .wrangler/acceptance-017/media-diagnostics.png.

Diagnóstico publicado no Worker espia, versão Cloudflare 4abf1852-d35f-467f-aa1a-725f93636663, preservando variáveis remotas. O app permanece 0.1.10 porque carrega o site remoto; não houve alteração do instalador ou código nativo. A sala aberta não foi recarregada pelo agente. Falta coletar os valores reais durante a perda de definição, preferencialmente também no transmissor, antes de escolher e validar uma correção.

### Primeira inspeção do diagnóstico no app — 2026-11-05

A pessoa usuária confirmou que a mesma transmissão parece melhor no navegador. Durante a inspeção seguinte, a fonte permaneceu no menu do jogo, com pouca movimentação; a pessoa usuária confirmou que naquele momento a imagem estava nítida. Portanto, não foi reproduzida a perda de definição durante essa coleta, nem foi realizada comparação medida entre app e navegador.

Três leituras pontuais do painel no app, sem alteração de controles:

| Leitura | Resolução recebida | FPS decodificados no intervalo | Mbps no intervalo | Perdas / congelamentos no intervalo | Buffer médio | Decodificação média |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 1920×1080 | 50,0 | 4,35 | 0 / 0 | 55,1 ms | 1,5 ms |
| 2 | 1920×1080 | 51,0 | 5,23 | 0 / 0 | 38,4 ms | 1,6 ms |
| 3 | 1920×1080 | 46,1 | 5,50 | 0 / 0 | 49,8 ms | 1,6 ms |

Essas são amostras esparsas de uma sessão, não uma medição contínua ou benchmark. Não comprovam ausência de perdas entre leituras, capacidade de 60 FPS, bitrate suficiente em movimento ou causa da diferença relatada entre app e navegador. Próximo passo: reproduzir um trecho movimentado com o painel aberto e identificar o intervalo em que a imagem perde definição; depois comparar o mesmo cenário no navegador.

### Reprodução com movimento — 2026-11-05

A pessoa usuária avisou quando o jogo voltou a se movimentar. A inspeção do app confirmou visualmente perda de definição e mudança das dimensões efetivamente recebidas, enquanto o cabeçalho continuava anunciando 1080p60. Foram observadas três leituras pontuais:

| Leitura | Resolução recebida | FPS decodificados | Mbps | Perdas / congelamentos | RTT | Buffer médio | Decodificação média |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 1280×720 | 49,3 | 4,70 | 0 / 0 | 44 ms | 121,3 ms | 1,4 ms |
| 2 | 640×360 | 49,7 | 5,28 | 0 / 0 | 44 ms | 94,9 ms | 0,7 ms |
| 3 | 640×360 | 49,0 | 4,63 | 0 / 0 | 43 ms | 81,9 ms | 0,7 ms |

O sintoma foi reproduzido com intervenção humana na fonte remota: jogo em movimento → definição baixa e recepção em 360p. Isso confirma a redução real da resolução decodificada, sem estabelecer ainda por que ela ocorreu ou que o problema seja exclusivo do app. Zero nos contadores dessas amostras não exclui perda/congestionamento em outros intervalos. Não se trata de uma medição de ganho de desempenho.

Hipóteses, em ordem de investigação: (1) adaptação do encoder para esse envio, verificável pela resolução e qualidadeLimitationReason no transmissor durante a redução; (2) redução da captura USB, verificável comparando dimensões da fonte com as saídas para diferentes espectadores; (3) perfil menor solicitado/aplicado pelo Espia, verificável pelos parâmetros efetivos do sender e modo de visualização. O perfil de miniatura previsto pelo código é 360p/15 FPS/0,5 Mbps e o governor de CPU reduz primeiro para 30 FPS; os valores observados diferem desses perfis, mas isso não prova que os parâmetros tenham sido aplicados corretamente em runtime.

Foi solicitado o campo Limitação do encoder no aparelho de quem transmite. Sem essa leitura e uma comparação medida com o navegador, a origem da adaptação continua pendente. Nenhuma alteração de bitrate, codec, resolução ou preferência de degradação foi aplicada a partir dessas amostras.

A pessoa usuária informou que não consegue conferir o transmissor neste momento. A inspeção termina com o sintoma reproduzido e a queda até 360p confirmada; a causa da redução e uma correção validada permanecem pendentes dos dados na origem ou de uma reprodução controlada equivalente.

### Inspeção no Chrome — 2026-11-05

A pessoa usuária abriu a mesma sala no Chrome com o painel de diagnóstico. A inspeção apenas leu a janela existente, sem alterar controles da sala. Cinco amostras foram coletadas pela acessibilidade, a cada aproximadamente dois segundos, entre 21:24:51 e 21:24:59 UTC (18:24:51–18:24:59 em America/Sao_Paulo):

| Amostra | Resolução recebida | FPS decodificados | Mbps | Perdas / congelamentos | Buffer médio | Decodificação média |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 1280×720 | 49,0 | 5,75 | 0 / 0 | 81,3 ms | 0,2 ms |
| 2 | 1280×720 | 48,0 | 5,62 | 0 / 0 | 79,9 ms | 0,3 ms |
| 3 | 1280×720 | 48,0 | 4,46 | 0 / 1 | 93,2 ms | 0,3 ms |
| 4 | 1280×720 | 50,0 | 5,96 | 0 / 0 | 82,8 ms | 0,3 ms |
| 5 | 1280×720 | 40,0 | 5,77 | 0 / 1 | 80,7 ms | 0,2 ms |

O cabeçalho continuava anunciando 1080p60. Receber menos que 1080p foi observado também no navegador; a hipótese de exclusividade desse aspecto ao app não se sustenta. O app chegou a 640×360 na inspeção anterior, enquanto o Chrome permaneceu em 1280×720 nessas cinco amostras. Isso não estabelece quanto um é pior que o outro: os trechos, o instante, o estado da conexão e o número de participantes diferem (quatro na inspeção anterior, cinco agora). Não houve benchmark controlado nem identificação do recurso limitante. Os contadores são por intervalo, e há lacunas entre as amostras; não interpretar os zeros como ausência global de perdas ou os dois intervalos com congelamento como total exaustivo da sessão.

Nenhuma correção adicional ou publicação foi realizada nesta inspeção. A origem da redução ainda depende das estatísticas de envio ou de uma reprodução controlada equivalente.
