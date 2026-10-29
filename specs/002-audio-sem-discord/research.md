# Research — 002 Som da transmissão sem vazar o Discord

## R1. Por que o Discord vazou

- **Decisão**: a causa é o pedido de "som do sistema" com a tela inteira. `src/lib/capture.ts` chama `getDisplayMedia` com `systemAudio: 'include'`. No Windows, o Chrome/Edge então oferece "Compartilhar áudio do sistema" no seletor da tela inteira, e esse som é a mistura de **tudo** que toca no PC, inclusive a chamada do Discord.
- **Evidência**: a documentação do Chrome ("Privacy-preserving screen sharing controls") avisa que o som do sistema inclui as vozes dos participantes remotos, que não deveriam ser mandadas de volta para eles. A recomendação para apps de chamada é não capturar o som do sistema.
- **Alternativas**: filtrar só o Discord do som do sistema. Não existe API para isso. `restrictOwnAudio` remove só o som **da própria página** que capturou, não de outros apps.

## R2. Tela inteira sem som

- **Decisão**: `systemAudio: 'exclude'`. Com isso o seletor oferece som só para aba e janela, nunca com a tela inteira (Chrome/Edge 105+).
- **Rede de segurança (FR-203)**: depois da captura, ler `videoTrack.getSettings().displaySurface`. Se for `'monitor'` e houver faixa de áudio, parar e remover a faixa **antes** de entregar para a malha (`track.stop()` + `stream.removeTrack`). Assim, mesmo que o navegador ignore a dica, nada sai.
- **Alternativas**: pedir `audio: false` sempre que a pessoa "talvez" escolha a tela inteira. Não dá, porque o site só sabe a superfície **depois** do seletor, e com `audio: false` a aba e a janela também perderiam o som.

## R3. Janela com som só dela

- **Decisão**: `windowAudio: 'window'` (Chrome 141+ estável, inclusive Windows). Ao compartilhar uma janela, o seletor oferece o som **daquela janela** (ex.: o jogo).
- **Valores possíveis**: `'exclude'` (janela sem som), `'window'` (som só da janela) e `'system'` (som do sistema junto com a janela, que é o que **não** queremos). É uma dica: o navegador pode ignorar.
- **Rede de segurança (FR-204)**: o site não distingue pela API se a faixa veio "da janela" ou "do sistema". Heurística: o rótulo da faixa de áudio (`track.label`). Hoje o Chrome usa "Tab audio" para aba e "System Audio" para o sistema. Se o rótulo indicar sistema com superfície `window`, descartar como no R2. **A confirmar no teste manual**: registrar o rótulo que o Edge/Chrome dá ao som de janela (T-Q1 do quickstart). Se não houver rótulo confiável, manter a regra "superfície `monitor` → descarta" e confiar na dica para janelas.
- **Navegador sem `windowAudio`** (< 141): no Windows, compartilhar janela historicamente não oferecia som nenhum, então cai naturalmente em "sem som" (FR-204). Opções desconhecidas no dicionário são ignoradas pelo navegador, sem erro.

## R4. Troca de superfície no meio da transmissão

- **Decisão**: manter `surfaceSwitching: 'include'` (botão "compartilhar esta aba" do navegador) e reavaliar a regra quando a faixa de vídeo disparar `configurationchange`, ou quando faixas de áudio forem adicionadas ou removidas do stream. A reavaliação reaplica R2/R3 e atualiza o rótulo anunciado à sala (FR-205).
- **Observação**: a troca do navegador, na prática, é **entre abas** (a superfície continua `browser`). O caso "trocou para tela inteira no meio" é tratado do mesmo jeito, por segurança.

## R5. Protocolo: tipo de som no `live`

- **Decisão**: `live.audio` deixa de ser `boolean` e passa a ser `'tab' | 'window' | 'none'`. O Worker valida e repassa em `peer-live`.
- **Compatibilidade**: uma aba aberta antes do deploy ainda manda `audio: boolean`. O Worker aceita os dois formatos e converte: `true → 'tab'` e `false → 'none'`. Para quem assiste, o que importa é "com som / sem som". O formato antigo pode ser removido depois de um tempo.
- **Alternativas**: um campo novo (`audioSource`) ao lado do booleano. Rejeitado porque gera dois campos que podem se contradizer.

## R6. Onde fica cada texto

| Lugar | Texto | Requisito |
|---|---|---|
| Modal Transmitir, sob "Compartilhar áudio" | "Vai o som da janela ou aba que você escolher — Discord e outros apps ficam de fora. Tela inteira vai sem som." | FR-206 |
| Card "Você está ao vivo" | "… · com som da aba" / "com som da janela" / "sem som" / "sem som (tela inteira)" | FR-207 |
| Toast ao começar tela inteira com som ligado | "Tela inteira vai sem som. Pra levar o som, transmita a janela do jogo ou uma aba." | FR-208 |
| Subtítulo de quem assiste | "1080p · 60 fps · com som" / "· sem som" | FR-209 |
| Toast da rede de segurança (som descartado) | mesmo texto do FR-208 | FR-203 |

## Fontes

- [Chrome — Privacy-preserving screen sharing controls](https://developer.chrome.com/docs/web-platform/screen-sharing-controls)
- [Chrome 141 release notes (windowAudio)](https://developer.chrome.com/release-notes/141)
- [blink-dev — Intent to Ship: windowAudio](https://www.mail-archive.com/blink-dev@chromium.org/msg14395.html)
- [W3C TAG review — windowAudio](https://github.com/w3ctag/design-reviews/issues/1110)
- [MDN — getDisplayMedia()](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getDisplayMedia)
- [Screen Capture spec](https://w3c.github.io/mediacapture-screen-share/)
- [blink-dev — Intent to Ship: restrictOwnAudio](https://groups.google.com/a/chromium.org/g/blink-dev/c/5SZfg29iwPc)
