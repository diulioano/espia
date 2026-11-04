import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  detectNative,
  matchLastSource,
  pollSources,
  shouldDropSound,
  SlotTable,
  soundBytes,
  sourceKey,
  startNative,
  type Bridge,
  type FrameMsg,
  type HostMessage,
} from '../../src/lib/native-capture';
import type { CaptureSource, GoLivePrefs } from '../../src/lib/types';

// Ponte falsa: o que o app desktop faria, sem o app (specs/005 contracts/native-capture.md).
class FakeBridge implements Bridge {
  calls: { cmd: string; args?: Record<string, unknown> }[] = [];
  handlers: Record<string, (args?: Record<string, unknown>) => unknown> = {};
  channels: ((m: unknown) => void)[] = [];
  bufferCb: ((b: ArrayBuffer, meta: unknown) => void) | null = null;
  released: ArrayBuffer[] = [];
  constructor(private inApp = true) {}
  available() {
    return this.inApp;
  }
  async invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    this.calls.push({ cmd, args });
    const h = this.handlers[cmd];
    if (!h) throw `command ${cmd} not found`;
    return (await h(args)) as T;
  }
  channel<T>(onmessage: (m: T) => void) {
    this.channels.push(onmessage as (m: unknown) => void);
    return { id: this.channels.length - 1 };
  }
  onSharedBuffer(cb: (b: ArrayBuffer, meta: unknown) => void) {
    this.bufferCb = cb;
    return () => (this.bufferCb = null);
  }
  releaseBuffer(b: ArrayBuffer) {
    this.released.push(b);
  }
  hostCb: ((m: HostMessage) => void) | null = null;
  onHostMessage(cb: (m: HostMessage) => void) {
    this.hostCb = cb;
    return () => (this.hostCb = null);
  }
  /** O app posta os 3 slots de uma geração. */
  postSlots(gen: number, width = 4, height = 2) {
    for (let slot = 0; slot < 3; slot++) {
      this.bufferCb?.(new ArrayBuffer((width * height * 3) / 2), { espia: 'frame-slot', gen, slot, width, height, format: 'NV12' });
    }
  }
}

const src = (over: Partial<CaptureSource>): CaptureSource => ({
  id: 'w:1',
  kind: 'window',
  title: 'Hollow Knight',
  app: 'hollow_knight.exe',
  icon: null,
  thumb: null,
  playing: false,
  primary: false,
  ...over,
});

describe('detectNative', () => {
  it('fora do app → false, sem chamar nada', async () => {
    const b = new FakeBridge(false);
    expect(await detectNative(b)).toBe(false);
    expect(b.calls).toEqual([]);
  });

  it('app com a ponte v1 → true', async () => {
    const b = new FakeBridge();
    b.handlers.capture_supported = () => ({ ok: true, version: 1, win11: true });
    expect(await detectNative(b)).toBe(true);
  });

  it('versão diferente, ok=false ou app antigo sem o comando → false (FR-511)', async () => {
    const b = new FakeBridge();
    b.handlers.capture_supported = () => ({ ok: true, version: 2 });
    expect(await detectNative(b)).toBe(false);
    b.handlers.capture_supported = () => ({ ok: false, version: 1 });
    expect(await detectNative(b)).toBe(false);
    delete b.handlers.capture_supported;
    expect(await detectNative(b)).toBe(false);
  });
});

describe('pollSources', () => {
  it('lista na hora e de novo a cada intervalo; para quando pede', async () => {
    const b = new FakeBridge();
    let n = 0;
    b.handlers.capture_list = () => [src({ id: `w:${++n}` })];
    const seen: string[] = [];
    const stop = pollSources((s) => seen.push(s[0].id), () => {}, b, 20);
    await new Promise((r) => setTimeout(r, 70));
    stop();
    const count = seen.length;
    await new Promise((r) => setTimeout(r, 50));
    expect(count).toBeGreaterThanOrEqual(3);
    expect(seen.length).toBe(count);
    expect(b.calls[0].args).toEqual({ thumbs: true });
  });

  it('falha → onError e não tenta de novo (vai pro seletor do Windows)', async () => {
    const b = new FakeBridge();
    b.handlers.capture_list = () => {
      throw 'list_failed';
    };
    const errors: unknown[] = [];
    pollSources(() => {}, (e) => errors.push(e), b, 10);
    await new Promise((r) => setTimeout(r, 40));
    expect(errors).toEqual(['list_failed']);
    expect(b.calls.length).toBe(1);
  });
});

describe('última fonte (US5)', () => {
  it('janela: programa + título; monitor: título', () => {
    expect(sourceKey(src({}))).toBe('hollow_knight.exe|Hollow Knight');
    expect(sourceKey(src({ kind: 'monitor', title: 'Tela 2', app: '' }))).toBe('m:Tela 2');
  });

  it('remarca se ainda existe, mesmo com outro id (programa reaberto)', () => {
    const list = [src({ id: 'w:99' }), src({ id: 'w:5', title: 'Spotify', app: 'Spotify' })];
    expect(matchLastSource(list, 'hollow_knight.exe|Hollow Knight')).toBe('w:99');
  });

  it('fechada ou sem chave → nada marcado', () => {
    expect(matchLastSource([src({})], 'Spotify|Spotify')).toBeNull();
    expect(matchLastSource([src({})], undefined)).toBeNull();
  });
});

describe('SlotTable', () => {
  const meta = (gen: number, slot: number) => ({ espia: 'frame-slot', gen, slot, width: 4, height: 2, format: 'NV12' });

  it('entrega o buffer do slot anunciado', () => {
    const t = new SlotTable(() => {});
    const bufs = [0, 1, 2].map(() => new ArrayBuffer(12));
    bufs.forEach((b, i) => t.add(b, meta(1, i)));
    expect(t.take({ gen: 1, slot: 2, seq: 0 })?.buffer).toBe(bufs[2]);
  });

  it('ignora o que não é nosso', () => {
    const t = new SlotTable(() => {});
    expect(t.add(new ArrayBuffer(1), { outro: true })).toBe(false);
    expect(t.add(new ArrayBuffer(1), null)).toBe(false);
  });

  it('geração nova libera a velha; aviso da geração velha é ignorado', () => {
    const released: ArrayBuffer[] = [];
    const t = new SlotTable((b) => released.push(b));
    const old = new ArrayBuffer(12);
    t.add(old, meta(1, 0));
    t.add(new ArrayBuffer(12), meta(2, 0));
    expect(released).toEqual([old]);
    expect(t.take({ gen: 1, slot: 0, seq: 5 })).toBeNull();
    expect(t.take({ gen: 2, slot: 0, seq: 6 })).not.toBeNull();
    // Buffer atrasado de uma geração que já passou: devolvido na hora.
    const late = new ArrayBuffer(12);
    t.add(late, meta(1, 1));
    expect(released).toContain(late);
  });

  it('aviso repetido/fora de ordem e aviso atrasado demais são descartados', () => {
    const t = new SlotTable(() => {});
    [0, 1, 2].forEach((i) => t.add(new ArrayBuffer(12), meta(1, i)));
    expect(t.take({ gen: 1, slot: 1, seq: 4 })).not.toBeNull();
    expect(t.take({ gen: 1, slot: 0, seq: 3 })).toBeNull();
    expect(t.stale({ seq: 4 }, 7)).toBe(true);
    expect(t.stale({ seq: 5 }, 7)).toBe(false);
  });
});

// ---------- startNative com stubs do Chromium (breakout box / WebCodecs) ----------

class FakeTrack {
  /** Gerador travado: as escritas nunca terminam (som acumulando). */
  static hold = false;
  stopped = false;
  contentHint = '';
  written: unknown[] = [];
  readonly writable: { getWriter(): { write(x: unknown): Promise<void> } };
  constructor(readonly init: { kind: 'video' | 'audio' }) {
    this.writable = {
      getWriter: () => ({
        write: (x) => {
          this.written.push(x);
          return FakeTrack.hold ? new Promise<void>(() => {}) : Promise.resolve();
        },
      }),
    };
  }
  get kind() {
    return this.init.kind;
  }
  stop() {
    this.stopped = true;
  }
}

class FakeStream {
  tracks: FakeTrack[];
  constructor(tracks: FakeTrack[]) {
    this.tracks = [...tracks];
  }
  addTrack(t: FakeTrack) {
    this.tracks.push(t);
  }
  removeTrack(t: FakeTrack) {
    this.tracks = this.tracks.filter((x) => x !== t);
  }
  getAudioTracks() {
    return this.tracks.filter((t) => t.kind === 'audio');
  }
  getVideoTracks() {
    return this.tracks.filter((t) => t.kind === 'video');
  }
}

class FakeVideoFrame {
  close() {}
  constructor(
    readonly data: Uint8Array,
    readonly init: { codedWidth: number; codedHeight: number; timestamp: number },
  ) {}
}

class FakeAudioData {
  close() {}
  constructor(readonly init: { timestamp: number; numberOfFrames: number; data: Float32Array }) {}
}

const G = globalThis as Record<string, unknown>;
const saved: Record<string, unknown> = {};
const STUBS = { MediaStreamTrackGenerator: FakeTrack, MediaStream: FakeStream, VideoFrame: FakeVideoFrame, AudioData: FakeAudioData };

beforeEach(() => {
  for (const [k, v] of Object.entries(STUBS)) {
    saved[k] = G[k];
    G[k] = v;
  }
});
afterEach(() => {
  for (const k of Object.keys(STUBS)) G[k] = saved[k];
});

const PREFS: GoLivePrefs = { resolution: 720, fps: 60, audio: true, surface: 'window' };
const quiet = { ended: () => {}, audio: () => {}, warn: () => {} };

function started(b: FakeBridge, audio: 'window' | 'pc' | 'none' = 'window') {
  b.handlers.capture_start = () => {
    b.postSlots(1);
    return { width: 4, height: 2, audio };
  };
  b.handlers.capture_stop = () => null;
  b.handlers.capture_set_quality = () => ({ width: 8, height: 4 });
}

describe('startNative', () => {
  it('stop idempotente não manda um segundo capture_stop nem altera hint após parada', async () => {
    const b = new FakeBridge(); started(b);
    let qualityReady!: () => void;
    b.handlers.capture_set_quality = () => new Promise<void>((resolve) => { qualityReady = resolve; });
    const s = await startNative(PREFS, 'w:1', quiet, b);
    const track = (s.stream as unknown as FakeStream).getVideoTracks()[0];
    const changing = s.setQuality(1080, 30);
    await Promise.resolve(); await Promise.resolve();
    await s.stop();
    qualityReady(); await changing;
    await s.stop();
    expect(track.contentHint).toBe('motion');
    expect(b.calls.filter((c) => c.cmd === 'capture_stop')).toHaveLength(1);
    expect(b.hostCb).toBeNull();
  });

  it('trocas de qualidade no mesmo tick aplicam só a escolha mais recente', async () => {
    const b = new FakeBridge(); started(b);
    const s = await startNative(PREFS, 'w:1', quiet, b);
    await Promise.all([s.setQuality(1080, 30), s.setQuality(720, 60)]);
    expect(b.calls.filter((c) => c.cmd === 'capture_set_quality')).toEqual([
      { cmd: 'capture_set_quality', args: { resolution: 720, fps: 60 } },
    ]);
    await s.stop();
  });
  it('abortar durante abertura descarta o resultado tardio e para o host', async () => {
    const b = new FakeBridge();
    started(b);
    let finishStart!: (value: unknown) => void;
    b.handlers.capture_start = () => new Promise((resolve) => { finishStart = resolve; });
    const controller = new AbortController();
    const opening = startNative(PREFS, 'w:1', quiet, b, controller.signal);
    controller.abort();
    finishStart({ width: 4, height: 2, audio: 'window' });
    await expect(opening).rejects.toMatchObject({ name: 'AbortError' });
    expect(b.hostCb).toBeNull();
    expect(b.bufferCb).toBeNull();
    expect(b.calls.some((c) => c.cmd === 'capture_stop')).toBe(true);
  });

  it('áudio recriado ignora conclusão do writer antigo e pacotes inválidos', async () => {
    const now = vi.spyOn(performance, 'now').mockReturnValue(1000);
    const b = new FakeBridge(); started(b);
    const s = await startNative(PREFS, 'w:1', quiet, b);
    try {
      b.channels[1](new ArrayBuffer(3));
      b.channels[1](new Float32Array(960).buffer);
      b.hostCb!({ espia: 'f', m: { t: 'audio', kind: 'none' } });
      now.mockReturnValue(11000);
      b.channels[1](new Float32Array(960).buffer);
      const tracks = (s.stream as unknown as FakeStream).getAudioTracks();
      expect(tracks).toHaveLength(1);
      expect((tracks[0].written[0] as FakeAudioData).init.timestamp).toBe(11_000_000);
    } finally { await s.stop(); now.mockRestore(); }
  });
  it.each([2000, 10000])('startup de %i ms não vira descompasso entre imagem e som', async (startupMs) => {
    const now = vi.spyOn(performance, 'now').mockReturnValue(1000);
    const b = new FakeBridge();
    started(b);
    b.handlers.capture_start = () => {
      now.mockReturnValue(1000 + startupMs);
      b.postSlots(1);
      return { width: 4, height: 2, audio: 'window' };
    };
    const s = await startNative(PREFS, 'w:1', quiet, b);
    try {
      b.hostCb!({ espia: 'f', m: { t: 'frame', gen: 1, slot: 0, seq: 0, ts: 0 } });
      b.channels[1](new Float32Array(960).buffer);
      const stream = s.stream as unknown as FakeStream;
      const video = (stream.getVideoTracks()[0].written[0] as FakeVideoFrame).init.timestamp;
      const audio = (stream.getAudioTracks()[0].written[0] as FakeAudioData).init.timestamp;
      expect(Math.abs(video - audio)).toBeLessThanOrEqual(20_000);
    } finally { await s.stop(); now.mockRestore(); }
  });

  it('áudio volta ao relógio atual depois de lacuna de 10 s', async () => {
    const now = vi.spyOn(performance, 'now').mockReturnValue(1000);
    const b = new FakeBridge();
    started(b);
    const s = await startNative(PREFS, 'w:1', quiet, b);
    try {
      b.channels[1](new Float32Array(960).buffer);
      await Promise.resolve();
      now.mockReturnValue(11000);
      b.channels[1](new Float32Array(960).buffer);
      const chunks = (s.stream as unknown as FakeStream).getAudioTracks()[0].written as FakeAudioData[];
      expect(Math.abs(chunks.at(-1)!.init.timestamp - 11_000_000)).toBeLessThanOrEqual(20_000);
    } finally { await s.stop(); now.mockRestore(); }
  });

  it('manda id, qualidade, som e os dois canais para o app', async () => {
    const b = new FakeBridge();
    started(b);
    await startNative(PREFS, 'w:7', quiet, b);
    const args = b.calls.find((c) => c.cmd === 'capture_start')!.args!;
    expect(args).toMatchObject({ id: 'w:7', resolution: 720, fps: 60, audio: true });
    expect(args.frames).toBeDefined();
    expect(args.sound).toBeDefined();
  });

  it('quadro anunciado vira VideoFrame NV12 no gerador de vídeo', async () => {
    const b = new FakeBridge();
    started(b);
    const s = await startNative(PREFS, 'w:1', quiet, b);
    const frames = b.channels[0] as (m: FrameMsg) => void;
    frames({ t: 'frame', gen: 1, slot: 0, seq: 0, ts: 1000 });
    await Promise.resolve();
    const video = (s.stream as unknown as FakeStream).getVideoTracks()[0];
    expect(video.written).toHaveLength(1);
    const f = video.written[0] as FakeVideoFrame;
    expect(f.init).toMatchObject({ codedWidth: 4, codedHeight: 2 });
    expect(f.data.byteLength).toBe(12);
    expect(video.contentHint).toBe('motion');
  });

  it('com som: faixa de áudio já no stream; pedaços viram AudioData com tempo contínuo', async () => {
    const b = new FakeBridge();
    started(b, 'window');
    const s = await startNative(PREFS, 'w:1', quiet, b);
    const stream = s.stream as unknown as FakeStream;
    expect(stream.getAudioTracks()).toHaveLength(1);
    const sound = b.channels[1];
    sound(new Float32Array(960).buffer);
    sound(new Float32Array(960).buffer);
    await Promise.resolve();
    const chunks = stream.getAudioTracks()[0].written as FakeAudioData[];
    expect(chunks[1].init.timestamp - chunks[0].init.timestamp).toBe(10_000);
    expect(chunks[0].init.numberOfFrames).toBe(480);
  });

  it('imagem e som no mesmo relógio: o receptor não atrasa a imagem (bug: imagem centenas de ms atrás do som)', async () => {
    // O receptor do WebRTC sincroniza pelos horários. Antes, a imagem contava do início da captura no app e o som
    // do 1º pedaço na página: zeros diferentes, e o receptor segurava a imagem até ~0,5 s de propósito.
    const now = vi.spyOn(performance, 'now').mockReturnValue(5_000);
    try {
      const b = new FakeBridge();
      started(b, 'window');
      const s = await startNative(PREFS, 'w:1', quiet, b);
      const stream = s.stream as unknown as FakeStream;
      now.mockReturnValue(6_000); // 1 s depois: chegam um quadro capturado 1 s após o início e o 1º som
      (b.channels[0] as (m: FrameMsg) => void)({ t: 'frame', gen: 1, slot: 0, seq: 0, ts: 1_000_000 });
      b.channels[1](new Float32Array(960).buffer);
      await Promise.resolve();
      const v = (stream.getVideoTracks()[0].written[0] as FakeVideoFrame).init.timestamp;
      const a = (stream.getAudioTracks()[0].written[0] as FakeAudioData).init.timestamp;
      expect(Math.abs(v - a)).toBeLessThanOrEqual(20_000);
      expect(v).toBe(6_000_000); // relógio da página em µs, como as câmeras do navegador
    } finally {
      now.mockRestore();
    }
  });

  it('som como lista de bytes (Tauri no modo postMessage) também vira AudioData, sem perder o valor', async () => {
    // Bug: o app real entregava number[] (CSP sem ipc.localhost) e a página descartava tudo: som mudo.
    const b = new FakeBridge();
    started(b, 'window');
    const s = await startNative(PREFS, 'w:1', quiet, b);
    const pcm = new Float32Array(960).fill(0.25);
    b.channels[1]([...new Uint8Array(pcm.buffer)]);
    await Promise.resolve();
    const chunks = (s.stream as unknown as FakeStream).getAudioTracks()[0].written as FakeAudioData[];
    expect(chunks).toHaveLength(1);
    expect(chunks[0].init.numberOfFrames).toBe(480);
    expect(chunks[0].init.data[0]).toBe(0.25);
  });

  it('som em base64 pelo canal de mensagens (app 0.1.7, specs/010) vira AudioData sem perder o valor', async () => {
    const b = new FakeBridge();
    started(b, 'window');
    const s = await startNative(PREFS, 'w:1', quiet, b);
    const pcm = new Float32Array(960).fill(0.5);
    b.hostCb!({ espia: 'pcm', d: btoa(String.fromCharCode(...new Uint8Array(pcm.buffer))) });
    await Promise.resolve();
    const chunks = (s.stream as unknown as FakeStream).getAudioTracks()[0].written as FakeAudioData[];
    expect(chunks).toHaveLength(1);
    expect(chunks[0].init.numberOfFrames).toBe(480);
    expect(chunks[0].init.data[0]).toBe(0.5);
  });

  it('som acumulado acima de 200 ms é descartado (specs/010)', async () => {
    expect(shouldDropSound(200)).toBe(false);
    expect(shouldDropSound(201)).toBe(true);
    const b = new FakeBridge();
    started(b, 'window');
    const s = await startNative(PREFS, 'w:1', quiet, b);
    FakeTrack.hold = true;
    try {
      for (let i = 0; i < 30; i++) b.channels[1](new Float32Array(960).buffer);
    } finally {
      FakeTrack.hold = false;
    }
    const chunks = (s.stream as unknown as FakeStream).getAudioTracks()[0].written as FakeAudioData[];
    // Limite inclui o novo pedaço: 20 × 10 ms, sem ultrapassar 200 ms.
    expect(chunks).toHaveLength(20);
    // O tempo dos que entram é contínuo; os descartados também contam (o som não atrasa em relação à imagem).
    expect(chunks[19].init.timestamp - chunks[0].init.timestamp).toBe(190_000);
  });

  it('avisos de quadro pelo canal de mensagens (app 0.1.7) também viram VideoFrame', async () => {
    const b = new FakeBridge();
    started(b);
    const s = await startNative(PREFS, 'w:1', quiet, b);
    b.hostCb!({ espia: 'f', m: { t: 'frame', gen: 1, slot: 0, seq: 0, ts: 1000 } });
    await Promise.resolve();
    expect((s.stream as unknown as FakeStream).getVideoTracks()[0].written).toHaveLength(1);
  });

  it('base64 inválido é ignorado', () => {
    expect(soundBytes('não é base64!')).toBeNull();
  });

  it("sem som ('none'): stream só com vídeo", async () => {
    const b = new FakeBridge();
    started(b, 'none');
    const s = await startNative({ ...PREFS, audio: false }, 'w:1', quiet, b);
    expect((s.stream as unknown as FakeStream).getAudioTracks()).toHaveLength(0);
  });

  it('vigia do Discord: som sai no meio → faixa removida e o evento avisa', async () => {
    const b = new FakeBridge();
    started(b, 'pc');
    const got: unknown[] = [];
    const s = await startNative({ ...PREFS, surface: 'monitor' }, 'm:1', { ...quiet, audio: (k, w) => got.push([k, w]) }, b);
    (b.channels[0] as (m: FrameMsg) => void)({ t: 'audio', kind: 'none', warn: 'many_discords' });
    expect((s.stream as unknown as FakeStream).getAudioTracks()).toHaveLength(0);
    expect(got).toEqual([['none', 'many_discords']]);
  });

  it('fonte fechou → ended uma vez, geradores parados e buffers devolvidos', async () => {
    const b = new FakeBridge();
    started(b);
    let ended = 0;
    const s = await startNative(PREFS, 'w:1', { ...quiet, ended: () => ended++ }, b);
    const frames = b.channels[0] as (m: FrameMsg) => void;
    frames({ t: 'ended', reason: 'source_closed' });
    frames({ t: 'ended', reason: 'stopped' });
    expect(ended).toBe(1);
    expect((s.stream as unknown as FakeStream).getVideoTracks()[0].stopped).toBe(true);
    expect(b.released).toHaveLength(3);
    expect(b.hostCb).toBeNull();
  });

  it('tela preta → aviso', async () => {
    const b = new FakeBridge();
    started(b);
    const warns: string[] = [];
    await startNative(PREFS, 'w:1', { ...quiet, warn: (r) => warns.push(r) }, b);
    (b.channels[0] as (m: FrameMsg) => void)({ t: 'warn', reason: 'black' });
    expect(warns).toEqual(['black']);
  });

  it('qualidade no ar: chama o app e troca para a geração nova de slots', async () => {
    const b = new FakeBridge();
    started(b);
    const s = await startNative(PREFS, 'w:1', quiet, b);
    await s.setQuality(1080, 30);
    expect(b.calls.at(-1)).toEqual({ cmd: 'capture_set_quality', args: { resolution: 1080, fps: 30 } });
    b.postSlots(2, 8, 4);
    expect(b.released).toHaveLength(3);
    const frames = b.channels[0] as (m: FrameMsg) => void;
    frames({ t: 'frame', gen: 1, slot: 0, seq: 0, ts: 0 });
    frames({ t: 'frame', gen: 2, slot: 0, seq: 1, ts: 1 });
    await Promise.resolve();
    const written = (s.stream as unknown as FakeStream).getVideoTracks()[0].written as FakeVideoFrame[];
    expect(written.map((f) => f.init.codedWidth)).toEqual([8]);
  });

  it('erro ao começar: limpa tudo e repassa o erro (source_gone)', async () => {
    const b = new FakeBridge();
    b.handlers.capture_start = () => {
      throw 'source_gone';
    };
    const ended = vi.fn();
    await expect(startNative(PREFS, 'w:1', { ...quiet, ended }, b)).rejects.toBe('source_gone');
    expect(ended).not.toHaveBeenCalled();
    expect(b.bufferCb).toBeNull();
  });

  it('parar avisa o app', async () => {
    const b = new FakeBridge();
    started(b);
    const s = await startNative(PREFS, 'w:1', quiet, b);
    await s.stop();
    expect(b.calls.at(-1)?.cmd).toBe('capture_stop');
  });
});
