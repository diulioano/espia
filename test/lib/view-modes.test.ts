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

  it('celular: só a selecionada (ou a primeira); as outras sem vídeo nem som', () => {
    expect(modes({ mobile: true })).toEqual({ lu: 'full', bia: 'off' });
    expect(modes({ mobile: true, focusedId: 'bia' })).toEqual({ lu: 'off', bia: 'full' });
  });

  it('parou de assistir: sem vídeo e sem som (off), mesmo focada', () => {
    expect(modes({ unwatched: new Set(['lu']) })).toEqual({ lu: 'off', bia: 'full' });
    expect(modes({ layout: 'focus', focusedId: 'lu', unwatched: new Set(['lu']) })).toEqual({ lu: 'off', bia: 'thumb' });
  });

  it('página escondida: sem vídeo, som continua (hidden), menos o picture-in-picture', () => {
    expect(modes({ pageVisible: false })).toEqual({ lu: 'hidden', bia: 'hidden' });
    expect(modes({ pageVisible: false, pipId: 'bia' })).toEqual({ lu: 'hidden', bia: 'full' });
  });

  it('não assistida ganha do picture-in-picture', () => {
    expect(modes({ pageVisible: false, pipId: 'bia', unwatched: new Set(['bia']) })).toEqual({ lu: 'hidden', bia: 'off' });
  });

  it('celular com a tela apagada: a selecionada segue com som, as outras sem nada', () => {
    expect(modes({ mobile: true, pageVisible: false })).toEqual({ lu: 'hidden', bia: 'off' });
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

describe('mensagem de como estou vendo (specs/014)', async () => {
  const { encodeView, decodeView } = await import('../../src/lib/peer');
  const { wantsSound, wantsVideo, sendParams } = await import('../../src/lib/quality');

  it('off vai como hidden + sound:false (versão antiga pausa vídeo e som); volta como off', () => {
    expect(encodeView('off')).toEqual({ t: 'view', mode: 'hidden', sound: false });
    expect(decodeView(encodeView('off'))).toBe('off');
    for (const m of ['full', 'thumb', 'hidden'] as const) expect(decodeView(encodeView(m))).toBe(m);
  });

  it('mensagem estranha: ignorada', () => {
    expect(decodeView({ t: 'view', mode: 'off' })).toBeNull();
    expect(decodeView({ t: 'outra', mode: 'full' })).toBeNull();
    expect(decodeView(null)).toBeNull();
  });

  it('segundo plano: sem vídeo, com som · parou de assistir: sem os dois', () => {
    expect([wantsVideo('hidden'), wantsSound('hidden')]).toEqual([false, true]);
    expect([wantsVideo('off'), wantsSound('off')]).toEqual([false, false]);
    expect([wantsVideo('full'), wantsSound('full')]).toEqual([true, true]);
    const q = { resolution: 1080 as const, fps: 60 as const, maxBitrate: 6_000_000 };
    expect(sendParams(q, 'off').active).toBe(false);
  });
});
