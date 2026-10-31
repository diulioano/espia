// Uma RTCPeerConnection com outra pessoa da sala (architecture.md §4.5).
// "Perfect negotiation": os dois lados podem renegociar; quando as ofertas se cruzam, o lado
// educado (polite) desiste da sua. Candidatos ICE vão aos poucos (trickle) pela sinalização.
import type { IceServer, SignalData } from '../shared/protocol';
import { sendParams, type Quality, type ViewMode } from './quality';

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
const VIEW_MODES: ViewMode[] = ['full', 'thumb', 'hidden'];

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
  /** Como essa pessoa vê a minha tela (ela me manda). Sem aviso, conta como tela cheia. */
  theirView: ViewMode = 'full';
  private restarted = false;
  private closed = false;
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
    this.pc = new RTCPeerConnection({ iceServers: iceServers as RTCIceServer[] });

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
    this.pc.ontrack = ({ track }) => {
      if (!this.remoteStream.getTrackById(track.id)) this.remoteStream.addTrack(track);
      this.cb.remote(this.remoteStream);
    };
    this.pc.onconnectionstatechange = () => {
      const s = this.pc.connectionState;
      console.debug('[peer]', this.remoteId.slice(0, 6), 'conexão:', s);
      if (s === 'connected') this.restarted = false;
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
    channel.onopen = () => this.sendView();
    channel.onmessage = (e) => {
      try {
        const msg = JSON.parse(String(e.data));
        if (msg?.t === 'view' && VIEW_MODES.includes(msg.mode) && msg.mode !== this.theirView) {
          this.theirView = msg.mode;
          this.cb.view(msg.mode);
          void this.applyEncodings();
        }
      } catch {
        // Mensagem estranha no ctl: ignora.
      }
    };
  }

  /** Avisa essa pessoa de como estou vendo a tela dela (só quando muda). */
  setMyView(mode: ViewMode) {
    if (mode === this.myView) return;
    this.myView = mode;
    this.sendView();
  }

  private sendView() {
    if (this.myView && this.ctl?.readyState === 'open') this.ctl.send(JSON.stringify({ t: 'view', mode: this.myView }));
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
  async setLocal(stream: MediaStream | null, quality: Quality | null) {
    if (this.closed) return;
    this.quality = quality;
    const videoTrack = stream?.getVideoTracks()[0] ?? null;
    const audioTrack = stream?.getAudioTracks()[0] ?? null;
    this.video = await this.attach(this.video, 'video', videoTrack, stream);
    this.audio = await this.attach(this.audio, 'audio', audioTrack, stream);
    await this.applyEncodings();
  }

  private async attach(
    t: RTCRtpTransceiver | null,
    kind: 'video' | 'audio',
    track: MediaStreamTrack | null,
    stream: MediaStream | null,
  ): Promise<RTCRtpTransceiver | null> {
    if (this.closed) return t;
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
            : [{ active: true, maxBitrate: AUDIO_BITRATE }];
      }
      const created = this.pc.addTransceiver(track, init);
      if (kind === 'video') preferHardwareCodec(created);
      return created;
    }
    try {
      await t.sender.replaceTrack(track);
      if (this.closed) return t;
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
        degradation: q.fps === 60 ? 'maintain-framerate' : 'maintain-resolution',
      });
      // O som vai sempre (specs/014): parar de assistir ou esconder a janela corta só o vídeo.
      await setEncoding(this.audio?.sender, { active: true, maxBitrate: AUDIO_BITRATE });
    });
    return this.encodingQueue;
  }

  /** O que o motor está enviando de vídeo para esta pessoa agora (specs/010). Sem envio: null. */
  async sendStats(): Promise<SendStats | null> {
    const sender = this.video?.sender;
    if (this.closed || !sender?.track) return null;
    let out: SendStats | null = null;
    (await sender.getStats()).forEach((s) => {
      if (s.type !== 'outbound-rtp' || s.kind !== 'video') return;
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
    });
    return out;
  }

  /** Credenciais TURN novas: vale para as próximas coletas de candidatos. */
  setIceServers(iceServers: IceServer[]) {
    if (!this.closed) this.pc.setConfiguration({ ...this.pc.getConfiguration(), iceServers: iceServers as RTCIceServer[] });
  }

  /** "Tentar de novo" no tile. */
  restart() {
    if (this.closed) return;
    this.restarted = true;
    this.pc.restartIce();
  }

  close() {
    this.closed = true;
    this.pc.close();
    for (const t of this.remoteStream.getTracks()) t.stop();
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
