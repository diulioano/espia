//! O redutor da sala: `reduce(estado, evento, agora_ms) -> (estado, efeitos)`.
//! Porte do estado e das decisões de `src/lib/stores/room.svelte.ts`. Cada chamada a rede, mídia,
//! captura, armazenamento, área de transferência e `setTimeout` da store virou um `Effect`, e cada
//! retorno assíncrono virou um `Event`. Os `$effect` do Svelte viraram `R::settle`, que roda no fim
//! de todo evento.

use std::collections::BTreeSet;

use crate::device::{GoLive, Surface};
use crate::devices::{audio_role, is_capture_card, preview_ids, suggest_audio, DeviceInfo, PreviewDevice};
use crate::live_label::{live_label, LabelInput};
use crate::media_policy::{
    direct_ice_servers, is_safe_sequence, same_transmission, BroadcastOrder, TransmissionKey, MAX_SAFE_INTEGER,
};
use crate::protocol::{ClientMessage, IceServer, LiveInfo, LiveSource, Peer, RoomInfo, ServerMessage, SignalData};
use crate::quality::{send_params, step_down, wants_sound, wants_video, Step, ViewMode};
use crate::view_model::{
    CameraPermission, CaptureSource, DeleteDialogView, LinkState, NameDialogMode, NameDialogView, RoomAction, ScreenSurface,
    StreamId, ToastKind, ToastView,
};
use crate::watch_state::{choose_watch, prune_watching};

use super::effect::{ApiCall, CaptureTarget, Destination, DeviceChange, Effect};
use super::event::{
    ApiEvent, CaptureEvent, Event, LinkEvent, PeerEvent, SendReading, StartFailure, TimerId, ViewMessage, WindowEvent,
};
use super::state::{
    owner_error_text, AudioPref, AudioWarn, EndReason, LiveRequest, MyBroadcast, PeerLinkState, PendingStart,
    RemoveDialog, RoomState, BLACK_WARN, COPIED_MS, DEFAULT_VOLUME, FIRST_FRAME_TIMEOUT_MS,
    RECONNECT_FAIL_MS, TOAST_MS,
};
use super::view::{stage_layout, view, view_modes, ViewModesInput};

type Modes = std::collections::BTreeMap<String, ViewMode>;

/// Aplica um evento ao estado da sala. Pura: sem relógio escondido, sem sorteio, sem entrada e saída.
/// Evento fora de hora (de quem não está na lista, depois de sair, depois de a sala acabar) é
/// ignorado: o estado volta igual e nenhum efeito sai.
pub fn reduce(state: RoomState, event: Event, now_ms: u64) -> (RoomState, Vec<Effect>) {
    let mut r = R { s: state, fx: Vec::new(), now: now_ms };
    if r.s.left {
        return (r.s, r.fx);
    }
    // Acabou a sala: só dá para sair dela.
    if r.s.ended.is_some() && !matches!(event, Event::Action(RoomAction::Leave | RoomAction::GoHome)) {
        return (r.s, r.fx);
    }
    match event {
        Event::Server(msg) => r.server(msg),
        Event::Link(e) => r.link_event(e),
        Event::Peer(e) => r.peer_event(e),
        Event::Capture(e) => r.capture_event(e),
        Event::Api(e) => r.api_event(e),
        Event::Action(a) => r.action(a),
        Event::Window(e) => r.window_event(e),
        Event::Timer(t) => r.timer(t),
    }
    r.settle();
    (r.s, r.fx)
}

struct R {
    s: RoomState,
    fx: Vec<Effect>,
    now: u64,
}

impl R {
    fn push(&mut self, e: Effect) {
        self.fx.push(e);
    }

    fn schedule(&mut self, timer: TimerId, after_ms: u64) {
        self.push(Effect::Schedule { timer, after_ms });
    }

    fn cancel(&mut self, timer: TimerId) {
        self.push(Effect::Cancel { timer });
    }

    // ---------- Avisos ----------

    fn toast(&mut self, kind: ToastKind, text: impl Into<String>, action: Option<&str>) -> String {
        self.s.toast_seq += 1;
        let id = format!("t{}", self.s.toast_seq);
        self.s.toasts.push(ToastView { id: id.clone(), kind, text: text.into(), action: action.map(str::to_string) });
        if kind != ToastKind::Error {
            self.schedule(TimerId::Toast(id.clone()), TOAST_MS);
        }
        id
    }

    fn dismiss(&mut self, id: &str) {
        self.s.toasts.retain(|t| t.id != id);
        self.s.toast_peer.remove(id);
    }

    fn copy_invite(&mut self, text: &str) {
        let url = self.s.invite_url();
        self.push(Effect::CopyInvite(url));
        self.s.copied = true;
        self.toast(ToastKind::Copied, text, None);
        self.schedule(TimerId::Copied, COPIED_MS);
    }

    // ---------- Ligação com o servidor ----------

    /// Caiu (ou "Tentar de novo"): `Reconnecting` e, sem volta em 60 s, `Failed`.
    fn mark_down(&mut self) {
        self.s.link = LinkState::Reconnecting;
        self.schedule(TimerId::LinkDown, RECONNECT_FAIL_MS);
    }

    fn link_event(&mut self, e: LinkEvent) {
        match e {
            LinkEvent::Connecting | LinkEvent::Open => {}
            // Queda não é sair (specs/011 US1): a sala continua na tela, com o aviso no palco.
            LinkEvent::Reconnecting => {
                if self.s.connected && self.s.link == LinkState::Ok {
                    self.mark_down();
                }
            }
            LinkEvent::Back => {
                if self.s.link == LinkState::Ok {
                    return;
                }
                self.cancel(TimerId::LinkDown);
                self.s.link = LinkState::Ok;
                self.toast(ToastKind::Copied, "De volta!", None);
            }
            LinkEvent::Session(session) => {
                self.s.session = session.clone();
                let code = self.s.code.clone();
                self.push(Effect::Save(DeviceChange::Session { code, session }));
            }
            LinkEvent::Ended(reason) => {
                self.close_media();
                self.stop_broadcast(false);
                // Eu mesmo apaguei a sala: volto pro início, sem o aviso de "apagaram a sala".
                if reason == EndReason::Deleted && self.s.deleting {
                    self.leave_after_delete();
                } else {
                    self.s.ended = Some(reason);
                    self.destroy();
                    self.push(Effect::Navigate(Destination::Ended(reason)));
                }
            }
        }
    }

    /// Saiu da tela da sala (botão, voltar): solta tudo e avisa o servidor, se ainda vale.
    fn destroy(&mut self) {
        self.stop_picker();
        self.leave_camera_tab();
        self.close_media();
        self.stop_broadcast(false);
        // O temporizador de "não deu pra voltar" só existe enquanto a ligação está caída (`clearTimeout(downTimer)`).
        if self.s.link == LinkState::Reconnecting {
            self.cancel(TimerId::LinkDown);
        }
        if self.s.ended.is_none() {
            self.push(Effect::Leave);
            // `signaling.leave()` do site também esquece a identidade (`onSession(null)`): quem sai de
            // propósito não retoma o lugar. Depois de `left` o redutor não ouve mais o laço.
            self.s.session = None;
            let code = self.s.code.clone();
            self.push(Effect::Save(DeviceChange::Session { code, session: None }));
        }
    }

    fn leave(&mut self) {
        self.destroy();
        self.s.left = true;
        self.push(Effect::Navigate(Destination::Home));
    }

    /// Apaguei a sala: vai pro início uma vez só (a resposta e o fechamento 4410 chegam em qualquer ordem).
    fn leave_after_delete(&mut self) {
        if self.s.left {
            return;
        }
        let code = self.s.code.clone();
        self.push(Effect::Save(DeviceChange::ForgetRoom { code: code.clone() }));
        self.push(Effect::Save(DeviceChange::ForgetOwnerKey { code }));
        self.leave();
    }

    // ---------- Mensagens do servidor ----------

    fn server(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Welcome { broadcast_order, you, peers, ice_servers, capacity, room, .. } => {
                self.welcome(broadcast_order, you, peers, ice_servers, capacity, room)
            }
            ServerMessage::LiveAck { request_id, broadcast_seq, live } => self.live_ack(&request_id, broadcast_seq, live),
            ServerMessage::PeerJoined { peer } => match self.s.peers.iter().position(|p| p.id == peer.id) {
                // Já na lista (veio no welcome): só atualiza, sem aviso.
                Some(i) => self.s.peers[i] = peer,
                None => {
                    let text = format!("{} entrou na sala", peer.nickname);
                    self.s.peers.push(peer);
                    self.toast(ToastKind::Join, text, None);
                }
            },
            // O dono renomeou (specs/011 FR-027): título e Suas salas mudam sem recarregar.
            ServerMessage::RoomRenamed { name } => {
                if matches!(self.s.room_info, RoomInfo::Fixed { .. }) {
                    self.s.room_info = RoomInfo::Fixed { name: name.clone() };
                }
                let code = self.s.code.clone();
                self.push(Effect::Save(DeviceChange::RenameRoom { code, name }));
            }
            ServerMessage::PeerLeft { id } => self.peer_left(&id),
            ServerMessage::PeerLive { id, live } => self.peer_live(&id, live),
            ServerMessage::Signal { from, data } => self.signal_in(from, data),
            ServerMessage::IceServers { ice_servers, .. } => {
                if self.s.mesh_id.is_some() {
                    let direct = direct_ice_servers(&ice_servers);
                    self.s.ice_servers = direct.clone();
                    self.push(Effect::SetIceServers(direct));
                }
            }
            // A sinalização trata estes (e o `error`, que o site só registra no console).
            ServerMessage::Error { .. }
            | ServerMessage::Pong
            | ServerMessage::RoomNotFound
            | ServerMessage::RoomFull { .. }
            | ServerMessage::ResumeFailed
            | ServerMessage::RoomDeleted
            | ServerMessage::Removed => {}
        }
    }

    fn welcome(
        &mut self,
        order: Option<BroadcastOrder>,
        you: Peer,
        peers: Vec<Peer>,
        ice_servers: Vec<IceServer>,
        capacity: u32,
        room: Option<RoomInfo>,
    ) {
        if self.s.me.as_ref().is_some_and(|m| m.id != you.id) {
            self.s.watching.clear();
            self.s.stopped.clear();
        }
        self.s.highest_broadcast_seq = order.as_ref().map_or(-1, |o| o.highest_broadcast_seq as i64);
        let same_request = match (&order, &self.s.capture_request) {
            (Some(o), Some(c)) => o.request_id == c.request_id,
            _ => false,
        };
        if !same_request {
            self.s.capture_request = None;
        }
        self.s.me = Some(you.clone());
        self.s.peers = peers;
        self.s.capacity = capacity;
        self.s.connected = true;
        self.s.room_info = room.unwrap_or(RoomInfo::Quick);
        // Entrou numa sala fixa: fica em Suas salas deste aparelho (specs/011 FR-016).
        if let RoomInfo::Fixed { name } = &self.s.room_info {
            let change = DeviceChange::RememberRoom { code: self.s.code.clone(), name: name.clone(), at_ms: self.now };
            self.push(Effect::Save(change));
        }
        if self.s.just_created {
            self.s.just_created = false;
            self.copy_invite("Sala criada — fixe o link no grupo");
        }
        if self.s.focused_id.as_ref().is_some_and(|f| !self.s.peers.iter().any(|p| &p.id == f)) {
            self.s.focused_id = None;
        }
        self.prune_watch();
        // Voltou como pessoa nova (passou da tolerância) com a captura ainda rodando: avisa de novo.
        if self.s.my_broadcast.is_some() && (you.live.is_none() || self.s.capture_request.is_none()) {
            self.send_live();
        }
        if self.s.my_broadcast.is_none() && you.live.is_some() {
            if let Some(o) = &order {
                self.push(Effect::Send(ClientMessage::Unlive {
                    request_id: o.request_id.clone(),
                    broadcast_seq: o.highest_broadcast_seq,
                }));
            }
        }
        self.sync_mesh(&you.id, &ice_servers);
        let key = match (&you.live, &self.s.my_broadcast, &order, &self.s.capture_request) {
            (Some(live), Some(_), Some(o), Some(c)) if o.request_id == c.request_id => {
                Some(TransmissionKey { peer_id: you.id.clone(), started_at: live.started_at })
            }
            _ => None,
        };
        self.set_local_key(key);
    }

    fn live_ack(&mut self, request_id: &str, broadcast_seq: u64, live: LiveInfo) {
        let Some(me_id) = self.s.me.as_ref().map(|m| m.id.clone()) else { return };
        let valid = self.s.my_broadcast.is_some()
            && self.s.capture_request.as_ref().is_some_and(|c| c.request_id == request_id && c.broadcast_seq == broadcast_seq);
        if !valid {
            return;
        }
        self.s.highest_broadcast_seq = self.s.highest_broadcast_seq.max(broadcast_seq as i64);
        let key = TransmissionKey { peer_id: me_id, started_at: live.started_at };
        if let Some(me) = self.s.me.as_mut() {
            me.live = Some(live);
        }
        self.set_local_key(Some(key));
    }

    fn peer_left(&mut self, id: &str) {
        let peer = self.s.peer(id).cloned();
        self.s.peers.retain(|p| p.id != id);
        if self.s.focused_id.as_deref() == Some(id) {
            self.s.focused_id = None;
        }
        // Saiu sozinha enquanto o dono confirmava: não tem mais quem tirar.
        if self.s.remove_dialog.as_ref().is_some_and(|d| d.peer_id == id) {
            self.s.remove_dialog = None;
        }
        self.remove_link(id);
        self.forget_media(id);
        self.prune_watch();
        if let Some(peer) = peer {
            self.toast(ToastKind::Leave, format!("{} saiu da sala", peer.nickname), None);
        }
    }

    fn peer_live(&mut self, id: &str, live: Option<LiveInfo>) {
        let Some(i) = self.s.peers.iter().position(|p| p.id == id) else { return };
        let started = self.s.peers[i].live.is_none() && live.is_some();
        self.s.peers[i].live = live.clone();
        if started {
            let text = format!("{} começou a transmitir", self.s.peers[i].nickname);
            let toast_id = self.toast(ToastKind::Live, text, Some("Assistir"));
            self.s.toast_peer.insert(toast_id, id.to_string());
        }
        if live.is_none() && self.s.focused_id.as_deref() == Some(id) {
            self.s.focused_id = None;
        }
        self.prune_watch();
        if started && self.s.watching.contains_key(id) {
            self.wait_first_frame(id);
        }
        if live.is_none() {
            self.clear_frame_timer(id);
        }
    }

    fn signal_in(&mut self, from: String, data: SignalData) {
        if self.s.mesh_id.is_none() {
            return;
        }
        if !self.s.links.contains_key(&from) {
            let views = self.my_views();
            self.create_link(&from, false, &views);
        }
        self.push(Effect::Signal { from, data });
    }

    // ---------- Assistir ----------

    /// Escolhas de "parar de assistir" que ainda valem depois de uma mudança na lista.
    fn prune_watch(&mut self) {
        let next = prune_watching(&self.s.watching, &self.s.peers);
        if next.len() != self.s.watching.len() {
            self.s.watching = next;
        }
        self.set_remote_keys();
        if let Some(f) = &self.s.floating_id {
            if !self.s.peers.iter().any(|p| &p.id == f && p.live.is_some()) {
                self.s.floating_id = None;
            }
        }
    }

    /// Assistir ou parar de assistir (`actions.watch`). Só `watching` e `stopped` mudam aqui; o que
    /// chega e o que sai segue disso, em `settle`.
    fn watch(&mut self, id: &str, on: bool) {
        let Some(peer) = self.s.peer(id).cloned() else { return };
        let Some(live) = peer.live.clone() else { return };
        self.s.watching = choose_watch(&self.s.watching, &peer, on, false);
        if on {
            self.s.stopped.remove(id);
        } else {
            self.s.stopped.insert(id.to_string(), live.started_at);
            self.clear_frame_timer(id);
            self.s.timed_out.remove(id);
            if let Some(m) = self.s.media.get_mut(id) {
                m.video = false;
            }
        }
    }

    // ---------- Vídeo de quem escolhi assistir ----------

    fn wait_first_frame(&mut self, id: &str) {
        self.s.frame_timers.insert(id.to_string());
        self.schedule(TimerId::FirstFrame(id.to_string()), FIRST_FRAME_TIMEOUT_MS);
    }

    fn clear_frame_timer(&mut self, id: &str) {
        if self.s.frame_timers.remove(id) {
            self.cancel(TimerId::FirstFrame(id.to_string()));
        }
    }

    fn forget_media(&mut self, id: &str) {
        self.clear_frame_timer(id);
        self.s.media.remove(id);
        self.s.timed_out.remove(id);
    }

    fn update_video(&mut self, id: &str, flowing: bool) {
        let watching = self.s.watching.contains_key(id);
        self.s.media.entry(id.to_string()).or_default().video = watching && flowing;
        if flowing && watching {
            self.clear_frame_timer(id);
            self.s.timed_out.remove(id);
        }
    }

    // ---------- Ligações com as pessoas ----------

    /// Como estou vendo a tela de cada pessoa ao vivo. Quem não está no mapa: `Off`.
    fn my_views(&self) -> Modes {
        let v = view(&self.s, self.now);
        let layout = stage_layout(&v);
        view_modes(&ViewModesInput {
            streams: v.streams.iter().map(|s| s.id.clone()).collect(),
            layout,
            focused_id: v.focused_id.clone(),
            watching: self.s.watching.keys().cloned().collect(),
            visible: self.s.visible,
            floating_id: self.s.floating_id.clone(),
            self_id: if self.s.self_on_stage { self.s.me.as_ref().map(|m| m.id.clone()) } else { None },
        })
    }

    /// Depois de cada welcome. Mesmo id (retomada): mantém as ligações vivas. Id novo: refaz tudo.
    fn sync_mesh(&mut self, my_id: &str, ice_servers: &[IceServer]) {
        if self.s.mesh_id.as_deref().is_some_and(|id| id != my_id) {
            self.close_media();
        }
        if self.s.mesh_id.is_none() {
            self.s.mesh_id = Some(my_id.to_string());
        }
        // O app só usa STUN: o que não for STUN fica de fora.
        let direct = direct_ice_servers(ice_servers);
        self.s.ice_servers = direct.clone();
        self.push(Effect::SetIceServers(direct));
        self.set_remote_keys();
        let keep: BTreeSet<String> = self.s.peers.iter().map(|p| p.id.clone()).collect();
        let stale: Vec<String> = self.s.links.keys().filter(|id| !keep.contains(*id)).cloned().collect();
        for id in stale {
            self.remove_link(&id);
        }
        let views = self.my_views();
        let missing: Vec<String> = self.s.peers.iter().map(|p| p.id.clone()).filter(|id| !self.s.links.contains_key(id)).collect();
        for id in missing {
            self.create_link(&id, true, &views);
        }
        if self.s.my_broadcast.is_some() {
            self.send_params_all();
        }
    }

    fn create_link(&mut self, id: &str, initiator: bool, views: &Modes) {
        let polite = self.s.mesh_id.as_deref().is_some_and(|me| me < id);
        self.push(Effect::OpenPeer { id: id.to_string(), initiator, polite });
        let mut link = PeerLinkState::new(self.s.remote_key(id));
        link.my_view = Some(views.get(id).copied().unwrap_or(ViewMode::Off));
        self.s.links.insert(id.to_string(), link);
        if self.s.my_broadcast.is_some() {
            self.send_params_for(id);
        }
    }

    fn remove_link(&mut self, id: &str) {
        if self.s.links.remove(id).is_some() {
            self.push(Effect::ClosePeer { id: id.to_string() });
        }
    }

    /// Fecha todas as ligações (e os temporizadores de quem estava esperando vídeo).
    fn close_media(&mut self) {
        let ids: Vec<String> = self.s.links.keys().cloned().collect();
        for id in ids {
            self.remove_link(&id);
        }
        self.s.mesh_id = None;
        let waiting: Vec<String> = self.s.frame_timers.iter().cloned().collect();
        for id in waiting {
            self.clear_frame_timer(&id);
        }
        self.s.media.clear();
        self.s.timed_out.clear();
    }

    /// Cada ligação confere se a transmissão da pessoa mudou: o que eu tinha dito (`my_view`) vale
    /// só para a transmissão antiga, então volta a `Off` e a pessoa é avisada.
    fn set_remote_keys(&mut self) {
        let ids: Vec<String> = self.s.links.keys().cloned().collect();
        for id in ids {
            let key = self.s.remote_key(&id);
            let Some(link) = self.s.links.get_mut(&id) else { continue };
            if link.remote_key == key {
                continue;
            }
            link.remote_key = key;
            link.my_view = Some(ViewMode::Off);
            self.send_view(&id);
        }
    }

    /// Chave da minha transmissão. Quem recebe a minha tela só vale para esta chave.
    fn set_local_key(&mut self, key: Option<TransmissionKey>) {
        if self.s.local_key == key {
            return;
        }
        self.s.local_key = key.clone();
        for link in self.s.links.values_mut() {
            link.grant_revision = -1;
            link.their_view = ViewMode::Off;
        }
        if self.s.my_broadcast.is_some() {
            self.send_params_all();
        }
        if let Some(key) = key {
            let open: Vec<String> = self.s.links.iter().filter(|(_, l)| l.ctl_open).map(|(id, _)| id.clone()).collect();
            for to in open {
                self.push(Effect::RequestView { to, transmission: key.clone() });
            }
        }
    }

    /// Avisa essa pessoa de como estou vendo a tela dela (só quando muda).
    fn set_my_view(&mut self, id: &str, mode: ViewMode) {
        let Some(link) = self.s.links.get_mut(id) else { return };
        if link.my_view == Some(mode) {
            return;
        }
        link.my_view = Some(mode);
        self.send_view(id);
    }

    fn send_view(&mut self, id: &str) {
        let Some(link) = self.s.links.get_mut(id) else { return };
        let (Some(key), true) = (link.remote_key.clone(), link.ctl_open) else { return };
        link.view_revision += 1;
        let message = ViewMessage { transmission: key, revision: link.view_revision, mode: link.my_view.unwrap_or(ViewMode::Off) };
        self.push(Effect::SendView { to: id.to_string(), message });
    }

    /// O que vai para essa pessoa agora: vídeo e som conforme como ela vê a minha tela. Sem pedido
    /// válido dela (`their_view` em `Off`) ou sem transmissão confirmada, nada sai.
    fn send_params_for(&mut self, id: &str) {
        let Some(q) = self.s.quality() else { return };
        let Some(link) = self.s.links.get(id) else { return };
        let confirmed = self.s.local_key.is_some();
        let mut params = send_params(&q, link.their_view);
        params.active = params.active && confirmed;
        let sound = confirmed && wants_sound(Some(link.their_view));
        self.push(Effect::SetSendParams { to: id.to_string(), params, sound });
    }

    fn send_params_all(&mut self) {
        let ids: Vec<String> = self.s.links.keys().cloned().collect();
        for id in ids {
            self.send_params_for(&id);
        }
    }

    /// Manda os modos e, para quem volta a ser visto, espera o vídeo de novo (não conta o tempo escondido).
    fn apply_views(&mut self, views: Modes) {
        for (id, mode) in &views {
            let before = self.s.sent_modes.get(id).copied();
            let has_video = self.s.media.get(id).is_some_and(|m| m.video);
            if wants_video(Some(*mode)) && !wants_video(before) && !has_video {
                self.s.timed_out.remove(id);
                if self.s.peers.iter().any(|p| &p.id == id && p.live.is_some()) {
                    self.wait_first_frame(id);
                }
            }
        }
        self.s.sent_modes = views.clone();
        let ids: Vec<String> = self.s.links.keys().cloned().collect();
        for id in ids {
            let mode = views.get(&id).copied().unwrap_or(ViewMode::Off);
            self.set_my_view(&id, mode);
        }
    }

    fn peer_event(&mut self, e: PeerEvent) {
        match e {
            PeerEvent::Signal { to, data } => {
                if self.s.mesh_id.is_some() {
                    self.push(Effect::Send(ClientMessage::Signal { to, data }));
                }
            }
            PeerEvent::SendStats(readings) => self.send_stats(&readings),
            PeerEvent::State { id, conn } => {
                if self.s.links.contains_key(&id) {
                    self.s.media.entry(id).or_default().conn = conn;
                }
            }
            PeerEvent::ControlOpen { id } => {
                let Some(link) = self.s.links.get_mut(&id) else { return };
                link.ctl_open = true;
                self.send_view(&id);
                if let Some(key) = self.s.local_key.clone() {
                    self.push(Effect::RequestView { to: id, transmission: key });
                }
            }
            PeerEvent::ControlClosed { id } => {
                let Some(link) = self.s.links.get_mut(&id) else { return };
                link.ctl_open = false;
                link.their_view = ViewMode::Off;
                link.grant_revision = -1;
                self.send_params_for(&id);
            }
            PeerEvent::View { id, message } => {
                let local = self.s.local_key.clone();
                let Some(link) = self.s.links.get_mut(&id) else { return };
                // Entrada externa inválida nunca autoriza mídia.
                if !link.ctl_open
                    || !same_transmission(Some(&message.transmission), local.as_ref())
                    || !is_safe_sequence(message.revision)
                    || (message.revision as i64) <= link.grant_revision
                {
                    return;
                }
                link.grant_revision = message.revision as i64;
                link.their_view = message.mode;
                self.send_params_for(&id);
            }
            PeerEvent::ViewSync { id, transmission } => {
                let Some(link) = self.s.links.get(&id) else { return };
                if same_transmission(Some(&transmission), link.remote_key.as_ref()) {
                    self.send_view(&id);
                }
            }
            PeerEvent::Remote { id, stream, audio, flowing } => {
                if !self.s.links.contains_key(&id) {
                    return;
                }
                let m = self.s.media.entry(id.clone()).or_default();
                m.stream = stream;
                m.audio = audio;
                self.update_video(&id, flowing);
            }
            PeerEvent::VideoFlow { id, flowing } => {
                if self.s.links.contains_key(&id) {
                    self.update_video(&id, flowing);
                }
            }
        }
    }

    /// CPU no limite em alguma pessoa em tela cheia por 10 s: um degrau a menos; 60 s de folga: um a mais.
    fn send_stats(&mut self, readings: &[SendReading]) {
        let Some(q) = self.s.quality() else { return };
        let full: Vec<&SendReading> = readings
            .iter()
            .filter(|r| self.s.links.get(&r.id).is_some_and(|l| l.their_view == ViewMode::Full))
            .collect();
        if full.is_empty() {
            return;
        }
        let down = step_down(Step { resolution: q.resolution, fps: q.fps });
        let can_go_down = down.resolution != q.resolution || down.fps != q.fps;
        if self.s.governor.0.tick(full.iter().any(|r| r.cpu_limited), can_go_down).is_some() {
            self.s.cpu_steps = self.s.governor.0.steps;
        }
    }

    // ---------- Transmissão ----------

    fn take_request_id(&mut self) -> Option<String> {
        let id = self.s.spare_request_id.take();
        // Reposição: o redutor não sorteia, o laço entrega outro.
        if !self.s.request_id_asked {
            self.s.request_id_asked = true;
            self.push(Effect::NewRequestId);
        }
        id
    }

    /// Anuncia a transmissão com a qualidade efetiva (quem assiste vê no subtítulo).
    fn send_live(&mut self) {
        let Some(b) = self.s.my_broadcast.clone() else { return };
        if !self.s.connected {
            return;
        }
        if self.s.capture_request.is_none() {
            if self.s.highest_broadcast_seq >= MAX_SAFE_INTEGER as i64 {
                return;
            }
            let Some(request_id) = self.take_request_id() else {
                self.s.live_pending = true;
                return;
            };
            self.s.highest_broadcast_seq += 1;
            self.s.capture_request =
                Some(LiveRequest { request_id, broadcast_seq: self.s.highest_broadcast_seq as u64 });
        }
        self.s.live_pending = false;
        let Some(req) = self.s.capture_request.clone() else { return };
        let (resolution, fps) = match self.s.quality() {
            Some(q) => (q.resolution, q.fps),
            None => (b.prefs.resolution, b.prefs.fps),
        };
        self.push(Effect::Send(ClientMessage::Live {
            request_id: req.request_id,
            broadcast_seq: req.broadcast_seq,
            resolution,
            fps: fps as u8,
            audio: b.audio_kind,
            source: Some(b.source.unwrap_or(LiveSource::Screen)),
            label: b.label,
        }));
    }

    /// Qualidade efetiva mudou durante a transmissão: aplica na captura e no envio e avisa a sala.
    fn apply_quality(&mut self) {
        let Some(q) = self.s.quality() else { return };
        let prev = self.s.applied_quality;
        if prev == Some(q) {
            return;
        }
        self.s.applied_quality = Some(q);
        self.push(Effect::SetCaptureQuality { resolution: q.resolution, fps: q.fps });
        self.send_params_all();
        if prev.is_none_or(|p| p.resolution != q.resolution || p.fps != q.fps) {
            self.send_live();
        }
    }

    fn stop_broadcast(&mut self, notify: bool) {
        // Uma captura que ainda estava abrindo é cancelada.
        if self.s.pending_start.take().is_some() {
            self.push(Effect::StopCapture);
        }
        if self.s.my_broadcast.is_none() {
            return;
        }
        self.s.my_broadcast = None;
        self.s.applied_quality = None;
        self.s.live_pending = false;
        self.push(Effect::StopCapture);
        self.set_local_key(None);
        if notify {
            if let Some(req) = self.s.capture_request.clone() {
                self.push(Effect::Send(ClientMessage::Unlive { request_id: req.request_id, broadcast_seq: req.broadcast_seq }));
            }
        }
        self.s.capture_request = None;
        if let Some(me) = self.s.me.as_mut() {
            me.live = None;
        }
    }

    fn start_broadcast(&mut self, prefs: GoLive) {
        self.stop_broadcast(true);
        if prefs.surface == Surface::Camera {
            return self.start_camera_broadcast(prefs);
        }
        // No app não há seletor do navegador para onde cair: sem fonte escolhida a grade segue aberta
        // e listando (o site pararia a lista e abriria o seletor do navegador).
        let Some(source) = self.s.selected_id.as_ref().and_then(|id| self.s.sources.iter().find(|s| &s.id == id)).cloned()
        else {
            return;
        };
        self.stop_picker();
        self.leave_camera_tab();
        let surface = match source.kind {
            ScreenSurface::Window => Surface::Window,
            ScreenSurface::Monitor => Surface::Monitor,
        };
        let prefs = GoLive { surface, last_source_key: Some(source_key(&source)), ..prefs };
        // Só o programa, nunca o título da janela (specs/012 FR-002).
        let label = live_label(&match source.kind {
            ScreenSurface::Monitor => LabelInput::NativeMonitor {
                title: source.title.clone(),
                monitors: self.s.sources.iter().filter(|s| s.kind == ScreenSurface::Monitor).count() as u32,
            },
            ScreenSurface::Window => LabelInput::NativeWindow { app: source.app.clone() },
        });
        self.s.pending_start =
            Some(PendingStart { prefs: prefs.clone(), source: None, source_title: Some(source.title.clone()), label });
        self.remember_prefs(&prefs);
        self.s.go_live_open = false;
        self.push(Effect::StartCapture { prefs, target: CaptureTarget::Source(source) });
    }

    fn start_camera_broadcast(&mut self, prefs: GoLive) {
        let Some(video) = self.s.cam_video_id.as_ref().and_then(|id| self.s.cam_videos.iter().find(|v| &v.id == id)).cloned()
        else {
            return;
        };
        self.stop_picker();
        let audio = if prefs.audio {
            self.s.cam_audio_id.as_ref().and_then(|id| self.s.cam_audios.iter().find(|a| &a.id == id)).cloned()
        } else {
            None
        };
        let mut chosen = GoLive { last_video_label: Some(video.label.clone()), ..prefs };
        if let Some(a) = &audio {
            chosen.last_audio_label = Some(a.label.clone());
            chosen.last_audio_role = Some(self.s.cam_audio_role);
        }
        self.remember_prefs(&chosen);
        self.s.go_live_open = false;
        // Fecha as prévias antes: alguns drivers de placa não abrem a mesma câmera duas vezes.
        self.leave_camera_tab();
        let label = live_label(&LabelInput::Camera { title: Some(video.label.clone()), facing: None });
        self.s.pending_start = Some(PendingStart {
            prefs: chosen.clone(),
            source: Some(LiveSource::Camera),
            source_title: Some(video.label.clone()),
            label,
        });
        self.push(Effect::StartCapture {
            prefs: chosen,
            target: CaptureTarget::Camera {
                video_id: video.id,
                audio_id: audio.map(|a| a.id),
                audio_role: self.s.cam_audio_role,
            },
        });
    }

    fn remember_prefs(&mut self, prefs: &GoLive) {
        self.s.go_live_prefs = prefs.clone();
        self.push(Effect::Save(DeviceChange::GoLivePrefs(prefs.clone())));
    }

    // ---------- Captura ----------

    fn capture_event(&mut self, e: CaptureEvent) {
        match e {
            CaptureEvent::Sources(sources) => {
                if !self.s.listing {
                    return;
                }
                self.s.sources_loading = false;
                // A escolhida fechou: some a marcação (US1 cenário 5).
                if self.s.selected_id.as_ref().is_some_and(|id| !sources.iter().any(|s| &s.id == id)) {
                    self.s.selected_id = None;
                }
                // Primeira lista: volta a última fonte, se ainda existir (US5).
                if self.s.first_list && self.s.selected_id.is_none() {
                    self.s.selected_id = match_last_source(&sources, self.s.go_live_prefs.last_source_key.as_deref());
                }
                self.s.first_list = false;
                self.s.sources = sources;
            }
            CaptureEvent::SourcesFailed => {
                self.s.sources_loading = false;
                self.stop_picker();
            }
            CaptureEvent::Started { request_id, stream, audio_kind, audio_warn } => {
                self.capture_started(request_id, stream, audio_kind, audio_warn)
            }
            CaptureEvent::StartFailed(reason) => self.capture_failed(reason),
            CaptureEvent::Ended => {
                if self.s.my_broadcast.is_some() {
                    self.stop_broadcast(true);
                }
            }
            CaptureEvent::AudioChanged { kind, warn } => {
                if self.s.my_broadcast.is_none() {
                    return;
                }
                if let Some(w) = warn {
                    self.toast(ToastKind::Error, w.text(), None);
                }
                let changed = self.s.my_broadcast.as_ref().is_some_and(|b| b.audio_kind != kind);
                if changed {
                    if let Some(b) = self.s.my_broadcast.as_mut() {
                        b.audio_kind = kind;
                    }
                    self.send_live();
                }
            }
            CaptureEvent::BlackScreen => {
                if self.s.my_broadcast.is_some() {
                    self.toast(ToastKind::Error, BLACK_WARN, None);
                }
            }
            CaptureEvent::Snapshot(shot) => {
                if self.s.my_broadcast.is_none() || !self.s.visible {
                    self.push(Effect::ReleaseSnapshot(shot));
                    return;
                }
                if let Some(old) = self.s.snapshot.replace(shot) {
                    self.push(Effect::ReleaseSnapshot(old));
                }
            }
            CaptureEvent::RequestId(id) => {
                self.s.spare_request_id = Some(id);
                self.s.request_id_asked = false;
                if self.s.live_pending {
                    self.s.live_pending = false;
                    self.send_live();
                }
            }
            CaptureEvent::CameraPermission(p) => {
                self.s.cam_permission = p;
                if p == CameraPermission::Granted && self.s.cam_active {
                    self.push(Effect::ListCameras);
                }
            }
            CaptureEvent::CameraAllowed { permission, error } => {
                self.s.cam_permission = permission;
                self.s.cam_error = error;
                // Abriu a câmera: os nomes já estão liberados.
                if permission == CameraPermission::Granted && self.s.cam_active && self.s.go_live_open {
                    self.push(Effect::ListCameras);
                }
            }
            CaptureEvent::CameraDevices { videos, audios } => self.camera_devices(videos, audios),
            CaptureEvent::CameraPreview { id, stream, busy } => {
                if !self.s.cam_active || !self.s.go_live_open || self.s.tab != Surface::Camera {
                    if let Some(s) = stream {
                        self.push(Effect::CloseStream(s));
                    }
                    return;
                }
                if busy {
                    self.s.cam_busy.insert(id.clone());
                }
                if let Some(s) = stream {
                    self.s.cam_previews.insert(id, s);
                }
            }
        }
    }

    fn capture_started(
        &mut self,
        request_id: String,
        stream: StreamId,
        audio_kind: crate::protocol::AudioKind,
        warn: Option<AudioWarn>,
    ) {
        // Chegou depois de a pessoa parar ou trocar: solta a captura que sobrou.
        let Some(p) = self.s.pending_start.take() else {
            self.push(Effect::StopCapture);
            return;
        };
        let camera = p.prefs.surface == Surface::Camera;
        self.s.spare_request_id = Some(request_id);
        self.s.my_broadcast = Some(MyBroadcast {
            prefs: p.prefs,
            started_at: self.now,
            stream: Some(stream),
            audio_kind,
            audio_reason_monitor: false,
            source_title: p.source_title,
            label: p.label,
            source: p.source,
        });
        // Começou a transmitir: o processador volta a zero degraus.
        self.s.governor.0.reset();
        self.s.cpu_steps = 0;
        if let Some(q) = self.s.quality() {
            self.s.applied_quality = Some(q);
            self.push(Effect::SetCaptureQuality { resolution: q.resolution, fps: q.fps });
        }
        self.send_params_all();
        self.send_live();
        self.toast(
            ToastKind::Live,
            if camera { "Sua câmera está ao vivo pra sala" } else { "Você está ao vivo pra sala" },
            None,
        );
        if let Some(w) = warn {
            self.toast(ToastKind::Error, w.text(), None);
        }
    }

    fn capture_failed(&mut self, reason: StartFailure) {
        let Some(p) = self.s.pending_start.take() else { return };
        if p.prefs.surface == Surface::Camera {
            if reason == StartFailure::PermissionDenied {
                self.s.cam_permission = CameraPermission::Denied;
            }
            match reason {
                StartFailure::Busy => {
                    self.toast(ToastKind::Error, "Essa câmera está sendo usada por outro programa.", None);
                }
                StartFailure::PermissionDenied => {}
                _ => {
                    self.toast(ToastKind::Error, "Não deu pra abrir a câmera.", None);
                }
            }
            self.s.go_live_open = true;
            self.s.tab = Surface::Camera;
            self.enter_camera_tab();
            return;
        }
        if reason == StartFailure::SourceGone {
            self.toast(ToastKind::Error, "Essa janela fechou — escolhe outra.", None);
            self.s.selected_id = None;
        } else {
            // FR-511: sem erro bloqueante; a pessoa tenta de novo.
            self.toast(
                ToastKind::Error,
                "Não deu pra capturar pelo app — tenta de novo, agora pelo seletor do Windows.",
                None,
            );
        }
        self.s.go_live_open = true;
        self.start_picker();
    }

    // ---------- Seletor ----------

    /// Grade aberta: lista agora e a cada 2 s (FR-504).
    fn start_picker(&mut self) {
        if self.s.listing {
            return;
        }
        self.s.listing = true;
        self.s.first_list = true;
        self.s.sources_loading = self.s.sources.is_empty();
        self.push(Effect::StartListingSources);
    }

    fn stop_picker(&mut self) {
        if self.s.listing {
            self.s.listing = false;
            self.push(Effect::StopListingSources);
        }
    }

    // ---------- Câmera / placa de captura (specs/006) ----------

    /// Abriu a aba: só consulta a permissão (FR-602); se já foi dada, lista e liga as prévias.
    fn enter_camera_tab(&mut self) {
        self.s.cam_entered = true;
        self.s.cam_active = true;
        self.s.cam_audio_manual = false;
        self.s.cam_role_manual = false;
        if self.s.cam_permission == CameraPermission::Granted {
            self.push(Effect::ListCameras);
        } else {
            self.push(Effect::QueryCameraPermission);
        }
    }

    /// Saiu da aba, fechou o seletor ou vai transmitir: nenhuma prévia fica ligada (FR-604).
    fn leave_camera_tab(&mut self) {
        let was_active = self.s.cam_active;
        self.s.cam_active = false;
        let previews: Vec<StreamId> = std::mem::take(&mut self.s.cam_previews).into_values().collect();
        if was_active {
            self.push(Effect::LeaveCameraTab);
        }
        for s in previews {
            self.push(Effect::CloseStream(s));
        }
    }

    fn paired_with_capture(&self, a: &DeviceInfo) -> bool {
        self.s.cam_videos.iter().any(|v| !v.group_id.is_empty() && v.group_id == a.group_id && is_capture_card(&v.label))
    }

    fn camera_devices(&mut self, videos: Vec<DeviceInfo>, audios: Vec<DeviceInfo>) {
        if !self.s.cam_active {
            return;
        }
        if !videos.is_empty() {
            self.s.cam_error = None;
        }
        self.s.cam_videos = videos;
        self.s.cam_audios = audios;
        // Escolha: a que já estava, senão a última transmitida (pelo nome, FR-609), senão a primeira.
        if !self.s.cam_videos.iter().any(|v| Some(&v.id) == self.s.cam_video_id.as_ref()) {
            let last = self.s.go_live_prefs.last_video_label.clone();
            self.s.cam_video_id = self
                .s
                .cam_videos
                .iter()
                .find(|v| last.as_ref() == Some(&v.label))
                .or(self.s.cam_videos.first())
                .map(|v| v.id.clone());
            self.suggest_camera_audio();
        }
        if self.s.cam_audio_id.as_ref().is_some_and(|id| !self.s.cam_audios.iter().any(|a| &a.id == id)) {
            self.s.cam_audio_id = None;
        }
        self.update_previews();
    }

    /// Som sugerido para a câmera escolhida (R4), a menos que a pessoa já tenha escolhido.
    fn suggest_camera_audio(&mut self) {
        let Some(video) = self.s.cam_video_id.as_ref().and_then(|id| self.s.cam_videos.iter().find(|v| &v.id == id)).cloned()
        else {
            return;
        };
        let same_video = self.s.go_live_prefs.last_video_label.as_ref() == Some(&video.label);
        if !self.s.cam_audio_manual {
            let last = self
                .s
                .go_live_prefs
                .last_audio_label
                .as_ref()
                .and_then(|l| self.s.cam_audios.iter().find(|a| &a.label == l));
            let pick = suggest_audio(&video, &self.s.cam_audios).or(if same_video { last } else { None });
            self.s.cam_audio_id = pick.map(|a| a.id.clone());
        }
        if !self.s.cam_role_manual {
            let chosen = self.s.cam_audio_id.as_ref().and_then(|id| self.s.cam_audios.iter().find(|a| &a.id == id));
            self.s.cam_audio_role = match (same_video, self.s.go_live_prefs.last_audio_role, chosen) {
                (true, Some(role), _) => role,
                (_, _, Some(a)) => audio_role(&a.label, self.paired_with_capture(a)),
                _ => crate::device::AudioRole::Voice,
            };
        }
    }

    /// Até 4 prévias ao vivo, a escolhida primeiro (FR-603).
    fn update_previews(&mut self) {
        let candidates: Vec<PreviewDevice> = self
            .s
            .cam_videos
            .iter()
            .map(|v| PreviewDevice { id: v.id.clone(), busy: self.s.cam_busy.contains(&v.id) })
            .collect();
        let want = preview_ids(&candidates, self.s.cam_video_id.as_deref());
        let drop: Vec<String> = self.s.cam_previews.keys().filter(|id| !want.contains(*id)).cloned().collect();
        for id in drop {
            if let Some(s) = self.s.cam_previews.remove(&id) {
                self.push(Effect::CloseStream(s));
            }
        }
        let missing: Vec<String> = want.into_iter().filter(|id| !self.s.cam_previews.contains_key(id)).collect();
        if !missing.is_empty() {
            self.push(Effect::OpenCameraPreviews { ids: missing });
        }
    }

    // ---------- API do dono ----------

    fn api_event(&mut self, e: ApiEvent) {
        match e {
            ApiEvent::Rename(result) => match result {
                // O nome novo chega pelo room-renamed.
                Ok(()) => self.s.name_dialog = None,
                Err(reason) => {
                    if let Some(d) = self.s.name_dialog.as_mut() {
                        d.busy = false;
                        d.error = Some(owner_error_text(reason).to_string());
                    }
                }
            },
            ApiEvent::Delete(result) => match result {
                Ok(()) => self.leave_after_delete(),
                Err(reason) => {
                    self.s.deleting = false;
                    if let Some(d) = self.s.delete_dialog.as_mut() {
                        d.busy = false;
                        d.error = Some(owner_error_text(reason).to_string());
                    }
                }
            },
            ApiEvent::Remove { peer_id, result } => {
                if self.s.remove_dialog.as_ref().map(|d| d.peer_id.as_str()) != Some(peer_id.as_str()) {
                    return;
                }
                match result {
                    // A pessoa some da lista pelo peer-left.
                    Ok(()) => self.s.remove_dialog = None,
                    Err(reason) => {
                        if let Some(d) = self.s.remove_dialog.as_mut() {
                            d.busy = false;
                            d.error = Some(owner_error_text(reason).to_string());
                        }
                    }
                }
            }
        }
    }

    // ---------- Janela e tempo ----------

    fn window_event(&mut self, e: WindowEvent) {
        match e {
            WindowEvent::Visible(v) => self.s.visible = v,
            WindowEvent::FloatingOpened(id) => self.pip(Some(id)),
            WindowEvent::FloatingClosed => self.s.floating_id = None,
        }
    }

    fn pip(&mut self, id: Option<String>) {
        if id.is_none() || self.s.peers.iter().any(|p| Some(&p.id) == id.as_ref() && p.live.is_some()) {
            self.s.floating_id = id;
        }
    }

    fn timer(&mut self, t: TimerId) {
        match t {
            TimerId::Toast(id) => self.dismiss(&id),
            TimerId::Copied => self.s.copied = false,
            TimerId::LinkDown => {
                if self.s.link == LinkState::Reconnecting {
                    self.s.link = LinkState::Failed;
                }
            }
            TimerId::FirstFrame(id) => {
                if !self.s.frame_timers.remove(&id) {
                    return;
                }
                // Escondida pra mim (parei de assistir, janela escondida): o vídeo não vem mesmo.
                if !wants_video(self.s.sent_modes.get(&id).copied()) {
                    return;
                }
                if !self.s.media.get(&id).is_some_and(|m| m.video) {
                    self.s.timed_out.insert(id);
                }
            }
        }
    }

    // ---------- Ações da tela ----------

    fn audio_of(&mut self, id: &str) -> &mut AudioPref {
        if !self.s.audio.contains_key(id) {
            let volume = self.s.peer(id).and_then(|p| self.s.saved_volume(&p.nickname)).unwrap_or(DEFAULT_VOLUME);
            self.s.audio.insert(id.to_string(), AudioPref { muted: true, volume });
        }
        self.s.audio.get_mut(id).expect("acabou de inserir")
    }

    fn fixed_and_up(&self) -> bool {
        matches!(self.s.room_info, RoomInfo::Fixed { .. }) && self.s.link == LinkState::Ok
    }

    fn action(&mut self, a: RoomAction) {
        match a {
            RoomAction::CopyInvite => self.copy_invite("Link copiado — é só mandar pra galera"),
            RoomAction::OpenGoLive => {
                // Sem servidor não dá pra avisar a sala (FR-007).
                if self.s.link != LinkState::Ok {
                    return;
                }
                self.s.permission_denied = false;
                self.s.go_live_open = true;
                self.start_picker();
                self.s.tab = self.s.go_live_prefs.surface;
                if self.s.tab == Surface::Camera {
                    self.enter_camera_tab();
                }
            }
            RoomAction::CloseGoLive => {
                self.s.go_live_open = false;
                self.stop_picker();
                self.leave_camera_tab();
            }
            RoomAction::SetSurface { surface } => {
                if surface == self.s.tab && surface == Surface::Camera && self.s.cam_entered {
                    return;
                }
                self.s.tab = surface;
                if surface == Surface::Camera {
                    self.enter_camera_tab();
                } else {
                    self.leave_camera_tab();
                }
            }
            RoomAction::AllowCamera => self.push(Effect::RequestCameraPermission),
            RoomAction::SelectCamera { id } => {
                if !self.s.cam_videos.iter().any(|v| v.id == id) {
                    return;
                }
                self.s.cam_video_id = Some(id);
                self.suggest_camera_audio();
                self.update_previews();
            }
            RoomAction::SelectAudio { id } => {
                self.s.cam_audio_id = id.clone();
                self.s.cam_audio_manual = true;
                let found = id.and_then(|id| self.s.cam_audios.iter().find(|a| a.id == id).cloned());
                if let Some(a) = found {
                    if !self.s.cam_role_manual {
                        self.s.cam_audio_role = audio_role(&a.label, self.paired_with_capture(&a));
                    }
                }
            }
            RoomAction::SetAudioRole { role } => {
                self.s.cam_audio_role = role;
                self.s.cam_role_manual = true;
            }
            RoomAction::StartGoLive { prefs } => self.start_broadcast(prefs),
            RoomAction::SelectSource { id } => {
                if self.s.sources.iter().any(|s| s.id == id) {
                    self.s.selected_id = Some(id);
                }
            }
            RoomAction::StopGoLive => self.stop_broadcast(true),
            RoomAction::DismissPermission => self.s.permission_denied = false,
            RoomAction::Focus { id } => {
                // Esc ou outra tela: a sua sai do palco (specs/014 US3).
                if self.s.self_on_stage && id.as_deref() != self.s.me.as_ref().map(|m| m.id.as_str()) {
                    self.s.self_on_stage = false;
                    if id.is_none() {
                        return;
                    }
                }
                self.s.focused_id = id;
            }
            RoomAction::ToggleSelfStage => {
                let live = self.s.my_broadcast.as_ref().is_some_and(|b| b.stream.is_some());
                self.s.self_on_stage = !self.s.self_on_stage && live;
            }
            RoomAction::Pip { id } => self.pip(id),
            RoomAction::Watch { id, on } => self.watch(&id, on),
            RoomAction::ToggleMute { id } => {
                let a = self.audio_of(&id);
                a.muted = !a.muted;
                if !a.muted && a.volume == 0.0 {
                    a.volume = DEFAULT_VOLUME;
                }
            }
            RoomAction::SetVolume { id, volume } => {
                let a = self.audio_of(&id);
                a.volume = volume;
                a.muted = volume == 0.0;
                if let Some(nickname) = self.s.peer(&id).map(|p| p.nickname.clone()) {
                    self.s.volumes.insert(nickname.to_lowercase(), volume);
                    self.push(Effect::Save(DeviceChange::Volume { nickname, volume }));
                }
            }
            RoomAction::Retry { id } => {
                if !self.s.watching.contains_key(&id) {
                    return;
                }
                self.s.timed_out.remove(&id);
                self.wait_first_frame(&id);
                if self.s.links.contains_key(&id) {
                    self.push(Effect::RetryPeer { id });
                } else if self.s.mesh_id.is_some() {
                    let views = self.my_views();
                    self.create_link(&id, true, &views);
                }
            }
            RoomAction::ToastAction { id } => {
                if let Some(peer_id) = self.s.toast_peer.get(&id).cloned() {
                    if self.s.peers.iter().any(|p| p.id == peer_id && p.live.is_some()) {
                        self.watch(&peer_id, true);
                        self.s.focused_id = Some(peer_id);
                    }
                }
                self.dismiss_by_user(&id);
            }
            RoomAction::DismissToast { id } => self.dismiss_by_user(&id),
            RoomAction::RetryConnection => {
                self.mark_down();
                self.push(Effect::RetryNow);
            }
            RoomAction::GoHome => {
                if self.s.my_broadcast.is_some() {
                    self.push(Effect::ConfirmLeave);
                } else {
                    self.leave();
                }
            }
            RoomAction::ToggleOwnerMenu { open } => self.s.owner_menu = open && self.s.link == LinkState::Ok,
            RoomAction::OpenRename => {
                self.s.owner_menu = false;
                let RoomInfo::Fixed { name } = &self.s.room_info else { return };
                if self.s.link != LinkState::Ok {
                    return;
                }
                self.s.name_dialog =
                    Some(NameDialogView { mode: NameDialogMode::Rename, value: name.clone(), busy: false, error: None });
            }
            RoomAction::SubmitRename { name } => {
                let Some(key) = self.s.owner_key.clone() else { return };
                let Some(d) = self.s.name_dialog.as_mut() else { return };
                d.value = name.clone();
                d.busy = true;
                d.error = None;
                let code = self.s.code.clone();
                self.push(Effect::Api(ApiCall::Rename { code, key, name }));
            }
            RoomAction::OpenDelete => {
                self.s.owner_menu = false;
                let RoomInfo::Fixed { name } = &self.s.room_info else { return };
                if self.s.link != LinkState::Ok {
                    return;
                }
                self.s.delete_dialog =
                    Some(DeleteDialogView { name: name.clone(), people: self.s.peers.len() as u32, busy: false, error: None });
            }
            RoomAction::ConfirmDelete => {
                let Some(key) = self.s.owner_key.clone() else { return };
                let Some(d) = self.s.delete_dialog.as_mut() else { return };
                d.busy = true;
                d.error = None;
                self.s.deleting = true;
                let code = self.s.code.clone();
                self.push(Effect::Api(ApiCall::Delete { code, key }));
            }
            RoomAction::AskRemove { peer_id } => {
                let Some(peer) = self.s.peer(&peer_id) else { return };
                if !self.fixed_and_up() || self.s.owner_key.is_none() {
                    return;
                }
                let name = peer.nickname.clone();
                self.s.remove_dialog = Some(RemoveDialog { peer_id, name, busy: false, error: None });
            }
            RoomAction::ConfirmRemove => {
                let Some(key) = self.s.owner_key.clone() else { return };
                let Some(d) = self.s.remove_dialog.as_mut() else { return };
                if d.busy {
                    return;
                }
                d.busy = true;
                d.error = None;
                let peer_id = d.peer_id.clone();
                let code = self.s.code.clone();
                self.push(Effect::Api(ApiCall::Remove { code, key, peer_id }));
            }
            RoomAction::CloseDialogs => {
                self.s.owner_menu = false;
                self.s.name_dialog = None;
                self.s.delete_dialog = None;
                self.s.remove_dialog = None;
            }
            RoomAction::Leave => self.leave(),
        }
    }

    /// Fechou o aviso: o temporizador dele não faz mais falta.
    fn dismiss_by_user(&mut self, id: &str) {
        let cancels = self.s.toasts.iter().any(|t| t.id == id && t.kind != ToastKind::Error);
        self.dismiss(id);
        if cancels {
            self.cancel(TimerId::Toast(id.to_string()));
        }
    }

    // ---------- Os $effect do Svelte ----------

    /// Depois de cada evento: como estou vendo cada tela, qualidade efetiva, quadros parados.
    fn settle(&mut self) {
        if self.s.left || self.s.ended.is_some() {
            return;
        }
        // Avisa cada pessoa de como estou vendo a tela dela (cheia, miniatura, escondida ou parada).
        let views = self.my_views();
        if views != self.s.sent_modes {
            self.apply_views(views);
        }
        // Teto de qualidade mudou durante a transmissão: aplica na captura, no envio e avisa a sala.
        self.apply_quality();
        // Parou de transmitir: some a prévia e a sua tela sai do palco.
        let broadcasting = self.s.my_broadcast.is_some();
        if !broadcasting {
            if let Some(old) = self.s.snapshot.take() {
                self.push(Effect::ReleaseSnapshot(old));
            }
            self.s.self_on_stage = false;
        }
        // Prévia parada (specs/014 US3): só transmitindo e com a janela à vista.
        let on = broadcasting && self.s.visible;
        if on != self.s.snapshots_on {
            self.s.snapshots_on = on;
            self.push(Effect::Snapshots { on });
        }
    }
}

/// Chave que lembra a última fonte transmitida (`sourceKey`): o programa e o título da janela, ou
/// o título do monitor.
pub(crate) fn source_key(s: &CaptureSource) -> String {
    match s.kind {
        ScreenSurface::Monitor => format!("m:{}", s.title),
        ScreenSurface::Window => format!("{}|{}", s.app, s.title),
    }
}

/// A fonte que bate com a última transmitida, ou `None` (`matchLastSource`).
pub(crate) fn match_last_source(sources: &[CaptureSource], key: Option<&str>) -> Option<String> {
    let key = key.filter(|k| !k.is_empty())?;
    sources.iter().find(|s| source_key(s) == key).map(|s| s.id.clone())
}
