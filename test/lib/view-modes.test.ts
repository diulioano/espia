import { describe, expect, it } from 'vitest';
import { pruneUnwatched, viewModes, type ViewModesInput } from '../../src/lib/room-view';

const base: ViewModesInput = {
  streams: [{ id: 'lu' }, { id: 'bia' }],
  layout: 'grid',
  focusedId: null,
  mobile: false,
  unwatched: new Set(),
  pageVisible: true,
  pipId: null,
  selfId: null,
};
const modes = (over: Partial<ViewModesInput>) => viewModes({ ...base, ...over });

describe('viewModes (specs/014 R5)', () => {
  it('grade: todo mundo cheio, como antes', () => {
    expect(modes({})).toEqual({ lu: 'full', bia: 'full' });
  });

  it('foco: focada cheia, outras miniatura', () => {
    expect(modes({ layout: 'focus', focusedId: 'bia' })).toEqual({ lu: 'thumb', bia: 'full' });
  });

  it('celular: só a selecionada (ou a primeira)', () => {
    expect(modes({ mobile: true })).toEqual({ lu: 'full', bia: 'hidden' });
    expect(modes({ mobile: true, focusedId: 'bia' })).toEqual({ lu: 'hidden', bia: 'full' });
  });

  it('parou de assistir: escondida, mesmo focada', () => {
    expect(modes({ unwatched: new Set(['lu']) })).toEqual({ lu: 'hidden', bia: 'full' });
    expect(modes({ layout: 'focus', focusedId: 'lu', unwatched: new Set(['lu']) })).toEqual({ lu: 'hidden', bia: 'thumb' });
  });

  it('página escondida: tudo escondido, menos o picture-in-picture', () => {
    expect(modes({ pageVisible: false })).toEqual({ lu: 'hidden', bia: 'hidden' });
    expect(modes({ pageVisible: false, pipId: 'bia' })).toEqual({ lu: 'hidden', bia: 'full' });
  });

  it('não assistida ganha do picture-in-picture', () => {
    expect(modes({ pageVisible: false, pipId: 'bia', unwatched: new Set(['bia']) })).toEqual({ lu: 'hidden', bia: 'hidden' });
  });

  it('celular com a tela apagada: tudo escondido', () => {
    expect(modes({ mobile: true, pageVisible: false })).toEqual({ lu: 'hidden', bia: 'hidden' });
  });

  it('a minha própria tela no palco nunca entra no mapa', () => {
    expect(modes({ streams: [{ id: 'lu' }, { id: 'eu' }], layout: 'focus', focusedId: 'eu', selfId: 'eu' })).toEqual({ lu: 'thumb' });
  });
});

describe('pruneUnwatched (specs/014 R4)', () => {
  const unwatched = new Map([['lu', 100]]);

  it('mesma transmissão: continua sem assistir (inclusive depois de uma queda)', () => {
    expect(pruneUnwatched(unwatched, [{ id: 'lu', live: { startedAt: 100 } }])).toEqual(new Map([['lu', 100]]));
  });

  it('parou, recomeçou ou saiu: volta a assistir', () => {
    expect(pruneUnwatched(unwatched, [{ id: 'lu', live: null }]).size).toBe(0);
    expect(pruneUnwatched(unwatched, [{ id: 'lu', live: { startedAt: 200 } }]).size).toBe(0);
    expect(pruneUnwatched(unwatched, []).size).toBe(0);
  });
});
