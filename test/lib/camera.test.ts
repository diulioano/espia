import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { listDevices, requestCamera } from '../../src/lib/camera';

// Navegador de mentira: como o Chromium/WebView2, só mostra o nome de um tipo de dispositivo depois que a
// permissão dele foi dada, e getUserMedia dá a permissão do que pediu.
function fakeBrowser(granted: { camera: boolean; microphone: boolean }, opts: { noMic?: boolean } = {}) {
  const devices = [
    { kind: 'videoinput', deviceId: 'v1', groupId: 'g1', label: 'HD Pro Webcam C920', perm: 'camera' as const },
    ...(opts.noMic ? [] : [{ kind: 'audioinput', deviceId: 'a1', groupId: 'g1', label: 'Microfone (C920)', perm: 'microphone' as const }]),
  ];
  const track = () => ({ stop: vi.fn() });
  const mediaDevices = {
    enumerateDevices: async () => devices.map(({ perm, ...d }) => ({ ...d, label: granted[perm] ? d.label : '' })),
    getUserMedia: vi.fn(async (c: { video?: boolean; audio?: boolean }) => {
      if (c.audio && opts.noMic) throw new DOMException('no mic', 'NotFoundError');
      if (c.video) granted.camera = true;
      if (c.audio) granted.microphone = true;
      return { getTracks: () => [track()] };
    }),
  };
  vi.stubGlobal('navigator', { mediaDevices });
  return mediaDevices;
}

describe('câmera com som (bug: câmera ia sem som)', () => {
  beforeEach(() => vi.spyOn(console, 'warn').mockImplementation(() => {}));
  afterEach(() => vi.unstubAllGlobals());

  it('câmera permitida mas microfone não: a lista pede a permissão de novo', async () => {
    fakeBrowser({ camera: true, microphone: false });
    const r = await listDevices();
    expect(r.videos).toHaveLength(1);
    expect(r.audios).toHaveLength(0);
    expect(r.unnamed).toBe(true);
  });

  it('"Permitir câmeras" pede câmera e microfone, e as entradas de som aparecem', async () => {
    const md = fakeBrowser({ camera: false, microphone: false });
    expect(await requestCamera()).toEqual({ permission: 'granted' });
    expect(md.getUserMedia).toHaveBeenCalledWith({ video: true, audio: true });
    const r = await listDevices();
    expect(r.audios.map((a) => a.label)).toEqual(['Microfone (C920)']);
    expect(r.unnamed).toBe(false);
  });

  it('sem microfone no PC: cai para só a câmera', async () => {
    const md = fakeBrowser({ camera: false, microphone: false }, { noMic: true });
    expect(await requestCamera()).toEqual({ permission: 'granted' });
    expect(md.getUserMedia).toHaveBeenLastCalledWith({ video: true });
    expect((await listDevices()).videos).toHaveLength(1);
  });
});
