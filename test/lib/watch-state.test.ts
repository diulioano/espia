import { expect, it } from 'vitest';
import { chooseWatch, pruneWatching } from '../../src/lib/watch-state';
const a = { id: 'a', live: { startedAt: 10 } };
it('only explicit selection authorizes the current transmission', () => {
    const empty = new Map();
    const selected = chooseWatch(empty, a, true, false);
    expect(selected.get('a')?.transmission.startedAt).toBe(10);
    expect(pruneWatching(selected, [a]).size).toBe(1);
    expect(pruneWatching(selected, [{ ...a, live: { startedAt: 11 } }]).size).toBe(0);
    expect(chooseWatch(selected, a, false, false).size).toBe(0);
});
it('mobile selection replaces the previous choice; no stale identity survives', () => {
    const selected = chooseWatch(chooseWatch(new Map(), a, true, false), { id: 'b', live: { startedAt: 20 } }, true, true);
    expect([...selected.keys()]).toEqual(['b']);
    expect(pruneWatching(selected, []).size).toBe(0);
});
