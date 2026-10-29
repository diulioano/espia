import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invoke = vi.hoisted(() => vi.fn(() => Promise.resolve()));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

import { installStopHook, openAction, reportLive, reportRoom } from '../../src/lib/desktop';

describe('openAction (link de sala vindo do app desktop)', () => {
  it('vai para a sala quando não está transmitindo', () => {
    expect(openAction('/s/k7f-q2x', '/', null)).toBe('go');
    expect(openAction('/s/k7f-q2x', '/s/abc-def', null)).toBe('go');
  });

  it('pergunta antes de sair se está transmitindo', () => {
    expect(openAction('/s/k7f-q2x', '/s/abc-def', 'Você está transmitindo.')).toBe('confirm');
  });

  it('ignora a mesma sala e caminhos inválidos', () => {
    expect(openAction('/s/k7f-q2x', '/s/k7f-q2x', 'x')).toBe('ignore');
    for (const bad of ['/', '/s/k7f-q2o', '/s/k7f-q2x/x', 'javascript:alert(1)', '/s/../x']) {
      expect(openAction(bad, '/', null)).toBe('ignore');
    }
  });
});

describe('bandeja do app desktop (specs/007)', () => {
  // Os testes rodam no Node: um window de mentira.
  let w: Record<string, unknown>;
  beforeEach(() => {
    w = {};
    vi.stubGlobal('window', w);
  });
  afterEach(() => {
    invoke.mockClear();
    vi.unstubAllGlobals();
  });

  it('reportLive não chama nada fora do app', () => {
    reportLive(true);
    expect(invoke).not.toHaveBeenCalled();
  });

  it('reportLive avisa o app com app_set_live', () => {
    w.__TAURI_INTERNALS__ = {};
    reportLive(true);
    reportLive(false);
    expect(invoke).toHaveBeenNthCalledWith(1, 'app_set_live', { live: true });
    expect(invoke).toHaveBeenNthCalledWith(2, 'app_set_live', { live: false });
  });

  it('reportRoom não chama nada fora do app', () => {
    reportRoom(true);
    expect(invoke).not.toHaveBeenCalled();
  });

  it('reportRoom avisa o app com app_set_room (specs/009)', () => {
    w.__TAURI_INTERNALS__ = {};
    reportRoom(true);
    reportRoom(false);
    expect(invoke).toHaveBeenNthCalledWith(1, 'app_set_room', { inRoom: true });
    expect(invoke).toHaveBeenNthCalledWith(2, 'app_set_room', { inRoom: false });
  });

  it('reportRoom engole erro do app', async () => {
    w.__TAURI_INTERNALS__ = {};
    invoke.mockReturnValueOnce(Promise.reject(new Error('sem comando')));
    expect(() => reportRoom(true)).not.toThrow();
    await Promise.resolve();
  });

  it('espiaStopLive chama a função instalada e some ao remover', () => {
    const stop = vi.fn();
    const remove = installStopHook(stop);
    (w.espiaStopLive as () => void)();
    expect(stop).toHaveBeenCalledOnce();
    remove();
    expect(w.espiaStopLive).toBeUndefined();
  });

  it('remover um gancho antigo não apaga o novo', () => {
    const removeOld = installStopHook(() => {});
    const fresh = () => {};
    installStopHook(fresh);
    removeOld();
    expect(w.espiaStopLive).toBe(fresh);
  });
});
