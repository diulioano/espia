# Quickstart: validar atraso da imagem

Este é o roteiro para a implementação. A fonte de ensaio e os novos testes ainda serão criados. Nenhuma medição real foi executada durante o planejamento.

## Pré-requisitos

- Node e npm conforme projeto; dependências por npm ci. Windows com Rust/MSVC/WebView2 para ensaio do app.
- Dois aparelhos reais para origem/destino; mais dois espectadores para ensaio com 3. Conexões estáveis, sem outra transmissão concorrente.
- Chrome/Edge desktop para transmitir; incluir recepção em Firefox/Safari e celular, e câmera no celular conforme suporte existente.
- Discord instalado para comparação. Não precisa de chamada de voz ou conta nova no Espia.
- Mesma qualidade e cena de movimento nos ensaios; registrar a qualidade efetiva quando a adaptação atuar.

## Preparar e executar

Na raiz do repositório, em terminais separados:

```powershell
npm ci
npm run dev:worker
```

```powershell
npm run dev
```

Abrir http://localhost:5173. O Vite encaminha /api e /ws ao Worker local na porta 8787. Criar uma sala e entrar pelo fluxo normal. Não publicar o site durante este roteiro.

Para app debug:

```powershell
$env:ESPIA_START = 'http://127.0.0.1:5173'
npm run desktop:dev
```

ESPIA_START é suportado por src-tauri/src/lib.rs em builds de debug. Confirmar que o app está em 127.0.0.1:5173 antes de atribuir resultados ao novo código; não alterar permissões de origem de produção. Encerrar qualquer instância anterior do Espia pela bandeja antes de iniciar o debug, pois o app é de instância única. Ao terminar, remover a variável com Remove-Item Env:ESPIA_START.

Depois de criada, abrir scripts/latency-source.html em um navegador e transmitir sua janela/aba. Ela deve mostrar contador com milissegundos, movimento contínuo e flash com beep sincronizado; iniciar som por clique. Para câmera, apontá-la à mesma fonte visual e sonora, mantendo distância fixa.

Para outros aparelhos, usar o fluxo existente npm run dev:lan, confiar no certificado local nos aparelhos de ensaio e entrar pela URL HTTPS da rede. Verificar conectividade e contexto seguro antes de medir; o Vite HTTP em outro aparelho não prova suporte de captura.

## Régua e registro

1. Colocar origem e tela recebida no mesmo campo de observação. A cada segundo, comparar o valor visível na origem com o recebido. Não subtrair relógios de aparelhos diferentes.
2. Registrar somente amostras numéricas, com elapsedMs, delayMs, avOffsetMs, tipo de transição/perturbação e qualidade efetiva. Não gravar conteúdo transmitido.
3. Capturar 1800 amostras por espectador em cada sessão de 30 min. Ordenar os atrasos e usar nearest-rank p95 (índice ceil(0,95×N)); registrar máximo e contagem ≥ 5000 ms. Amostras ausentes são lacunas a investigar, nunca zeros.
4. Identificar previamente os intervalos estáveis e perturbados. Não excluir trecho apenas porque seu atraso foi alto. Depois da normalização, o prazo de recuperação também é avaliado separadamente.
5. Para SC-005, usar flash/beep e instrumento local de temporização A/V capaz de resolver ≤ 20 ms, registrando apenas o desvio numérico. Ouvir informalmente não comprova 200 ms. Não salvar mídia da sessão como diagnóstico do produto.
6. Anotar a linha de base e resultados após correção em relatorio.md da feature, com versões/aparelhos/rota host, srflx ou relay, sem nomes, IPs, SDP ou código de sala.

## Ensaios de aceitação

### T1 — Linha de base e estabilidade (SC-001)

Antes e depois da alteração: sessões de 30 min com 1 e com 3 espectadores, com som e sem som. Começar pelo app/tela ou janela afetada e repetir no browser. Usar 1080p60 quando suportado e incluir 720p30 como controle. Critério por espectador: p95 ≤ 1000 ms e zero amostras ≥ 5000 ms nos trechos estáveis.

### T2 — Discord (SC-002)

Repetir a mesma cena por 30 min no Discord, com aparelhos, rede e qualidade equivalente. Fazer uma rodada Espia→Discord e outra Discord→Espia; nunca transmitir simultaneamente. Comparar p95 de cada par. Diferença máxima aceita: 500 ms. Documentar qualidade não equivalente como limitação do ensaio, sem marcar aprovação.

### T3 — Startup e interrupções (SC-003/005)

- Reproduzir startup de captura atrasado em 2 s e 10 s no fake Bridge e, quando possível, no debug nativo; o tempo de abertura não pode virar offset permanente de vídeo.
- Limitar a conexão efetivamente usada pelo WebRTC por 10 s; depois restaurar. O throttle HTTP do DevTools sozinho pode não limitar UDP de mídia. Usar ferramenta de rede do ambiente e registrar banda/perda/latência aplicadas.
- Sobrecarregar transmissor e, separadamente, espectador por 10 s. Fazer 5 repetições por tipo de perturbação.
- Medir desde a retirada da perturbação: em ≤ 5 s o atraso deve ser ≤ 1000 ms e A/V ≤ 200 ms, sem ação na sala.
- Incluir pausa/retorno do áudio e audio:none→áudio para verificar reancoragem.

### T4 — Transições e isolamento (SC-004)

Cinco repetições de cada ação: começar a assistir, queda/retorno, miniatura↔destaque, mudança de qualidade e ocultar/restaurar janela. Para início/retorno, medir o prazo após a primeira imagem; nas demais, após a ação. Em ≤ 5 s, atraso ≤ 1000 ms. Nos ensaios com 3 espectadores, causar dificuldade em apenas um: os demais continuam atendendo T1.

### T5 — Encerrar e começar de novo (SC-006)

Cinco repetições: atrasar captura/escrita, parar a transmissão, iniciar outra e liberar callbacks antigos. Repetir saída da sala/close. Nenhum som ou imagem da sessão encerrada pode reaparecer; buffers, timers e writers antigos são liberados.

## Testes automatizados e regressão

```powershell
npm run check
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run desktop:build
```

Cargo/build desktop são necessários quando houver alteração Rust. Testes novos devem cobrir startup lento, reancoragem/lacunas, writer que demora/rejeita, completions de geração velha, mailbox cheia, stop/start, deltas sem suporte/reset/denominador zero e receiver setter ausente/que lança. Não substituir T1–T5 por mocks.

Abrir /?mock=room&scene=grid para regressão visual. Verificar áudio em segundo plano, mute/volume, off (sem som/vídeo), PiP, miniatura e encerramento; repetir no celular. Verificar modos e limites com sala de 10 pessoas, sem exigir qualidade máxima para todos. Conferir que tetos de upload e histerese de CPU da 010 permanecem.

## Saída

Aprovar somente com evidências para SC-001–006 e gates de implementação passando. Registrar capacidades ausentes e falhas; jitterBufferTarget=0 não é resultado de teste. Se o IPC continuar acumulando no renderer ou o fallback exceder o prazo, revisar o design com as medidas antes de marcar a correção concluída.

## Estado após implementação local

A fonte scripts/latency-source.html e os testes citados já existem. A fonte foi aberta via Vite em http://127.0.0.1:5173/scripts/latency-source.html; contador/animação iniciaram por clique. O mock de sala abriu e os controles Parar de assistir/Assistir foram conferidos. Isso não substitui a captura dessa fonte por app/câmera nem as medições T1–T5.

Enquanto estiver numa sala real, o console permite `JSON.stringify(window.espiaMediaDiagnostics())` para obter até 60 amostras técnicas por link, sem nomes, endereços ou sala. O hook é removido ao destruir o store. getStats não mede atraso ponta a ponta: continuar usando a referência visual comum.

O build local do app foi gerado sem instalação, release ou deploy. Para testar as correções TypeScript junto ao app, usar ESPIA_START em debug com o site local; o instalador release continua abrindo o site publicado. As medições reais permanecem pendentes conforme relatorio.md.
