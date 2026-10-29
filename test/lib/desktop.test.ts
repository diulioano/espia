import { describe, expect, it } from 'vitest';
import { openAction } from '../../src/lib/desktop';

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
