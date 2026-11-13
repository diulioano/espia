//! Redutor da sala (specs/019, R4, T021): o estado e as decisões de `src/lib/stores/room.svelte.ts`,
//! sem rede, mídia, tela nem relógio.
//!
//! # Desenho
//!
//! ```text
//! reduce(estado, evento, agora_ms) -> (estado, efeitos)
//! view(&estado, agora_ms) -> RoomView
//! ```
//!
//! O binário `espia` é o único dono do `RoomState`. Ele roda um laço só: recebe um `Event`, chama
//! `reduce`, guarda o estado novo, executa os `Effect` na ordem em que vieram e entrega `view()` à
//! tela. Nada aqui fala com `espia-net`, `espia-media`, `espia-capture` nem `espia-shell`: essas
//! crates produzem eventos e executam efeitos, e esta crate não as conhece.
//!
//! ## Regras
//!
//! 1. **Pura**: mesmo estado, mesmo evento e mesmo `agora_ms` dão o mesmo resultado. Sem relógio
//!    escondido, sem sorteio (ids de pedido chegam dentro do evento), sem entrada e saída.
//! 2. **Tempo por temporizador nomeado**: o redutor nunca espera. Ele emite
//!    `Effect::Schedule { timer, after_ms }` e `Effect::Cancel { timer }`; o laço devolve
//!    `Event::Timer(timer)` quando vence. Agendar um temporizador que já existe o substitui.
//! 3. **Mídia por identificador opaco**: vídeo e som aparecem só como `StreamId`/`SnapshotId`
//!    (`view_model`). Quem tem o quadro de verdade é o laço.
//! 4. **Evento desconhecido ou fora de hora não quebra**: é ignorado, devolvendo o estado igual
//!    e nenhum efeito (por exemplo `peer-left` de quem não está na lista).
//! 5. **A regra do Assistir vale nos dois sentidos** (FR-021): só chega vídeo de quem a pessoa
//!    escolheu assistir, e só sai vídeo para quem mandou `ViewMessage` válida. Abrir, fechar,
//!    aproximar ou destacar uma tela nunca muda `watching`, `stopped` nem os modos de `view_modes`.
//!
//! ## Estado (`state.rs`)
//!
//! `RoomState` espelha os campos de `RoomStore`, agrupados:
//!
//! - **Ligação com o servidor**: `connected`, `ended`, `link` (ok, reconectando, falhou), sessão.
//! - **Sala e pessoas**: `room_info`, `capacity`, `me`, `peers`, servidores STUN.
//! - **O que eu vejo**: `focused_id`, `watching`, `stopped`, `sent_modes`, `media` por pessoa
//!   (tem vídeo, tem som, estado da ligação), `timed_out`, `audio` (mudo e volume por pessoa),
//!   `visible` (janela à vista), `floating_id` (janela flutuante), `self_on_stage`.
//! - **O que eu transmito**: `my_broadcast`, `their_views`, `capture_request`,
//!   `highest_broadcast_seq`, `cpu_steps` e o `CpuGovernor`, preferências (`go_live_prefs`).
//! - **Seletor de transmissão**: aberto ou não, aba, fontes, escolhida, câmeras e entradas de som.
//! - **Avisos e diálogos**: `toasts`, `copied`, `permission_denied`, menu do dono, diálogos de
//!   nome, apagar e tirar da sala, `deleting`.
//!
//! ## Eventos (`event.rs`)
//!
//! - `Server(ServerMessage)`: mensagem do servidor, já decodificada.
//! - `Link(LinkEvent)`: estado da sinalização (abrindo, aberta, reconectando, voltou, sessão nova,
//!   fim com o motivo).
//! - `Peer(PeerEvent)`: da ligação com cada pessoa (estado da ligação, chegou ou sumiu vídeo ou
//!   som, quadros chegando ou não, `ViewMessage` recebida, estatística de envio "limitado pelo
//!   processador").
//! - `Capture(CaptureEvent)`: da captura (fontes listadas, começou, falhou, acabou sozinha, tela
//!   preta, aviso de som, quadro parado novo) e das câmeras.
//! - `Api(ApiEvent)`: resposta de renomear, apagar e tirar da sala.
//! - `Action(RoomAction)`: o que a pessoa fez na tela (`view_model::RoomAction`).
//! - `Window(WindowEvent)`: janela à vista ou escondida, janela flutuante aberta ou fechada.
//! - `Timer(TimerId)`: um temporizador venceu.
//!
//! ## Efeitos (`effect.rs`)
//!
//! - **Servidor**: `Send(ClientMessage)`, `Leave`, `RetryNow`, `Api(ApiCall)`.
//! - **Ligações**: `OpenPeer`, `ClosePeer`, `SetIceServers`, `SendView { to, message }`,
//!   `SetSendParams { to, params }`, `SetCaptureQuality`.
//! - **Captura**: listar fontes (começar e parar), começar e parar a captura, câmeras.
//! - **Aparelho**: `CopyInvite(String)`, `Save(DeviceChange)` (apelido, preferências, volume,
//!   sessão, lembrar, renomear e esquecer sala, esquecer chave de dono).
//! - **Tempo**: `Schedule`, `Cancel`.
//! - **Rota**: `Navigate` (sair da sala, sala apagada, removido).
//!
//! Aviso na tela não é efeito: é estado (`toasts`), com um temporizador para sumir.
//!
//! ## O que não entra aqui
//!
//! A negociação entre pessoas (ofertas, respostas, candidatos) fica em `espia-media` (T037): o
//! redutor só repassa `signal` como efeito e evento. Tela cheia do celular e tudo que é só de
//! celular ficam no site (R17).
//!
//! ## Notas de implementação
//!
//! O que a implementação decidiu onde o site não coube no desenho acima (T021):
//!
//! - **Identificador de pedido sem sorteio.** O site sorteia `crypto.randomUUID()` em `sendLive`. Aqui o
//!   redutor guarda um identificador de reserva (`spare_request_id`): `CaptureEvent::Started` traz o
//!   primeiro e o redutor pede o seguinte com `Effect::NewRequestId`; a resposta é
//!   `CaptureEvent::RequestId`. Sem reserva na hora de anunciar (por exemplo, `welcome` de quem voltou
//!   como pessoa nova), `live_pending` fica ligado e o anúncio sai quando o identificador chegar.
//! - **`their_views` é derivado.** O site copia `mesh.theirViews()` a cada callback; aqui o estado só
//!   guarda `their_view` em cada ligação e `RoomState::their_views()` filtra as que estão de pé.
//! - **Revisões e chaves da ligação** (`PeerLinkState`) são o que o `PeerLink` do site guarda e que muda o
//!   que sai e o que entra: `my_view`, `view_revision`, `their_view`, `grant_revision`, `remote_key`.
//!   A mensagem do canal `ctl` ganhou aqui o codec (`ViewMessage::to_json`/`decode`, `view_sync_json`,
//!   `decode_view_sync`) porque os testes do site (`view-modes.test.ts`) cobrem `encodeView`/`decodeView`.
//! - **Temporizadores** viram `Schedule`/`Cancel`. Fechar um aviso na mão cancela o temporizador dele (o
//!   site deixa o `setTimeout` correr; o efeito na tela é o mesmo).
//! - **`viewer_stream`** recebe o id da tela cheia por parâmetro (o `RoomView` do app não tem `fullscreenId`).
//! - **Sair de propósito esquece a identidade** (`signaling.leave()` do site chama `onSession(null)`):
//!   o redutor emite `Save(Session { session: None })` junto com `Leave`.
//! - **Sem fonte escolhida, `StartGoLive` não faz nada** e a grade segue listando; o site cairia no seletor
//!   do navegador, que o app não tem.
//!
//! Do `RoomStore` que **não** foi portado, e por quê (a fase 4 completa):
//!
//! - Caminho de captura do navegador (`startCapture`, `applyAudioRule`, `warnScreenAudio`, `reapplyAudio`,
//!   `SCREEN_NO_AUDIO`, `pickerMode`, `detectNative`): o app só tem a captura nativa.
//! - Tudo de celular: `mobile`, `fullscreenId`, `keepFullscreen`, `camFacing`/`setFacing`, `muteTimer`.
//! - `turn-refresh` e `scheduleTurnRefresh` (o app só usa STUN e nunca manda `turn-refresh`).
//! - Dentro da malha (`mesh.ts`, `peer.ts`, T037): negociação, troca da ligação quando o certificado do
//!   outro lado muda, rota por retransmissão (`policyRebuilt`), `attach`/`setEncoding`/preferência de
//!   codec, filas de `getParameters`, `contentHint`, coleta de estatísticas e diagnóstico
//!   (`espiaMediaDiagnostics`, o log de envio a cada 10 s).
//! - `statsTimer` de 1 s e `SNAPSHOT_MS`: quem conta o intervalo é o laço (`PeerEvent::SendStats` e
//!   `CaptureEvent::Snapshot`); o redutor só liga e desliga (`Effect::Snapshots`).
//! - Corridas de captura (`broadcastEpoch`, `qualityRevision`, `qualityQueue`): o laço serializa os
//!   efeitos; as respostas atrasadas morrem em `pending_start`/`my_broadcast` (ver `capture_started`).
//! - `refreshDevices` com reabertura quando os nomes das câmeras vêm vazios e `watchDevices`: o laço
//!   responde a `Effect::ListCameras` e repete quando o Windows avisa que os aparelhos mudaram.
//! - Leitura de `localStorage` (`loadOwnerKey`, `loadVolume`, `loadSavedRooms`): vira `RoomInit` e
//!   `Effect::Save(DeviceChange::..)`.

mod effect;
mod event;
mod reduce;
mod state;
mod view;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_view;

pub use effect::*;
pub use event::*;
pub use reduce::reduce;
pub use state::*;
pub use view::*;
