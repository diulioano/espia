import { describe, expect, it } from 'vitest';
import { header } from '../../src/lib/room-view';
import type { RoomView, StreamView } from '../../src/lib/types';

const lu: StreamView = { id: 'lu', name: 'Lu', tone: 'accent', quality: '1080p · 60 fps · com som', status: 'ok', muted: true, volume: 1 };
const bia: StreamView = { ...lu, id: 'bia', name: 'Bia', tone: 'sage' };

function view(over: Partial<RoomView>): RoomView {
  return { roomId: 'k7f-q2x', status: 'ready', streams: [], focusedId: null, myBroadcast: null, room: { kind: 'quick' }, ...over } as RoomView;
}

const fixed = { kind: 'fixed', name: 'CS da galera', own: false } as const;

describe('header (cabeçalho da sala)', () => {
  it('sala rápida: igual a antes', () => {
    expect(header(view({}))).toEqual({ title: 'Sala da galera', subtitle: 'Ninguém transmitindo agora' });
  });

  it('sala fixa: o nome é o título (specs/011)', () => {
    expect(header(view({ room: fixed }))).toEqual({ title: 'CS da galera', subtitle: 'Ninguém transmitindo agora' });
    expect(header(view({ room: fixed, streams: [lu] })).subtitle).toBe('Lu está ao vivo · 1080p · 60 fps · com som');
    expect(header(view({ room: fixed, streams: [lu, bia] })).subtitle).toBe('2 telas ao vivo · escolha qual assistir');
    expect(header(view({ room: fixed, streams: [lu, bia], focusedId: 'bia' })).subtitle).toBe('Vendo Bia · 2 telas ao vivo');
  });

  it('uma transmissão: o que está sendo transmitido vem antes da qualidade (specs/012)', () => {
    expect(header(view({ streams: [{ ...lu, label: 'Hollow Knight' }] })).subtitle).toBe('Hollow Knight · 1080p · 60 fps · com som');
    expect(header(view({ room: fixed, streams: [{ ...lu, label: 'Hollow Knight' }] })).subtitle).toBe(
      'Lu está ao vivo · Hollow Knight · 1080p · 60 fps · com som',
    );
  });

  it('sala fixa conectando: ainda não sabe o nome', () => {
    expect(header(view({ room: fixed, status: 'connecting' })).subtitle).toBe('Conectando…');
  });
});
