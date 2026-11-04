import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { PeerLink, preferLowLatency } from '../../src/lib/peer';
import { Mesh } from '../../src/lib/mesh';

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => { resolve = r; });
  return { promise, resolve };
}

class FakePC {
  static instances: FakePC[] = [];
  getStats = vi.fn(() => Promise.resolve(new Map()));
  close = vi.fn();
  createDataChannel = vi.fn(() => ({ readyState: 'connecting' }));
  receivers: { jitterBufferTarget: number | null }[] = [];
  getReceivers = () => this.receivers;
  constructor() { FakePC.instances.push(this); }
}

const callbacks = () => ({ signal: vi.fn(), remote: vi.fn(), state: vi.fn(), view: vi.fn() });
const report = () => new Map([['in', { id: 'in', type: 'inbound-rtp', kind: 'video', timestamp: 1000 }]]);

describe('diagnóstico por peer', () => {
  beforeEach(() => {
    FakePC.instances = [];
    vi.stubGlobal('RTCPeerConnection', FakePC);
    vi.stubGlobal('MediaStream', class { getTracks() { return []; } });
  });
  afterEach(() => vi.unstubAllGlobals());

  it('coleta de quem apenas assiste e tolera campos indisponíveis', async () => {
    const link = new PeerLink('private-peer', false, [], false, callbacks());
    FakePC.instances[0].getStats.mockResolvedValue(report());
    const result = await link.pollStats();
    expect(result?.send).toBeNull();
    expect(result?.sample.streams[0].jitterMs).toBeNull();
    expect(link.diagnostics()).toHaveLength(1);
    expect(JSON.stringify(link.diagnostics())).not.toContain('private-peer');
    link.close();
    expect(link.diagnostics()).toEqual([]);
  });

  it('compartilha consulta pendente e ignora conclusão depois de fechar', async () => {
    const link = new PeerLink('private-peer', false, [], false, callbacks());
    const pending = deferred<Map<string, unknown>>();
    FakePC.instances[0].getStats.mockReturnValue(pending.promise);
    const first = link.pollStats(), second = link.pollStats();
    expect(FakePC.instances[0].getStats).toHaveBeenCalledTimes(1);
    link.close();
    pending.resolve(report());
    expect(await first).toBeNull();
    expect(await second).toBeNull();
    expect(link.diagnostics()).toEqual([]);
  });

  it('reset invalida a leitura em andamento sem abrir uma segunda consulta', async () => {
    const link = new PeerLink('p', false, [], false, callbacks());
    const pending = deferred<Map<string, unknown>>();
    FakePC.instances[0].getStats.mockReturnValue(pending.promise);
    const reading = link.pollStats();
    link.clearDiagnostics();
    const afterReset = link.pollStats();
    expect(FakePC.instances[0].getStats).toHaveBeenCalledTimes(1);
    pending.resolve(report());
    expect(await reading).toBeNull();
    expect(await afterReset).toBeNull();
    expect(link.diagnostics()).toEqual([]);
  });

  it('rejeição do motor não quebra a próxima coleta', async () => {
    const link = new PeerLink('p', false, [], false, callbacks());
    const pc = FakePC.instances[0];
    pc.getStats.mockRejectedValueOnce(new Error('unsupported'));
    expect(await link.pollStats()).toBeNull();
    pc.getStats.mockResolvedValue(report());
    expect((await link.pollStats())?.sample.streams).toHaveLength(1);
  });

  it('preferência opcional tolera ausência e recusa do setter', () => {
    expect(() => preferLowLatency({} as RTCRtpReceiver)).not.toThrow();
    const unsupported = Object.defineProperty({}, 'jitterBufferTarget', { get: () => null, set: () => { throw new Error('unsupported'); } });
    expect(() => preferLowLatency(unsupported as RTCRtpReceiver)).not.toThrow();
    const receiver = { jitterBufferTarget: null };
    preferLowLatency(receiver as RTCRtpReceiver);
    expect(receiver.jitterBufferTarget).toBe(0);
  });

  it('retorno à visualização ativa reaplica a preferência sem alterar modos no fio', () => {
    const link = new PeerLink('p', false, [], false, callbacks());
    const receiver = { jitterBufferTarget: 200 };
    FakePC.instances[0].receivers.push(receiver);
    link.setMyView('full');
    expect(receiver.jitterBufferTarget).toBe(0);
    receiver.jitterBufferTarget = 200;
    link.setMyView('hidden');
    expect(receiver.jitterBufferTarget).toBe(200);
    link.setMyView('thumb');
    expect(receiver.jitterBufferTarget).toBe(0);
  });

  it('malha descarta link removido no meio da coleta e exporta índice local', async () => {
    const mesh = new Mesh('me', callbacks());
    mesh.sync(['private-remote']);
    const pending = deferred<Map<string, unknown>>();
    FakePC.instances[0].getStats.mockReturnValue(pending.promise);
    const reading = mesh.pollStats();
    mesh.remove('private-remote');
    mesh.sync(['private-remote']);
    pending.resolve(report());
    expect(await reading).toEqual([]);
    FakePC.instances[1].getStats.mockResolvedValue(report());
    await mesh.pollStats();
    const exported = mesh.diagnostics();
    expect(exported[0].link).toBe(2);
    expect(exported[0].samples).toHaveLength(1);
    expect(JSON.stringify(exported)).not.toContain('private-remote');
    mesh.close();
    expect(mesh.diagnostics()).toEqual([]);
  });
});
