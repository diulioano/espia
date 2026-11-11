import { describe, expect, it } from 'vitest';
import { isPhoneLayout } from '../../src/lib/layout';

const touch = { coarse: true, hover: false };
const mouse = { coarse: false, hover: true };

describe('isPhoneLayout (specs/018 R1)', () => {
  it('celular em pé e deitado', () => {
    expect(isPhoneLayout({ width: 390, height: 844, ...touch })).toBe(true);
    expect(isPhoneLayout({ width: 844, height: 390, ...touch })).toBe(true);
    expect(isPhoneLayout({ width: 932, height: 430, ...touch })).toBe(true);
  });
  it('tablet deitado e notebook baixo ficam no computador', () => {
    expect(isPhoneLayout({ width: 1024, height: 768, ...touch })).toBe(false);
    expect(isPhoneLayout({ width: 1280, height: 500, ...mouse })).toBe(false);
  });
  it('janela estreita no computador continua celular, como antes', () => {
    expect(isPhoneLayout({ width: 600, height: 900, ...mouse })).toBe(true);
  });
});
