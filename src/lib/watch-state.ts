import type { TransmissionKey } from '../shared/media';
export interface WatchChoice {
    transmission: TransmissionKey;
    revision: number;
    status: 'opening' | 'playing' | 'unavailable';
}
type LivePeer = {
    id: string;
    live: {
        startedAt: number;
    } | null;
};
export function chooseWatch(choices: ReadonlyMap<string, WatchChoice>, peer: LivePeer, on: boolean, mobile: boolean): Map<string, WatchChoice> {
    const next = mobile && on ? new Map<string, WatchChoice>() : new Map(choices);
    if (on && peer.live)
        next.set(peer.id, { transmission: { peerId: peer.id, startedAt: peer.live.startedAt }, revision: (choices.get(peer.id)?.revision ?? 0) + 1, status: 'opening' });
    else
        next.delete(peer.id);
    return next;
}
export function pruneWatching(choices: ReadonlyMap<string, WatchChoice>, peers: LivePeer[]): Map<string, WatchChoice> {
    return new Map([...choices].filter(([id, choice]) => peers.some(p => p.id === id && p.live?.startedAt === choice.transmission.startedAt)));
}
