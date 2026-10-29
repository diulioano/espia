import { describe, expect, it } from 'vitest';
import { parseRoomId } from '../../src/lib/room-id';

describe('parseRoomId (código ou link colado na tela inicial)', () => {
  it('aceita código com e sem hífen', () => {
    expect(parseRoomId('k7f-q2x')).toBe('k7f-q2x');
    expect(parseRoomId(' K7F Q2X ')).toBe('k7f-q2x');
    expect(parseRoomId('k7fq2x')).toBe('k7f-q2x');
  });

  it('aceita o link do site e o link do app desktop', () => {
    expect(parseRoomId('https://espia.espia.workers.dev/s/k7f-q2x')).toBe('k7f-q2x');
    expect(parseRoomId('espia://s/k7f-q2x')).toBe('k7f-q2x');
  });

  it('recusa o que não é sala', () => {
    expect(parseRoomId('espia://qualquer-coisa')).toBeNull();
    expect(parseRoomId('k7f-q2o')).toBeNull();
    expect(parseRoomId('')).toBeNull();
  });
});
