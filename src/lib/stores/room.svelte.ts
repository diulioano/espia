// Sala real: liga a sinalização e a malha WebRTC ao RoomView que as telas já usam com os mocks.
import type { IceServer, Peer, RoomInfo, ServerMessage } from '../../shared/protocol';
import { applyAudioRule, canShareScreen, isPermissionDenied, startCapture, stopCapture } from '../capture';
import {
  cameraPermission,
  canUseCamera,
  isBusy,
  listDevices,
  openPreview,
  requestCamera,
  startCamera,
  stopStream,
  watchDevices,
  type CameraError,
  type CameraPermission,
} from '../camera';
import { audioRole, isCaptureCard, previewIds, suggestAudio, type AudioRole, type DeviceInfo } from '../devices';
import { untrack } from 'svelte';
import { Mesh } from '../mesh';
import { detectNative, matchLastSource, pollSources, sourceKey, startNative, type AudioWarn, type NativeSession } from '../native-capture';
import { CpuGovernor } from '../cpu-governor';
import { countFullViewers, effectiveQuality, isAdjusted, stepDown, type Quality, type ViewMode } from '../quality';
import { pruneUnwatched, stageLayout, viewModes } from '../room-view';
import {
  loadGoLivePrefs,
  forgetOwnerKey,
  loadOwnerKey,
  loadSession,
  loadVolume,
  saveGoLivePrefs,
  saveSession,
  saveVolume,
} from '../prefs';
import { liveLabel } from '../live-label';
import { grabSnapshot, SNAPSHOT_MS } from '../snapshot';
import { roomPath } from '../room-id';
import { forgetRoom, loadSavedRooms, rememberRoom, renameSaved, saveSavedRooms } from '../saved-rooms';
import { deleteRoom, ownerErrorText, renameRoom } from '../api';
import { Signaling, type SignalingEnd } from '../signaling';
import type {
  CameraView,
  CaptureSource,
  DeleteDialogView,
  GoLivePrefs,
  MyBroadcastView,
  NameDialogView,
  RoomActions,
  RoomView,
  ShareSurface,
  StreamView,
  ToastView,
} from '../types';

const DEFAULT_VOLUME = 0.68;
const TOAST_MS = 4_000;
const SCREEN_NO_AUDIO = 'Tela inteira vai sem som. Pra levar o som, transmita a janela do jogo ou uma aba.';
/** Avisos do som no app desktop (specs/005). */
const NATIVE_AUDIO_WARN: Record<AudioWarn, string> = {
  unsupported: 'Seu Windows não deixa separar o som da janela — foi só a imagem.',
  many_discords: 'Mais de um Discord aberto — tela inteira vai sem som.',
};
const BLACK_WARN = 'A tela tá saindo preta — tenta deixar o jogo em modo janela ou sem bordas.';
/** Sem quadro de vídeo nesse tempo depois de "começou a transmitir" → tile "falhou". */
const FIRST_FRAME_TIMEOUT_MS = 10_000;
/** Sem voltar nesse tempo → "Não deu pra voltar pra sala" (specs/011 FR-004). */
const RECONNECT_FAIL_MS = 60_000;
/** Pede credenciais TURN novas esse tempo antes de vencerem. */
const TURN_REFRESH_EARLY_MS = 10 * 60_000;

interface RemoteMedia {
  stream: MediaStream | null;
  /** Só as faixas de som de `stream` (specs/014): toca à parte do vídeo. */
  audio: MediaStream | null;
  /** Chegando quadros de vídeo agora (faixa remota sem "mute"). */
  video: boolean;
  conn: RTCPeerConnectionState;
}

export class RoomStore {
  connected = $state(false);
  ended = $state<SignalingEnd | null>(null);
  /** Ligação com o servidor (specs/011 US1). */
  link = $state<'ok' | 'reconnecting' | 'failed'>('ok');
  /** Rápida ou fixa (vem no welcome, specs/011). */
  roomInfo = $state<RoomInfo>({ kind: 'quick' });
  /** Dono da sala fixa (specs/011 US4). */
  ownerMenu = $state(false);
  nameDialog = $state<NameDialogView | undefined>(undefined);
  deleteDialog = $state<DeleteDialogView | undefined>(undefined);
  /** Confirmou "Apagar sala": o fechamento 4410 que vem em seguida não é surpresa. */
  private deleting = false;
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
  /** Quem eu parei de assistir → início daquela transmissão (specs/014 US1, research R4). Trocado por inteiro. */
  unwatched = $state.raw<Map<string, number>>(new Map());
  /** Página à vista (specs/014 US2): minimizada, na bandeja ou aba de trás = false. Nunca pelo foco. */
  pageVisible = $state(typeof document === 'undefined' || document.visibilityState === 'visible');
  /** Tela em picture-in-picture: continua chegando com a página escondida (specs/014 R3). */
  pipId = $state<string | null>(null);
  /** A sua tela grande no palco, ao vivo (specs/014 US3). Volta a false com Esc, outro foco ou o fim. */
  selfOnStage = $state(false);
  /** Quadro parado da sua transmissão (specs/014 US3). ImageBitmap não vira proxy: $state.raw. */
  snapshot = $state.raw<ImageBitmap | null>(null);
  /** Último modo mandado a cada pessoa: hidden → visível de novo reinicia a espera do vídeo. */
  private sentModes: Record<string, ViewMode> = {};
  /** Seletor do app desktop (specs/005): null = ainda descobrindo se a ponte existe. */
  pickerMode = $state<'native' | 'browser' | null>(null);
  sources = $state<CaptureSource[]>([]);
  sourcesLoading = $state(false);
  selectedId = $state<string | null>(null);
  /** Aba aberta no seletor (liga/desliga as prévias de câmera). */
  tab = $state<ShareSurface>('window');
  /** Aba Câmeras (specs/006). */
  camPermission = $state<CameraPermission>('unknown');
  /** A câmera não abriu (ocupada, não encontrada…): a aba explica e oferece tentar de novo. */
  camError = $state<CameraError | null>(null);
  camVideos = $state<DeviceInfo[]>([]);
  camAudios = $state<DeviceInfo[]>([]);
  /** MediaStream não pode virar proxy: $state.raw, trocado por inteiro. */
  camPreviews = $state.raw<Record<string, MediaStream>>({});
  camBusy = $state<Record<string, boolean>>({});
  camVideoId = $state<string | null>(null);
  camAudioId = $state<string | null>(null);
  camAudioRole = $state<AudioRole>('voice');
  camFacing = $state<'user' | 'environment'>('user');

  /** Degraus baixados porque o PC não dá conta de comprimir (specs/010). Sem aviso: o indicador já mostra. */
  cpuSteps = $state(0);
  private governor = new CpuGovernor();
  private sendBusy = false;
  private sendTicks = 0;
  private sendLast = new Map<string, number>();

  /** Qualidade efetiva da minha transmissão: o escolhido, com teto pelo nº de espectadores e pela CPU. */
  readonly quality: Quality | null = $derived(
    this.myBroadcast ? effectiveQuality(this.myBroadcast, countFullViewers(Object.values(this.theirViews)), this.cpuSteps) : null,
  );
  private readonly broadcasting = $derived(!!this.myBroadcast);

  readonly view: RoomView = $derived.by(() => this.buildView());
  readonly actions: RoomActions;

  private signaling: Signaling;
  private toastSeq = 0;
  /** Toast "começou a transmitir" → quem focar no "Assistir". */
  private toastPeer = new Map<string, string>();
  /** Vira 'failed' depois de RECONNECT_FAIL_MS em 'reconnecting'. */
  private downTimer: ReturnType<typeof setTimeout> | undefined;
  private stream: MediaStream | null = null;
  private mesh: Mesh | null = null;
  private frameTimers = new Map<string, ReturnType<typeof setTimeout>>();
  private turnTimer: ReturnType<typeof setTimeout> | undefined;
  private native: NativeSession | null = null;
  private stopPoll: (() => void) | null = null;
  /** A pessoa trocou o som/perfil nesta abertura: não sugerir por cima. */
  private camAudioManual = false;
  private camRoleManual = false;
  private stopWatchDevices: (() => void) | null = null;
  /** Aumenta a cada entrada/saída da aba: prévias de uma abertura antiga são descartadas. */
  private camGen = 0;
  private muteTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(
    readonly code: string,
    nickname: string,
    private onLeave: () => void,
    /** Acabou de criar a sala fixa (specs/011 S1): copia o convite e avisa no primeiro welcome. */
    private justCreated = false,
  ) {
    this.signaling = new Signaling({
      code,
      nickname,
      session: loadSession(code),
      onMessage: (m) => this.receive(m),
      onSession: (s) => saveSession(code, s),
      // Queda não é sair (specs/011 US1): a sala continua na tela, com o aviso no palco.
      onStatus: (s) => {
        if (s === 'reconnecting' && this.connected && this.link === 'ok') this.markDown();
      },
      onBack: () => {
        if (this.link === 'ok') return;
        clearTimeout(this.downTimer);
        this.link = 'ok';
        this.toast({ kind: 'copied', text: 'De volta!' });
      },
      onEnd: (reason) => {
        this.closeMedia();
        this.stopBroadcast(false);
        // Eu mesmo apaguei a sala: volto pro início, sem o aviso de "apagaram a sala".
        if (reason === 'deleted' && this.deleting) return this.leaveAfterDelete();
        this.ended = reason;
      },
    });
    this.actions = this.buildActions();
    document.addEventListener('visibilitychange', this.onVisibility);
    // App desktop com a ponte de captura? Fora dele, o seletor da 004.
    detectNative().then((ok) => {
      if (this.pickerMode === null) this.pickerMode = ok ? 'native' : 'browser';
      if (this.goLiveOpen) this.startPicker();
    });
    this.stopEffects = $effect.root(() => {
      // Avisa cada pessoa de como estou vendo a tela dela (cheia, miniatura ou escondida).
      $effect(() => {
        const views = this.myViews();
        untrack(() => this.applyViews(views));
      });
      // Teto de qualidade mudou durante a transmissão: aplica na captura, no envio e avisa a sala.
      $effect(() => {
        const q = this.quality;
        if (q) untrack(() => this.applyQuality(q));
      });
      // Prévia parada (specs/014 US3): um quadro logo ao começar e a cada SNAPSHOT_MS, só com a página à vista.
      $effect(() => {
        const track = this.myBroadcast?.stream?.getVideoTracks()[0];
        if (!track || !this.pageVisible) return;
        let alive = true;
        const take = async () => {
          const shot = await grabSnapshot(track);
          if (!alive) return shot?.close();
          if (!shot) return;
          untrack(() => this.snapshot)?.close();
          this.snapshot = shot;
        };
        void take();
        const timer = setInterval(() => void take(), SNAPSHOT_MS);
        return () => {
          alive = false;
          clearInterval(timer);
        };
      });
      // Parou de transmitir: some a prévia e a sua tela sai do palco.
      $effect(() => {
        if (this.broadcasting) return;
        untrack(() => {
          this.snapshot?.close();
          this.snapshot = null;
          this.selfOnStage = false;
        });
      });
      // Transmitindo: uma vez por segundo, olha se o motor está limitando por CPU (specs/010, research R5).
      $effect(() => {
        if (!this.broadcasting) return;
        untrack(() => this.resetCpu());
        const timer = setInterval(() => void this.watchSend(), 1000);
        return () => clearInterval(timer);
      });
    });
  }

  private stopEffects: () => void;
  /** Última qualidade aplicada/anunciada, para só agir quando mudar. */
  private appliedQuality: Quality | null = null;

  /** Saiu da tela da sala (botão, voltar do navegador). */
  destroy() {
    document.removeEventListener('visibilitychange', this.onVisibility);
    this.stopPicker();
    this.leaveCameraTab();
    this.stopEffects();
    this.closeMedia();
    this.stopBroadcast(false);
    clearTimeout(this.downTimer);
    if (!this.ended) this.signaling.leave();
  }

  private onVisibility = () => {
    const visible = document.visibilityState === 'visible';
    if (visible === this.pageVisible) return;
    this.pageVisible = visible;
    console.info(`[espia] página ${visible ? 'visível' : 'escondida'}`);
  };

  /** Escolhas de "parar de assistir" que ainda valem depois de uma mudança na lista. */
  private pruneWatch() {
    const next = pruneUnwatched(this.unwatched, this.peers);
    if (next.size !== this.unwatched.size) this.unwatched = next;
    if (this.pipId && !this.peers.some((p) => p.id === this.pipId && p.live)) this.pipId = null;
  }

  /** Manda os modos e, pra quem volta a ser visto, espera o vídeo de novo (não conta o tempo escondido). */
  private applyViews(views: Record<string, ViewMode>) {
    for (const [id, mode] of Object.entries(views)) {
      if (mode !== 'hidden' && (this.sentModes[id] ?? 'hidden') === 'hidden' && !this.media[id]?.video) {
        this.timedOut[id] = false;
        if (this.peers.some((p) => p.id === id && p.live)) this.waitFirstFrame(id);
      }
    }
    this.sentModes = views;
    this.mesh?.setViews(views);
  }

  private copyInvite(text: string) {
    navigator.clipboard?.writeText(this.view.inviteUrl).catch(() => {});
    this.copied = true;
    this.toast({ kind: 'copied', text });
    setTimeout(() => (this.copied = false), 2000);
  }

  /** Apaguei a sala: vai pro início uma vez só (a resposta e o fechamento 4410 chegam em qualquer ordem). */
  private leftAfterDelete = false;
  private leaveAfterDelete() {
    if (this.leftAfterDelete) return;
    this.leftAfterDelete = true;
    // Antes de ir pro início, que lê a lista na hora.
    saveSavedRooms(forgetRoom(loadSavedRooms(), this.code));
    forgetOwnerKey(this.code);
    this.destroy();
    this.onLeave();
  }

  /** Caiu (ou "Tentar de novo"): 'reconnecting' e, sem volta em 60 s, 'failed'. */
  private markDown() {
    clearTimeout(this.downTimer);
    this.link = 'reconnecting';
    this.downTimer = setTimeout(() => (this.link = 'failed'), RECONNECT_FAIL_MS);
  }

  // ---------- Mensagens do servidor ----------

  private receive(msg: ServerMessage) {
    switch (msg.t) {
      case 'welcome': {
        this.me = msg.you;
        this.peers = msg.peers;
        this.capacity = msg.capacity;
        this.connected = true;
        this.roomInfo = msg.room ?? { kind: 'quick' };
        // Entrou numa sala fixa: fica em Suas salas deste aparelho (specs/011 FR-016).
        if (this.roomInfo.kind === 'fixed') {
          saveSavedRooms(rememberRoom(loadSavedRooms(), { code: this.code, name: this.roomInfo.name }, Date.now()));
        }
        if (this.justCreated) {
          this.justCreated = false;
          this.copyInvite('Sala criada — fixe o link no grupo');
        }
        if (this.focusedId && !this.peers.some((p) => p.id === this.focusedId)) this.focusedId = null;
        this.pruneWatch();
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
      case 'room-renamed': {
        // O dono renomeou (specs/011 FR-027): título e Suas salas mudam sem recarregar.
        if (this.roomInfo.kind === 'fixed') this.roomInfo = { kind: 'fixed', name: msg.name };
        saveSavedRooms(renameSaved(loadSavedRooms(), this.code, msg.name));
        break;
      }
      case 'peer-left': {
        const peer = this.peers.find((p) => p.id === msg.id);
        this.peers = this.peers.filter((p) => p.id !== msg.id);
        if (this.focusedId === msg.id) this.focusedId = null;
        this.mesh?.remove(msg.id);
        this.forgetMedia(msg.id);
        this.pruneWatch();
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
        this.pruneWatch();
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
      ...(me ? [{ id: me.id, name: me.nickname, tone: me.tone, you: true, live: !!this.myBroadcast, liveLabel: this.myBroadcast?.label }] : []),
      ...this.peers.map((p) => ({ id: p.id, name: p.nickname, tone: p.tone, you: false, live: !!p.live, liveLabel: p.live?.label })),
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
          audioStream: m?.audio ?? undefined,
          source: p.live!.source,
          label: p.live!.label,
          watching: !this.unwatched.has(p.id),
        };
      });
    // A sua tela no palco (specs/014 US3): uma tela a mais, sempre muda, fora do que vai pra sala.
    const self = this.selfOnStage && me && this.myBroadcast?.stream ? this.myBroadcast : null;
    if (self && me) {
      streams.push({
        id: me.id,
        name: 'Você',
        tone: me.tone,
        status: 'ok',
        muted: true,
        volume: 0,
        stream: self.stream,
        source: self.source,
        label: self.label,
        you: true,
      });
    }
    const focusedId = self && me ? me.id : streams.some((s) => s.id === this.focusedId) ? this.focusedId : null;
    return {
      roomId: this.code,
      inviteUrl: location.origin + roomPath(this.code),
      status: this.connected ? 'ready' : 'connecting',
      participants,
      capacity: this.capacity,
      streams,
      focusedId,
      myBroadcast: this.myBroadcast && {
        ...this.myBroadcast,
        // Mostra a qualidade efetiva, não só a escolhida.
        resolution: this.quality?.resolution ?? this.myBroadcast.resolution,
        fps: this.quality?.fps ?? this.myBroadcast.fps,
        adjusted: !!this.quality && isAdjusted(this.myBroadcast, this.quality),
        // Quem está recebendo o vídeo de verdade (specs/014): parar de assistir ou esconder a janela não conta.
        viewers: Object.values(this.theirViews).filter((m) => m !== 'hidden').length,
        snapshot: this.snapshot,
        onStage: this.selfOnStage,
      },
      link: this.link,
      room: this.roomInfo.kind === 'fixed' ? { ...this.roomInfo, own: !!loadOwnerKey(this.code) } : { kind: 'quick' },
      ownerMenu: this.ownerMenu,
      nameDialog: this.nameDialog,
      deleteDialog: this.deleteDialog,
      copied: this.copied,
      goLiveOpen: this.goLiveOpen,
      goLivePrefs: this.goLivePrefs,
      picker: {
        mode: this.pickerMode === 'native' ? 'native' : 'browser',
        sources: this.sources,
        loading: this.sourcesLoading,
        selectedId: this.selectedId,
      },
      camera: this.cameraView(),
      permissionDenied: this.permissionDenied,
      toasts: this.toasts,
      canShare: canShareScreen() || canUseCamera(),
      canShareScreen: canShareScreen(),
    };
  }

  // ---------- Ações das telas ----------

  private buildActions(): RoomActions {
    return {
      copyInvite: () => this.copyInvite('Link copiado — é só mandar pra galera'),
      openGoLive: () => {
        // Sem servidor não dá pra avisar a sala (FR-007).
        if (this.link !== 'ok') return;
        this.permissionDenied = false;
        this.goLiveOpen = true;
        this.startPicker();
        // Celular: só câmera (specs/006 US4).
        this.tab = canShareScreen() ? this.goLivePrefs.surface : 'camera';
        if (this.tab === 'camera') void this.enterCameraTab();
      },
      closeGoLive: () => {
        this.goLiveOpen = false;
        this.stopPicker();
        this.leaveCameraTab();
      },
      setSurface: (surface) => {
        if (surface === this.tab && surface === 'camera' && this.camGen > 0) return;
        this.tab = surface;
        if (surface === 'camera') void this.enterCameraTab();
        else this.leaveCameraTab();
      },
      allowCamera: () => void this.allowCamera(),
      selectCamera: (id) => {
        if (!this.camVideos.some((v) => v.id === id)) return;
        this.camVideoId = id;
        this.suggestCameraAudio();
        void this.updatePreviews();
      },
      selectAudio: (id) => {
        this.camAudioId = id;
        this.camAudioManual = true;
        const a = this.camAudios.find((x) => x.id === id);
        if (a && !this.camRoleManual) this.camAudioRole = audioRole(a.label, this.pairedWithCapture(a));
      },
      setAudioRole: (role) => {
        this.camAudioRole = role;
        this.camRoleManual = true;
      },
      setFacing: (facing) => {
        this.camFacing = facing;
      },
      startGoLive: (prefs) => this.startBroadcast(prefs),
      selectSource: (id) => {
        if (this.sources.some((s) => s.id === id)) this.selectedId = id;
      },
      stopGoLive: () => this.stopBroadcast(true),
      dismissPermission: () => {
        this.permissionDenied = false;
      },
      focus: (id) => {
        // Esc ou outra tela: a sua sai do palco (specs/014 US3).
        if (this.selfOnStage && id !== this.me?.id) {
          this.selfOnStage = false;
          if (id === null) return;
        }
        this.focusedId = id;
      },
      toggleSelfStage: () => {
        this.selfOnStage = !this.selfOnStage && !!this.myBroadcast?.stream;
      },
      pip: (id) => {
        if (id === null || this.peers.some((p) => p.id === id && p.live)) this.pipId = id;
      },
      watch: (id, on) => {
        const peer = this.peers.find((p) => p.id === id);
        if (!peer?.live) return;
        const next = new Map(this.unwatched);
        if (on) next.delete(id);
        else next.set(id, peer.live.startedAt);
        this.unwatched = next;
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
      toggleOwnerMenu: (open) => (this.ownerMenu = open && this.link === 'ok'),
      openRename: () => {
        this.ownerMenu = false;
        if (this.roomInfo.kind !== 'fixed' || this.link !== 'ok') return;
        this.nameDialog = { mode: 'rename', value: this.roomInfo.name, busy: false, error: null };
      },
      submitRename: async (name) => {
        const key = loadOwnerKey(this.code);
        if (!key || !this.nameDialog) return;
        this.nameDialog = { ...this.nameDialog, value: name, busy: true, error: null };
        const res = await renameRoom(this.code, key, name);
        if (res.ok) this.nameDialog = undefined; // O nome novo chega pelo room-renamed.
        else if (this.nameDialog) this.nameDialog = { ...this.nameDialog, busy: false, error: ownerErrorText(res.reason) };
      },
      openDelete: () => {
        this.ownerMenu = false;
        if (this.roomInfo.kind !== 'fixed' || this.link !== 'ok') return;
        this.deleteDialog = { name: this.roomInfo.name, people: this.peers.length, busy: false, error: null };
      },
      confirmDelete: async () => {
        const key = loadOwnerKey(this.code);
        if (!key || !this.deleteDialog) return;
        this.deleteDialog = { ...this.deleteDialog, busy: true, error: null };
        this.deleting = true;
        const res = await deleteRoom(this.code, key);
        if (!res.ok) {
          this.deleting = false;
          if (this.deleteDialog) this.deleteDialog = { ...this.deleteDialog, busy: false, error: ownerErrorText(res.reason) };
          return;
        }
        this.leaveAfterDelete();
      },
      closeDialogs: () => {
        this.ownerMenu = false;
        this.nameDialog = undefined;
        this.deleteDialog = undefined;
      },
      goHome: () => {
        if (this.myBroadcast && !confirm('Você está transmitindo. Parar e sair da sala?')) return;
        this.destroy();
        this.onLeave();
      },
      retryConnection: () => {
        this.markDown();
        this.signaling.retryNow();
      },
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

  // ---------- Seletor do app desktop (specs/005) ----------

  /** Grade aberta: lista agora e a cada 2 s (FR-504). Falhou → seletor do navegador (FR-511). */
  private startPicker() {
    if (this.pickerMode !== 'native' || this.stopPoll) return;
    let first = true;
    this.sourcesLoading = this.sources.length === 0;
    this.stopPoll = pollSources(
      (sources) => {
        this.sources = sources;
        this.sourcesLoading = false;
        // A escolhida fechou: some a marcação (US1 cenário 5).
        if (this.selectedId && !sources.some((s) => s.id === this.selectedId)) this.selectedId = null;
        // Primeira lista: volta a última fonte, se ainda existir (US5).
        if (first && !this.selectedId) this.selectedId = matchLastSource(sources, this.goLivePrefs.lastSourceKey);
        first = false;
      },
      (err) => {
        console.warn('[espia] lista de janelas falhou; usando o seletor do Windows', err);
        this.pickerMode = 'browser';
        this.sourcesLoading = false;
        this.stopPicker();
      },
    );
  }

  private stopPicker() {
    this.stopPoll?.();
    this.stopPoll = null;
  }

  private async startBroadcast(prefs: GoLivePrefs) {
    this.stopPicker();
    if (prefs.surface === 'camera') return this.startCameraBroadcast(prefs);
    this.leaveCameraTab();
    const source = this.pickerMode === 'native' ? this.sources.find((s) => s.id === this.selectedId) : undefined;
    if (source) return this.startNativeBroadcast({ ...prefs, surface: source.kind, lastSourceKey: sourceKey(source) }, source);
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
      // O tipo que o navegador entregou (specs/012 FR-004): ele não conta o nome da janela/aba.
      label: liveLabel({ kind: 'browser', surface: video?.getSettings().displaySurface }),
    };
    const q = this.quality!;
    this.appliedQuality = q;
    await this.constrainCapture(q);
    this.mesh?.setLocal(stream, q);
    this.sendLive();
    this.toast({ kind: 'live', text: 'Você está ao vivo pra sala' });
    if (a.warn) this.warnScreenAudio();
  }

  /** App desktop: o próprio app captura a fonte escolhida na grade, sem o seletor do Windows. */
  private async startNativeBroadcast(prefs: GoLivePrefs, source: CaptureSource) {
    this.goLivePrefs = prefs;
    saveGoLivePrefs(prefs);
    this.goLiveOpen = false;
    let session: NativeSession;
    try {
      session = await startNative(prefs, source.id, {
        ended: () => this.stopBroadcast(true),
        audio: (kind, warn) => {
          const b = this.myBroadcast;
          if (!b || this.native !== session) return;
          if (warn) this.toast({ kind: 'error', text: NATIVE_AUDIO_WARN[warn] });
          if (b.audioKind !== kind) {
            b.audioKind = kind;
            this.sendLive();
          }
        },
        warn: () => this.toast({ kind: 'error', text: BLACK_WARN }),
      });
    } catch (err) {
      if (err === 'source_gone') {
        this.toast({ kind: 'error', text: 'Essa janela fechou — escolhe outra.' });
        this.selectedId = null;
      } else {
        // FR-511: sem erro bloqueante; a próxima tentativa usa o seletor do Windows.
        console.warn('[espia] captura nativa falhou', err);
        this.pickerMode = 'browser';
        this.toast({ kind: 'error', text: 'Não deu pra capturar pelo app — tenta de novo, agora pelo seletor do Windows.' });
      }
      this.goLiveOpen = true;
      this.startPicker();
      return;
    }
    const stream = session.stream;
    this.native = session;
    this.stream = stream;
    this.warnedScreenAudio = false;
    // O som pode entrar/sair no meio (vigia do Discord): a malha manda o estado atual.
    const resend = () => {
      if (this.stream === stream && this.quality) this.mesh?.setLocal(stream, this.quality);
    };
    stream.addEventListener('addtrack', resend);
    stream.addEventListener('removetrack', resend);
    this.myBroadcast = {
      ...prefs,
      startedAt: Date.now(),
      viewers: this.peers.length,
      stream,
      audioKind: session.result.audio,
      sourceTitle: source.title,
      // Só o programa, nunca o título da janela (specs/012 FR-002).
      label: liveLabel(
        source.kind === 'monitor'
          ? { kind: 'native-monitor', title: source.title, monitors: this.sources.filter((s) => s.kind === 'monitor').length }
          : { kind: 'native-window', app: source.app },
      ),
    };
    const q = this.quality!;
    this.appliedQuality = q;
    await this.constrainCapture(q);
    this.mesh?.setLocal(stream, q);
    this.sendLive();
    this.toast({ kind: 'live', text: 'Você está ao vivo pra sala' });
    if (session.result.audioWarn) this.toast({ kind: 'error', text: NATIVE_AUDIO_WARN[session.result.audioWarn] });
  }

  // ---------- Câmera / placa de captura (specs/006) ----------

  private cameraView(): CameraView {
    return {
      permission: this.camPermission,
      devices: this.camVideos.map((v) => ({
        id: v.id,
        label: v.label,
        kind: isCaptureCard(v.label) ? 'capture' : 'camera',
        preview: this.camPreviews[v.id] ?? null,
        busy: !!this.camBusy[v.id],
      })),
      audios: this.camAudios.map((a) => ({ id: a.id, label: a.label })),
      videoId: this.camVideoId,
      audioId: this.camAudioId,
      audioRole: this.camAudioRole,
      mobile: !canShareScreen(),
      facing: this.camFacing,
      error: this.camError ?? undefined,
    };
  }

  /** Abriu a aba: só consulta a permissão (FR-602); se já foi dada, lista e liga as prévias. */
  private async enterCameraTab() {
    const gen = ++this.camGen;
    this.camAudioManual = this.camRoleManual = false;
    if (!canShareScreen()) return; // Celular: frontal/traseira, sem lista nem prévias.
    if (this.camPermission !== 'granted') this.camPermission = await cameraPermission();
    if (gen !== this.camGen) return;
    if (this.camPermission === 'granted') await this.refreshDevices(gen);
  }

  private async allowCamera() {
    const gen = this.camGen;
    const r = await requestCamera();
    this.camPermission = r.permission;
    this.camError = r.error ?? null;
    // Abriu a câmera: os nomes já estão liberados, não precisa abrir de novo.
    if (r.permission === 'granted' && gen === this.camGen && this.goLiveOpen) await this.refreshDevices(gen, true);
  }

  /** Saiu da aba, fechou o seletor ou vai transmitir: nenhuma prévia fica ligada (FR-604). */
  private leaveCameraTab() {
    this.camGen++;
    this.stopWatchDevices?.();
    this.stopWatchDevices = null;
    for (const s of Object.values(this.camPreviews)) stopStream(s);
    this.camPreviews = {};
  }

  private async refreshDevices(gen: number, retried = false): Promise<void> {
    const { videos, audios, unnamed } = await listDevices();
    if (gen !== this.camGen) return;
    // Permissão dada, mas sem nomes ainda: abre e fecha a câmera uma vez (a pessoa já clicou na aba).
    if (unnamed && !retried) {
      const r = await requestCamera();
      this.camPermission = r.permission;
      this.camError = r.error ?? null;
      if (gen !== this.camGen || r.permission !== 'granted') return;
      return this.refreshDevices(gen, true);
    }
    if (videos.length) this.camError = null;
    this.camVideos = videos;
    this.camAudios = audios;
    this.stopWatchDevices ??= watchDevices(() => void this.refreshDevices(this.camGen));
    // Escolha: a que já estava, senão a última transmitida (pelo nome, FR-609), senão a primeira.
    if (!videos.some((v) => v.id === this.camVideoId)) {
      this.camVideoId = (videos.find((v) => v.label === this.goLivePrefs.lastVideoLabel) ?? videos[0])?.id ?? null;
      this.suggestCameraAudio();
    }
    if (this.camAudioId && !audios.some((a) => a.id === this.camAudioId)) this.camAudioId = null;
    await this.updatePreviews();
  }

  private pairedWithCapture(a: DeviceInfo) {
    return this.camVideos.some((v) => v.groupId && v.groupId === a.groupId && isCaptureCard(v.label));
  }

  /** Som sugerido para a câmera escolhida (R4), a menos que a pessoa já tenha escolhido. */
  private suggestCameraAudio() {
    const video = this.camVideos.find((v) => v.id === this.camVideoId);
    if (!video) return;
    if (!this.camAudioManual) {
      const last = this.camAudios.find((a) => a.label === this.goLivePrefs.lastAudioLabel);
      const pick = suggestAudio(video, this.camAudios) ?? (video.label === this.goLivePrefs.lastVideoLabel ? last : null) ?? null;
      this.camAudioId = pick?.id ?? null;
    }
    if (!this.camRoleManual) {
      const a = this.camAudios.find((x) => x.id === this.camAudioId);
      this.camAudioRole =
        video.label === this.goLivePrefs.lastVideoLabel && this.goLivePrefs.lastAudioRole
          ? this.goLivePrefs.lastAudioRole
          : a
            ? audioRole(a.label, this.pairedWithCapture(a))
            : 'voice';
    }
  }

  /** Até 4 prévias ao vivo, a escolhida primeiro (FR-603). */
  private async updatePreviews() {
    const gen = this.camGen;
    const want = new Set(previewIds(this.camVideos.map((v) => ({ id: v.id, busy: this.camBusy[v.id] })), this.camVideoId));
    const next: Record<string, MediaStream> = {};
    for (const [id, s] of Object.entries(this.camPreviews)) {
      if (want.has(id)) next[id] = s;
      else stopStream(s);
    }
    this.camPreviews = next;
    // Uma por vez: alguns drivers não gostam de abrir várias câmeras ao mesmo tempo.
    for (const id of want) {
      if (this.camPreviews[id]) continue;
      const { stream, busy } = await openPreview(id);
      if (gen !== this.camGen || !this.goLiveOpen || this.tab !== 'camera') {
        stopStream(stream);
        return;
      }
      if (busy) this.camBusy[id] = true;
      if (stream) this.camPreviews = { ...this.camPreviews, [id]: stream };
    }
  }

  private async startCameraBroadcast(prefs: GoLivePrefs) {
    const mobile = !canShareScreen();
    const video = this.camVideos.find((v) => v.id === this.camVideoId);
    if (!mobile && !video) return;
    const audio = prefs.audio ? this.camAudios.find((a) => a.id === this.camAudioId) : undefined;
    // Celular: 720p30 fixo (bateria/4G, research R5).
    const chosen: GoLivePrefs = {
      ...prefs,
      ...(mobile ? { resolution: 720 as const, fps: 30 as const } : {}),
      ...(video ? { lastVideoLabel: video.label } : {}),
      ...(audio ? { lastAudioLabel: audio.label, lastAudioRole: this.camAudioRole } : {}),
    };
    this.goLivePrefs = chosen;
    saveGoLivePrefs(chosen);
    this.goLiveOpen = false;
    // Fecha as prévias antes: alguns drivers de placa não abrem a mesma câmera duas vezes.
    this.leaveCameraTab();
    let stream: MediaStream;
    try {
      stream = await startCamera({
        videoId: mobile ? undefined : video!.id,
        facing: this.camFacing,
        // Celular: o microfone padrão, como voz.
        audioId: mobile ? (prefs.audio ? '' : null) : (audio?.id ?? null),
        audioRole: mobile ? 'voice' : this.camAudioRole,
        resolution: chosen.resolution,
        fps: chosen.fps,
      });
    } catch (err) {
      if (isPermissionDenied(err)) this.camPermission = 'denied';
      if (isBusy(err)) this.toast({ kind: 'error', text: 'Essa câmera está sendo usada por outro programa.' });
      else if (!isPermissionDenied(err)) this.toast({ kind: 'error', text: 'Não deu pra abrir a câmera.' });
      this.goLiveOpen = true;
      this.tab = 'camera';
      void this.enterCameraTab();
      return;
    }
    this.stream = stream;
    const track = stream.getVideoTracks()[0];
    // Câmera desplugada / celular trocou de app: a transmissão acaba (FR-608). O som nunca fica
    // sem a câmera (FR-606b): stopBroadcast para todas as faixas.
    track?.addEventListener('ended', () => this.stopBroadcast(true));
    if (mobile && track) {
      track.addEventListener('mute', () => {
        clearTimeout(this.muteTimer);
        this.muteTimer = setTimeout(() => this.stream === stream && this.stopBroadcast(true), 3_000);
      });
      track.addEventListener('unmute', () => clearTimeout(this.muteTimer));
    }
    this.myBroadcast = {
      ...chosen,
      startedAt: Date.now(),
      viewers: this.peers.length,
      stream,
      audioKind: stream.getAudioTracks().length ? 'device' : 'none',
      source: 'camera',
      sourceTitle: video?.label ?? (this.camFacing === 'user' ? 'câmera frontal' : 'câmera traseira'),
      // No celular, "frontal/traseira" diz mais que o nome técnico do sensor (specs/012).
      label: liveLabel(canShareScreen() ? { kind: 'camera', title: video?.label } : { kind: 'camera', title: undefined, facing: this.camFacing }),
    };
    const q = this.quality!;
    this.appliedQuality = q;
    await this.constrainCapture(q);
    this.mesh?.setLocal(stream, q);
    this.sendLive();
    this.toast({ kind: 'live', text: 'Sua câmera está ao vivo pra sala' });
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
    // "Compartilhar esta aba" no meio: o tipo pode ter mudado (specs/012).
    const label = liveLabel({ kind: 'browser', surface: stream.getVideoTracks()[0]?.getSettings().displaySurface });
    const relabel = label !== b.label;
    b.label = label;
    // Faixa de áudio pode ter entrado ou saído: manda o estado atual (setLocal é idempotente).
    if (this.quality) this.mesh?.setLocal(stream, this.quality);
    if (a.warn) this.warnScreenAudio();
    if (relabel || a.kind !== b.audioKind || a.reason !== b.audioReason) {
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
    this.signaling.send({
      t: 'live',
      resolution: q.resolution,
      fps: q.fps,
      audio: b.audioKind,
      source: b.source ?? 'screen',
      ...(b.label ? { label: b.label } : {}),
    });
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
    // App desktop: o track é gerado; quem reduz é o app (capture_set_quality).
    if (this.native) {
      try {
        await this.native.setQuality(q.resolution, q.fps);
      } catch (err) {
        console.warn('[espia] capture_set_quality', err);
      }
      return;
    }
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
    return viewModes({
      streams: v.streams,
      layout: stageLayout(v),
      focusedId: v.focusedId,
      mobile: this.mobile,
      unwatched: new Set(this.unwatched.keys()),
      pageVisible: this.pageVisible,
      pipId: this.pipId,
      selfId: this.selfOnStage ? (this.me?.id ?? null) : null,
    });
  }

  private resetCpu() {
    this.governor.reset();
    this.cpuSteps = 0;
    this.sendTicks = 0;
    this.sendLast.clear();
  }

  /** CPU no limite em alguma pessoa em tela cheia por 10 s → um degrau a menos; 60 s de folga → um a mais. */
  private async watchSend() {
    const q = this.quality;
    if (!this.mesh || !q || this.sendBusy) return;
    this.sendBusy = true;
    try {
      const all = await this.mesh.sendStats();
      const full = all.filter((x) => x.view === 'full');
      if (full.length) {
        const down = stepDown(q);
        const canGoDown = down.resolution !== q.resolution || down.fps !== q.fps;
        if (this.governor.tick(full.some((x) => x.stats.limitation === 'cpu'), canGoDown)) this.cpuSteps = this.governor.steps;
      }
      // Log de envio a cada 10 s: Mbps por pessoa e por que o motor está limitando (SC-1003).
      if (++this.sendTicks % 10 === 0) {
        const pessoas: Record<string, string> = {};
        let total = 0;
        for (const { id, view, stats } of all) {
          const mbps = ((stats.bytesSent - (this.sendLast.get(id) ?? stats.bytesSent)) * 8) / 10 / 1e6;
          this.sendLast.set(id, stats.bytesSent);
          total += mbps;
          pessoas[id.slice(0, 6)] = `${mbps.toFixed(1)} Mbps ${view} ${stats.limitation} pli=${stats.pli} ${stats.encoder}${stats.hardware ? ' (placa)' : ''}`;
        }
        console.debug('[espia] envio', { total: `${total.toFixed(1)} Mbps`, qualidade: `${q.resolution}p${q.fps}`, cpuSteps: this.cpuSteps, pessoas });
      }
    } catch (err) {
      console.warn('[espia] stats de envio', err);
    } finally {
      this.sendBusy = false;
    }
  }

  private stopBroadcast(notify: boolean) {
    if (!this.stream) return;
    if (this.native) {
      const native = this.native;
      this.native = null;
      void native.stop();
    } else stopCapture(this.stream);
    clearTimeout(this.muteTimer);
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
    untrack(() => this.applyViews(this.myViews()));
    if (this.stream && this.quality) this.mesh.setLocal(this.stream, this.quality);
  }

  private entry(id: string): RemoteMedia {
    return (this.media[id] ??= { stream: null, audio: null, video: false, conn: 'new' });
  }

  /** Acompanha quando os quadros de vídeo dessa pessoa começam e param de chegar. */
  private attachRemote(id: string, stream: MediaStream) {
    const e = this.entry(id);
    e.stream = stream;
    const sound = stream.getAudioTracks();
    const had = e.audio?.getAudioTracks() ?? [];
    if (sound.length !== had.length || sound.some((t, i) => t !== had[i])) e.audio = sound.length ? new MediaStream(sound) : null;
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
        // Escondida pra mim (parei de assistir, página escondida): o vídeo não vem mesmo.
        if ((this.sentModes[id] ?? 'hidden') === 'hidden') return;
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
