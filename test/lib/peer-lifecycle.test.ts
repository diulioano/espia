import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { PeerLink } from '../../src/lib/peer';

class Track { constructor(readonly kind: string, readonly id: string) {} }
class Stream {
  constructor(readonly tracks: Track[] = []) {}
  getVideoTracks() { return this.tracks.filter((t) => t.kind === 'video'); }
  getAudioTracks() { return this.tracks.filter((t) => t.kind === 'audio'); }
  getTracks() { return this.tracks; }
}
type Sender = { track: Track | null; hold: Promise<void> | null; replaceTrack: (t: Track | null) => Promise<void>; setStreams: ReturnType<typeof vi.fn> };
class PC {
  static last: PC;
  signalingState = 'stable';
  transceivers: { sender: Sender; direction: string }[] = [];
  close = vi.fn();
  constructor() { PC.last = this; }
  addTransceiver(track: Track, init: RTCRtpTransceiverInit) {
    const sender = {
      track, hold: null, setStreams: vi.fn(),
      async replaceTrack(this: Sender, value: Track | null) { await this.hold; this.track = value; },
      getParameters: () => ({ encodings: [{}] }), setParameters: vi.fn(async () => {}),
    } as Sender;
    const t = { sender, direction: init.direction ?? 'sendonly' };
    this.transceivers.push(t);
    return t;
  }
}
const q = { resolution: 720 as const, fps: 30 as const, maxBitrate: 2_500_000 };
const media = (id: string) => new Stream([new Track('video', id), new Track('audio', id + '-a')]) as unknown as MediaStream;
const create = () => new PeerLink('p', false, [], false, { signal() {}, remote() {}, state() {}, view() {} });

describe('setLocal com revisões', () => {
  beforeEach(() => {
    vi.stubGlobal('RTCPeerConnection', PC);
    vi.stubGlobal('MediaStream', Stream);
    vi.stubGlobal('RTCRtpSender', { getCapabilities: () => ({ codecs: [] }) });
  });
  afterEach(() => vi.unstubAllGlobals());

  it('start/start/stop no mesmo tick não cria envios antigos', async () => {
    const link = create();
    await Promise.all([link.setLocal(media('old'), q), link.setLocal(media('new'), q), link.setLocal(null, null)]);
    expect(PC.last.transceivers).toHaveLength(0);
  });

  it('stop vence replaceTrack em andamento sem reaplicar áudio antigo', async () => {
    const link = create();
    await link.setLocal(media('first'), q);
    const sender = PC.last.transceivers[0].sender;
    let release!: () => void;
    sender.hold = new Promise<void>((r) => { release = r; });
    const updating = link.setLocal(media('second'), q);
    await Promise.resolve(); await Promise.resolve();
    const stopping = link.setLocal(null, null);
    release();
    await Promise.all([updating, stopping]);
    expect(PC.last.transceivers).toHaveLength(2);
    expect(PC.last.transceivers.map((t) => t.sender.track)).toEqual([null, null]);
    expect(PC.last.transceivers.map((t) => t.direction)).toEqual(['inactive', 'inactive']);
  });

  it('close impede setStreams tardio e nova transmissão', async () => {
    const link = create();
    await link.setLocal(media('first'), q);
    const sender = PC.last.transceivers[0].sender;
    sender.setStreams.mockClear();
    let release!: () => void;
    sender.hold = new Promise<void>((r) => { release = r; });
    const updating = link.setLocal(media('second'), q);
    await Promise.resolve(); await Promise.resolve();
    link.close();
    release();
    await updating;
    await link.setLocal(media('third'), q);
    expect(sender.setStreams).not.toHaveBeenCalled();
    expect(PC.last.close).toHaveBeenCalledTimes(1);
    expect(PC.last.transceivers).toHaveLength(2);
  });
});
