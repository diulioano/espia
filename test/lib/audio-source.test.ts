import { describe, expect, it } from 'vitest';
import { audioLabelText, decideAudio, nativeAudioText } from '../../src/lib/audio-source';

// Uma linha da tabela de specs/002-audio-sem-discord/data-model.md = um caso.
describe('decideAudio', () => {
  const cases = [
    ['desligado, sem faixa', false, 'window', null, { kind: 'none', drop: false, warn: false }],
    ['desligado, veio faixa', false, 'browser', 'Tab audio', { kind: 'none', drop: true, warn: false }],
    ['aba com som', true, 'browser', 'Tab audio', { kind: 'tab', drop: false, warn: false }],
    ['janela com som da janela', true, 'window', 'Window audio', { kind: 'window', drop: false, warn: false }],
    ['janela com som do sistema', true, 'window', 'System Audio', { kind: 'none', drop: true, warn: true }],
    ['tela inteira, veio som', true, 'monitor', 'System Audio', { kind: 'none', drop: true, warn: true }],
    ['tela inteira, sem som', true, 'monitor', null, { kind: 'none', drop: false, warn: true }],
    ['aba sem faixa', true, 'browser', null, { kind: 'none', drop: false, warn: false }],
    ['janela sem faixa', true, 'window', null, { kind: 'none', drop: false, warn: false }],
    ['superfície desconhecida com faixa', true, undefined, 'Audio', { kind: 'none', drop: true, warn: false }],
  ] as const;

  for (const [name, wanted, surface, audioLabel, expected] of cases) {
    it(name, () => expect(decideAudio({ wanted, surface, audioLabel })).toEqual(expected));
  }
});

describe('audioLabelText', () => {
  it('textos do card', () => {
    expect(audioLabelText('tab')).toBe('com som da aba');
    expect(audioLabelText('window')).toBe('com som da janela');
    expect(audioLabelText('none')).toBe('sem som');
    expect(audioLabelText('none', 'monitor')).toBe('sem som (tela inteira)');
    expect(audioLabelText('pc')).toBe('com som do PC (sem Discord)');
    expect(audioLabelText('device')).toBe('com som do dispositivo');
  });
});

describe('nativeAudioText (seletor do app, specs/005)', () => {
  it('janela escolhida com som cita o título', () => {
    expect(nativeAudioText({ surface: 'window', audio: true, selectedTitle: 'Hollow Knight' })).toBe(
      'Só o som de Hollow Knight — nada de notificação vazando.',
    );
  });
  it('janela ainda não escolhida', () => {
    expect(nativeAudioText({ surface: 'window', audio: true })).toBe(
      'Só o som da janela que você escolher — Discord e notificações ficam de fora.',
    );
  });
  it('tela inteira com som: PC menos o Discord', () => {
    expect(nativeAudioText({ surface: 'monitor', audio: true, selectedTitle: 'Tela 1' })).toBe(
      'Todo o som do computador vai junto — menos o Discord.',
    );
  });
  it('som desligado', () => {
    expect(nativeAudioText({ surface: 'monitor', audio: false })).toBe('Só a imagem vai pra sala.');
    expect(nativeAudioText({ surface: 'window', audio: false, selectedTitle: 'X' })).toBe('Só a imagem vai pra sala.');
  });
});
