// Sala real: liga a sinalização e a malha WebRTC ao RoomView que as telas já usam com os mocks.
import type { IceServer, Peer, ServerMessage } from '../../shared/protocol';
import { applyAudioRule, canShare, isPermissionDenied, startCapture, stopCapture } from '../capture';
import { untrack } from 'svelte';
import { Mesh } from '../mesh';
import { countFullViewers, effectiveQuality, isAdjusted, type Quality, type ViewMode } from '../quality';
import { stageLayout } from '../room-view';
import {
  loadGoLivePrefs,
  loadSession,
  loadVolume,
  saveGoLivePrefs,
  saveSession,
  saveVolume,
} from '../prefs';
import { roomPath } from '../room-id';
import { Signaling, type SignalingEnd } from '../signaling';
import type { GoLivePrefs, MyBroadcastView, RoomActions, RoomView, StreamView, ToastView } from '../types';

const DEFAULT_VOLUME = 0.68;
const TOAST_MS = 4_000;
const SCREEN_NO_AUDIO = 'Tela inteira vai sem som. Pra levar o som, transmita a janela do jogo ou uma aba.';
/** Sem quadro de vídeo nesse tempo depois de "começou a transmitir" → tile "falhou". */
const FIRST_FRAME_TIMEOUT_MS = 10_000;
/** Pede credenciais TURN novas esse tempo antes de vencerem. */
const TURN_REFRESH_EARLY_MS = 10 * 60_000;

interface RemoteMedia {
  stream: MediaStream | null;
  /** Chegando quadros de vídeo agora (faixa remota sem "mute"). */
  video: boolean;
  conn: RTCPeerConnectionState;
}

export class RoomStore {
  connected = $state(false);
  ended = $state<SignalingEnd | null>(null);
  me = $state<Peer | null>(null);
  peers = $state<Peer[]>([]);
  focusedId = $state<string | null>(null);
  copied = $state(false);
  goLiveOpen = $state(false);
  goLivePrefs = $state<GoLivePrefs>(loadGoLivePrefs());
  permissionDenied = $state(false);
  toasts = $state<ToastView[]>([]);
  myBroadcast = $state<MyBroadcastView | null>(null);
  /** Mudo/volume por pessoa que transmite. Começa mudo (autoplay). */
  audio = $state<Record<string, { muted: boolean; volume: number }>>({});
  capacity = $state(10);
  /** Vídeo/conexão de cada pessoa. */
  media = $state<Record<string, RemoteMedia>>({});
  /** Começou a transmitir e o vídeo não chegou a tempo. */
  timedOut = $state<Record<string, boolean>>({});
  /** Como cada pessoa conectada está vendo a minha tela (vem pelo `ctl`). */
  theirViews = $state<Record<string, ViewMode>>({});
  /** Layout de celular (a rota avisa). Muda como eu vejo as telas dos outros. */
  mobile = $state(false);

  /** Qualidade efetiva da minha transmissão: o escolhido, com teto pelo nº de espectadores. */
  readonly quality: Quality | null = $derived(
    this.myBroadcast ? effectiveQuality(this.myBroadcast, countFullViewers(Object.values(this.theirViews))) : null,
  );

  readonly view: RoomView = $derived.by(() => this.buildView());
  readonly actions: RoomActions;

  private signaling: Signaling;
  private toastSeq = 0;
  /** Toast "começou a transmitir" → quem focar no "Assistir". */
  private toastPeer = new Map<string, string>();
  private reconnectToast: string | null = null;
  private stream: MediaStream | null = null;
  private mesh: Mesh | null = null;
  private frameTimers = new Map<string, ReturnType<typeof setTimeout>>();
  private turnTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(
    readonly code: string,
    nickname: string,
    private onLeave: () => void,
  ) {
    this.signaling = new Signaling({
      code,
      nickname,
      session: loadSession(code),
      onMessage: (m) => this.receive(m),
      onSession: (s) => saveSession(code, s),
      onStatus: (s) => {
        if (s === 'reconnecting' && this.connected && !this.reconnectToast) {
          this.reconnectToast = this.toast({ kind: 'error', text: 'A conexão com a sala caiu. Reconectando…' });
        }
      },
      onEnd: (reason) => {
        this.closeMedia();
        this.stopBroadcast(false);
        this.ended = reason;
      },
    });
    this.actions = this.buildActions();
    this.stopEffects = $effect.root(() => {
      // Avisa cada pessoa de como estou vendo a tela dela (cheia, miniatura ou escondida).
      $effect(() => {
        const views = this.myViews();
        untrack(() => this.mesh?.setViews(views));
      });
      // Teto de qualidade mudou durante a transmissão: aplica na captura, no envio e avisa a sala.
      $effect(() => {
        const q = this.quality;
        if (q) untrack(() => this.applyQuality(q));
      });
    });
  }

  private stopEffects: () => void;
  /** Última qualidade aplicada/anunciada, para só agir quando mudar. */
  private appliedQuality: Quality | null = null;

  /** Saiu da tela da sala (botão, voltar do navegador). */
  destroy() {
    this.stopEffects();
    this.closeMedia();
    this.stopBroadcast(false);
    if (!this.ended) this.signaling.leave();
  }

  // ---------- Mensagens do servidor ----------

  private receive(msg: ServerMessage) {
    switch (msg.t) {
      case 'welcome': {
        this.me = msg.you;
        this.peers = msg.peers;
        this.capacity = msg.capacity;
        this.connected = true;
        if (this.reconnectToast) this.dismiss(this.reconnectToast);
        this.reconnectToast = null;
        if (this.focusedId && !this.peers.some((p) => p.id === this.focusedId)) this.focusedId = null;
        // Voltou como pessoa nova (passou da tolerância) com a captura ainda rodando: avisa de novo.
        if (this.myBroadcast && !msg.you.live) this.sendLive();
        this.syncMesh(msg.you.id, msg.iceServers);
        this.scheduleTurnRefresh(msg.iceExpiresAt);
        for (const p of this.peers) if (p.live && !this.media[p.id]?.video) this.waitFirstFrame(p.id);
        return;
      }
      case 'peer-joined': {
        // Já na lista (veio no welcome): só atualiza, sem aviso.
        const i = this.peers.findIndex((p) => p.id === msg.peer.id);
        if (i >= 0) this.peers[i] = msg.peer;
        else {
          this.peers.push(msg.peer);
          this.toast({ kind: 'join', text: `${msg.peer.nickname} entrou na sala` });
        }
        return;
      }
      case 'peer-left': {
        const peer = this.peers.find((p) => p.id === msg.id);
        this.peers = this.peers.filter((p) => p.id !== msg.id);
        if (this.focusedId === msg.id) this.focusedId = null;
        this.mesh?.remove(msg.id);
        this.forgetMedia(msg.id);
        if (peer) this.toast({ kind: 'leave', text: `${peer.nickname} saiu da sala` });
        return;
      }
      case 'peer-live': {
        const peer = this.peers.find((p) => p.id === msg.id);
        if (!peer) return;
        const started = !peer.live && msg.live;
        peer.live = msg.live;
        if (started) {
          const id = this.toast({ kind: 'live', text: `${peer.nickname} começou a transmitir`, action: 'Assistir' });
          this.toastPeer.set(id, peer.id);
        }
        if (!msg.live && this.focusedId === msg.id) this.focusedId = null;
        if (started) this.waitFirstFrame(peer.id);
        if (!msg.live) this.clearFrameTimer(peer.id);
        return;
      }
      case 'signal':
        this.mesh?.receive(msg.from, msg.data);
        return;
      case 'ice-servers':
        this.mesh?.setIceServers(msg.iceServers);
        this.scheduleTurnRefresh(msg.iceExpiresAt);
        return;
      case 'error':
        console.warn('[espia] servidor recusou mensagem:', msg.code);
        return;
    }
  }

  // ---------- View ----------

  private buildView(): RoomView {
    const me = this.me;
    const participants = [
      ...(me ? [{ id: me.id, name: me.nickname, tone: me.tone, you: true, live: !!this.myBroadcast }] : []),
      ...this.peers.map((p) => ({ id: p.id, name: p.nickname, tone: p.tone, you: false, live: !!p.live })),
    ];
    const streams: StreamView[] = this.peers
      .filter((p) => p.live)
      .map((p) => {
        const a = this.audio[p.id] ?? { muted: true, volume: loadVolume(p.nickname) ?? DEFAULT_VOLUME };
        const m = this.media[p.id];
        const flowing = !!m?.video && !!m.stream;
        const failed = m?.conn === 'failed' || !!this.timedOut[p.id];
        return {
          id: p.id,
          name: p.nickname,
          tone: p.tone,
          quality: `${p.live!.resolution}p · ${p.live!.fps} fps · ${p.live!.audio === 'none' ? 'sem som' : 'com som'}`,
          status: flowing ? 'ok' : failed ? 'failed' : 'loading',
          muted: a.muted,
          volume: a.volume,
          stream: flowing ? m.stream! : undefined,
        };
      });
    return {
      roomId: this.code,
      inviteUrl: location.origin + roomPath(this.code),
      status: this.connected ? 'ready' : 'connecting',
      participants,
      capacity: this.capacity,
      streams,
      focusedId: streams.some((s) => s.id === this.focusedId) ? this.focusedId : null,
      myBroadcast: this.myBroadcast && {
        ...this.myBroadcast,
        // Mostra a qualidade efetiva, não só a escolhida.
        resolution: this.quality?.resolution ?? this.myBroadcast.resolution,
        fps: this.quality?.fps ?? this.myBroadcast.fps,
        adjusted: !!this.quality && isAdjusted(this.myBroadcast, this.quality),
        viewers: Object.values(this.media).filter((m) => m.conn === 'connected').length,
      },
      copied: this.copied,
      goLiveOpen: this.goLiveOpen,
      goLivePrefs: this.goLivePrefs,
      permissionDenied: this.permissionDenied,
      toasts: this.toasts,
      canShare: canShare(),
    };
  }

  // ---------- Ações das telas ----------

  private buildActions(): RoomActions {
    return {
      copyInvite: () => {
        navigator.clipboard?.writeText(this.view.inviteUrl).catch(() => {});
        this.copied = true;
        this.toast({ kind: 'copied', text: 'Link copiado — é só mandar pra galera' });
        setTimeout(() => (this.copied = false), 2000);
      },
      openGoLive: () => {
        this.permissionDenied = false;
        this.goLiveOpen = true;
      },
      closeGoLive: () => {
        this.goLiveOpen = false;
      },
      startGoLive: (prefs) => this.startBroadcast(prefs),
      stopGoLive: () => this.stopBroadcast(true),
      dismissPermission: () => {
        this.permissionDenied = false;
      },
      focus: (id) => {
        this.focusedId = id;
      },
      toggleMute: (id) => {
        const a = this.audioOf(id);
        a.muted = !a.muted;
        if (!a.muted && a.volume === 0) a.volume = DEFAULT_VOLUME;
      },
      setVolume: (id, volume) => {
        const a = this.audioOf(id);
        a.volume = volume;
        a.muted = volume === 0;
        const peer = this.peers.find((p) => p.id === id);
        if (peer) saveVolume(peer.nickname, volume);
      },
      retry: (id) => {
        this.timedOut[id] = false;
        this.waitFirstFrame(id);
        this.mesh?.retry(id);
      },
      toastAction: (id) => {
        const peerId = this.toastPeer.get(id);
        if (peerId && this.peers.some((p) => p.id === peerId && p.live)) this.focusedId = peerId;
        this.dismiss(id);
      },
      dismissToast: (id) => this.dismiss(id),
      leave: () => {
        this.destroy();
        this.onLeave();
      },
    };
  }

  private audioOf(id: string) {
    if (!this.audio[id]) {
      const peer = this.peers.find((p) => p.id === id);
      this.audio[id] = { muted: true, volume: (peer && loadVolume(peer.nickname)) ?? DEFAULT_VOLUME };
    }
    return this.audio[id];
  }

  private async startBroadcast(prefs: GoLivePrefs) {
    this.goLivePrefs = prefs;
    saveGoLivePrefs(prefs);
    this.goLiveOpen = false;
    let stream: MediaStream;
    try {
      stream = await startCapture(prefs);
    } catch (err) {
      if (isPermissionDenied(err)) this.permissionDenied = true;
      else this.toast({ kind: 'error', text: 'Não deu pra capturar a tela' });
      return;
    }
    // Antes de qualquer envio: tira o som que não pode sair (specs/002).
    const wanted = prefs.audio && prefs.surface !== 'monitor';
    const a = applyAudioRule(stream, wanted);
    this.stream = stream;
    this.warnedScreenAudio = false;
    const video = stream.getVideoTracks()[0];
    // Botão nativo "Parar de compartilhar".
    video?.addEventListener('ended', () => this.stopBroadcast(true));
    // Trocou o que compartilha no meio ("compartilhar esta aba"): reavalia o som (FR-205).
    video?.addEventListener('configurationchange', () => this.reapplyAudio(stream, wanted));
    stream.addEventListener('addtrack', () => this.reapplyAudio(stream, wanted));
    stream.addEventListener('removetrack', () => this.reapplyAudio(stream, wanted));
    this.myBroadcast = {
      ...prefs,
      startedAt: Date.now(),
      viewers: this.peers.length,
      stream,
      audioKind: a.kind,
      audioReason: a.reason,
    };
    const q = this.quality!;
    this.appliedQuality = q;
    await this.constrainCapture(q);
    this.mesh?.setLocal(stream, q);
    this.sendLive();
    this.toast({ kind: 'live', text: 'Você está ao vivo pra sala' });
    if (a.warn) this.warnScreenAudio();
  }

  /** Aviso do FR-208: no máximo uma vez por transmissão. */
  private warnedScreenAudio = false;
  private warnScreenAudio() {
    if (this.warnedScreenAudio) return;
    this.warnedScreenAudio = true;
    this.toast({ kind: 'live', text: SCREEN_NO_AUDIO });
  }

  private reapplyAudio(stream: MediaStream, wanted: boolean) {
    const b = this.myBroadcast;
    if (!b || this.stream !== stream) return;
    const a = applyAudioRule(stream, wanted);
    // Faixa de áudio pode ter entrado ou saído: manda o estado atual (setLocal é idempotente).
    if (this.quality) this.mesh?.setLocal(stream, this.quality);
    if (a.warn) this.warnScreenAudio();
    if (a.kind !== b.audioKind || a.reason !== b.audioReason) {
      b.audioKind = a.kind;
      b.audioReason = a.reason;
      this.sendLive();
    }
  }

  /** Anuncia a transmissão com a qualidade efetiva (quem assiste vê no subtítulo). */
  private sendLive() {
    const b = this.myBroadcast;
    if (!b) return;
    const q = this.quality ?? b;
    this.signaling.send({ t: 'live', resolution: q.resolution, fps: q.fps, audio: b.audioKind });
  }

  private async applyQuality(q: Quality) {
    const prev = this.appliedQuality;
    if (!this.stream || (prev && prev.resolution === q.resolution && prev.fps === q.fps && prev.maxBitrate === q.maxBitrate)) return;
    this.appliedQuality = q;
    await this.constrainCapture(q);
    this.mesh?.setQuality(q);
    if (!prev || prev.resolution !== q.resolution || prev.fps !== q.fps) this.sendLive();
  }

  /** Reduz na captura (alivia a CPU de todos os envios de uma vez). */
  private async constrainCapture(q: Quality) {
    const track = this.stream?.getVideoTracks()[0];
    if (!track) return;
    const width = q.resolution === 1080 ? 1920 : 1280;
    try {
      await track.applyConstraints({ width: { max: width }, height: { max: q.resolution }, frameRate: { max: q.fps } });
    } catch (err) {
      console.warn('[espia] applyConstraints', err);
    }
  }

  /** Como estou vendo a tela de cada pessoa ao vivo. Quem não está no mapa: escondida. */
  private myViews(): Record<string, ViewMode> {
    const v = this.view;
    const out: Record<string, ViewMode> = {};
    if (this.mobile) {
      // Celular: só a selecionada aparece; as outras ficam nos chips, sem vídeo.
      const selected = v.focusedId ?? v.streams[0]?.id;
      for (const s of v.streams) out[s.id] = s.id === selected ? 'full' : 'hidden';
    } else if (stageLayout(v) === 'focus') {
      for (const s of v.streams) out[s.id] = s.id === v.focusedId ? 'full' : 'thumb';
    } else {
      for (const s of v.streams) out[s.id] = 'full';
    }
    return out;
  }

  private stopBroadcast(notify: boolean) {
    if (!this.stream) return;
    stopCapture(this.stream);
    this.stream = null;
    this.myBroadcast = null;
    this.appliedQuality = null;
    this.mesh?.setLocal(null, null);
    if (notify) this.signaling.send({ t: 'unlive' });
  }

  // ---------- WebRTC ----------

  /** Depois de cada welcome. Mesmo id (resume): mantém as conexões vivas. Id novo: refaz tudo. */
  private syncMesh(myId: string, iceServers: IceServer[]) {
    if (this.mesh && this.mesh.id !== myId) this.closeMedia();
    if (!this.mesh) {
      this.mesh = new Mesh(myId, {
        signal: (to, data) => this.signaling.send({ t: 'signal', to, data }),
        remote: (id, stream) => this.attachRemote(id, stream),
        state: (id, conn) => {
          this.entry(id).conn = conn;
          this.theirViews = this.mesh?.theirViews() ?? {};
        },
        view: () => {
          this.theirViews = this.mesh?.theirViews() ?? {};
        },
      });
    }
    this.mesh.setIceServers(iceServers);
    this.mesh.sync(this.peers.map((p) => p.id));
    this.mesh.setViews(untrack(() => this.myViews()));
    if (this.stream && this.quality) this.mesh.setLocal(this.stream, this.quality);
  }

  private entry(id: string): RemoteMedia {
    return (this.media[id] ??= { stream: null, video: false, conn: 'new' });
  }

  /** Acompanha quando os quadros de vídeo dessa pessoa começam e param de chegar. */
  private attachRemote(id: string, stream: MediaStream) {
    this.entry(id).stream = stream;
    for (const track of stream.getVideoTracks()) {
      const update = () => {
        this.entry(id).video = !track.muted && track.readyState === 'live';
        if (!track.muted) {
          this.clearFrameTimer(id);
          this.timedOut[id] = false;
        }
      };
      track.onmute = update;
      track.onunmute = update;
      track.onended = update;
      update();
    }
  }

  private waitFirstFrame(id: string) {
    this.clearFrameTimer(id);
    this.frameTimers.set(
      id,
      setTimeout(() => {
        if (!this.media[id]?.video) this.timedOut[id] = true;
      }, FIRST_FRAME_TIMEOUT_MS),
    );
  }

  private clearFrameTimer(id: string) {
    clearTimeout(this.frameTimers.get(id));
    this.frameTimers.delete(id);
  }

  private forgetMedia(id: string) {
    this.clearFrameTimer(id);
    delete this.media[id];
    delete this.timedOut[id];
  }

  private scheduleTurnRefresh(expiresAt: number) {
    clearTimeout(this.turnTimer);
    const delay = Math.max(30_000, expiresAt - Date.now() - TURN_REFRESH_EARLY_MS);
    this.turnTimer = setTimeout(() => this.signaling.send({ t: 'turn-refresh' }), delay);
  }

  private closeMedia() {
    this.mesh?.close();
    this.mesh = null;
    clearTimeout(this.turnTimer);
    for (const id of [...this.frameTimers.keys()]) this.clearFrameTimer(id);
    this.media = {};
    this.timedOut = {};
  }

  // ---------- Toasts ----------

  private toast(t: Omit<ToastView, 'id'>): string {
    const id = `t${++this.toastSeq}`;
    this.toasts.push({ id, ...t });
    if (t.kind !== 'error') setTimeout(() => this.dismiss(id), TOAST_MS);
    return id;
  }

  private dismiss(id: string) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
    this.toastPeer.delete(id);
  }
}
