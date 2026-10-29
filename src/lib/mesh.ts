// Malha da sala: uma PeerLink por pessoa (architecture.md §4.1). Quem chega inicia a conexão com
// quem já está; quem já está espera a primeira mensagem e cria a conexão nessa hora.
import type { IceServer, SignalData } from '../shared/protocol';
import { fingerprintOf, PeerLink } from './peer';
import type { Quality, ViewMode } from './quality';

export interface MeshCallbacks {
  signal(to: string, data: SignalData): void;
  remote(peerId: string, stream: MediaStream): void;
  state(peerId: string, state: RTCPeerConnectionState): void;
  /** Como essa pessoa está vendo a minha tela mudou. */
  view(peerId: string, mode: ViewMode): void;
}

export class Mesh {
  private links = new Map<string, PeerLink>();
  private iceServers: IceServer[] = [];
  private local: MediaStream | null = null;
  private quality: Quality | null = null;
  private myViews: Record<string, ViewMode> = {};

  constructor(
    private myId: string,
    private readonly cb: MeshCallbacks,
  ) {}

  get id() {
    return this.myId;
  }

  setIceServers(iceServers: IceServer[]) {
    this.iceServers = iceServers;
    for (const link of this.links.values()) link.setIceServers(iceServers);
  }

  /**
   * Depois de um welcome: deixa a malha igual à lista da sala. Quem ficou sem conexão (entrou
   * enquanto estávamos fora, ou é a primeira vez) recebe uma conexão iniciada por nós.
   */
  sync(peerIds: string[]) {
    const keep = new Set(peerIds);
    for (const id of [...this.links.keys()]) if (!keep.has(id)) this.remove(id);
    for (const id of peerIds) if (!this.links.has(id)) this.create(id, true);
  }

  remove(id: string) {
    this.links.get(id)?.close();
    this.links.delete(id);
  }

  receive(from: string, data: SignalData) {
    let link = this.links.get(from);
    // Oferta de uma conexão nova da mesma pessoa (ela recarregou a página): descarta a antiga.
    if (link?.remoteFingerprint && 'description' in data && data.description.type === 'offer') {
      const fp = fingerprintOf(data.description.sdp);
      if (fp && fp !== link.remoteFingerprint) {
        this.remove(from);
        link = undefined;
      }
    }
    (link ?? this.create(from, false)).receive(data);
  }

  /** Começa (stream) ou para (null) de transmitir para todo mundo. */
  setLocal(stream: MediaStream | null, quality: Quality | null) {
    this.local = stream;
    this.quality = quality;
    for (const link of this.links.values()) void link.setLocal(stream, quality);
  }

  setQuality(quality: Quality) {
    this.quality = quality;
    for (const link of this.links.values()) link.setQuality(quality);
  }

  /** Como eu estou vendo a tela de cada pessoa (quem não está na lista: escondida). */
  setViews(views: Record<string, ViewMode>) {
    this.myViews = views;
    for (const [id, link] of this.links) link.setMyView(views[id] ?? 'hidden');
  }

  /** Como cada pessoa conectada está vendo a minha tela. */
  theirViews(): Record<string, ViewMode> {
    const out: Record<string, ViewMode> = {};
    for (const [id, l] of this.links) if (l.pc.connectionState === 'connected') out[id] = l.theirView;
    return out;
  }

  retry(id: string) {
    this.links.get(id)?.restart();
  }

  /** Quantas conexões estão de pé (espectadores possíveis da sua transmissão). */
  connectedCount() {
    return [...this.links.values()].filter((l) => l.pc.connectionState === 'connected').length;
  }

  close() {
    for (const id of [...this.links.keys()]) this.remove(id);
  }

  private create(id: string, initiator: boolean) {
    const link = new PeerLink(id, this.myId < id, this.iceServers, initiator, {
      signal: (data) => this.cb.signal(id, data),
      remote: (stream) => this.cb.remote(id, stream),
      state: (state) => this.cb.state(id, state),
      view: (mode) => this.cb.view(id, mode),
    });
    this.links.set(id, link);
    link.setMyView(this.myViews[id] ?? 'hidden');
    if (this.local) void link.setLocal(this.local, this.quality);
    return link;
  }
}
