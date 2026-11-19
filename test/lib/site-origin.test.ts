import { describe, expect, it } from 'vitest';
import { inviteOrigin, LEGACY_HOSTS, OFFICIAL_ORIGIN } from '../../src/lib/site-origin';

const at = (origin: string) => ({ origin, hostname: new URL(origin).hostname });

describe('inviteOrigin (origem que vai no convite)', () => {
  it('fixa o endereço oficial e o antigo', () => {
    expect(OFFICIAL_ORIGIN).toBe('https://espia.diuliano.com');
    expect(LEGACY_HOSTS).toEqual(['espia.espia.workers.dev']);
  });

  it('no endereço oficial devolve o oficial', () => {
    expect(inviteOrigin(at('https://espia.diuliano.com'))).toBe('https://espia.diuliano.com');
  });

  it('no endereço antigo devolve o oficial', () => {
    expect(inviteOrigin(at('https://espia.espia.workers.dev'))).toBe('https://espia.diuliano.com');
  });

  it('no teste local devolve o endereço da própria página, com a porta (FR-007)', () => {
    expect(inviteOrigin(at('http://localhost:5173'))).toBe('http://localhost:5173');
    expect(inviteOrigin(at('http://127.0.0.1:8787'))).toBe('http://127.0.0.1:8787');
  });

  it('em qualquer outro host devolve o endereço da própria página', () => {
    expect(inviteOrigin(at('https://teste.example'))).toBe('https://teste.example');
    expect(inviteOrigin(at('https://espia.diuliano.com.evil.example'))).toBe('https://espia.diuliano.com.evil.example');
    expect(inviteOrigin(at('https://outro.espia.workers.dev'))).toBe('https://outro.espia.workers.dev');
  });

  it('aceita o próprio location da página', () => {
    const loc: Pick<Location, 'hostname' | 'origin'> = { hostname: 'espia.espia.workers.dev', origin: 'https://espia.espia.workers.dev' };
    expect(inviteOrigin(loc)).toBe(OFFICIAL_ORIGIN);
  });
});
