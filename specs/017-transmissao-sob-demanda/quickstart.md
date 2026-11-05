# Validação da transmissão sob demanda

Este roteiro descreve validação futura. Nenhum resultado de desempenho foi obtido pelo planejamento.

## Preparação

Node 24, dependências instaladas com `npm ci`, dois aparelhos para o ensaio mínimo e até nove para o cenário-alvo. Usar app 0.1.9 ou superior com site de teste atualizado; confirmar versão nativa e WebView2 de cada aparelho. A fonte existente é scripts/latency-source.html; para texto, definir lista de vinte itens antes da rodada. Para movimento, usar cena repetível com contador e tarefa/jogo concorrente.

Rodar `npm run dev` para interface/mocks e `npm run dev:worker` para assets e sinalização locais. Usar `npm run dev:lan` somente em rede de ensaio com HTTPS confiável nos clientes. Captura real requer contexto seguro; não mascarar falha de certificado como falha de mídia. App debug pode usar ESPIA_START conforme configuração da 016, verificando origem e capabilities de desenvolvimento.

## Checks locais após implementação

1. `npm run check`, `npm test`, `npm run build`.
2. Abrir `/?mock=room&scene=watch-available`, `watch-stopped`, `watch-opening`, `watch-unavailable`, `watch-playing` e `watch-reconnecting`. Cenas propostas, a implementar antes de usar como evidência.
3. Executar testes de reducer, grants e filas de peer com operações pendentes; Worker deve recusar protocolo antigo e nunca chamar API TURN na política direct-only.
4. Conferir servidor com chaves TURN ausentes e presentes em ambiente de teste: ambos retornam somente STUN, sem imprimir segredos. Não usar conta produtiva para demonstrar orçamento.

## Sem clique e lifecycle

Duas transmissões anunciadas. Aguardar 60 segundos e confirmar zero conteúdo de mídia, nenhum decoder/player ativo e tracks de envio ausentes para esses receptores; presença de sinalização não falha o teste. Cliques em destaque, volume e restaurar janela não iniciam mídia.

Escolher uma transmissão; em até 3 s ela aparece, a outra continua sem mídia. Parar corta imagem/som e contagem em até 2 s. Repetir com clique rápido, renegociação pendente, unlive/live, saída, interrupção do controle e retomada. Em cada caso comparar a chave e revisão sem salvar identidade nos artefatos. Verificar grades, teclado, celular, janela escondida e PiP.

Testar cliente antigo/novo e sockets antigos retomados: atualização explícita, sem receber conteúdo antes da escolha. Um teste com mocks não prova ausência de pacotes numa sessão real; verificar getStats e fluxo real por recepção.

## Atraso e recursos

Definir antes do ensaio aparelhos/GPU, rede, RTT, capacidade e cenas. Perfil inicial de referência: transmissores com pelo menos 25 Mbps de upload disponível cada, receptores com pelo menos 10 Mbps de download disponível cada, RTT até 80 ms e sem competição deliberada no trecho estável. São condições propostas para testar 720p30 no máximo, não promessa de funcionar em todo enlace. Registrar capacidade efetiva e carga do jogo; repetir em aparelhos mais modestos como caracterização.

Medir baseline da 0.1.9 e candidato em rodadas alternadas, mesmos aparelhos, conteúdo, rota direta, qualidade efetiva e configurações/limite de FPS do jogo. Matriz 1/2 transmissões × 1/4/7 espectadores, 30 minutos estáveis por cenário, excluindo apenas os intervalos de perturbação definidos antes do ensaio. Cada recepção tem 1.800 instantes previstos a 1 Hz, sem misturar receptores.

Exigir pelo menos 1.782 instantes com instrumento funcionando, no máximo 2 lacunas instrumentais consecutivas e pelo menos 1.710 instantes com atraso ≤500 ms. O denominador de sucesso é sempre 1.800, não a quantidade de amostras válidas. Instrumento funcionando sem vídeo é falha de entrega; não classificar como lacuna instrumental. Uma imagem congelada pode ter atraso medido pelo contador antigo e deve entrar no resultado. Lacuna instrumental não vira zero; cobertura abaixo de 99% ou lacuna maior que 2 instantes torna a execução inconclusiva. Atraso ≥5 s ou ausência de imagem por ≥5 s reprova mesmo com p95 baixo. Se houver simultaneamente falha comprovada e cobertura insuficiente, manter a reprovação e registrar a limitação, sem usar inconclusivo para apagar a falha.

Comparar origem e receptor no mesmo campo de observação ou usar instrumento calibrado, sem subtrair relógios não sincronizados. Para A/V usar resolução de medição ≤20 ms e calibrar fonte. getStats e RTT explicam etapas, não substituem atraso visual. Registrar p95 no índice ceil(0,95×N), máximo e amostras ≥5 s. Critérios SC-003/004 permanecem pendentes sem instrumentos e espectadores reais.

Cinco repetições de cada condição: queda breve da rede, limitação de upload, carga temporária do emissor, restaurar janela, troca grade/destaque e troca de qualidade. Marcar previamente intervalos estáveis e perturbados; medir até 5 s após normalização/transição, sem excluir resultados ruins a posteriori. Manter pausas de observação para distinguir espera por frame novo de atraso real.

Texto: pelo menos 19 dos vinte itens lidos corretamente, com fonte, tamanho, resolução de origem e janela fixados antes das rodadas. Movimento em visualização completa: fonte produzindo ≥30 fps, média de ≥27 quadros distintos decodificados por segundo em todo o trecho estável, interrupções somadas ≤1% desse tempo e nenhuma pausa >1 s. Interrupção é tempo sem novo quadro quando a fonte de movimento continua produzindo; medir intervalos entre quadros, sem contar repetição do mesmo quadro como decodificação nova. Incluir falhas de vídeo no denominador. Sem comprovar a cadência da origem, marcar ensaio pendente em vez de culpar a rede.

Mediana dos FPS do jogo/tarefa concorrente no candidato deve ser ≥95% da baseline equivalente, mantendo mesma cena, configurações, aparelhos e limite de FPS. Sem tarefa mensurável equivalente, o aceite desse item fica pendente. Guardar CPU/GPU, memória e encoder como diagnóstico; não reduzir referências depois de ver o resultado. `powershell -File scripts/memoria.ps1` auxilia memória do app. Desconectar DevTools durante ensaio de recursos para não medir retenção do próprio diagnóstico, conforme relatório da 010.

Repetir live de A após unlive(A), live(A) após início de B e unlive(A) enquanto B está ativo. Repetir também depois de retomada, hibernação e reinício do Worker. O servidor deve manter o estado atual, não gerar peer-live para pedido encerrado, e conservar o mesmo ACK para repetição de live ainda ativo.

## Custo e redes restritivas

Conferir ausência de TURN no welcome/refresh/resume e ausência de geração de credenciais. Simular falhas de política e conexões antigas sem consumir saldo real. Em rede que exige relay, resultado esperado é mensagem clara e nova tentativa manual, preservando pares diretos.

Não declarar proteção da conta até concluir o procedimento de drenagem em [media-policy.md](contracts/media-policy.md). Saldo positivo e SFU remota ficam pendentes enquanto não houver provas e governança. Não marcar essa parte aprovada por testes de STUN-only.

## Fechamento

Produzir relatório local sem conteúdo gravado, IPs, SDP, nomes ou identificadores permanentes. Separar checks locais, medições reais e condições não testadas. Registrar todos os SC individualmente como pass/fail/pending. Sem espectadores/instrumentos, entregar apenas evidência disponível e pendências. Planejamento, testes e build não autorizam deploy.
