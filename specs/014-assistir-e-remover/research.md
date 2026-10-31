# Research — 014 Assistir quando quiser, telas que dormem e remover pessoas

## R1. Onde está o custo de quem transmite

**Decision**: atacar três coisas, nesta ordem de ganho esperado:

1. vídeo **recebido** de quem a pessoa não está olhando (decodificar custa GPU/CPU mesmo sem desenhar);
2. o app desktop desenhando com a janela minimizada ou na bandeja (ver R2);
3. a prévia ao vivo na barra (desenho a 60 fps de um `<video>` local).

**Rationale**: o `ViewMode` da 010 já pausa o envio por pessoa (`sendParams(...,'hidden')` → `active: false` na codificação daquela conexão). Como a malha tem uma conexão por pessoa, "escondida" corta o vídeo só daquele par, e o som continua no transceiver de áudio. Basta a página decidir `hidden` em mais casos (US1 e US2): quem transmite economiza uma codificação, e quem assiste economiza a decodificação.

**Alternatives considered**: desligar o `track.enabled` do lado de quem assiste. Isso não economiza nada: o vídeo continua chegando e sendo decodificado. Fechar e reabrir o transceiver exige renegociação e é mais lento que os 3 s do FR-004.

## R2. "Fora de vista" no site e no app

**Decision**: a página usa `document.visibilityState` com o evento `visibilitychange`, e nada mais (sem `blur`/`focus`, por causa do FR-007).

- **Site**: aba de trás, janela minimizada e tela do celular apagada já dão `hidden` no Chrome/Edge/Safari.
- **App**: é preciso confirmar (**tarefa de spike, a primeira da US2**). Com a janela minimizada ou depois de `window.hide()` (bandeja), o WebView2 só reporta `hidden` se o controller estiver com `IsVisible = false`. Não está garantido que o Tauri/wry faça isso.
  - Se o spike mostrar `visible`: o `engine.rs`, que já recebe `WindowMode::{Visible, Minimized, Hidden}` (009), passa a chamar `controller.SetIsVisible(false/true)` junto do `MemoryUsageTargetLevel`.
  - É só Rust e não cria API para a página, então **não precisa de emenda**. Isso pede app 0.1.8. O site funciona com o app antigo: só não economiza no app minimizado.
  - Isso também explicaria parte do FPS perdido: o app minimizado continuaria desenhando as telas.

**Risco**: com `IsVisible = false`, o WebView2 freia os timers da página (1 por segundo). O som nativo chega por `PostWebMessageAsJson` (010, evento, não timer) e o envio continua no motor de WebRTC. Mesmo assim, o roteiro manual confere o som com o app na bandeja por 5 minutos (quickstart M3). Os timers da store que importam com a página escondida são o `watchSend` de 1 s e o refresh de TURN, e os dois aceitam ser freados.

**Alternatives considered**:
- `blur`: descartado pelo Q2.
- Um comando novo `app_set_visible` da página: desnecessário, porque o Rust já sabe o estado da janela, e exigiria emenda.

## R3. Picture-in-picture com a página escondida

**Decision**: o tile avisa a store quando entra ou sai do PiP (eventos `enterpictureinpicture` / `leavepictureinpicture` do `<video>`) por uma ação `pip(id | null)`. A store mantém `pipId`. Com a página escondida, `pipId` continua `full`. Parar de assistir a tela do PiP fecha o PiP (`document.exitPictureInPicture()` no tile, quando o `view.watching` vira `false`).

## R4. Escolha de assistir: chave e duração

**Decision**: `unwatched: Map<peerId, startedAt>` na store da sala.

- Uma transmissão é "a mesma" enquanto o `startedAt` do `live` não muda. O servidor preserva o `startedAt` entre `live` repetidos (troca de qualidade) e em queda/retomada, porque o `live` vive no membro, que sobrevive como fantasma.
- `peer-live` com `live: null` ou outro `startedAt`, ou `peer-left`, apaga a entrada (FR-005: volta a assistir sozinho na próxima transmissão).
- A store é a mesma durante uma queda (011), então a escolha sobrevive à reconexão sem guardar nada em disco. Nada vai pro `localStorage` (princípio III: é escolha de momento, não preferência).

## R5. `myViews()` com as regras novas

**Decision**: ordem de precedência, por transmissão `s` (função pura nova `viewModes()` em `room-view.ts`, testável):

1. `s` não está sendo assistida → `hidden`;
2. página escondida e `s.id !== pipId` → `hidden`;
3. celular: selecionada → `full`, outras → `hidden` (como hoje);
4. layout foco: focada → `full`, outras → `thumb`; grade/única → `full` (como hoje).

A própria tela no palco (R6) nunca entra no mapa. O teto de qualidade (`countFullViewers`) já conta só os `full`, então quem parou de assistir ou escondeu a janela libera qualidade para os outros sem código novo.

**"N assistindo"**: hoje `viewers` = conexões de pé (`connectedCount`). Passa a ser quantas pessoas me veem com modo ≠ `hidden` (`theirViews()`), atendendo o cenário US1-3.

## R6. Prévia parada e a própria tela no palco

**Decision (prévia)**: a cada 5 s, com a página visível, a store pega um quadro do track de vídeo da própria transmissão com `ImageCapture.grabFrame()`. Isso funciona em qualquer `MediaStreamTrack` de vídeo no Chromium, inclusive o `MediaStreamTrackGenerator` do app (005). O quadro é reduzido para 320 px de largura num `OffscreenCanvas` e entregue como `ImageBitmap` em `myBroadcast.snapshot`. O bitmap anterior é fechado (`close()`) para não vazar memória. O `StreamTile` ganha o modo "still": desenha o bitmap num `<canvas>` em vez de montar um `<video>`.

- **Fallback**: sem `ImageCapture` (Safari, que só transmite câmera no celular), `MediaStreamTrackProcessor` lê um quadro de um clone do track e o fecha. Se nenhum dos dois existir, a prévia mostra o placeholder "sua tela" (sem vídeo).
- Primeira imagem: logo ao começar a transmitir (≤ 1 s), para não ficar 5 s vazia.

**Decision (palco)**: `selfOnStage: boolean` na store. Quando é `true`, `view.streams` ganha uma entrada `{ id: <meu id>, you: true, stream: <meu stream>, muted: true }` e `focusedId = meu id`.

- Sai de `true` com Esc, foco em outra tela, outro clique na prévia ou o fim da transmissão.
- Com a página escondida, o navegador já não desenha nada. O `<video>` local não decodifica (é um stream local), então não precisa de regra extra. O FR-011 é atendido sem mudar `selfOnStage`.
- Fica fora de `viewModes()` (FR-012) e de qualquer mensagem.

**Alternatives considered**: um `<video>` escondido fora do DOM, copiado para o canvas. Funciona, mas mantém um elemento de mídia vivo o tempo todo. O `grabFrame` só toca o track a cada 5 s.

## R7. Remover pessoa: transporte e prova de dono

**Decision**: HTTP, igual a renomear/apagar da 011. `POST /api/rooms/:code/remove` com `Authorization: Bearer <ownerKey>` e corpo `{ "peerId": "…" }`, passando pelo mesmo `ownerAction` (mesma origem, `RL_CREATE`, hash da chave). Chama a RPC nova do DO, `removePerson(ownerHash, peerId)`.

**Rationale**: reaproveita a validação de dono que já existe e foi testada, sem mexer no parser do WebSocket nem mandar a chave por ele.

**No DO**:

1. confere o dono (`checkOwner`);
2. acha o membro conectado e/ou o fantasma com aquele id;
3. no socket conectado: marca o attachment como `joined: false` (o `webSocketClose` não cria fantasma), manda `{ t: 'removed' }` e fecha com `CLOSE.removed = 4403`;
4. tira do `roster`/fantasmas (`dropMembers`), avisa todos com `peer-left` e revoga o TURN da pessoa;
5. devolve `'ok'` mesmo se a pessoa já tinha saído (idempotente). Não guarda nada sobre quem foi removido (FR-017).

Remover o próprio dono: o servidor não sabe qual `peerId` é do dono (a chave não está ligada a uma conexão). A tela nunca oferece isso. Se alguém forçar, o efeito é igual a "Sair da sala", sem dano.

**Mesma pessoa em vários lugares**: duas abas do mesmo navegador dividem a identidade (`resumeToken` no `localStorage`, commit 1105657), então só uma está conectada e a outra já cedeu. App e navegador são pessoas diferentes na lista (identidades diferentes) e o dono remove cada uma. O cenário US4-6 fica atendido assim, e a spec é ajustada na tarefa de acabamento.

**No cliente removido**: `signaling.ts` trata `removed` / 4403 como fim `'removed'`, sem reconectar, e apaga a sessão guardada (`resumeToken`). A `RoomRoute` volta pro início com o toast "O dono te tirou da sala". Uma pessoa removida no meio de uma queda tenta `resume`, recebe `resume-failed` e entra de novo como pessoa nova. Isso é aceito, porque o FR-017 diz que remover não bloqueia: a remoção já fez efeito (ela saiu da lista e a transmissão acabou).

**Compatibilidade**: um cliente antigo removido recebe 4403, que ele não conhece. Ele trata como queda e reconecta com `resume`, que falha, e então entra como pessoa nova. Aceitável, porque toda aba recarrega na próxima publicação.

## R8. Celular

**Decision**: o `MobileRoomPage` ganha o mesmo controle "Parar de assistir" / "Assistir" na tela selecionada. As outras já ficam `hidden` (só chips). A prévia parada vale no celular que transmite câmera. A própria tela no palco não existe no celular (não há barra lateral): aceito, e anotado na spec no acabamento.
