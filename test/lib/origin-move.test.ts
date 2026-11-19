import { describe, expect, it } from 'vitest';
import { importMove, leaveUrl, mergeMove, MOVE_MAX, packMove, readItems, unpackMove, type MoveItems } from '../../src/lib/origin-move';

const OWNER_A = 'a'.repeat(64);
const OWNER_B = 'b'.repeat(64);

/** localStorage de mentira: só o que as funções usam. */
function fakeStorage(initial: Record<string, string> = {}) {
  const map = new Map(Object.entries(initial));
  return {
    get length() {
      return map.size;
    },
    key: (i: number) => [...map.keys()][i] ?? null,
    getItem: (k: string) => map.get(k) ?? null,
    setItem: (k: string, v: string) => void map.set(k, v),
    dump: () => Object.fromEntries(map),
  };
}

const rooms = (...list: { code: string; name: string; lastUsed: number }[]) => JSON.stringify(list);
const room = (code: string, lastUsed: number, name = `Sala ${code}`) => ({ code, name, lastUsed });

function decode(packed: string) {
  const b64 = packed.replace(/-/g, '+').replace(/_/g, '/');
  const bin = atob(b64 + '='.repeat((4 - (b64.length % 4)) % 4));
  return JSON.parse(new TextDecoder().decode(Uint8Array.from(bin, (c) => c.charCodeAt(0))));
}

function encode(value: unknown) {
  const bytes = new TextEncoder().encode(typeof value === 'string' ? value : JSON.stringify(value));
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

describe('readItems (o que sai do endereço antigo)', () => {
  it('pega só as chaves espia:*, com o valor cru', () => {
    const storage = fakeStorage({ 'espia:nickname': 'Rafa', 'espia:golive': '{"fps":60', outra: 'x', 'debug': '1' });
    expect(readItems(storage)).toEqual({ 'espia:nickname': 'Rafa', 'espia:golive': '{"fps":60' });
  });

  it('sem armazenamento devolve pacote vazio', () => {
    expect(readItems(null)).toEqual({});
    const broken = {
      get length(): number {
        throw new Error('bloqueado');
      },
      key: () => null,
      getItem: () => null,
    };
    expect(readItems(broken)).toEqual({});
  });
});

describe('packMove (empacotar)', () => {
  it('monta { v: 1, items } em base64url sem preenchimento', () => {
    const items = { 'espia:nickname': 'Rafa', 'espia:rooms': rooms(room('k7f-q2x', 1762860000000, 'Noite de jogo')), 'espia:owner:k7f-q2x': OWNER_A };
    const packed = packMove(items)!;
    expect(packed).toMatch(/^[A-Za-z0-9_-]+$/);
    expect(decode(packed)).toEqual({ v: 1, items });
  });

  it('é o pacote de exemplo do contrato', () => {
    const items = {
      'espia:nickname': 'Rafa',
      'espia:rooms': '[{"code":"k7f-q2x","name":"Noite de jogo","lastUsed":1762860000000}]',
      'espia:owner:k7f-q2x': OWNER_A,
    };
    expect(packMove(items)).toBe(
      'eyJ2IjoxLCJpdGVtcyI6eyJlc3BpYTpuaWNrbmFtZSI6IlJhZmEiLCJlc3BpYTpyb29tcyI6Ilt7XCJjb2RlXCI6XCJrN2YtcTJ4XCIsXCJuYW1lXCI6XCJOb2l0ZSBkZSBqb2dvXCIsXCJsYXN0VXNlZFwiOjE3NjI4NjAwMDAwMDB9XSIsImVzcGlhOm93bmVyOms3Zi1xMngiOiJhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhIn19',
    );
  });

  it('aguenta acento e emoji', () => {
    const items = { 'espia:nickname': 'João 🎮' };
    expect(unpackMove('#mv=' + packMove(items))).toEqual(items);
  });

  it('sem nada para levar não gera pacote', () => {
    expect(packMove({})).toBeNull();
  });

  it('o teto é 16.384 caracteres', () => {
    expect(MOVE_MAX).toBe(16_384);
  });

  it('acima do teto descarta na ordem: sessões, volumes, golive, appinvite', () => {
    const big = 'x'.repeat(9000);
    const base = {
      'espia:nickname': 'Rafa',
      'espia:rooms': rooms(room('k7f-q2x', 5)),
      'espia:owner:k7f-q2x': OWNER_A,
      'espia:appinvite': 'dismissed',
    };

    // Só as sessões estouram: saem todas, o resto fica.
    const a = decode(packMove({ ...base, 'espia:session:k7f-q2x': big, 'espia:session:abc-def': big, 'espia:volume:ana': '0.5', 'espia:golive': '{}' })!);
    expect(Object.keys(a.items).sort()).toEqual(['espia:appinvite', 'espia:golive', 'espia:nickname', 'espia:owner:k7f-q2x', 'espia:rooms', 'espia:volume:ana']);

    // Volumes também estouram: saem sessões e volumes.
    const b = decode(packMove({ ...base, 'espia:session:k7f-q2x': big, 'espia:volume:ana': big, 'espia:volume:bia': big, 'espia:golive': '{}' })!);
    expect(Object.keys(b.items).sort()).toEqual(['espia:appinvite', 'espia:golive', 'espia:nickname', 'espia:owner:k7f-q2x', 'espia:rooms']);

    // golive estoura sozinho: saem sessões, volumes e golive.
    const c = decode(packMove({ ...base, 'espia:session:k7f-q2x': '{}', 'espia:volume:ana': '0.5', 'espia:golive': big + big })!);
    expect(Object.keys(c.items).sort()).toEqual(['espia:appinvite', 'espia:nickname', 'espia:owner:k7f-q2x', 'espia:rooms']);

    // appinvite estoura sozinho: sobra só o que sempre vai.
    const d = decode(packMove({ ...base, 'espia:appinvite': big + big })!);
    expect(Object.keys(d.items).sort()).toEqual(['espia:nickname', 'espia:owner:k7f-q2x', 'espia:rooms']);
  });

  it('o que cabe no teto vai inteiro e nunca passa dele', () => {
    const items = { 'espia:nickname': 'Rafa', 'espia:session:k7f-q2x': 'x'.repeat(11_000) };
    const packed = packMove(items)!;
    expect(packed.length).toBeLessThanOrEqual(MOVE_MAX);
    expect(decode(packed).items).toEqual(items);
  });

  it('se o que sempre vai não cabe, não há pacote', () => {
    expect(packMove({ 'espia:nickname': 'Rafa', 'espia:rooms': 'x'.repeat(20_000) })).toBeNull();
  });
});

describe('leaveUrl (para onde ir)', () => {
  it('leva caminho, busca e o pacote', () => {
    const storage = fakeStorage({ 'espia:nickname': 'Rafa' });
    const url = leaveUrl({ pathname: '/s/k7f-q2x', search: '?x=1' }, storage);
    expect(url).toBe('https://espia.diuliano.com/s/k7f-q2x?x=1#mv=' + packMove({ 'espia:nickname': 'Rafa' }));
  });

  it('sem armazenamento ou sem dados vai sem #mv=', () => {
    expect(leaveUrl({ pathname: '/', search: '' }, null)).toBe('https://espia.diuliano.com/');
    expect(leaveUrl({ pathname: '/s/k7f-q2x', search: '' }, fakeStorage({ outra: 'x' }))).toBe('https://espia.diuliano.com/s/k7f-q2x');
  });

  it('acima do teto vai sem #mv= e não apaga nada no antigo', () => {
    const storage = fakeStorage({ 'espia:rooms': 'x'.repeat(20_000) });
    const before = storage.dump();
    expect(leaveUrl({ pathname: '/', search: '' }, storage)).toBe('https://espia.diuliano.com/');
    expect(storage.dump()).toEqual(before);
  });
});

describe('unpackMove (desempacotar e validar)', () => {
  const pack = (items: unknown, v: unknown = 1) => '#mv=' + encode({ v, items });

  it('pacote inválido devolve nada, sem erro', () => {
    expect(unpackMove('')).toBeNull();
    expect(unpackMove('#outra=1')).toBeNull();
    expect(unpackMove('#mv=')).toBeNull();
    expect(unpackMove('#mv=%%%')).toBeNull();
    expect(unpackMove('#mv=' + encode('não é json'))).toBeNull();
    expect(unpackMove('#mv=' + encode('[1,2]'))).toBeNull();
    expect(unpackMove('#mv=' + encode('null'))).toBeNull();
    expect(unpackMove(pack({ 'espia:nickname': 'Rafa' }, 2))).toBeNull();
    expect(unpackMove(pack(['espia:nickname']))).toBeNull();
    expect(unpackMove(pack('texto'))).toBeNull();
    expect(unpackMove('#mv=' + 'A'.repeat(MOVE_MAX + 4))).toBeNull();
  });

  it('aceita o pacote do contrato', () => {
    const items = { 'espia:nickname': 'Rafa', 'espia:rooms': rooms(room('k7f-q2x', 1762860000000, 'Noite de jogo')), 'espia:owner:k7f-q2x': OWNER_A };
    expect(unpackMove(pack(items))).toEqual(items);
  });

  it('ignora chave desconhecida e valor que não é texto', () => {
    expect(unpackMove(pack({ outra: 'x', 'espia:desconhecida': 'x', 'espia:nickname': 7, 'espia:golive': '{"fps":60}' }))).toEqual({ 'espia:golive': '{"fps":60}' });
  });

  it('item inválido é ignorado em silêncio, sem derrubar os outros', () => {
    const got = unpackMove(
      pack({
        'espia:nickname': 'x'.repeat(25),
        'espia:owner:k7f-q2x': OWNER_A,
        'espia:owner:abc-def': OWNER_A.toUpperCase(),
        'espia:owner:xyz-234': 'a'.repeat(63),
        'espia:owner:K7F-Q2X': OWNER_A,
        'espia:owner:sala': OWNER_A,
        'espia:session:k7f-q2x': '{"peerId":"p"}',
        'espia:session:nada': '{}',
        'espia:volume:ana': '0.5',
        'espia:volume:bia': '1.5',
        'espia:volume:caio': '-0.1',
        'espia:volume:duda': 'alto',
        'espia:volume:edu': '',
        'espia:appinvite': 'dismissed',
      }),
    );
    expect(got).toEqual({
      'espia:owner:k7f-q2x': OWNER_A,
      'espia:session:k7f-q2x': '{"peerId":"p"}',
      'espia:volume:ana': '0.5',
      'espia:appinvite': 'dismissed',
    });
  });

  it('apelido de até 24 e volume nos limites passam', () => {
    const items = { 'espia:nickname': 'x'.repeat(24), 'espia:volume:ana': '0', 'espia:volume:bia': '1' };
    expect(unpackMove(pack(items))).toEqual(items);
  });

  it('em espia:rooms, tira só as salas inválidas', () => {
    const list = [
      room('k7f-q2x', 10, 'x'.repeat(40)),
      room('abc-def', 9, 'x'.repeat(41)),
      room('sala', 8),
      { code: 'xyz-234', name: 'Sem data' },
      { code: 'xyz-234', name: 7, lastUsed: 6 },
      null,
      room('mnp-789', 5),
    ];
    const got = unpackMove(pack({ 'espia:rooms': JSON.stringify(list) }))!;
    expect(JSON.parse(got['espia:rooms'])).toEqual([room('k7f-q2x', 10, 'x'.repeat(40)), room('mnp-789', 5)]);
  });

  it('espia:rooms que não é lista some', () => {
    expect(unpackMove(pack({ 'espia:rooms': '{"code":"k7f-q2x"}', 'espia:nickname': 'Rafa' }))).toEqual({ 'espia:nickname': 'Rafa' });
    expect(unpackMove(pack({ 'espia:rooms': '[', 'espia:nickname': 'Rafa' }))).toEqual({ 'espia:nickname': 'Rafa' });
  });
});

describe('mergeMove (juntar)', () => {
  it('só acrescenta: o que o oficial já tem fica', () => {
    const current: MoveItems = {
      'espia:nickname': 'Oficial',
      'espia:golive': '{"fps":30}',
      'espia:volume:ana': '0.2',
      'espia:owner:k7f-q2x': OWNER_A,
      'espia:session:k7f-q2x': '{"peerId":"oficial"}',
      'espia:appinvite': 'dismissed',
    };
    const incoming: MoveItems = {
      'espia:nickname': 'Antigo',
      'espia:golive': '{"fps":60}',
      'espia:volume:ana': '0.9',
      'espia:owner:k7f-q2x': OWNER_B,
      'espia:session:k7f-q2x': '{"peerId":"antigo"}',
      'espia:appinvite': 'dismissed',
    };
    expect(mergeMove(current, incoming)).toEqual({});
  });

  it('o que o oficial não tem entra', () => {
    const incoming: MoveItems = {
      'espia:nickname': 'Antigo',
      'espia:golive': '{"fps":60}',
      'espia:volume:ana': '0.9',
      'espia:owner:k7f-q2x': OWNER_B,
      'espia:session:k7f-q2x': '{"peerId":"antigo"}',
      'espia:appinvite': 'dismissed',
    };
    expect(mergeMove({}, incoming)).toEqual(incoming);
    expect(mergeMove({ 'espia:nickname': 'Oficial' }, incoming)).toEqual({ ...incoming, 'espia:nickname': undefined });
  });

  it('salas: união por código, mais recente primeiro', () => {
    const current = { 'espia:rooms': rooms(room('aaa-aaa', 30), room('bbb-bbb', 10)) };
    const incoming = { 'espia:rooms': rooms(room('ccc-ccc', 20), room('aaa-aaa', 5, 'Nome velho')) };
    const out = mergeMove(current, incoming);
    expect(JSON.parse(out['espia:rooms']!)).toEqual([room('aaa-aaa', 30), room('ccc-ccc', 20), room('bbb-bbb', 10)]);
  });

  it('mesmo código dos dois lados: fica a de maior lastUsed, com o nome dela', () => {
    const current = { 'espia:rooms': rooms(room('aaa-aaa', 10, 'Nome do oficial')) };
    const incoming = { 'espia:rooms': rooms(room('aaa-aaa', 20, 'Nome do antigo')) };
    expect(JSON.parse(mergeMove(current, incoming)['espia:rooms']!)).toEqual([room('aaa-aaa', 20, 'Nome do antigo')]);
    // Empate: fica a do oficial.
    const tie = { 'espia:rooms': rooms(room('aaa-aaa', 10, 'Nome do antigo')) };
    expect(mergeMove(current, tie)).toEqual({});
  });

  it('corta em 5 e nunca descarta chave de dono de sala que ficou fora', () => {
    const current = { 'espia:rooms': rooms(room('aaa-aaa', 60), room('bbb-bbb', 50), room('ccc-ccc', 40)) };
    const incoming = {
      'espia:rooms': rooms(room('ddd-ddd', 55), room('eee-eee', 45), room('fff-fff', 1), room('ggg-ggg', 2)),
      'espia:owner:fff-fff': OWNER_A,
      'espia:owner:zzz-zzz': OWNER_B,
    };
    const out = mergeMove(current, incoming);
    expect(JSON.parse(out['espia:rooms']!).map((r: { code: string }) => r.code)).toEqual(['aaa-aaa', 'ddd-ddd', 'bbb-bbb', 'eee-eee', 'ccc-ccc']);
    expect(out['espia:owner:fff-fff']).toBe(OWNER_A);
    expect(out['espia:owner:zzz-zzz']).toBe(OWNER_B);
  });

  it('lista corrompida no oficial não impede a do pacote de entrar', () => {
    const incoming = { 'espia:rooms': rooms(room('aaa-aaa', 10)) };
    expect(JSON.parse(mergeMove({ 'espia:rooms': 'lixo' }, incoming)['espia:rooms']!)).toEqual([room('aaa-aaa', 10)]);
  });

  it('pacote sem salas não mexe na lista do oficial', () => {
    expect(mergeMove({ 'espia:rooms': rooms(room('aaa-aaa', 10)) }, { 'espia:nickname': 'Rafa' })).toEqual({ 'espia:nickname': 'Rafa' });
  });
});

describe('importMove (chegada no oficial)', () => {
  const official = () =>
    fakeStorage({
      'espia:nickname': 'Oficial',
      'espia:rooms': rooms(room('aaa-aaa', 30), room('bbb-bbb', 10)),
      'espia:owner:aaa-aaa': OWNER_A,
      outra: 'intocada',
    });
  const legacy = () =>
    fakeStorage({
      'espia:nickname': 'Antigo',
      'espia:rooms': rooms(room('ccc-ccc', 20), room('aaa-aaa', 5)),
      'espia:owner:ccc-ccc': OWNER_B,
      'espia:volume:ana': '0.4',
    });
  const hashOf = (url: string) => url.slice(url.indexOf('#'));

  it('junta o que veio do antigo e grava', () => {
    const storage = official();
    importMove(hashOf(leaveUrl({ pathname: '/', search: '' }, legacy())), storage);
    expect(storage.dump()).toEqual({
      'espia:nickname': 'Oficial',
      'espia:rooms': rooms(room('aaa-aaa', 30), room('ccc-ccc', 20), room('bbb-bbb', 10)),
      'espia:owner:aaa-aaa': OWNER_A,
      'espia:owner:ccc-ccc': OWNER_B,
      'espia:volume:ana': '0.4',
      outra: 'intocada',
    });
  });

  it('repetir a ida não duplica nem apaga', () => {
    const storage = official();
    const hash = hashOf(leaveUrl({ pathname: '/', search: '' }, legacy()));
    importMove(hash, storage);
    const once = storage.dump();
    importMove(hash, storage);
    importMove(hash, storage);
    expect(storage.dump()).toEqual(once);
  });

  it('sem pacote, com pacote inválido ou sem armazenamento, não faz nada e não lança', () => {
    const storage = official();
    const before = storage.dump();
    importMove('', storage);
    importMove('#mv=lixo', storage);
    expect(storage.dump()).toEqual(before);
    expect(() => importMove('#mv=' + packMove({ 'espia:nickname': 'Rafa' }), null)).not.toThrow();
    const full = { ...fakeStorage(), setItem: () => { throw new Error('cheio'); } };
    expect(() => importMove('#mv=' + packMove({ 'espia:nickname': 'Rafa' }), full)).not.toThrow();
  });
});
