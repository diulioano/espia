import { describe, expect, it } from 'vitest';
import { parseClientMessage } from '../../src/worker/validate';
import type { ClientMessage, ServerMessage } from '../../src/shared/protocol';

// No ambiente do Workers não há `fs`: o Vite embute os arquivos como texto.
const arquivos = import.meta.glob('../../app/fixtures/protocol/*.json', { eager: true, query: '?raw', import: 'default' }) as Record<string, string>;

const porNome = new Map<string, string>(Object.entries(arquivos).map(([caminho, texto]) => [caminho.split('/').pop()!.replace(/\.json$/, ''), texto]));

const CLIENTE: ClientMessage['t'][] = ['join', 'resume', 'signal', 'live', 'unlive', 'turn-refresh'];
const SERVIDOR: ServerMessage['t'][] = [
  'welcome', 'live-ack', 'room-not-found', 'room-full', 'resume-failed', 'peer-joined', 'peer-left', 'peer-live',
  'signal', 'ice-servers', 'error', 'room-renamed', 'room-deleted', 'removed', 'pong',
];

/** Campos obrigatórios de cada tipo de `ServerMessage` (src/shared/protocol.ts), além de `t`. */
const OBRIGATORIOS: Record<ServerMessage['t'], string[]> = {
  welcome: ['mediaProtocol', 'mediaPolicy', 'broadcastOrder', 'you', 'peers', 'iceServers', 'iceExpiresAt', 'resumeToken', 'capacity'],
  'live-ack': ['requestId', 'broadcastSeq', 'live'],
  'room-not-found': [],
  'room-full': ['capacity'],
  'resume-failed': [],
  'peer-joined': ['peer'],
  'peer-left': ['id'],
  'peer-live': ['id', 'live'],
  signal: ['from', 'data'],
  'ice-servers': ['iceServers', 'iceExpiresAt'],
  error: ['code', 'message'],
  'room-renamed': ['name'],
  'room-deleted': [],
  removed: [],
  pong: [],
};

const texto = (nome: string): string => {
  const t = porNome.get(nome);
  if (t === undefined) throw new Error(`falta a fixture ${nome}.json`);
  return t;
};

describe('fixtures do protocolo (app/fixtures/protocol)', () => {
  it('existem as 21 fixtures, uma por tipo de mensagem, e nenhuma a mais', () => {
    const esperados = [...CLIENTE.map((t) => `client-${t}`), ...SERVIDOR.map((t) => `server-${t}`)].sort();
    expect(esperados).toHaveLength(21);
    expect([...porNome.keys()].sort()).toEqual(esperados);
  });

  describe('mensagens do cliente passam por parseClientMessage', () => {
    for (const t of CLIENTE) {
      it(`client-${t}`, () => {
        const bruto = texto(`client-${t}`);
        const msg = parseClientMessage(bruto);
        expect(msg).not.toBeNull();
        expect(msg!.t).toBe(t);
        expect(JSON.parse(bruto).t).toBe(t);
      });
    }
  });

  describe('mensagens do servidor têm os campos do tipo ServerMessage', () => {
    for (const t of SERVIDOR) {
      it(`server-${t}`, () => {
        const msg = JSON.parse(texto(`server-${t}`)) as Record<string, unknown>;
        expect(msg.t).toBe(t);
        for (const campo of OBRIGATORIOS[t]) expect(msg, `campo ${campo}`).toHaveProperty(campo);
      });
    }
  });
});
