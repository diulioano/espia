// Validação de cada mensagem do cliente (security.md A4/A5). Escrita à mão: são poucos tipos.
import { AUDIO_KINDS, LIVE_SOURCES, NICKNAME_MAX, type AudioKind, type ClientMessage, type LiveSource, type SignalData } from '../shared/protocol';

type Obj = Record<string, unknown>;

const isObj = (v: unknown): v is Obj => typeof v === 'object' && v !== null && !Array.isArray(v);
const hasOnly = (o: Obj, keys: string[]) => Object.keys(o).every((k) => keys.includes(k));

/** Controles, marcas de direção (U+200E/F, U+202A–202E, U+2066–2069) e espaços de largura zero. */
const UNSAFE = /[\p{Cc}​-‏‪-‮⁦-⁩﻿]/gu;

/** Normaliza o apelido; devolve null se ficar vazio ou passar do limite. */
export function cleanNickname(raw: unknown): string | null {
  if (typeof raw !== 'string' || raw.length > NICKNAME_MAX * 4) return null;
  const s = raw.normalize('NFC').replace(UNSAFE, '').replace(/\s+/g, ' ').trim();
  const len = [...s].length;
  return len >= 1 && len <= NICKNAME_MAX ? s : null;
}

const SDP_TYPES = ['offer', 'answer', 'pranswer', 'rollback'];

function signalData(v: unknown): SignalData | null {
  if (!isObj(v)) return null;
  if ('description' in v && hasOnly(v, ['description'])) {
    const d = v.description;
    if (!isObj(d) || !hasOnly(d, ['type', 'sdp'])) return null;
    if (typeof d.type !== 'string' || !SDP_TYPES.includes(d.type)) return null;
    if (d.sdp !== undefined && typeof d.sdp !== 'string') return null;
    return { description: { type: d.type, sdp: d.sdp as string | undefined } };
  }
  if ('candidate' in v && hasOnly(v, ['candidate'])) {
    const c = v.candidate;
    if (c === null) return { candidate: null };
    return isObj(c) ? { candidate: c } : null;
  }
  return null;
}

const ID_RE = /^[0-9a-f]{16}$/;
const TOKEN_RE = /^[0-9a-f]{64}$/;

/** Devolve a mensagem tipada ou null se for inválida de qualquer jeito. */
export function parseClientMessage(raw: string): ClientMessage | null {
  let m: unknown;
  try {
    m = JSON.parse(raw);
  } catch {
    return null;
  }
  if (!isObj(m) || typeof m.t !== 'string') return null;

  switch (m.t) {
    case 'join': {
      if (!hasOnly(m, ['t', 'nickname'])) return null;
      const nickname = cleanNickname(m.nickname);
      return nickname ? { t: 'join', nickname } : null;
    }
    case 'resume':
      if (!hasOnly(m, ['t', 'peerId', 'resumeToken'])) return null;
      if (typeof m.peerId !== 'string' || !ID_RE.test(m.peerId)) return null;
      if (typeof m.resumeToken !== 'string' || !TOKEN_RE.test(m.resumeToken)) return null;
      return { t: 'resume', peerId: m.peerId, resumeToken: m.resumeToken };
    case 'signal': {
      if (!hasOnly(m, ['t', 'to', 'data'])) return null;
      if (typeof m.to !== 'string' || !ID_RE.test(m.to)) return null;
      const data = signalData(m.data);
      return data ? { t: 'signal', to: m.to, data } : null;
    }
    case 'live': {
      if (!hasOnly(m, ['t', 'resolution', 'fps', 'audio', 'source'])) return null;
      if (m.resolution !== 720 && m.resolution !== 1080) return null;
      if (m.fps !== 30 && m.fps !== 60) return null;
      // boolean: abas abertas antes da 002 (compatibilidade temporária, contracts/signaling-delta.md).
      if (typeof m.audio === 'boolean') return { t: 'live', resolution: m.resolution, fps: m.fps, audio: m.audio ? 'tab' : 'none', source: 'screen' };
      if (!AUDIO_KINDS.includes(m.audio as AudioKind)) return null;
      // Sem source: cliente de antes da 006 → tela (specs/006 contracts/signaling-live.md).
      const source = m.source === undefined ? 'screen' : m.source;
      if (!LIVE_SOURCES.includes(source as LiveSource)) return null;
      return { t: 'live', resolution: m.resolution, fps: m.fps, audio: m.audio as AudioKind, source: source as LiveSource };
    }
    case 'unlive':
    case 'turn-refresh':
      return hasOnly(m, ['t']) ? { t: m.t } : null;
    default:
      return null;
  }
}
