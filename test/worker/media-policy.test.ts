import { afterEach, expect, it, vi } from 'vitest';
import { mintIceServers } from '../../src/worker/turn';
afterEach(() => vi.restoreAllMocks());
it('fails closed across concurrent requests with or without configured credentials', async () => {
    const fetcher = vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('No network allowed'));
    const grants = await Promise.all([{}, { TURN_KEY_ID: 'test-key', TURN_KEY_API_TOKEN: 'test-token' }, {}].map(env => mintIceServers(env)));
    for (const grant of grants) {
        expect(grant.iceServers).toEqual([{ urls: 'stun:stun.cloudflare.com:3478' }]);
        expect(grant.username).toBeUndefined();
    }
    expect(fetcher).not.toHaveBeenCalled();
});
