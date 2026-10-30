// Cenas mockadas da sala não encontrada (specs/011, N1–N4). `&saved=1` = estava em Suas salas.
import type { NotFoundView } from '../lib/types';

export const NOTFOUND_SCENES = [
  { id: 'quick', code: 'N4', label: 'Sala rápida que acabou' },
  { id: 'deleted', code: 'N1', label: 'Apagada pelo dono' },
  { id: 'kicked', code: 'N2', label: 'Apagada com você dentro' },
  { id: 'expired', code: 'N3', label: 'Expirada (30 dias)' },
  { id: 'mobile', code: 'N·M', label: 'Apagada, no celular', device: 'mobile' },
] as const;

export type NotFoundScene = (typeof NOTFOUND_SCENES)[number]['id'];

export function buildNotFoundScene(scene: NotFoundScene, saved: boolean): NotFoundView {
  const reason = scene === 'mobile' ? 'deleted' : scene;
  return { reason, name: reason === 'quick' ? null : 'CS da galera', saved: saved && reason !== 'quick' };
}
