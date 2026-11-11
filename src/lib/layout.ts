// Quando a sala usa a versão de celular (specs/018 R1). Celular deitado passa de 640 px de largura,
// então além da largura vale "toque sem hover e pouca altura" (tablet deitado tem 700 px ou mais).

export const PHONE_LAYOUT_QUERY = '(max-width: 639px), (pointer: coarse) and (hover: none) and (max-height: 540px)';

/** A mesma regra do PHONE_LAYOUT_QUERY, em código (testes). */
export function isPhoneLayout(i: { width: number; height: number; coarse: boolean; hover: boolean }): boolean {
  return i.width <= 639 || (i.coarse && !i.hover && i.height <= 540);
}
