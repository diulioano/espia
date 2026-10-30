import { describe, expect, it } from 'vitest';
import {
  buildSavedRooms,
  forgetRoom,
  recentLabel,
  rememberRoom,
  renameSaved,
  roomTone,
  savedRoomSub,
  type SavedRoom,
} from '../../src/lib/saved-rooms';
import type { RoomStatus } from '../../src/shared/protocol';

const DAY = 24 * 60 * 60_000;
const NOW = new Date(2026, 9, 30, 15, 0).getTime();
const room = (code: string, name: string, daysAgo: number): SavedRoom => ({ code, name, lastUsed: NOW - daysAgo * DAY });

describe('rememberRoom (limite de 5, pedido de 2026-10-30)', () => {
  it('a sala nova vai pro topo e a 6ª tira a usada há mais tempo', () => {
    let list: SavedRoom[] = [];
    ['aaa-aaa', 'bbb-bbb', 'ccc-ccc', 'ddd-ddd', 'eee-eee'].forEach((c, i) => (list = rememberRoom(list, { code: c, name: c }, NOW - (10 - i) * DAY)));
    list = rememberRoom(list, { code: 'fff-fff', name: 'Nova' }, NOW);
    expect(list.map((r) => r.code)).toEqual(['fff-fff', 'eee-eee', 'ddd-ddd', 'ccc-ccc', 'bbb-bbb']);
  });

  it('entrar de novo não duplica, só atualiza nome e uso', () => {
    let list = [room('aaa-aaa', 'Velho', 3), room('bbb-bbb', 'B', 1)];
    list = rememberRoom(list, { code: 'aaa-aaa', name: 'Novo' }, NOW);
    expect(list).toEqual([{ code: 'aaa-aaa', name: 'Novo', lastUsed: NOW }, room('bbb-bbb', 'B', 1)]);
  });

  it('tirar e renomear', () => {
    const list = [room('aaa-aaa', 'A', 1), room('bbb-bbb', 'B', 2)];
    expect(forgetRoom(list, 'aaa-aaa').map((r) => r.code)).toEqual(['bbb-bbb']);
    expect(renameSaved(list, 'bbb-bbb', 'B2')[1].name).toBe('B2');
  });
});

describe('textos', () => {
  it('recentLabel', () => {
    expect(recentLabel(NOW - 60_000, NOW)).toBe('usada hoje');
    expect(recentLabel(NOW - DAY, NOW)).toBe('usada ontem');
    expect(recentLabel(NOW - 3 * DAY, NOW)).toBe('há 3 dias');
    expect(recentLabel(NOW - 8 * DAY, NOW)).toBe('há 1 semana');
    expect(recentLabel(NOW - 15 * DAY, NOW)).toBe('há 2 semanas');
    expect(recentLabel(NOW - 40 * DAY, NOW)).toBe('há mais de um mês');
  });

  it('savedRoomSub', () => {
    expect(savedRoomSub({ people: 3, streaming: 1, recent: 'usada ontem' })).toBe('3 pessoas agora · 1 transmitindo');
    expect(savedRoomSub({ people: 1, streaming: 0, recent: 'usada ontem' })).toBe('1 pessoa agora');
    expect(savedRoomSub({ people: 0, streaming: 0, recent: 'usada ontem' })).toBe('Vazia · usada ontem');
  });

  it('roomTone é estável', () => {
    expect(roomTone('k7f-q2x')).toBe(roomTone('k7f-q2x'));
  });
});

describe('buildSavedRooms (painel)', () => {
  const list = [room('aaa-aaa', 'Filme de sexta', 1), room('bbb-bbb', 'CS da galera', 2), room('ccc-ccc', 'Trampo do TCC', 3)];
  const ok = (code: string, people: number, streaming = 0, name: string | null = null): RoomStatus => ({ code, state: 'ok', name, people, streaming });

  it('lista vazia → null (mostra a ilustração)', () => {
    expect(buildSavedRooms([], null, () => false, NOW)).toBeNull();
  });

  it('sem números ainda: todas carregando, na ordem de uso', () => {
    const v = buildSavedRooms(list, null, () => false, NOW)!;
    expect(v.groups[0].rooms.map((r) => [r.code, r.state])).toEqual([
      ['aaa-aaa', 'loading'],
      ['bbb-bbb', 'loading'],
      ['ccc-ccc', 'loading'],
    ]);
  });

  it('com gente primeiro (mais gente antes), depois as mais recentes; "Sua" só pro dono', () => {
    const v = buildSavedRooms(list, [ok('aaa-aaa', 0), ok('bbb-bbb', 3, 1), ok('ccc-ccc', 2)], (c) => c === 'bbb-bbb', NOW)!;
    expect(v.note).toBe('neste aparelho');
    expect(v.groups).toHaveLength(1);
    expect(v.groups[0].rooms.map((r) => r.code)).toEqual(['bbb-bbb', 'ccc-ccc', 'aaa-aaa']);
    expect(v.groups[0].rooms.map((r) => r.own)).toEqual([true, false, false]);
  });

  it('nome novo do servidor substitui o guardado', () => {
    const v = buildSavedRooms(list, [ok('aaa-aaa', 0, 0, 'Filme de sábado')], () => false, NOW)!;
    expect(v.groups[0].rooms.find((r) => r.code === 'aaa-aaa')!.name).toBe('Filme de sábado');
  });

  it('apagada ou inexistente → gone, sem "Sua"', () => {
    const v = buildSavedRooms(list, [{ code: 'aaa-aaa', state: 'gone', reason: 'deleted' }, { code: 'bbb-bbb', state: 'notfound' }], () => true, NOW)!;
    const gone = v.groups[0].rooms.filter((r) => r.state === 'gone');
    expect(gone.map((r) => [r.code, r.own])).toEqual([
      ['aaa-aaa', false],
      ['bbb-bbb', false],
    ]);
  });

  it('4+ salas com gente e vazias → dois grupos e a contagem', () => {
    const five = [...list, room('ddd-ddd', 'Revisão', 4), room('eee-eee', 'Lu joga', 5)];
    const v = buildSavedRooms(five, [ok('aaa-aaa', 0), ok('bbb-bbb', 4, 2), ok('ccc-ccc', 2), ok('ddd-ddd', 0), ok('eee-eee', 3, 1)], () => false, NOW)!;
    expect(v.note).toBe('5 salas neste aparelho');
    expect(v.groups.map((g) => [g.label, g.rooms.map((r) => r.code)])).toEqual([
      ['Com gente agora', ['bbb-bbb', 'eee-eee', 'ccc-ccc']],
      ['Usadas recentemente', ['aaa-aaa', 'ddd-ddd']],
    ]);
  });

  it('5 salas todas vazias → um grupo só', () => {
    const five = [...list, room('ddd-ddd', 'D', 4), room('eee-eee', 'E', 5)];
    const v = buildSavedRooms(five, five.map((r) => ok(r.code, 0)), () => false, NOW)!;
    expect(v.groups).toHaveLength(1);
    expect(v.groups[0].label).toBeNull();
  });
});
