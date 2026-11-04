// Indicadores locais de etapas da mídia. Não são atraso ponta a ponta.
export interface StatsRow {
  id: string;
  type: string;
  timestamp: number;
  [key: string]: unknown;
}

export interface StreamMetrics {
  direction: 'send' | 'receive';
  kind: 'audio' | 'video';
  encodeMs: number | null;
  sendQueueMs: number | null;
  jitterMs: number | null;
  decodeMs: number | null;
  mbps: number | null;
  frames: number | null;
  lostPackets: number | null;
  freezes: number | null;
}

export interface MediaStatsSample {
  elapsedMs: number;
  route: 'host' | 'srflx' | 'relay' | null;
  rttMs: number | null;
  streams: StreamMetrics[];
}

const HISTORY_SIZE = 60;
const COUNTERS = ['framesEncoded', 'totalEncodeTime', 'packetsSent', 'totalPacketSendDelay',
  'bytesSent', 'bytesReceived', 'jitterBufferDelay', 'jitterBufferEmittedCount',
  'totalDecodeTime', 'framesDecoded', 'packetsLost', 'freezeCount'] as const;

function number(row: StatsRow | undefined, key: string): number | null {
  const value = row?.[key];
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : null;
}

function delta(row: StatsRow, before: StatsRow | undefined, key: string): number | null {
  if (!before || !Number.isFinite(row.timestamp) || row.timestamp <= before.timestamp) return null;
  const a = number(row, key), b = number(before, key);
  return a !== null && b !== null && a >= b ? a - b : null;
}

function ratioMs(row: StatsRow, before: StatsRow | undefined, time: string, count: string): number | null {
  const seconds = delta(row, before, time), n = delta(row, before, count);
  return seconds !== null && n !== null && n > 0 ? seconds * 1000 / n : null;
}

function copy(sample: MediaStatsSample): MediaStatsSample {
  return { ...sample, streams: sample.streams.map((s) => ({ ...s })) };
}

export class MediaStats {
  private previous = new Map<string, StatsRow>();
  private samples: MediaStatsSample[] = [];
  private origin: number | null = null;

  read(rows: readonly StatsRow[], nowMs: number): MediaStatsSample {
    this.origin ??= nowMs;
    const next = new Map<string, StatsRow>();
    const streams: StreamMetrics[] = [];
    for (const row of rows) {
      if ((row.type !== 'outbound-rtp' && row.type !== 'inbound-rtp') ||
          (row.kind !== 'audio' && row.kind !== 'video') || !Number.isFinite(row.timestamp)) continue;
      const key = JSON.stringify([row.type, row.id, row.ssrc, row.kind]);
      let before = this.previous.get(key);
      // Um contador que reinicia invalida a linha toda, não mistura épocas.
      if (before && COUNTERS.some((field) => {
        const a = number(row, field), b = number(before, field);
        return a !== null && b !== null && a < b;
      })) before = undefined;
      const outgoing = row.type === 'outbound-rtp';
      const bytes = delta(row, before, outgoing ? 'bytesSent' : 'bytesReceived');
      const elapsed = before ? row.timestamp - before.timestamp : 0;
      streams.push({
        direction: outgoing ? 'send' : 'receive', kind: row.kind,
        encodeMs: outgoing ? ratioMs(row, before, 'totalEncodeTime', 'framesEncoded') : null,
        sendQueueMs: outgoing ? ratioMs(row, before, 'totalPacketSendDelay', 'packetsSent') : null,
        jitterMs: outgoing ? null : ratioMs(row, before, 'jitterBufferDelay', 'jitterBufferEmittedCount'),
        decodeMs: outgoing ? null : ratioMs(row, before, 'totalDecodeTime', 'framesDecoded'),
        mbps: bytes !== null && elapsed > 0 ? bytes * 8 / (elapsed * 1000) : null,
        frames: delta(row, before, outgoing ? 'framesEncoded' : 'framesDecoded'),
        lostPackets: outgoing ? null : delta(row, before, 'packetsLost'),
        freezes: outgoing ? null : delta(row, before, 'freezeCount'),
      });
      // Só os contadores necessários ficam retidos: nunca SDP, endereços ou outros campos do report.
      const snapshot: StatsRow = { id: row.id, type: row.type, timestamp: row.timestamp };
      for (const field of COUNTERS) snapshot[field] = number(row, field);
      next.set(key, snapshot);
    }
    this.previous = next;
    const byId = new Map(rows.map((r) => [r.id, r]));
    const transport = rows.find((r) => r.type === 'transport' && typeof r.selectedCandidatePairId === 'string');
    const pair = transport ? byId.get(String(transport.selectedCandidatePairId)) : undefined;
    const candidates = pair ? [byId.get(String(pair.localCandidateId)), byId.get(String(pair.remoteCandidateId))] : [];
    const types = candidates.map((c) => c?.candidateType);
    const route = types.includes('relay') ? 'relay' : types.includes('srflx') ? 'srflx' : types.includes('host') ? 'host' : null;
    const rtt = number(pair, 'currentRoundTripTime');
    const sample: MediaStatsSample = { elapsedMs: Math.max(0, nowMs - this.origin), route,
      rttMs: rtt === null ? null : rtt * 1000, streams };
    this.samples.push(copy(sample));
    if (this.samples.length > HISTORY_SIZE) this.samples.shift();
    return sample;
  }

  history(): MediaStatsSample[] {
    return this.samples.map(copy);
  }

  clear() {
    this.previous.clear();
    this.samples = [];
    this.origin = null;
  }
}
