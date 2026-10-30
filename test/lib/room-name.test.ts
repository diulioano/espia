import { describe, expect, it } from 'vitest';
import { normalizeRoomName } from '../../src/shared/room-name';

describe('normalizeRoomName (specs/011 R10)', () => {
  it('tira espaços das pontas e junta os repetidos', () => {
    expect(normalizeRoomName('  CS   da galera ')).toBe('CS da galera');
  });

  it('tab e quebra de linha viram espaço', () => {
    expect(normalizeRoomName('CS\tda\ngalera')).toBe('CS da galera');
  });

  it('recusa vazio', () => {
    expect(normalizeRoomName('')).toBeNull();
    expect(normalizeRoomName('   ')).toBeNull();
  });

  it('conta caracteres, não unidades de UTF-16: 40 emojis passam, 41 não', () => {
    expect(normalizeRoomName('\u{1F3AE}'.repeat(40))).toBe('\u{1F3AE}'.repeat(40));
    expect(normalizeRoomName('\u{1F3AE}'.repeat(41))).toBeNull();
    expect(normalizeRoomName('a'.repeat(41))).toBeNull();
  });

  it('recusa caracteres invisíveis e de controle', () => {
    expect(normalizeRoomName('a‮b')).toBeNull();
    expect(normalizeRoomName('a​b')).toBeNull();
    expect(normalizeRoomName('a\u0007b')).toBeNull();
  });

  it('aceita emoji composto (ZWJ)', () => {
    const family = '\u{1F468}‍\u{1F469}‍\u{1F467}';
    expect(normalizeRoomName(`Família ${family}`)).toBe(`Família ${family}`);
  });
});
