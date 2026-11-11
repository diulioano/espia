import { describe, expect, it } from 'vitest';
import { header, keepFullscreen, viewerStream } from '../../src/lib/room-view';
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

describe('header com a sua tela no palco (specs/014 US3)', () => {
  const me: StreamView = { ...lu, id: 'eu', name: 'Você', you: true };
  it('só a sua: "Sua tela, ao vivo"', () => {
    expect(header(view({ streams: [me] }))).toEqual({ title: 'Sua tela, ao vivo', subtitle: 'Do jeito que a sala vê · Esc volta' });
  });
  it('com outras, em foco', () => {
    expect(header(view({ streams: [lu, me], focusedId: 'eu' }))).toEqual({ title: 'Sua tela, ao vivo', subtitle: '1 tela da galera · Esc volta' });
  });
  it('sala fixa: o nome continua no título', () => {
    expect(header(view({ streams: [me], room: fixed })).title).toBe('CS da galera');
  });
});

describe('viewerStream (tela cheia do celular, specs/018 R5)', () => {
  const watched = { ...lu, watching: true };
  const base = (over: Partial<RoomView>) => view({ streams: [watched, { ...bia, watching: false }], link: 'ok', fullscreenId: 'lu', ...over });

  it('devolve a transmissão aberta que estou assistindo', () => {
    expect(viewerStream(base({}))?.id).toBe('lu');
  });
  it('fechado, id que não existe ou não assistida: nada', () => {
    expect(viewerStream(base({ fullscreenId: null }))).toBeUndefined();
    expect(viewerStream(base({ fullscreenId: 'tomas' }))).toBeUndefined();
    expect(viewerStream(base({ fullscreenId: 'bia' }))).toBeUndefined();
  });
  it('ligação em failed esconde; reconnecting mantém (FR-008)', () => {
    expect(viewerStream(base({ link: 'failed' }))).toBeUndefined();
    expect(viewerStream(base({ link: 'reconnecting' }))?.id).toBe('lu');
  });
});

describe('keepFullscreen (store, specs/018 FR-007)', () => {
  it('continua enquanto assisto à mesma transmissão', () => {
    expect(keepFullscreen('lu', new Set(['lu']), null)).toBe('lu');
  });
  it('fim, reinício (escolha some) ou parar de assistir fecha', () => {
    expect(keepFullscreen('lu', new Set(), null)).toBeNull();
    expect(keepFullscreen('lu', new Set(['bia']), null)).toBeNull();
  });
  it('picture-in-picture da mesma tela fecha', () => {
    expect(keepFullscreen('lu', new Set(['lu']), 'lu')).toBeNull();
  });
});
