// O que o aparelho deixa fazer com a mídia (specs/018 R8).

let volumeCap: boolean | undefined;

/**
 * O volume de um <audio> pode ser mudado pela página? No iPhone não: fica sempre em 1 e só os botões
 * do aparelho mexem. Um teste só por página.
 */
export function canSetVolume(): boolean {
  if (volumeCap !== undefined) return volumeCap;
  if (typeof document === 'undefined') return true;
  try {
    const a = document.createElement('audio');
    a.volume = 0.5;
    volumeCap = a.volume === 0.5;
  } catch {
    volumeCap = false;
  }
  return volumeCap;
}
