# Relatório — 2026-10-29 · Espia v0.1.3 (seletor de tela nativo no app)

## O que foi pedido

No app desktop, o seletor "Escolha o que compartilhar" deve seguir o design (`tela-esperada.png`): grade com as janelas reais, miniaturas e selo "tocando som", e nenhuma segunda escolha no seletor do Windows. No site, continua o seletor do Chrome. Na tela inteira, vai o som do PC **menos o Discord** (Q1: B).

## Como ficou

| Onde | O que a pessoa vê | Som que vai pra sala |
|---|---|---|
| **App** · Janelas | Grade com miniaturas (a cada 2 s), ícone, programa, "tocando som" e ✓ na escolhida. Transmitir começa **direto**. | Só o programa da janela ("Só o som de Hollow Knight — nada de notificação vazando."). |
| **App** · Telas inteiras | Um cartão por monitor ("Tela 1 · Principal"). | O PC inteiro **menos o Discord** ("Todo o som do computador vai junto — menos o Discord."). |
| **App** · tela + 2 Discords abertos | Aviso "Mais de um Discord aberto — tela inteira vai sem som." | Nenhum. O Windows só deixa excluir um programa. |
| **App** · Windows sem som por processo | Aviso "Seu Windows não deixa separar o som da janela — foi só a imagem." | Nenhum. Nunca cai para o som do PC com o Discord junto. |
| **Site** (Chrome/Edge) | Igual à v0.1.2: cartão "Você escolhe a janela no próximo passo" + o seletor do navegador. | Regras da 002/004. |
| **App v0.1.2 + site novo** | Igual à v0.1.2 (a página não acha a ponte). | Regras da 002/004. |

Outros comportamentos:

- Se a captura nativa falhar, o app avisa e volta para o seletor do Windows na próxima tentativa.
- Se a janela escolhida fechar durante a transmissão, a transmissão acaba sozinha.
- Se os primeiros quadros saírem pretos (jogo em tela cheia exclusiva), o app sugere o modo janela/sem bordas.
- O seletor lembra a última janela transmitida, se ela ainda estiver aberta.

## Como funciona por dentro (resumo)

- **Escolha da fonte**: o WebView2 não deixa a página escolher a fonte (o `ScreenCaptureStarting` só aceita ou cancela). Por isso quem lista e captura é o app em Rust (`src-tauri/src/capture/`):
  - `EnumWindows`/`EnumDisplayMonitors` + `PrintWindow`/WIC para as miniaturas;
  - Windows.Graphics.Capture + `ID3D11VideoProcessor` (NV12 na GPU);
  - loopback de processo do WASAPI (incluir a árvore da janela, ou excluir a árvore do Discord);
  - um vigia de 1 s para quando o Discord abre ou reinicia.
- **Do app para a página**: 3 buffers compartilhados do WebView2 + um `Channel` do Tauri por quadro. A página monta `VideoFrame` → `MediaStreamTrackGenerator` (e `AudioData` para o som), e o stream entra na **mesma** malha P2P de antes (`src/lib/native-capture.ts`).
- **Segurança** (constituição 1.2.0):
  - 5 comandos, liberados por capability **só** para `https://espia.espia.workers.dev` (e para o Worker local em debug), só na janela `main`;
  - cada comando confere a origem de novo;
  - as miniaturas só são geradas com o Espia em foco;
  - nada sai do PC além do vídeo/som que já ia pela malha.
- **Protocolo**: `AudioKind` ganhou `'pc'` (o Worker aceita). Quem assiste vê "· com som".

## Diferenças em relação ao design

1. O botão continua **"Transmitir"** (decisão de 2026-10-28), não "Go Live".
2. Os ícones dos programas vêm do próprio Windows, não das letras do design. A letra aparece só quando o programa não tem ícone (e nos monitores).
3. Janela minimizada ou que o Windows não consegue desenhar mostra o ícone grande no lugar da miniatura.

## Testes

- `npm run check`: 0 erros. `npm test`: **85 passando**. Os novos cobrem:
  - a ponte com uma ponte falsa (detecção e fallback, consulta da lista a cada 2 s, slots e gerações, quadros → VideoFrame, som → AudioData com tempo contínuo, vigia do Discord tirando a faixa, fim da fonte, qualidade no ar, erro ao começar);
  - os textos de som;
  - `'pc'` no Worker.
- `cargo test`: **34 passando** (filtro de janelas, raiz da árvore de processos, raízes do Discord, plano de som, tamanhos/letterbox/NV12, miniatura). `cargo clippy --all-targets`: 0 avisos.
- Mocks conferidos no Edge headless contra `tela-esperada.png`: `?mock=room&scene=pickerwindows`, `pickerscreens`, `pickerempty`, `pickerfallback`, `livepc`.
- **Não testado: a captura real.** Testar exigiria capturar a sua tela de verdade, que é justamente o que a memória do projeto proíbe nos testes automáticos. Faltam:
  - o roteiro manual N1–N8 (`quickstart.md`), com o Discord ligado e dois monitores se der;
  - a medição de CPU a 1080p60 (T549; o console mostra `[espia] captura nativa:` com enviados/descartados a cada 10 s).
- **Não refeito**: o E2E de 18 casos da 004 (T542). O script daquela sessão não está no repositório. O modo do site é o mesmo código da 004 (mock `pickerfallback` igual ao `modal`), e os testes de unidade dele continuam passando.

## Tamanho

Instalador `Espia_0.1.3_x64-setup.exe`: **1,51 MB** (1,44 MiB), +0,32 MB (v0.1.2: 1,13 MB).

## ⚠️ O que precisa de você

1. ~~**Publicar o site**~~ — feito em 2026-10-29 (versão do Worker `e1b24e47`). O seletor nativo é parte do site: sem o deploy, o app 0.1.3 continua mostrando o seletor da v0.1.2.
2. **Commit + tag `v0.1.3`**. Não fiz commit nem push: me diga se é para fazer.
3. **Teste manual N1–N8** com a galera:
   - N4: janela com som + chamada do Discord → ninguém ouve o Discord;
   - N5/N6: tela inteira com Spotify + Discord, inclusive reiniciando o Discord;
   - N7: dois Discords.
4. **Windows 10**: se alguém tiver, vale conferir o som por processo (pode cair no aviso "Seu Windows não deixa separar o som…") e a borda amarela da captura.
