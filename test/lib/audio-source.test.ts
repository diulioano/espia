import { describe, expect, it } from 'vitest';
import { audioLabelText, decideAudio } from '../../src/lib/audio-source';

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
  });
});
