import type { IceServer } from './protocol';
export interface TransmissionKey {
    peerId: string;
    startedAt: number;
}
export interface BroadcastOrder {
    highestBroadcastSeq: number;
    requestId: string;
    lastStartedAt: number;
    status: 'live' | 'ended';
}
export const MEDIA_PROTOCOL = 2;
export const DIRECT_ONLY = { kind: 'direct-only', reason: 'unverified-budget' } as const;
export type MediaPolicy = {
    kind: 'direct-only';
    reason: 'unverified-budget' | 'exhausted';
};
export const isSafeSequence = (v: unknown): v is number => typeof v === 'number' && Number.isSafeInteger(v) && v >= 0;
export const isRequestId = (v: unknown): v is string => typeof v === 'string' && /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(v);
export function sameTransmission(a: TransmissionKey | null, b: TransmissionKey | null) { return !!a && !!b && a.peerId === b.peerId && a.startedAt === b.startedAt; }
export function directIceServers(servers: IceServer[]): IceServer[] {
    return servers.flatMap(server => { const urls = (typeof server.urls === 'string' ? [server.urls] : server.urls).filter(url => /^stuns?:/i.test(url)); return urls.length ? [{ urls }] : []; });
}
export function nextBroadcastOrder(current: BroadcastOrder | null, msg: {
    t: 'live' | 'unlive';
    requestId: string;
    broadcastSeq: number;
}, now: number): BroadcastOrder | null {
    if (!isRequestId(msg.requestId) || !isSafeSequence(msg.broadcastSeq) || !isSafeSequence(now))
        return null;
    if (!current && msg.broadcastSeq !== 0)
        return null;
    if (current && msg.broadcastSeq < current.highestBroadcastSeq)
        return null;
    if (current && msg.broadcastSeq === current.highestBroadcastSeq) {
        if (msg.requestId !== current.requestId || (msg.t === 'live' && current.status === 'ended'))
            return null;
        return { ...current, status: msg.t === 'unlive' ? 'ended' : 'live' };
    }
    if (msg.t === 'unlive' || (current && msg.requestId === current.requestId))
        return null;
    if (current && msg.broadcastSeq !== current.highestBroadcastSeq + 1)
        return null;
    const lastStartedAt = Math.max(now, (current?.lastStartedAt ?? -1) + 1);
    return isSafeSequence(lastStartedAt) ? { highestBroadcastSeq: msg.broadcastSeq, requestId: msg.requestId, lastStartedAt, status: 'live' } : null;
}
