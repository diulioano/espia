import { describe, expect, it, vi } from 'vitest';
import { parseClientMessage } from '../../src/worker/validate';
import { nextBroadcastOrder } from '../../src/shared/media';
import { mintIceServers } from '../../src/worker/turn';
const a = { t: 'live' as const, requestId: '00000000-0000-4000-8000-000000000001', broadcastSeq: 0 };
const b = { ...a, requestId: '00000000-0000-4000-8000-000000000002', broadcastSeq: 1 };
describe('media protocol v2', () => {
    it('rejects sequence overflow, negative values and skipped initial sequence', () => {
        for(const broadcastSeq of [-1,Number.MAX_SAFE_INTEGER+1]) {
            expect(parseClientMessage(JSON.stringify({...a,broadcastSeq,resolution:720,fps:30,audio:'none'}))).toBeNull();
        }
        expect(nextBroadcastOrder(null,{...a,broadcastSeq:1},100)).toBeNull();
    });
    it('requires version and a valid capture identity', () => {
        expect(parseClientMessage(JSON.stringify({ t: 'join', nickname: 'Ana' }))).toBeNull();
        expect(parseClientMessage(JSON.stringify({ t: 'join', nickname: 'Ana', mediaProtocol: 2 }))).toMatchObject({ mediaProtocol: 2 });
        expect(parseClientMessage(JSON.stringify({ ...a, resolution: 720, fps: 30, audio: 'none' }))).not.toBeNull();
        expect(parseClientMessage(JSON.stringify({ ...a, requestId: 'old', resolution: 720, fps: 30, audio: 'none' }))).toBeNull();
    });
    it('never reopens ended/superseded requests and preserves canonical time', () => {
        const first = nextBroadcastOrder(null, a, 100)!;
        expect(nextBroadcastOrder(first, a, 200)?.lastStartedAt).toBe(100);
        const ended = nextBroadcastOrder(first, { ...a, t: 'unlive' }, 200)!;
        expect(nextBroadcastOrder(ended, a, 300)).toBeNull();
        const newer = nextBroadcastOrder(ended, b, 1)!;
        expect(newer.lastStartedAt).toBe(101);
        expect(nextBroadcastOrder(newer, { ...a, t: 'unlive' }, 300)).toBeNull();
        expect(nextBroadcastOrder(JSON.parse(JSON.stringify(newer)), a, 300)).toBeNull();
    });
    it('does not issue TURN even when secrets exist', async () => {
        const fetcher = vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(null, { status: 500 }));
        const ice = await mintIceServers({ TURN_KEY_ID: 'test-key', TURN_KEY_API_TOKEN: 'test-token' });
        expect(ice.iceServers.flatMap(s => s.urls)).toEqual(['stun:stun.cloudflare.com:3478']);
        expect(fetcher).not.toHaveBeenCalled();
        fetcher.mockRestore();
    });
});
