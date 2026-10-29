import { describe, expect, it } from 'vitest';
import { audioConstraints, audioRole, isCaptureCard, previewIds, suggestAudio, videoConstraints } from '../../src/lib/devices';

const dev = (id: string, label: string, groupId = '') => ({ id, label, groupId });

describe('isCaptureCard', () => {
  it('placas conhecidas', () => {
    for (const l of ['Elgato HD60 X', 'Cam Link 4K', 'AVerMedia Live Gamer MINI', 'USB3 Video', 'HDMI Capture']) {
      expect(isCaptureCard(l), l).toBe(true);
    }
  });
  it('webcams e câmeras virtuais não', () => {
    for (const l of ['HD Pro Webcam C920 (046d:082d)', 'OBS Virtual Camera', 'Integrated Camera']) {
      expect(isCaptureCard(l), l).toBe(false);
    }
  });
});

describe('audioRole (voz × jogo)', () => {
  it('microfones são voz', () => {
    expect(audioRole('Microfone (HD Pro Webcam C920)')).toBe('voice');
    expect(audioRole('Headset Microphone (HyperX)')).toBe('voice');
  });
  it('entrada da placa é jogo', () => {
    expect(audioRole('Digital Audio Interface (HD60 X)')).toBe('game');
    expect(audioRole('Line In (Realtek)')).toBe('game');
    expect(audioRole('Qualquer coisa', true)).toBe('game');
  });
  it('na dúvida, voz', () => {
    expect(audioRole('Entrada desconhecida')).toBe('voice');
  });
});

describe('suggestAudio', () => {
  const audios = [
    dev('default', 'Padrão - Microfone (Realtek)', 'g0'),
    dev('a1', 'Microfone (Realtek Audio)', 'g0'),
    dev('a2', 'Digital Audio Interface (HD60 X)', 'g-elgato'),
    dev('a3', 'Microfone (HD Pro Webcam C920)', 'g-c920'),
  ];
  it('mesmo aparelho (groupId) primeiro', () => {
    expect(suggestAudio(dev('v1', 'Elgato HD60 X', 'g-elgato'), audios)?.id).toBe('a2');
  });
  it('sem groupId igual: nome parecido', () => {
    expect(suggestAudio(dev('v2', 'HD Pro Webcam C920 (046d:082d)', 'outro'), audios)?.id).toBe('a3');
  });
  it('nada parecido: nenhuma', () => {
    expect(suggestAudio(dev('v3', 'OBS Virtual Camera', 'g-obs'), audios)).toBeNull();
  });
  it('nunca sugere as entradas genéricas "default"/"communications"', () => {
    expect(suggestAudio(dev('v4', 'Webcam', 'g0'), [dev('default', 'Padrão', 'g0'), dev('communications', 'Comunicações', 'g0')])).toBeNull();
  });
});

describe('constraints', () => {
  it('jogo vai cru, em estéreo', () => {
    expect(audioConstraints('a2', 'game')).toEqual({
      deviceId: { exact: 'a2' },
      echoCancellation: false,
      noiseSuppression: false,
      autoGainControl: false,
      channelCount: { ideal: 2 },
    });
  });
  it('voz com cancelamento de eco, em mono', () => {
    expect(audioConstraints('a1', 'voice')).toMatchObject({ echoCancellation: true, noiseSuppression: true, autoGainControl: true, channelCount: { ideal: 1 } });
  });
  it('vídeo por dispositivo ou por lado (celular)', () => {
    expect(videoConstraints({ id: 'v1' }, 1080, 60)).toEqual({ deviceId: { exact: 'v1' }, width: { ideal: 1920 }, height: { ideal: 1080 }, frameRate: { ideal: 60 } });
    expect(videoConstraints({ facing: 'environment' }, 720, 30)).toMatchObject({ facingMode: { ideal: 'environment' }, height: { ideal: 720 } });
  });
});

describe('previewIds', () => {
  const devices = ['a', 'b', 'c', 'd', 'e', 'f'].map((id) => ({ id, busy: id === 'b' }));
  it('no máximo 4, sem os ocupados', () => {
    expect(previewIds(devices, null)).toEqual(['a', 'c', 'd', 'e']);
  });
  it('a escolhida sempre entra, primeiro', () => {
    expect(previewIds(devices, 'f')).toEqual(['f', 'a', 'c', 'd']);
  });
});
