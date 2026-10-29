import { describe, expect, it } from 'vitest';
import { appInviteVisible, type InviteEnv } from '../../src/lib/app-invite';

const UA = {
  winChrome: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36',
  winFirefox: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:144.0) Gecko/20100101 Firefox/144.0',
  mac: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15',
  linux: 'Mozilla/5.0 (X11; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0',
  android: 'Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Mobile Safari/537.36',
  iphone: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1',
};
const env = (e: Partial<InviteEnv>): InviteEnv => ({ ua: UA.winChrome, inApp: false, dismissed: false, ...e });

describe('convite do app (specs/008)', () => {
  it('Windows no Chrome/Edge (userAgentData) e no Firefox (só UA) → mostra', () => {
    expect(appInviteVisible(env({ platform: 'Windows', mobile: false }))).toBe(true);
    expect(appInviteVisible(env({ ua: UA.winFirefox }))).toBe(true);
  });

  it('Mac, Linux, Android e iPhone → não mostra', () => {
    expect(appInviteVisible(env({ platform: 'macOS', mobile: false, ua: UA.mac }))).toBe(false);
    expect(appInviteVisible(env({ ua: UA.mac }))).toBe(false);
    expect(appInviteVisible(env({ ua: UA.linux }))).toBe(false);
    expect(appInviteVisible(env({ platform: 'Android', mobile: true, ua: UA.android }))).toBe(false);
    expect(appInviteVisible(env({ ua: UA.iphone }))).toBe(false);
  });

  it('userAgentData manda, mesmo com UA de Windows (UA congelado/falso)', () => {
    expect(appInviteVisible(env({ platform: 'Linux' }))).toBe(false);
    expect(appInviteVisible(env({ platform: 'Windows', mobile: true }))).toBe(false);
  });

  it('dentro do app ou depois do "Agora não" → não mostra', () => {
    expect(appInviteVisible(env({ platform: 'Windows', inApp: true }))).toBe(false);
    expect(appInviteVisible(env({ platform: 'Windows', dismissed: true }))).toBe(false);
  });
});
