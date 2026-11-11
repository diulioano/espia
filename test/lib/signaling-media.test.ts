import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { Signaling } from '../../src/lib/signaling';
class Socket {
    static OPEN = 1;
    static instances: Socket[] = [];
    readyState = 1;
    onopen: (() => void) | null = null;
    onmessage: ((event: {
        data: string;
    }) => void) | null = null;
    onclose: ((event: {
        code: number;
    }) => void) | null = null;
    send = vi.fn();
    close = vi.fn();
    constructor() { Socket.instances.push(this); }
}
beforeEach(() => { Socket.instances = []; vi.useFakeTimers(); vi.stubGlobal('WebSocket', Socket); vi.stubGlobal('location', { protocol: 'http:', host: 'test' }); vi.stubGlobal('addEventListener', vi.fn()); vi.stubGlobal('removeEventListener', vi.fn()); });
afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals(); });
const open = () => {
    const onEnd = vi.fn();
    const signal = new Signaling({ code: 'aaa-aaa', nickname: 'Ana', session: null, onMessage() { }, onStatus() { }, onSession() { }, onEnd });
    const socket = Socket.instances[0];
    socket.onopen?.();
    return { signal, socket, onEnd };
};
it('announces v2 and refuses an old welcome', () => {
    const { socket, onEnd } = open();
    expect(JSON.parse(socket.send.mock.calls[0][0])).toMatchObject({ t: 'join', mediaProtocol: 2 });
    socket.onmessage?.({ data: JSON.stringify({ t: 'welcome' }) });
    expect(onEnd).toHaveBeenCalledWith('update');
    vi.runAllTimers();
    expect(Socket.instances).toHaveLength(1);
});
it('4426 is terminal and cannot trigger a reconnect loop', () => {
    const { socket, onEnd } = open();
    socket.onclose?.({ code: 4426 });
    expect(onEnd).toHaveBeenCalledWith('update');
    vi.runAllTimers();
    expect(Socket.instances).toHaveLength(1);
});

it('a queued welcome cannot reopen a terminal connection', () => {
    const { socket, onEnd } = open();
    socket.onclose?.({ code: 4426 });
    expect(() => socket.onmessage?.({ data: JSON.stringify({ t: 'welcome', mediaProtocol: 2, mediaPolicy: { kind: 'direct-only' } }) })).not.toThrow();
    expect(onEnd).toHaveBeenCalledTimes(1);
});
