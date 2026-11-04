# Relatório: Reduzir atraso da imagem

Data: 2026-11-04. Não contém gravações de mídia nem identidade das pessoas.

## Relato e estado

A pessoa usuária confirmou nesta execução: **app do Windows; imagem e som atrasam juntos** em alguns casos, cerca de 10 s comparado ao Discord. Isso orienta o diagnóstico, sem confirmar a causa.

Estado de AcceptanceRun: **baseline em preparação**. Nenhuma sessão real de 30 min ou comparação com Discord foi medida nesta execução. Dados ausentes não contam como aprovação.

## Inventário

| Campo | Valor verificado |
|---|---|
| Revisão inicial | fe6b282; branch main |
| Projeto | Espia 0.1.8 |
| Máquina disponível | Um ambiente Windows 11 Home, versão 10.0.26300, build 26300 |
| Node / npm | v24.15.0 / 11.12.1 |
| Rust | rustc 1.99.1 |
| App instalado e versão WebView2 em execução | Não verificados |
| Navegadores de origem/destino e versões | Não verificados |
| 1/3 espectadores em aparelhos reais | Não disponíveis como sessões conectadas nesta execução |
| Instrumento A/V com resolução ≤ 20 ms | Não disponibilizado/verificado |
| Origem, qualidade efetiva, áudio, rota | Preencher por ensaio; não deduzir do relato |
| Início/fim de perturbações | Nenhuma perturbação real executada |

## AcceptanceRun: protocolo de medição

Estados: baseline → after-change → pass/fail. Cada execução registra build/revisão, versões de navegador/WebView2, aparelhos sem identificação pessoal, origem, qualidade efetiva, som, número de espectadores, tipo de rota (host/srflx/relay, sem IP) e início/fim de cada perturbação.

Abrir scripts/latency-source.html, clicar em iniciar e transmitir a janela/aba. Para câmera, apontá-la à mesma referência visual e sonora. Manter a origem visível. Calibrar o desvio flash/beep da própria origem antes de medir a transmissão: o agendamento comum não elimina latência de display/alto-falante.

Comparar contador da origem e imagem recebida no mesmo campo de observação; não subtrair relógios de aparelhos diferentes. Coletar uma amostra numérica por segundo e por espectador, com elapsedMs, delayMs, avOffsetMs quando medido, condição/transição e qualidade efetiva. Em 30 min, são 1800 amostras esperadas. Sem conteúdo de mídia ou identidade das pessoas.

Ordenar os N atrasos válidos e obter p95 no índice ceil(0,95×N), usando índice iniciado em 1. Registrar também máximo, número de amostras ≥ 5000 ms e lacunas. Lacunas não viram zero; definir os trechos estáveis e de perturbação antecipadamente, sem excluir atrasos altos depois de vê-los.

Para A/V, é obrigatório um instrumento local com resolução ≤ 20 ms, registrando apenas o desvio numérico. O instrumento ainda não foi disponibilizado: SC-005 não pode ser aprovado por ouvir informalmente o beep.

Comparação com Discord ocorre em rodadas separadas Espia→Discord e Discord→Espia, mantendo aparelhos, rede, cena e qualidade equivalente. Os indicadores getStats explicam partes da cadeia e não substituem a medida visual ponta a ponta.

## Checks iniciais

| Comando | Resultado |
|---|---|
| npm run check | PASS: svelte-check sem erros/avisos; tsc worker passou |
| npm test | PASS: 17 arquivos, 211 testes |
| npm run build | PASS: Vite gerou dist |

O repositório é privado para publicação npm (`private: true`), portanto .npmignore não é necessário. Ignorados node_modules, dist, target, arquivos locais de ambiente e artefatos temporários; sem Docker, ESLint, Prettier, Terraform ou Helm configurados que exijam novos ignores específicos.

## Debug nativo

Permissões de produção continuam restritas à origem do Espia em src-tauri/capabilities/capture.json. O setup Rust já adiciona capabilities-dev/capture-dev.json somente em debug, com http://127.0.0.1:*/* e http://localhost:*/*. ESPIA_START é reconhecido por src-tauri/src/lib.rs. Isso confirma o caminho configurado, ainda não uma sessão de captura validada em execução.

## Evidências de aceitação

| Critério | Estado | Evidência |
|---|---|---|
| SC-001 atraso estável | Pendente | Sem ensaio de 30 min em aparelhos reais |
| SC-002 comparação Discord | Pendente | Sem rodada comparativa equivalente |
| SC-003 recuperação | Pendente | Sem 5 repetições por perturbação |
| SC-004 transições | Pendente | Sem matriz real de transições |
| SC-005 A/V e ausência de ação manual | Pendente | Instrumento A/V e sessões reais indisponíveis |
| SC-006 encerramento | Pendente | Sem ensaio de callbacks e encerramento real |

## Limitações

O workflow exige baseline antes de alterar o comportamento de captura. A preparação e a instrumentação podem ser verificadas localmente; resultados de desempenho e eficácia permanecem pendentes até haver os aparelhos e as sessões necessários. Não houve deploy ou atualização de app.

## Progresso local: preparação e diagnóstico

T001–T007: preparação e instrumentação implementadas. T004 verificou o caminho debug configurado e registrou a ausência de sessões/aparelhos externos; não se atribui aprovação de captura real a essa inspeção. A régua foi aberta no navegador local, iniciou por clique e avançou de 0 a 3,201 s, com animação visível; o agendamento de áudio iniciou sem erro. Isso não calibra saída sonora nem mede A/V.

MediaStats e adapters: 11 testes novos passaram, cobrindo deltas/ausência/reset/SSRC/denominador zero, limite de 60 amostras, exportação sanitizada, consulta compartilhada, falha do motor e descarte após fechamento/substituição de peer. O ciclo de sala coleta também quando apenas assiste; `JSON.stringify(window.espiaMediaDiagnostics())` permite inspeção manual enquanto a sala existe, sem upload. Destruir a sala remove o hook.

Ajuste do plano de execução: T008 e as medições externas continuam pendentes como gates de aceitação, mas não impedem corrigir defeitos com reprodução determinística no caminho nativo. Justificativa: somente um ambiente conectado e ausência de espectadores reais, comparação Discord e instrumento A/V; preservar código inicial por revisão fe6b282 para baseline posterior. Não há afirmação de redução dos 10 s nem de causa confirmada.

## Implementação local e regressões

Implementadas correções de relógio de captura, reancoragem do áudio, descarte limitado no gerador, serialização de qualidade, cancelamento de abertura, aplicação serial de mídia por peer e preferência opcional de baixa latência. No Rust, a mailbox interna está em capture/publish.rs, usada por capture/mod.rs: um frame pendente, até 200 ms de PCM e cancelamento por sessão. A captura WASAPI também limita seu backlog e o pool de vídeo prioriza o quadro mais recente.

O comando `npm test -- test/lib/native-capture.test.ts` reproduziu três falhas antes da correção:

- startup artificial de 2 s: diferença de timestamps A/V de 2.000.000 µs (limite do teste: 20.000 µs);
- startup artificial de 10 s: diferença de 10.000.000 µs;
- lacuna de áudio de 10 s: próximo timestamp ficou 9.990.000 µs atrás do relógio atual.

Após correção, esses casos passam no mesmo caminho startNative com Bridge/geradores falsos. Isso demonstra os defeitos de timestamps e a correção das invariantes; **não comprova que eram a causa dos 10 s percebidos no receptor real**. As outras hipóteses (fila IPC e buffering do motor) permanecem dependentes de medições reais.

A regressão cobre também startup abortado, pausa/retorno de áudio, descarte de burst, stop idempotente, qualidade supersedida, resultado tardio de qualidade e setLocal/replaceTrack depois de stop/close. A política de escritores fecha objetos rejeitados sem usar os já transferidos ao motor.

### Resultados locais

- npm run check: sem erros nem avisos; worker tsc passou.
- npm test: a rodada anterior à última revisão passou com 238 testes; resultado da rodada final será registrado abaixo.
- cargo test --manifest-path src-tauri/Cargo.toml: **51 testes passaram**.
- npm run desktop:build: **PASS**, gerou src-tauri/target/release/Espia.exe e bundle/nsis/Espia_0.1.8_x64-setup.exe (1,62 MiB). A versão permanece 0.1.8: este é um build local, sem release ou instalação. O app continua apontando ao site publicado; testar o conjunto completo exige o site local atualizado no debug ou uma publicação futura autorizada.
- Aviso preexistente do Tauri: identificador dev.espia.app termina em .app, desaconselhado para macOS; o alvo deste projeto é Windows e o build terminou com sucesso.
- Mock /?mock=room&scene=grid: abriu no navegador; parar de assistir mostrou o cartão correto e Assistir restaurou o tile e controles. PiP está desabilitado no mock, sem mídia real; não foi atestado autoplay/A/V/PiP em transmissão real.

T002/T004 foram mantidas desmarcadas porque a fonte local e a configuração debug foram verificadas, mas a captura real/abertura do app debug em sessão não foi executada. T012/T027/T030 têm cobertura local parcial e continuam desmarcadas pelas verificações reais que faltam. A checklist de requisitos foi tratada como gate somente leitura e não alterada.

### Rastreabilidade final

| Requisito | Evidência local | Pendente |
|---|---|---|
| FR-001/002 | Testes de época/monotonicidade, descarte de frames velhos; preferência receiver | SC-001/002 em aparelhos reais |
| FR-003/004 | Reancoragem, fila ≤ 200 ms, mailbox latest frame, descarte PCM | SC-003 por perturbação real |
| FR-005 | Mesma época nos testes de startup/lacuna | SC-005 por instrumento A/V |
| FR-006 | Qualidade serial, modos existentes, retomada de target | SC-004 e transições reais |
| FR-007 | Sem mudança no governor global; operações por peer | Isolamento e limites nos espectadores reais |
| FR-008 | Fluxo e controles preservados; mock conferido | Transmissão real sem intervenção |
| FR-009 | Cancelamento JS/Rust, terminalidade e testes de lifecycle | SC-006 com encerramentos reais |

Nenhum SC-001–006 foi marcado como aprovado. Faltam T008, T013/T014, T022/T023, T028/T029 e partes externas de T002/T004/T012/T027/T030/T032. A feature permanece **implementada localmente, com aceitação real pendente**. Não houve deploy, alteração de protocolo, aumento de versão mínima nem mudança de permissões de produção.

## Fechamento desta execução

Rodada final: **21 arquivos / 239 testes TypeScript passaram**, `npm run check` sem erros/avisos, build Vite passou, **51 testes Rust passaram** e build release Windows/NSIS passou. `git diff --check` passou. A alteração TypeScript final não muda o executável nativo gerado, que continua carregando o site remoto conforme a configuração existente.

Contagem em tasks.md: **19 concluídas / 13 pendentes**. As pendentes são T002/T004 (validação real da fonte/debug), T008/T012/T013/T014 (baseline, compatibilidade instalada e comparação), T022/T023 (perturbações/encerramentos reais), T027/T028/T029/T030 (A/V/PiP, transições, aparelhos e recursos reais) e T032 (consolidação final após esses resultados). Elas não foram marcadas como completas por mocks ou testes unitários.

Nenhum hook before_implement/after_implement estava configurado: .specify/extensions.yml não existe. Não houve pedido de aprovação nem rejeição automática. O servidor Vite e as abas de teste abertos por esta execução foram encerrados. Nenhum app foi instalado nem serviço publicado.

## Publicação em produção — 2026-11-04

Autorização explícita da pessoa usuária: "suba em prod para fazer o teste real". Esta seção sucede o estado anterior de alterações somente locais.

- Versão do app: **0.1.9**, sincronizada em package.json/package-lock.json, src-tauri/tauri.conf.json, Cargo.toml e Cargo.lock. Sem alteração de dependências.
- Fluxo executado: npm run desktop:release → npm run build → npx wrangler deploy --dry-run → npm run deploy.
- Alvo verificado: Worker existente **espia**, configuração padrão de produção em wrangler.jsonc, origem **https://espia.espia.workers.dev**. Configuração/bindings remotos conferidos antes da publicação; credenciais, segredos e migrações não foram alterados.
- Versão Cloudflare anterior: b6fbc4de-f4c4-49df-8654-a3bdad2bfd50.
- Versão Cloudflare publicada: **60d9b5e3-4064-454e-8dff-1887bf16ac71**.
- Download público: **https://espia.espia.workers.dev/download/Espia-setup.exe**.
- Instalador: 1.702.161 bytes; SHA-256 **3fb50adba3d77eb850b7c18179241f0f551650bfa125f28faf581ae81dd8b312**.

### Verificação pós-publicação

O smoke em .wrangler/deploy-smoke-016.mjs verificou HTTP 200 e hash idêntico ao build local para index.html, bundles JS/CSS referenciados e instalador. Também criou uma sala rápida exclusiva de teste, conectou dois clientes sintéticos e confirmou welcome/peer-joined, ping/pong, repasse de sinalização, live com áudio window e unlive. Os clientes de teste foram fechados com código de saída explícita. Nenhuma sala existente foi usada e não houve captura de conteúdo de pessoas.

Resultado: **PASS** para entrega do site/app e sinalização em produção. A medição de atraso ponta a ponta permanece pendente para o teste real da pessoa usuária. Quem transmite deve sair do app pela bandeja e instalar a **0.1.9**, pois as correções Rust exigem novo executável. O site novo sozinho não atualiza o código nativo do app 0.1.8.
