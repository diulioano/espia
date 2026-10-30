// Convite para baixar o app desktop na tela inicial (specs/008, research R1/R2). Só no Windows, fora do app.

/**
 * Sempre o instalador da versão mais nova, servido pelo próprio Worker (sem GitHub): `npm run desktop:release`
 * copia o instalador para public/download/ e o `npm run deploy` publica junto com o site.
 */
export const DOWNLOAD_URL = '/download/Espia-setup.exe';

export interface InviteEnv {
  /** `navigator.userAgentData.platform` (Chrome/Edge); falta no Firefox e no Safari. */
  platform?: string;
  mobile?: boolean;
  ua: string;
  inApp: boolean;
  dismissed: boolean;
}

/** Mostrar o convite? Na dúvida, não. */
export function appInviteVisible(env: InviteEnv): boolean {
  return !env.dismissed && appDownloadVisible(env);
}

/**
 * Botão "Baixar o app" no topo da tela inicial (specs/013): a mesma regra do convite, mas continua lá
 * depois do "Agora não". Só no site, no Windows, fora do celular. Na dúvida, não.
 */
export function appDownloadVisible(env: Omit<InviteEnv, 'dismissed'>): boolean {
  if (env.inApp || env.mobile) return false;
  if (env.platform) return env.platform === 'Windows';
  return /Windows NT/.test(env.ua) && !/Mobile|Android/.test(env.ua);
}

export function readInviteEnv(): Omit<InviteEnv, 'dismissed'> {
  const nav = navigator as Navigator & { userAgentData?: { platform?: string; mobile?: boolean } };
  return {
    platform: nav.userAgentData?.platform || undefined,
    mobile: nav.userAgentData?.mobile,
    ua: nav.userAgent,
    inApp: '__TAURI_INTERNALS__' in window,
  };
}
