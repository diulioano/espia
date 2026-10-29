# Research: Memória do app (009)

## R1 — Medir: régua única para todos os SC

- **Decision**: `scripts/memoria.ps1` acha o `Espia.exe` e soma os filhos dele (recursivo, via `Win32_Process.ParentProcessId`). Mostra a soma da memória **privada** de conjunto de trabalho (a coluna "Memória" da aba Processos do Gerenciador) e o detalhe por tipo (`--type=renderer`, `gpu-process`, `utility`, principal do motor, `Espia.exe`). Tem um modo `-Watch` (1 linha a cada 5 s) para ver a memória cair depois de parar.
- **Rationale**: a spec mede "o que o Gerenciador mostra". Somar só pelo nome `msedgewebview2.exe` contaria o WhatsApp e a Pesquisa do Windows, que usam o mesmo motor (medido: 2 árvores de outros apps).
- **Base medida (2026-10-29, v0.1.5)**:
  - **Prints do Gerenciador** (janela minimizada, numa sala, ninguém mais transmitindo): **243,3 MB** transmitindo e **228,3 MB** 2 min depois de parar. Página 104 → 94, GPU 89 → 84, Manager 27 → 26, utilitários + Crashpad ~24. **Esta é a régua.**
  - Uma medida anterior por conjunto de trabalho (~490 MB) contava memória compartilhada e foi descartada.
- **Conclusão**: parar já devolve a memória da transmissão. O custo é o motor carregado (~228 MB), e só descarregá-lo (US1) tira isso.
- **Alternatives**: medir no Gerenciador à mão (sem repetição, sem detalhe).

## R2 — O que sobra depois de "Parar"

Revisão do código atual:

| Peça | Hoje ao parar | Ação |
|---|---|---|
| Captura do navegador / câmera (`stopCapture`) | para as faixas | conferir na medição |
| Captura nativa (005): sessão WGC, pool, som | `Session::stop` fecha a sessão, o pool e o som | ok |
| Buffers compartilhados (3 × NV12, ~3 MB cada em 1080p) | `SlotSet::release` → `Close()`; a página chama `releaseBuffer` em `slots.clear()` | ok, conferir que a página não guarda referência |
| Aparelho D3D11 + `Pipeline` da 005 | cai com a `Video` | conferir com a medição (GPU) |
| Canais do Tauri (`frames`, `sound`) | os callbacks ficam registrados em `window` | **conferir**: se o `Channel` do `@tauri-apps/api` continua registrado depois do `capture_stop`, soltar no `stop()` (suspeita, não confirmada) |
| Codificador do WebRTC por pessoa | `mesh.setLocal(null)` → `replaceTrack(null)` | o codificador é liberado sem faixa. Conferir com a medição |
| `MediaStreamTrackGenerator` | `video.stop()` / `audio.stop()` | ok |

- **Decision**: revisar o código (tabela acima) e cobrir os caminhos além do botão (sair da sala e sala acabar transmitindo). Medir 5 ciclos para vazamento. **Sem** "memória baixa por 10 s depois de parar".
- **Rationale**: os prints mostram −15 MB ao parar pelo botão (SC-901 já atende). A memória baixa depois de parar daria pouco e conflitava com a regra da bandeja (`apply` voltava a `Normal` na hora; analyze I2). Removida.
- **Alternatives**: recarregar a página depois de parar. Descartado: derruba quem você está assistindo e pisca a tela.

## R3 — Modo de memória baixa do WebView2

- **Decision**: `ICoreWebView2_19::put_MemoryUsageTargetLevel`:
  - `LOW` com a janela **minimizada** ou **escondida** (numa sala, ou esperando os 30 s fora de sala);
  - `NORMAL` com a janela visível.
  - Nunca `LOW` enquanto está **ao vivo** (pode atrasar o codificador).
- **Rationale**: é a API feita para isso ("app inativo"). Os scripts e a mídia continuam rodando. Diferente do `TrySuspend`, que congela a página e derrubaria a sala.
- **Risco**: a documentação não garante nada sobre o som em `LOW`. Quem está minimizado ou escondido ouvindo a galera não pode perder o som (roteiro M6). **Plano B, já decidido**: se cortar, a regra devolve `Normal` para "minimizado/escondido numa sala" (o FR-904a e o FR-904c ficam sem efeito, e isso é registrado). A US1 (descarregar fora de sala) não depende disso.
- **Alternatives**: `TrySuspend` (congela: descartado numa sala, mas desnecessário fora dela, onde o motor é destruído); `SetProcessWorkingSetSize` nos processos do motor (só troca para o disco, a memória volta na hora e o Gerenciador engana).

## R4 — Como o app sabe se está numa sala

- **Decision**: a página avisa, com o comando `app_set_room(inRoom: bool)`:
  - `true` quando chega o `welcome` (entrou de fato);
  - `false` ao sair, quando a sala acaba, quando a sala falha de vez, e quando a rota da sala é desmontada.
  - O padrão no app é `false`: página nova, recarga e `PageLoadEvent::Started` zeram, como o `live`.
- **Rationale**: só a página sabe o estado real. O endereço `/s/xxx-xxx` vale também para o formulário de apelido e para a tela "a sala acabou".
- **Alternatives**: ler o endereço no Rust (sem emenda, mas erra nos casos acima); um comando único `app_set_state({live, inRoom})` (mudaria um contrato que já está publicado, e o site novo precisa funcionar com o app 0.1.4/0.1.5).
- **Compatibilidade**: site novo com app antigo → o `invoke` falha em silêncio (`.catch`), como no `reportLive`. App novo com site antigo → nunca chega `true` → escondido = solta o motor mesmo numa sala. Por isso o **site sobe antes** do app 0.1.6 (o app carrega sempre o site publicado).

## R5 — Soltar o motor e recriar; Job Object; Gerenciador

- **Soltar**: `WebviewWindow::destroy()` na janela `main`. O wry fecha o controller. Quando o último controller e o ambiente são soltos, o processo principal do motor sai e leva os filhos (alguns segundos). **Confirmar na etapa 4** com `memoria.ps1`: 0 processos do motor depois de 10 s. Se o Tauri segurar o ambiente, plano B: `ICoreWebView2Environment` explícito por janela, solto junto (ou `ICoreWebView2_..::BrowserProcessId` + esperar o processo sair).
- **Não sair sem janela**: hoje o Tauri sai quando a última janela fecha. Trocar `.run(ctx)` por `.build(ctx)?.run(|app, ev| …)` e, em `RunEvent::ExitRequested { code: None, api }`, chamar `api.prevent_exit()` quando a saída não veio do "Sair" (o "Sair" chama `app.exit(0)`, que traz `code: Some`).
- **Recriar**: `create_main(app, room)` é o mesmo builder de hoje, extraído. É chamado por:
  - clique no ícone da bandeja e "Abrir o Espia";
  - segunda abertura (single-instance), passando a sala do link `espia://`, se houver;
  - o `tauri-plugin-window-state`, que devolve posição e tamanho.
  - O `share_bar::install` roda de novo.
- **Esperar 30 s antes de soltar**, em todos os casos (inclusive sair da sala já escondido; analyze I1): quem esconde e reabre logo não espera a recarga. Mostrar a janela ou entrar numa sala cancela o timer.
- **Spike primeiro (analyze U1)**: antes de ligar tudo, criar e destruir a janela no debug e contar os processos com o `memoria.ps1`. Se o motor não sair em ≤ 10 s, investigar quem segura o ambiente (Tauri/wry) antes de seguir.
  - **Resultado (2026-10-29, build de debug)**: `window.destroy()` → o motor (7 processos) saiu em **< 2 s**; o `espia.exe` ficou vivo com **8 MB**. A segunda abertura (`espia.exe espia://s/k7f-q2x`) recriou a janela "Espia" no mesmo processo; o motor voltou em < 2 s (~80 MB no carregamento). Não precisou de plano B.
- **Job Object: descartado (analyze C1).** Com "matar ao fechar", qualquer programa aberto pelo Espia nasce dentro do job, inclusive o navegador aberto por um link externo (`opener`) quando ainda não estava aberto. Sair do Espia fecharia o navegador da pessoa (viola o FR-908). O processo principal do WebView2 já sai quando o app que o abriu morre. A conferência fica no roteiro M9. Só se sobrar processo: job com `JOB_OBJECT_LIMIT_BREAKAWAY_OK` e links externos abertos com `CREATE_BREAKAWAY_FROM_JOB`.
- **Gerenciador** (print): o Windows já agrupa os 8 processos do motor do Espia num grupo com o ícone do castor, chamado "WebView2 Manager" (o nome do processo principal do motor). Não há API para trocar o nome. Aceito (spec, FR-906). Fora de sala e escondido, o motor nem existe mais (US1).

## R6 — Flags do motor para gastar menos (opcional)

Candidatas, só se a medição mostrar ganho sem quebrar vídeo e som:
- `--disable-features=AudioServiceOutOfProcess` (um utilitário a menos, o som roda no processo principal do motor; testar o som da 005 e o da câmera);
- `--renderer-process-limit=1` (já é uma página só, provavelmente sem efeito);
- **não** usar `--disable-gpu` (o vídeo passaria a ser decodificado e codificado na CPU).

Hoje o app de release usa só os padrões do wry: `additional_browser_args` só existe em debug. Qualquer flag nova precisa repetir os padrões (`--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`).

## R7 — Memória da página ao assistir

- **Decision**: ao sair de uma sala, `closeMedia()` já fecha a malha. A rota da sala desmonta os `<video>`. Medir (M4) que assistir 10 min e sair volta para perto da base. Fora isso, nada específico: com a janela escondida fora de sala, o motor é destruído de qualquer forma.

## R8 — Ao vivo e minimizado

- **Ao vivo, nada muda** (FR-905), esteja a janela visível, minimizada ou escondida:
  - nem `LOW`, nem soltar o motor;
  - o timer de 30 s não começa;
  - ao parar escondido, ainda numa sala, vale `LOW`;
  - ao parar e sair da sala escondido, começa o timer (FR-904b).
- **Minimizado** (pedido de 2026-10-29: o print era com a janela minimizada): `LOW`, nunca descarregar, porque o botão na barra de tarefas precisa restaurar a janela. A detecção é `WindowEvent::Resized` + `window.is_minimized()`.
