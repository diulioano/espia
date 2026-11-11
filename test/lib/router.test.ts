import { describe, expect, it } from 'vitest';
import { parseRoute, sameRoute } from '../../src/lib/route';

describe('rotas (specs/018 R3)', () => {
  it('sala e início', () => {
    expect(parseRoute('/')).toEqual({ name: 'home' });
    expect(parseRoute('/coisa')).toEqual({ name: 'home' });
  });
  it('voltar no mesmo endereço (fechar a tela cheia) é a mesma rota', () => {
    const a = parseRoute('/');
    expect(sameRoute(a, parseRoute('/'))).toBe(true);
    expect(sameRoute({ name: 'room', code: 'k7f-q2x' }, { name: 'room', code: 'k7f-q2x' })).toBe(true);
  });
  it('outro endereço muda a rota', () => {
    expect(sameRoute({ name: 'room', code: 'k7f-q2x' }, { name: 'room', code: 'abc-def' })).toBe(false);
    expect(sameRoute({ name: 'room', code: 'k7f-q2x' }, { name: 'home' })).toBe(false);
    expect(sameRoute({ name: 'home' }, { name: 'room', code: 'k7f-q2x' })).toBe(false);
  });
});
