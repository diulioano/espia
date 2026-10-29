// Ajuste automático por CPU de quem transmite (specs/010, data-model.md). Função pura: quem chama lê o
// `qualityLimitationReason` do motor uma vez por segundo e aplica os degraus em `effectiveQuality`. Sem aviso:
// o indicador de qualidade ajustada que já existe mostra a mudança.

/** Segundos seguidos no limite para baixar um degrau. */
export const DOWN_AFTER = 10;
/** Segundos seguidos com folga para subir um degrau. */
export const UP_AFTER = 60;
/** 1080p60 → 1080p30 → 720p30. */
export const MAX_STEPS = 2;

export class CpuGovernor {
  steps = 0;
  private limitedFor = 0;
  private freeFor = 0;

  /** Um segundo a mais. `canGoDown`: ainda há degrau abaixo da qualidade em vigor. */
  tick(limited: boolean, canGoDown: boolean): 'down' | 'up' | null {
    if (limited) {
      this.freeFor = 0;
      this.limitedFor++;
      if (this.limitedFor >= DOWN_AFTER && canGoDown && this.steps < MAX_STEPS) {
        this.steps++;
        this.limitedFor = 0;
        return 'down';
      }
      return null;
    }
    this.limitedFor = 0;
    this.freeFor++;
    if (this.freeFor >= UP_AFTER && this.steps > 0) {
      this.steps--;
      this.freeFor = 0;
      return 'up';
    }
    return null;
  }

  reset() {
    this.steps = this.limitedFor = this.freeFor = 0;
  }
}
