import { describe, expect, it } from 'vitest';
import { liveLabel } from '../../src/lib/live-label';

describe('liveLabel (specs/012)', () => {
  it('app, janela: só o nome do programa', () => {
    expect(liveLabel({ kind: 'native-window', app: 'Hollow Knight' })).toBe('Hollow Knight');
    expect(liveLabel({ kind: 'native-window', app: '  ' })).toBe('Janela');
  });

  it('app, tela: "Tela inteira", ou o nome com mais de um monitor', () => {
    expect(liveLabel({ kind: 'native-monitor', title: 'Tela 1', monitors: 1 })).toBe('Tela inteira');
    expect(liveLabel({ kind: 'native-monitor', title: 'Tela 2', monitors: 2 })).toBe('Tela 2');
  });

  it('site: o tipo que o navegador entregou', () => {
    expect(liveLabel({ kind: 'browser', surface: 'monitor' })).toBe('Tela inteira');
    expect(liveLabel({ kind: 'browser', surface: 'window' })).toBe('Janela');
    expect(liveLabel({ kind: 'browser', surface: 'browser' })).toBe('Aba do navegador');
    expect(liveLabel({ kind: 'browser', surface: undefined })).toBeUndefined();
  });

  it('câmera', () => {
    expect(liveLabel({ kind: 'camera', title: 'HD Pro Webcam C920' })).toBe('Câmera · HD Pro Webcam C920');
    expect(liveLabel({ kind: 'camera', title: undefined, facing: 'user' })).toBe('Câmera frontal');
    expect(liveLabel({ kind: 'camera', title: '', facing: 'environment' })).toBe('Câmera traseira');
  });

  it('corta em 60 com reticências', () => {
    const label = liveLabel({ kind: 'native-window', app: 'x'.repeat(80) })!;
    expect([...label]).toHaveLength(60);
    expect(label.endsWith('…')).toBe(true);
  });
});
