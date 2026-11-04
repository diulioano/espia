import { describe, expect, it } from 'vitest';
import { MediaStats, type StatsRow } from '../../src/lib/media-stats';

const out = (timestamp: number, over: Partial<StatsRow> = {}): StatsRow => ({
  id: 'out-private-id', type: 'outbound-rtp', timestamp, kind: 'video', ssrc: 12,
  framesEncoded: timestamp / 100, totalEncodeTime: timestamp / 10_000,
  packetsSent: timestamp / 10, totalPacketSendDelay: timestamp / 100_000,
  bytesSent: timestamp * 100, ...over,
});

describe('MediaStats — diagnóstico local, não atraso ponta a ponta', () => {
  it('usa deltas, converte segundos para ms e não inventa a primeira amostra', () => {
    const stats = new MediaStats();
    expect(stats.read([out(1000)], 1000).streams[0].encodeMs).toBeNull();
    const row = stats.read([out(2000)], 2000).streams[0];
    expect(row.encodeMs).toBeCloseTo(10);
    expect(row.sendQueueMs).toBeCloseTo(0.1);
    expect(row.mbps).toBeCloseTo(0.8);
    expect(row.jitterMs).toBeNull();
  });

  it('mede jitter e decode separadamente para recepção de áudio e vídeo', () => {
    const stats = new MediaStats();
    const inbound = (timestamp: number): StatsRow => ({ id: 'in', type: 'inbound-rtp', kind: 'video', timestamp,
      jitterBufferDelay: timestamp / 10_000, jitterBufferEmittedCount: timestamp / 100,
      totalDecodeTime: timestamp / 100_000, framesDecoded: timestamp / 100 });
    stats.read([inbound(1000)], 1000);
    const row = stats.read([inbound(2000)], 2000).streams[0];
    expect(row.jitterMs).toBeCloseTo(10);
    expect(row.decodeMs).toBeCloseTo(1);
    expect(row.encodeMs).toBeNull();
  });

  it('retorna null para ausência, denominador zero, reset e mudança de SSRC', () => {
    const stats = new MediaStats();
    stats.read([out(1000)], 1000);
    expect(stats.read([out(2000, { framesEncoded: 10 })], 2000).streams[0].encodeMs).toBeNull();
    expect(stats.read([out(3000, { framesEncoded: 0 })], 3000).streams[0].encodeMs).toBeNull();
    expect(stats.read([out(4000, { ssrc: 99 })], 4000).streams[0].encodeMs).toBeNull();
    expect(stats.read([out(4000, { ssrc: 99 })], 5000).streams[0].encodeMs).toBeNull();
    expect(stats.read([{ id: 'unsupported', type: 'inbound-rtp', kind: 'audio', timestamp: 6000 }], 6000).streams[0].jitterMs).toBeNull();
  });

  it('não reutiliza baselines de faixas que sumiram e limpa histórico', () => {
    const stats = new MediaStats();
    stats.read([out(1000)], 1000);
    stats.read([], 2000);
    expect(stats.read([out(3000)], 3000).streams[0].encodeMs).toBeNull();
    stats.clear();
    expect(stats.history()).toEqual([]);
    expect(stats.read([out(4000)], 4000).streams[0].encodeMs).toBeNull();
  });

  it('limita a 60 amostras e exporta somente números e enums sanitizados', () => {
    const stats = new MediaStats();
    for (let t = 1; t <= 65; t++) stats.read([out(t * 1000)], t * 1000);
    expect(stats.history()).toHaveLength(60);
    const exported = JSON.stringify(stats.history());
    expect(exported).not.toContain('private-id');
    expect(exported).not.toContain('ssrc');
    const copy = stats.history();
    copy[0].streams.length = 0;
    expect(stats.history()[0].streams).toHaveLength(1);
  });

  it('seleciona somente o tipo da rota ativa, sem endereço ou ID', () => {
    const stats = new MediaStats();
    const rows: StatsRow[] = [
      { id: 'transport', type: 'transport', timestamp: 1, selectedCandidatePairId: 'pair' },
      { id: 'pair', type: 'candidate-pair', timestamp: 1, localCandidateId: 'local', remoteCandidateId: 'remote', currentRoundTripTime: 0.04 },
      { id: 'local', type: 'local-candidate', timestamp: 1, candidateType: 'host', address: 'secret' },
      { id: 'remote', type: 'remote-candidate', timestamp: 1, candidateType: 'relay', address: 'secret' },
    ];
    const sample = stats.read(rows, 1);
    expect(sample.route).toBe('relay');
    expect(sample.rttMs).toBe(40);
    expect(JSON.stringify(sample)).not.toContain('secret');
  });
});
