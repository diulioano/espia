import { describe, expect, it } from 'vitest';
import { CpuGovernor, DOWN_AFTER, UP_AFTER } from '../../src/lib/cpu-governor';

const run = (g: CpuGovernor, n: number, limited: boolean, canGoDown = true) => {
  const out: (string | null)[] = [];
  for (let i = 0; i < n; i++) out.push(g.tick(limited, canGoDown));
  return out;
};

describe('CpuGovernor (specs/010)', () => {
  it('baixa depois de 10 s seguidos no limite, não antes', () => {
    const g = new CpuGovernor();
    expect(run(g, DOWN_AFTER - 1, true).every((x) => x === null)).toBe(true);
    expect(g.tick(true, true)).toBe('down');
    expect(g.steps).toBe(1);
  });

  it('um segundo com folga zera a contagem do limite', () => {
    const g = new CpuGovernor();
    run(g, DOWN_AFTER - 1, true);
    g.tick(false, true);
    expect(run(g, DOWN_AFTER - 1, true).includes('down')).toBe(false);
  });

  it('sobe depois de 60 s seguidos com folga', () => {
    const g = new CpuGovernor();
    run(g, DOWN_AFTER, true);
    expect(run(g, UP_AFTER - 1, false).includes('up')).toBe(false);
    expect(g.tick(false, true)).toBe('up');
    expect(g.steps).toBe(0);
  });

  it('não passa de 2 degraus nem baixa sem degrau disponível', () => {
    const g = new CpuGovernor();
    run(g, DOWN_AFTER * 5, true);
    expect(g.steps).toBe(2);
    const h = new CpuGovernor();
    run(h, DOWN_AFTER * 2, true, false);
    expect(h.steps).toBe(0);
  });

  it('sem degrau baixado, folga não sobe nada', () => {
    const g = new CpuGovernor();
    expect(run(g, UP_AFTER * 2, false).includes('up')).toBe(false);
  });

  it('reset volta ao começo', () => {
    const g = new CpuGovernor();
    run(g, DOWN_AFTER * 2, true);
    g.reset();
    expect(g.steps).toBe(0);
    expect(run(g, DOWN_AFTER - 1, true).includes('down')).toBe(false);
  });
});
