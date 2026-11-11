import { expect, it } from 'vitest';
import { directIceServers } from '../../src/shared/media';
it('does not retain TURN credentials or URLs from any previous envelope', () => {
    expect(directIceServers([{ urls: ['turn:relay', 'stun:stun.example'], username: 'old', credential: 'old' }, { urls: 'turns:relay' }])).toEqual([{ urls: ['stun:stun.example'] }]);
    expect(directIceServers([{ urls: 'https://invalid' }])).toEqual([]);
});
