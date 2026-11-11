// Uma RTCPeerConnection com outra pessoa da sala (architecture.md §4.5).
// "Perfect negotiation": os dois lados podem renegociar; quando as ofertas se cruzam, o lado
// educado (polite) desiste da sua. Candidatos ICE vão aos poucos (trickle) pela sinalização.
import { directIceServers, isSafeSequence, sameTransmission, type TransmissionKey } from '../shared/media';
import type { IceServer, SignalData } from '../shared/protocol';
import { contentPreference, sendParams, wantsSound, type Quality, type ViewMode } from './quality';
import { MediaStats, type MediaStatsSample, type StatsRow } from './media-stats';

export interface PeerCallbacks {
  signal(data: SignalData): void;
  /** Stream remoto desta pessoa (sempre o mesmo objeto; as faixas dele ligam e desligam). */
  remote(stream: MediaStream): void;
  state(state: RTCPeerConnectionState): void;
  /** Como essa pessoa está vendo a NOSSA tela mudou. */
  view(mode: ViewMode): void;
}

const AUDIO_BITRATE = 128_000;

export interface SendStats {
  bytesSent: number;
  /** 'none' | 'cpu' | 'bandwidth' | 'other' (qualityLimitationReason do motor). */
  limitation: string;
  pli: number;
  keyFrames: number;
  /** encoderImplementation do motor (ex.: "MediaFoundationVideoEncodeAccelerator" = placa de vídeo). */
  encoder: string;
  /** powerEfficientEncoder: true = compressão na placa de vídeo. null = o motor não informou. */
  hardware: boolean | null;
}
export interface PeerStats {
  send: SendStats | null;
  sample: MediaStatsSample;
}
export interface ViewMessage { t:'view'; version:2; transmission:TransmissionKey; revision:number; mode:ViewMode }
export function encodeView(mode:ViewMode, transmission:TransmissionKey, revision:number):ViewMessage { return {t:'view',version:2,transmission,revision,mode}; }
export function decodeView(msg:unknown): ViewMessage | null {
  if(typeof msg!=='object'||msg===null||!('t' in msg)||msg.t!=='view'||!('version' in msg)||msg.version!==2||!('transmission' in msg)||!('revision' in msg)||!('mode' in msg)) return null;
  const key=msg.transmission;
  if(typeof key!=='object'||key===null||!('peerId' in key)||typeof key.peerId!=='string'||!(/^[0-9a-f]{16}$/).test(key.peerId)||!('startedAt' in key)||!isSafeSequence(key.startedAt)||!isSafeSequence(msg.revision)) return null;
  if(msg.mode!=='full'&&msg.mode!=='thumb'&&msg.mode!=='hidden'&&msg.mode!=='off') return null;
  return {t:'view',version:2,transmission:{peerId:key.peerId,startedAt:key.startedAt},revision:msg.revision,mode:msg.mode};
}

export class PeerLink {
  readonly pc: RTCPeerConnection;
  private makingOffer = false;
  private ignoreOffer = false;
  /** Mensagens processadas em ordem: um candidato não pode passar na frente da oferta. */
  private queue: Promise<void> = Promise.resolve();
  private remoteStream = new MediaStream();
  /** Nossos dois canais de envio, reaproveitados entre uma transmissão e outra. */
  private video: RTCRtpTransceiver | null = null;
  private audio: RTCRtpTransceiver | null = null;
  private quality: Quality | null = null;
  /** Canal de dados `ctl` (architecture.md §4.2). */
  private ctl: RTCDataChannel | null = null;
  /** Como eu vejo a tela dessa pessoa (mando pra ela). */
  private myView: ViewMode | null = null;
  /** Como essa pessoa vê a minha tela (ela me manda). Sem autorização válida, nenhum track é enviado. */
  theirView: ViewMode = 'off';
  private localKey: TransmissionKey | null = null;
  private remoteKey: TransmissionKey | null = null;
  private localStream: MediaStream | null = null;
  private grantRevision = -1;
  private viewRevision = 0;
  private encoderImplementation: string | null = null;
  private encoderPowerEfficient: boolean | null = null;
  private restarted = false;
  private closed = false;
  private mediaStats = new MediaStats();
  private statsPending: Promise<PeerStats | null> | null = null;
  private statsEpoch = 0;
  private localRevision = 0;
  private localQueue: Promise<void> = Promise.resolve();
  /** getParameters/setParameters não podem se cruzar (dá InvalidModificationError). */
  private encodingQueue: Promise<void> = Promise.resolve();
  /** Certificado DTLS do outro lado. Muda quando a pessoa recarrega a página (conexão nova). */
  remoteFingerprint: string | null = null;

  constructor(
    readonly remoteId: string,
    private readonly polite: boolean,
    iceServers: IceServer[],
    initiator: boolean,
    private readonly cb: PeerCallbacks,
  ) {
    this.pc = new RTCPeerConnection({ iceServers: directIceServers(iceServers) as RTCIceServer[] });

    this.pc.onnegotiationneeded = async () => {
      try {
        this.makingOffer = true;
        await this.pc.setLocalDescription();
        this.sendDescription();
      } catch (err) {
        console.warn('[peer] falhou ao criar oferta', err);
      } finally {
        this.makingOffer = false;
      }
    };
    this.pc.onicecandidate = ({ candidate }) => {
      if (candidate) this.cb.signal({ candidate: candidate.toJSON() as Record<string, unknown> });
    };
    this.pc.ontrack = ({ track, receiver }) => {
      if (this.closed) return;
      preferLowLatency(receiver);
      if (!this.remoteStream.getTrackById(track.id)) this.remoteStream.addTrack(track);
      this.cb.remote(this.remoteStream);
    };
    this.pc.onconnectionstatechange = () => {
      const s = this.pc.connectionState;
      console.debug('[peer]', this.remoteId.slice(0, 6), 'conexão:', s);
      // Uma recuperação não repõe o orçamento de tentativas desta conexão.
      // Uma tentativa automática; se falhar de novo, o tile mostra "Tentar de novo".
      if (s === 'failed' && !this.restarted) {
        this.restarted = true;
        this.pc.restartIce();
      }
      this.cb.state(s);
    };
    // Os limites de envio só valem depois que a negociação assenta.
    this.pc.onsignalingstatechange = () => {
      if (this.pc.signalingState === 'stable') void this.applyEncodings();
    };

    // Quem chega abre o canal `ctl`: faz a conexão nascer mesmo sem ninguém ao vivo (§3.2).
    if (initiator) this.setupCtl(this.pc.createDataChannel('ctl'));
    this.pc.ondatachannel = ({ channel }) => {
      if (channel.label === 'ctl') this.setupCtl(channel);
    };
  }

  private setupCtl(channel: RTCDataChannel) {
    this.ctl = channel;
    channel.onopen = () => { this.sendView(); this.requestView(); };
    channel.onclose = () => { this.theirView='off'; this.grantRevision=-1; this.cb.view('off'); void this.setLocal(this.localStream,this.quality); };
    channel.onmessage = (e) => {
      if(this.closed || this.ctl!==channel || channel.readyState!=='open') return;
      try {
        const raw:unknown=JSON.parse(String(e.data));
        if(typeof raw==='object'&&raw!==null&&'t' in raw&&raw.t==='view-sync'&&'version' in raw&&raw.version===2&&'transmission' in raw) {
          const request=decodeView({...raw,t:'view',revision:0,mode:'off'});
          if(request&&sameTransmission(request.transmission,this.remoteKey)) this.sendView();
          return;
        }
        const grant=decodeView(raw);
        if(!grant||!sameTransmission(grant.transmission,this.localKey)||grant.revision<=this.grantRevision) return;
        this.grantRevision=grant.revision;
        this.theirView=grant.mode;
        this.cb.view(grant.mode);
        void this.setLocal(this.localStream,this.quality);
      } catch { /* Entrada externa inválida nunca autoriza mídia. */ }
    };
  }

  setLocalKey(key:TransmissionKey|null) {
    if(sameTransmission(key,this.localKey)||(!key&&!this.localKey)) return;
    this.localKey=key; this.grantRevision=-1; this.theirView='off'; this.cb.view('off');
    void this.setLocal(this.localStream,this.quality);
    this.requestView();
  }

  setRemoteKey(key:TransmissionKey|null) {
    if(sameTransmission(key,this.remoteKey)||(!key&&!this.remoteKey)) return;
    this.remoteKey=key; this.myView='off'; this.sendView();
  }

  private requestView() {
    if(this.localKey&&this.ctl?.readyState==='open') this.ctl.send(JSON.stringify({t:'view-sync',version:2,transmission:this.localKey}));
  }

  /** Avisa essa pessoa de como estou vendo a tela dela (só quando muda). */
  setMyView(mode: ViewMode) {
    if (mode === this.myView) return;
    this.myView = mode;
    if (mode === 'full' || mode === 'thumb') {
      for (const receiver of this.pc.getReceivers()) preferLowLatency(receiver);
    }
    this.sendView();
  }

  private sendView() {
    if (this.remoteKey && this.ctl?.readyState === 'open') this.ctl.send(JSON.stringify(encodeView(this.myView ?? 'off',this.remoteKey,++this.viewRevision)));
  }

  receive(data: SignalData) {
    this.queue = this.queue.then(() => this.handle(data)).catch((err) => console.warn('[peer] sinal', err));
  }

  private async handle(data: SignalData) {
    if (this.closed) return;
    if ('description' in data) {
      const description = data.description as RTCSessionDescriptionInit;
      const collision = description.type === 'offer' && (this.makingOffer || this.pc.signalingState !== 'stable');
      this.ignoreOffer = !this.polite && collision;
      if (this.ignoreOffer) return;
      await this.pc.setRemoteDescription(description);
      this.remoteFingerprint = fingerprintOf(description.sdp) ?? this.remoteFingerprint;
      if (description.type === 'offer') {
        await this.pc.setLocalDescription();
        this.sendDescription();
      }
    } else if (data.candidate) {
      if(typeof data.candidate.candidate==='string' && /\btyp relay\b/i.test(data.candidate.candidate)) return;
      try {
        await this.pc.addIceCandidate(data.candidate as RTCIceCandidateInit);
      } catch (err) {
        if (!this.ignoreOffer) throw err;
      }
    }
  }

  private sendDescription() {
    const d = this.pc.localDescription;
    if (d) this.cb.signal({ description: { type: d.type, sdp: d.sdp } });
  }

  /** Começa ou para de enviar a nossa tela para esta pessoa. */
  setLocal(stream: MediaStream | null, quality: Quality | null): Promise<void> {
    if (this.closed) return Promise.resolve();
    this.localStream = stream;
    this.quality = quality;
    const revision = ++this.localRevision;
    this.localQueue = this.localQueue.catch(() => {}).then(async () => {
      if (this.closed || revision !== this.localRevision) return;
      this.video = await this.attach(this.video, 'video', this.localKey && (this.theirView==='full'||this.theirView==='thumb') ? stream?.getVideoTracks()[0] ?? null : null, stream, revision);
      if (this.closed || revision !== this.localRevision) return;
      this.audio = await this.attach(this.audio, 'audio', this.localKey && wantsSound(this.theirView) ? stream?.getAudioTracks()[0] ?? null : null, stream, revision);
      if (this.closed || revision !== this.localRevision) return;
      await this.applyEncodings();
    });
    return this.localQueue;
  }

  private async attach(
    t: RTCRtpTransceiver | null,
    kind: 'video' | 'audio',
    track: MediaStreamTrack | null,
    stream: MediaStream | null,
    revision: number,
  ): Promise<RTCRtpTransceiver | null> {
    if (this.closed || revision !== this.localRevision) return t;
    if (!t) {
      if (!track || !stream) return null;
      // O teto entra junto com a faixa: antes da primeira negociação o setParameters não tem efeito, e o
      // motor mandava no próprio padrão até a sinalização assentar (specs/010, research R3).
      const init: RTCRtpTransceiverInit = { direction: 'sendonly', streams: [stream] };
      if (this.quality) {
        const p = sendParams(this.quality, this.theirView);
        init.sendEncodings =
          kind === 'video'
            ? [{ active: p.active, maxBitrate: p.maxBitrate, maxFramerate: p.maxFramerate, scaleResolutionDownBy: p.scaleResolutionDownBy }]
            : [{ active: wantsSound(this.theirView), maxBitrate: AUDIO_BITRATE }];
      }
      const created = this.pc.addTransceiver(track, init);
      if (kind === 'video') preferHardwareCodec(created);
      return created;
    }
    try {
      await t.sender.replaceTrack(track);
      if (this.closed || revision !== this.localRevision) return t;
      if (track && stream) t.sender.setStreams(stream);
      // Mudar a direção dispara a renegociação (onnegotiationneeded).
      t.direction = track ? 'sendonly' : 'inactive';
    } catch (err) {
      // Conexão fechou no meio (a pessoa saiu): nada a fazer.
      if (!this.closed) throw err;
    }
    return t;
  }

  /** Qualidade efetiva mudou (teto pelo nº de espectadores). */
  setQuality(quality: Quality) {
    this.quality = quality;
    void this.applyEncodings();
  }

  private applyEncodings(): Promise<void> {
    this.encodingQueue = this.encodingQueue.then(async () => {
      const q = this.quality;
      // No meio de uma renegociação os parâmetros mudam por baixo; quando assentar, roda de novo.
      if (!q || this.closed || this.pc.signalingState !== 'stable') return;
      const p = sendParams(q, this.theirView);
      await setEncoding(this.video?.sender, {
        ...p,
        degradation: contentPreference(this.localStream?.getVideoTracks()[0]?.contentHint ?? '', 'screen'),
      });
      // Segundo plano corta só o vídeo; parar de assistir corta vídeo e som (specs/014).
      await setEncoding(this.audio?.sender, { active: wantsSound(this.theirView), maxBitrate: AUDIO_BITRATE });
    });
    return this.encodingQueue;
  }

  /** O que o motor está enviando de vídeo para esta pessoa agora (specs/010). Sem envio: null. */
  async sendStats(): Promise<SendStats | null> {
    return (await this.pollStats())?.send ?? null;
  }

  /** Uma consulta por link, compartilhada pelo diagnóstico e pelo governor. */
  pollStats(): Promise<PeerStats | null> {
    if (this.closed) return Promise.resolve(null);
    if (this.statsPending) return this.statsPending;
    const epoch = this.statsEpoch;
    this.statsPending = this.collectStats(epoch).catch(() => null).finally(() => {
      this.statsPending = null;
    });
    return this.statsPending;
  }

  diagnostics(): MediaStatsSample[] {
    return this.mediaStats.history();
  }

  encodingInfo() {
    const hint=this.localStream?.getVideoTracks()[0]?.contentHint;
    return { profile:hint==='motion'||hint==='detail'||hint==='text'?hint:null, implementation:this.encoderImplementation, hardware:this.encoderPowerEfficient };
  }

  clearDiagnostics() {
    this.statsEpoch++;
    this.mediaStats.clear();
  }

  private async collectStats(epoch: number): Promise<PeerStats | null> {
    const report = await this.pc.getStats();
    if (this.closed || epoch !== this.statsEpoch) return null;
    const rows: StatsRow[] = [];
    let out: SendStats | null = null;
    report.forEach((s) => {
      rows.push(s as unknown as StatsRow);
      if (!this.video?.sender.track || s.type !== 'outbound-rtp' || s.kind !== 'video') return;
      const r = s as RTCOutboundRtpStreamStats & {
        qualityLimitationReason?: string;
        pliCount?: number;
        keyFramesEncoded?: number;
        encoderImplementation?: string;
        powerEfficientEncoder?: boolean;
      };
      out = {
        bytesSent: r.bytesSent ?? 0,
        limitation: r.qualityLimitationReason ?? 'none',
        pli: r.pliCount ?? 0,
        keyFrames: r.keyFramesEncoded ?? 0,
        encoder: r.encoderImplementation ?? '?',
        hardware: r.powerEfficientEncoder ?? null,
      };
      this.encoderImplementation=r.encoderImplementation?.slice(0,120)??null;
      this.encoderPowerEfficient=r.powerEfficientEncoder??null;
    });
    return { send: out, sample: this.mediaStats.read(rows, performance.now()) };
  }

  /** Credenciais TURN novas: vale para as próximas coletas de candidatos. */
  setIceServers(iceServers: IceServer[]) {
    if (!this.closed) this.pc.setConfiguration({ ...this.pc.getConfiguration(), iceServers: directIceServers(iceServers) as RTCIceServer[] });
  }

  /** "Tentar de novo" no tile. */
  restart() {
    if (this.closed) return;
    this.restarted = true;
    this.pc.restartIce();
  }

  close() {
    this.localStream = null;
    this.localKey = null;
    this.closed = true;
    this.localRevision++;
    this.clearDiagnostics();
    this.pc.close();
    for (const t of this.remoteStream.getTracks()) t.stop();
  }
}

/** Preferência opcional, em ms. Não garante o atraso efetivo do motor. */
export function preferLowLatency(receiver: RTCRtpReceiver | undefined) {
  if (!receiver || !('jitterBufferTarget' in receiver)) return;
  try {
    (receiver as RTCRtpReceiver & { jitterBufferTarget: number | null }).jitterBufferTarget = 0;
  } catch {
    // Motor sem suporte ou que recusou a preferência: recepção padrão continua.
  }
}

/**
 * H.264 primeiro: no Windows o Chrome/Edge comprime H.264 na placa de vídeo (NVENC/AMF/QuickSync),
 * enquanto o VP8 padrão roda no processador — uma compressão por pessoa na malha, disputando com o jogo.
 * Sem H.264 no motor, fica a ordem padrão. Quem recebe segue a ordem da oferta.
 */
export function orderCodecs<C extends { mimeType: string; sdpFmtpLine?: string }>(codecs: C[]): C[] {
  const rank = (c: C) => {
    if (c.mimeType.toLowerCase() !== 'video/h264') return 2;
    // packetization-mode=1 é o que os decodificadores de celular e Safari esperam.
    return /packetization-mode=1/.test(c.sdpFmtpLine ?? '') ? 0 : 1;
  };
  return codecs.map((c, i) => ({ c, i })).sort((a, b) => rank(a.c) - rank(b.c) || a.i - b.i).map(({ c }) => c);
}

function preferHardwareCodec(t: RTCRtpTransceiver) {
  try {
    const codecs = RTCRtpSender.getCapabilities?.('video')?.codecs;
    if (!codecs?.some((c) => c.mimeType.toLowerCase() === 'video/h264') || !t.setCodecPreferences) return;
    t.setCodecPreferences(orderCodecs(codecs));
  } catch (err) {
    console.warn('[peer] setCodecPreferences', err);
  }
}

/** "a=fingerprint:sha-256 AB:CD:…" de uma SDP. */
export function fingerprintOf(sdp: string | undefined): string | null {
  return sdp?.match(/^a=fingerprint:(.+)$/m)?.[1].trim() ?? null;
}

async function setEncoding(
  sender: RTCRtpSender | undefined,
  opts: {
    active: boolean;
    maxBitrate: number;
    maxFramerate?: number;
    scaleResolutionDownBy?: number;
    degradation?: RTCDegradationPreference;
  },
) {
  if (!sender?.track) return;
  const params = sender.getParameters();
  // Antes da primeira negociação ainda não há encodings: tenta de novo quando assentar.
  if (!params.encodings?.length) return;
  const e = params.encodings[0];
  e.active = opts.active;
  e.maxBitrate = opts.maxBitrate;
  if (opts.maxFramerate) e.maxFramerate = opts.maxFramerate;
  if (opts.scaleResolutionDownBy) e.scaleResolutionDownBy = opts.scaleResolutionDownBy;
  if (opts.degradation) (params as RTCRtpSendParameters & { degradationPreference?: string }).degradationPreference = opts.degradation;
  try {
    await sender.setParameters(params);
  } catch (err) {
    // Uma renegociação começou entre o get e o set: normal, roda de novo quando assentar.
    if ((err as DOMException).name !== 'InvalidModificationError') console.warn('[peer] setParameters', err);
  }
}
