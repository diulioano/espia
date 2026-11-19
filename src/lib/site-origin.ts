/** Endereço oficial do site: é o que vai em todo convite. */
export const OFFICIAL_ORIGIN = 'https://espia.diuliano.com';
/** Endereços antigos que ainda respondem; quem está neles convida pelo oficial. */
export const LEGACY_HOSTS = ['espia.espia.workers.dev'] as const;

const OFFICIAL_HOST = new URL(OFFICIAL_ORIGIN).hostname;

/** Origem que vai no convite. Fora do oficial e do antigo (teste local), fica a da própria página. */
export function inviteOrigin(loc: Pick<Location, 'hostname' | 'origin'>): string {
  const host = loc.hostname;
  if (host === OFFICIAL_HOST || (LEGACY_HOSTS as readonly string[]).includes(host)) return OFFICIAL_ORIGIN;
  return loc.origin;
}
