// Ponte de captura do app desktop (specs/005, contracts/native-capture.md). No navegador nada disso
// existe: detectNative() responde false e o seletor segue o fluxo da 004 (getDisplayMedia).
//
// Vídeo: o app escreve NV12 em 3 buffers compartilhados e avisa por um Channel qual slot ficou pronto;
// aqui viram VideoFrames num MediaStreamTrackGenerator. Som: pedaços de 10 ms de f32 → AudioData.
import { Channel, invoke } from '@tauri-apps/api/core';
import type { AudioKind, CaptureSource, GoLivePrefs } from './types';
import { CaptureTiming, MAX_MEDIA_AGE_US } from './capture-timing';

export const NATIVE_VERSION = 1;
export const POLL_MS = 2_000;

export type FrameMsg =
  | { t: 'frame'; gen: number; slot: number; seq: number; ts: number }
  | { t: 'ended'; reason: 'source_closed' | 'stopped' | 'error' }
  | { t: 'audio'; kind: AudioKind; warn?: AudioWarn }
  | { t: 'warn'; reason: 'black' };

export type AudioWarn = 'unsupported' | 'many_discords';

export interface StartResult {
  width: number;
  height: number;
  audio: AudioKind;
  audioWarn?: AudioWarn;
}

export interface SlotMeta {
  espia: 'frame-slot';
  gen: number;
  slot: number;
  width: number;
  height: number;
  format: 'NV12';
}

/** O que a ponte precisa do ambiente. Injetável para testar sem o app. */
export interface Bridge {
  available(): boolean;
  invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
  /** Cria um canal do Tauri; o objeto devolvido vai nos args do invoke. */
  channel<T>(onmessage: (m: T) => void): unknown;
  /** Buffers compartilhados postados pelo app (chrome.webview "sharedbufferreceived"). */
  onSharedBuffer(cb: (buffer: ArrayBuffer, meta: unknown) => void): () => void;
  releaseBuffer(buffer: ArrayBuffer): void;
  /**
   * Mensagens do app 0.1.7+ pelo canal do WebView2 (chrome.webview "message"), sem um script por mensagem
   * (specs/010): `{espia:'pcm', d: base64}` (som) e `{espia:'f', m: FrameMsg}` (avisos de quadro e de estado).
   */
  onHostMessage(cb: (msg: HostMessage) => void): () => void;
}

export type HostMessage = { espia: 'pcm'; d: string } | { espia: 'f'; m: FrameMsg };

interface WebViewHost {
  addEventListener(type: 'sharedbufferreceived', cb: (e: { getBuffer(): ArrayBuffer; additionalData: unknown }) => void): void;
  addEventListener(type: 'message', cb: (e: { data: unknown }) => void): void;
  removeEventListener(type: 'sharedbufferreceived' | 'message', cb: (e: never) => void): void;
  releaseBuffer(buffer: ArrayBuffer): void;
}

function webviewHost(): WebViewHost | undefined {
  return (globalThis as { chrome?: { webview?: WebViewHost } }).chrome?.webview;
}

export const tauriBridge: Bridge = {
  available: () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window && !!webviewHost(),
  invoke: (cmd, args) => invoke(cmd, args),
  channel<T>(onmessage: (m: T) => void) {
    const ch = new Channel<T>();
    ch.onmessage = onmessage;
    return ch;
  },
  onSharedBuffer(cb) {
    const host = webviewHost();
    if (!host) return () => {};
    const listener = (e: { getBuffer(): ArrayBuffer; additionalData: unknown }) => {
      let meta = e.additionalData;
      if (typeof meta === 'string') {
        try {
          meta = JSON.parse(meta);
        } catch {
          // Não é nosso.
        }
      }
      cb(e.getBuffer(), meta);
    };
    host.addEventListener('sharedbufferreceived', listener);
    return () => host.removeEventListener('sharedbufferreceived', listener as (e: never) => void);
  },
  releaseBuffer: (buffer) => webviewHost()?.releaseBuffer(buffer),
  onHostMessage(cb) {
    const host = webviewHost();
    if (!host) return () => {};
    const listener = (e: { data: unknown }) => {
      const m = e.data as { espia?: unknown } | null;
      if (m?.espia === 'pcm' || m?.espia === 'f') cb(m as HostMessage);
    };
    host.addEventListener('message', listener);
    return () => host.removeEventListener('message', listener as (e: never) => void);
  },
};

/** Dentro do app desktop com a ponte certa? Qualquer erro → false (fluxo da 004, FR-511). */
export async function detectNative(bridge: Bridge = tauriBridge): Promise<boolean> {
  if (!bridge.available()) return false;
  try {
    const r = await bridge.invoke<{ ok: boolean; version: number }>('capture_supported');
    return !!r?.ok && r.version === NATIVE_VERSION;
  } catch {
    return false;
  }
}

export function listSources(thumbs: boolean, bridge: Bridge = tauriBridge): Promise<CaptureSource[]> {
  return bridge.invoke<CaptureSource[]>('capture_list', { thumbs });
}

/** Busca a lista agora e a cada 2 s até chamar a função devolvida (FR-504). */
export function pollSources(
  onUpdate: (sources: CaptureSource[]) => void,
  onError: (err: unknown) => void,
  bridge: Bridge = tauriBridge,
  intervalMs = POLL_MS,
): () => void {
  let stopped = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const tick = async () => {
    try {
      const sources = await listSources(true, bridge);
      if (!stopped) onUpdate(sources);
    } catch (err) {
      if (!stopped) onError(err);
      return;
    }
    if (!stopped) timer = setTimeout(tick, intervalMs);
  };
  void tick();
  return () => {
    stopped = true;
    clearTimeout(timer);
  };
}

/** Chave estável entre aberturas (o HWND muda quando o programa reabre). */
export function sourceKey(s: Pick<CaptureSource, 'kind' | 'app' | 'title'>): string {
  return s.kind === 'monitor' ? `m:${s.title}` : `${s.app}|${s.title}`;
}

/** A fonte que bate com a última transmitida, ou null. */
export function matchLastSource(sources: CaptureSource[], key: string | undefined): string | null {
  if (!key) return null;
  return sources.find((s) => sourceKey(s) === key)?.id ?? null;
}

// ---------- Quadros ----------

/** Guarda os slots por geração e decide qual buffer usar para cada aviso de quadro. */
export class SlotTable {
  private slots = new Map<string, { buffer: ArrayBuffer; meta: SlotMeta }>();
  gen = 0;
  private lastSeq = -1;

  constructor(private release: (b: ArrayBuffer) => void) {}

  add(buffer: ArrayBuffer, meta: unknown): boolean {
    const m = meta as Partial<SlotMeta> | null;
    if (!m || m.espia !== 'frame-slot' || typeof m.gen !== 'number' || typeof m.slot !== 'number') return false;
    if (m.gen < this.gen) {
      this.release(buffer);
      return true;
    }
    if (m.gen > this.gen) {
      this.dropBelow(m.gen);
      this.gen = m.gen;
      this.lastSeq = -1;
    }
    this.slots.set(`${m.gen}:${m.slot}`, { buffer, meta: m as SlotMeta });
    return true;
  }

  /** Buffer do quadro anunciado, ou null (geração velha, slot que não chegou, aviso atrasado). */
  take(msg: { gen: number; slot: number; seq: number }): { buffer: ArrayBuffer; meta: SlotMeta } | null {
    if (msg.gen !== this.gen || msg.seq <= this.lastSeq) return null;
    this.lastSeq = msg.seq;
    return this.slots.get(`${msg.gen}:${msg.slot}`) ?? null;
  }

  /** O app reescreve um slot depois de anunciar os outros dois: atraso > 2 = slot já sobrescrito. */
  stale(msg: { seq: number }, newest: number): boolean {
    return newest - msg.seq > 2;
  }

  dropBelow(gen: number) {
    for (const [k, v] of this.slots) {
      if (v.meta.gen < gen) {
        this.release(v.buffer);
        this.slots.delete(k);
      }
    }
  }

  clear() {
    this.dropBelow(Number.POSITIVE_INFINITY);
  }
}

// Tipos do Chromium (breakout box), fora do lib.dom.
declare class MediaStreamTrackGenerator extends MediaStreamTrack {
  constructor(init: { kind: 'video' | 'audio' });
  readonly writable: WritableStream<VideoFrame | AudioData>;
}

const NV12_COLOR: VideoColorSpaceInit = { primaries: 'bt709', transfer: 'bt709', matrix: 'bt709', fullRange: false };
const RATE = 48_000;
const CHANNELS = 2;

/** Som na fila do gerador acima disso: o pedaço novo é descartado em vez de acumular (specs/010, FR-1001). */
export const SOUND_MAX_QUEUE_MS = 200;

export function shouldDropSound(queuedMs: number): boolean {
  return queuedMs > SOUND_MAX_QUEUE_MS;
}

/**
 * Bytes de um pedaço de som. O app 0.1.7+ manda texto base64 pelo canal de mensagens do WebView2 (specs/010,
 * research R2). Apps antigos mandam ArrayBuffer pelo protocolo `ipc.localhost`, ou uma lista de
 * bytes se o Tauri cair no modo postMessage (ex.: CSP sem `ipc.localhost`).
 */
export function soundBytes(msg: unknown): ArrayBuffer | null {
  if (typeof msg === 'string') {
    try {
      const bin = atob(msg);
      const out = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
      return out.buffer;
    } catch {
      return null;
    }
  }
  if (msg instanceof ArrayBuffer) return msg;
  if (ArrayBuffer.isView(msg)) return new Uint8Array(msg.buffer, msg.byteOffset, msg.byteLength).slice().buffer;
  if (Array.isArray(msg)) return new Uint8Array(msg).buffer;
  return null;
}

export interface NativeSession {
  stream: MediaStream;
  result: StartResult;
  setQuality(resolution: 720 | 1080, fps: 30 | 60): Promise<void>;
  stop(): Promise<void>;
}

export interface NativeEvents {
  ended(): void;
  audio(kind: AudioKind, warn?: AudioWarn): void;
  warn(reason: 'black'): void;
}

/** Começa a transmitir a fonte escolhida (US1/US2/US3). Lança 'source_gone' / 'capture_failed'. */
export async function startNative(
  prefs: GoLivePrefs,
  sourceId: string,
  events: NativeEvents,
  bridge: Bridge = tauriBridge,
  signal?: AbortSignal,
): Promise<NativeSession> {
  if (signal?.aborted) throw new DOMException('Captura cancelada', 'AbortError');
  const slots = new SlotTable((b) => bridge.releaseBuffer(b));
  const offBuffers = bridge.onSharedBuffer((buffer, meta) => slots.add(buffer, meta));
  const video = new MediaStreamTrackGenerator({ kind: 'video' });
  const videoWriter = video.writable.getWriter();
  const stream = new MediaStream([video]);
  // Faixa de som só existe enquanto chega som (o vigia do Discord pode ligar/desligar no meio).
  let audio: MediaStreamTrackGenerator | null = null;
  let audioWriter: WritableStreamDefaultWriter<VideoFrame | AudioData> | null = null;
  // O relógio nativo só começa DEPOIS de preparar GPU/permissões, não antes do invoke.
  const timing = new CaptureTiming();
  let audioGeneration = 0;
  /** Som já entregue ao gerador e ainda não consumido (µs). */
  let audioQueuedUs = 0;
  let soundDropped = 0;
  const addAudio = () => {
    audio = new MediaStreamTrackGenerator({ kind: 'audio' });
    audioWriter = audio.writable.getWriter();
    stream.addTrack(audio);
  };
  const removeAudio = () => {
    audioGeneration++;
    timing.resetAudio();
    if (!audio) return;
    stream.removeTrack(audio);
    audio.stop();
    void audioWriter?.abort?.().catch(() => {});
    audio = audioWriter = null;
    audioQueuedUs = 0;
  };
  let newest = -1;
  let busy = false;
  let done = false;
  let qualityRevision = 0;
  let qualityQueue: Promise<void> = Promise.resolve();
  let hostStopped = false;
  const stopHost = async () => {
    if (hostStopped) return;
    hostStopped = true;
    await bridge.invoke('capture_stop').catch(() => {});
  };
  let offAbort = () => {};
  let dropped = 0;
  let sent = 0;
  const stats = setInterval(() => {
    console.debug('[espia] captura nativa:', { fonte: sourceId, gen: slots.gen, enviados: sent, descartados: dropped, somDescartado: soundDropped });
    sent = dropped = soundDropped = 0;
  }, 10_000);

  const finish = (notify = true) => {
    if (done) return;
    done = true;
    qualityRevision++;
    offAbort();
    timing.stop();
    clearInterval(stats);
    offBuffers();
    offHost();
    slots.clear();
    video.stop();
    audio?.stop();
    void videoWriter.abort?.().catch(() => {});
    void audioWriter?.abort?.().catch(() => {});
    if (notify) events.ended();
  };

  const onFrame = (msg: FrameMsg) => {
    if (done) return;
    if (msg.t === 'ended') return finish();
    if (msg.t === 'audio') {
      if (msg.kind === 'none') removeAudio();
      return events.audio(msg.kind, msg.warn);
    }
    if (msg.t === 'warn') return events.warn(msg.reason);
    if (!Number.isSafeInteger(msg.seq) || msg.seq < 0 || !Number.isSafeInteger(msg.ts) || msg.ts < 0) return;
    newest = Math.max(newest, msg.seq);
    const slot = slots.stale(msg, newest) || busy ? null : slots.take(msg);
    if (!slot) {
      dropped++;
      return;
    }
    const timestamp = timing.video(msg.ts, msg.seq);
    if (timestamp === null) { dropped++; return; }
    const { width, height } = slot.meta;
    let frame: VideoFrame;
    try {
      // Copia na hora: depois disso o app pode reescrever o slot.
      frame = new VideoFrame(new Uint8Array(slot.buffer, 0, (width * height * 3) / 2), {
        format: 'NV12',
        codedWidth: width,
        codedHeight: height,
        timestamp,
        colorSpace: NV12_COLOR,
      });
    } catch (err) {
      console.warn('[espia] quadro inválido', err);
      dropped++;
      return;
    }
    busy = true;
    sent++;
    try {
      videoWriter.write(frame).then(
        () => (busy = false),
        () => { frame.close(); busy = false; },
      );
    } catch { frame.close(); busy = false; }
  };

  const onSound = (msg: unknown) => {
    const buf = soundBytes(msg);
    if (done || !buf || !buf.byteLength || buf.byteLength % (CHANNELS * 4) !== 0) return;
    if (!audioWriter) addAudio();
    const data = new Float32Array(buf);
    const frames = data.length / CHANNELS;
    const us = Math.round((frames / RATE) * 1_000_000);
    const ts = timing.audio(frames);
    if (ts === null || audioQueuedUs + us > MAX_MEDIA_AGE_US) {
      soundDropped++;
      return;
    }
    const chunk = new AudioData({ format: 'f32', sampleRate: RATE, numberOfChannels: CHANNELS, numberOfFrames: frames, timestamp: ts, data });
    audioQueuedUs += us;
    const generation = audioGeneration;
    const consumed = () => {
      if (generation === audioGeneration) audioQueuedUs = Math.max(0, audioQueuedUs - us);
    };
    try { audioWriter!.write(chunk).then(consumed, () => { chunk.close(); consumed(); }); }
    catch { chunk.close(); consumed(); }
  };
  // App 0.1.7+: pelo canal de mensagens do WebView2. Apps antigos: pelos canais do Tauri.
  const offHost = bridge.onHostMessage((m) => (m.espia === 'pcm' ? onSound(m.d) : onFrame(m.m)));
  const frames = bridge.channel<FrameMsg>(onFrame);
  const sound = bridge.channel<ArrayBuffer | number[]>(onSound);
  const abort = () => {
    finish();
    void stopHost();
  };
  signal?.addEventListener('abort', abort, { once: true });
  offAbort = () => signal?.removeEventListener('abort', abort);

  let result: StartResult;
  try {
    result = await bridge.invoke<StartResult>('capture_start', {
      id: sourceId,
      resolution: prefs.resolution,
      fps: prefs.fps,
      audio: prefs.audio,
      frames,
      sound,
    });
  } catch (err) {
    finish(false);
    throw err;
  }

  if (done) {
    await stopHost();
    throw new DOMException('Captura encerrada durante a abertura', 'AbortError');
  }

  // Já com a faixa de som, se vai ter som: a malha manda as duas de uma vez.
  if (result.audio !== 'none' && !audio) addAudio();
  video.contentHint = prefs.fps === 60 ? 'motion' : 'detail';

  return {
    stream,
    result,
    async setQuality(resolution, fps) {
      if (done) return;
      const revision = ++qualityRevision;
      qualityQueue = qualityQueue.catch(() => {}).then(async () => {
        if (done || revision !== qualityRevision) return;
        await bridge.invoke('capture_set_quality', { resolution, fps });
        if (!done && revision === qualityRevision) video.contentHint = fps === 60 ? 'motion' : 'detail';
      });
      await qualityQueue;
    },
    async stop() {
      finish();
      await stopHost();
    },
  };
}
