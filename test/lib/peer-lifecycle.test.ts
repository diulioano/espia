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
  ondatachannel: ((event:{channel:RTCDataChannel})=>void)|null=null;
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
const key={peerId:'0123456789abcdef',startedAt:1};
const create = () => new PeerLink('p', false, [], false, { signal() {}, remote() {}, state() {}, view() {} });
const consent=(link:PeerLink)=>{
 link.setLocalKey(key);
 const channel={label:'ctl',readyState:'open',onmessage:null,send:vi.fn()} as unknown as RTCDataChannel;
 PC.last.ondatachannel?.({channel});
 channel.onmessage?.({data:JSON.stringify({t:'view',version:2,transmission:key,revision:1,mode:'full'})} as MessageEvent);
 return channel;
};

describe('setLocal com revisões', () => {
  beforeEach(() => {
    vi.stubGlobal('RTCPeerConnection', PC);
    vi.stubGlobal('MediaStream', Stream);
    vi.stubGlobal('RTCRtpSender', { getCapabilities: () => ({ codecs: [] }) });
  });
  afterEach(() => vi.unstubAllGlobals());

  it('never attaches audio or video before consent',async()=>{const link=create();link.setLocalKey(key);await link.setLocal(media('first'),q);expect(PC.last.transceivers).toHaveLength(0);});

  it('new transmission and closed control revoke the grant',async()=>{
    const link=create();const channel=consent(link);await link.setLocal(media('first'),q);
    expect(PC.last.transceivers.map(t=>t.sender.track?.kind)).toEqual(['video','audio']);
    link.setLocalKey({...key,startedAt:2});
    channel.onmessage?.({data:JSON.stringify({t:'view',version:2,transmission:key,revision:100,mode:'full'})} as MessageEvent);
    await link.setLocal(media('second'),q);
    expect(PC.last.transceivers.map(t=>t.sender.track)).toEqual([null,null]);
  });

  it('a queued message on a closed control cannot authorize media',async()=>{
    const link=create();const channel=consent(link);await link.setLocal(media('first'),q);
    Object.assign(channel,{readyState:'closed'});channel.onclose?.(new Event('close'));
    channel.onmessage?.({data:JSON.stringify({t:'view',version:2,transmission:key,revision:999,mode:'full'})} as MessageEvent);
    await link.setLocal(media('second'),q);
    expect(PC.last.transceivers.map(t=>t.sender.track)).toEqual([null,null]);
  });

  it('start/start/stop no mesmo tick não cria envios antigos', async () => {
    const link = create();
    await Promise.all([link.setLocal(media('old'), q), link.setLocal(media('new'), q), link.setLocal(null, null)]);
    expect(PC.last.transceivers).toHaveLength(0);
  });

  it('stop vence replaceTrack em andamento sem reaplicar áudio antigo', async () => {
    const link = create();
    consent(link);
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
    consent(link);
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
