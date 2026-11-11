import { describe, expect, it } from 'vitest';
import { viewModes, type ViewModesInput } from '../../src/lib/room-view';

const base: ViewModesInput = {
  streams: [{ id: 'lu' }, { id: 'bia' }],
  layout: 'grid',
  focusedId: null,
  mobile: false,
  watching: new Set(['lu','bia']),
  pageVisible: true,
  pipId: null,
  selfId: null,
};
const modes = (over: Partial<ViewModesInput>) => viewModes({ ...base, ...over });

describe('viewModes (specs/014 R5)', () => {
  it('sem escolha não recebe mídia nem em destaque',()=>{ expect(modes({watching:new Set(),focusedId:'lu'})).toEqual({lu:'off',bia:'off'}); });

  it('grade: somente escolhas explícitas', () => {
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
    expect(modes({ watching: new Set(['bia']) })).toEqual({ lu: 'off', bia: 'full' });
    expect(modes({ layout: 'focus', focusedId: 'lu', watching: new Set(['bia']) })).toEqual({ lu: 'off', bia: 'thumb' });
  });

  it('página escondida: sem vídeo, som continua (hidden), menos o picture-in-picture', () => {
    expect(modes({ pageVisible: false })).toEqual({ lu: 'hidden', bia: 'hidden' });
    expect(modes({ pageVisible: false, pipId: 'bia' })).toEqual({ lu: 'hidden', bia: 'full' });
  });

  it('não assistida ganha do picture-in-picture', () => {
    expect(modes({ pageVisible: false, pipId: 'bia', watching: new Set(['lu']) })).toEqual({ lu: 'hidden', bia: 'off' });
  });

  it('celular com a tela apagada: a selecionada segue com som, as outras sem nada', () => {
    expect(modes({ mobile: true, pageVisible: false })).toEqual({ lu: 'hidden', bia: 'off' });
  });

  it('a minha própria tela no palco nunca entra no mapa', () => {
    expect(modes({ streams: [{ id: 'lu' }, { id: 'eu' }], layout: 'focus', focusedId: 'eu', selfId: 'eu' })).toEqual({ lu: 'thumb' });
  });
});

describe('mensagem de como estou vendo (specs/014)', async () => {
  const { encodeView, decodeView } = await import('../../src/lib/peer');
  const { wantsSound, wantsVideo, sendParams } = await import('../../src/lib/quality');

  it('v2 identifica a transmissão e a revisão; não aceita v1', () => {
    const key={peerId:'0123456789abcdef',startedAt:100};
    for(const mode of ['full','thumb','hidden','off'] as const) expect(decodeView(encodeView(mode,key,1))).toMatchObject({mode,transmission:key,version:2});
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

  // specs/018 FR-005: o visualizador em tela cheia é só apresentação. viewModes nem recebe fullscreenId,
  // então abrir, girar ou aproximar nunca muda o que chega: no celular a aberta já é a 'full'.
  it('tela cheia no celular: a mesma recepção de antes', () => {
    expect(modes({ mobile: true, focusedId: 'lu' })).toEqual({ lu: 'full', bia: 'off' });
    expect(Object.keys(base)).not.toContain('fullscreenId');
  });
});
