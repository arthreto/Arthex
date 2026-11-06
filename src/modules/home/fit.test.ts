import { describe, expect, it } from 'vitest';
import { fitScale } from './fit';

describe('fitScale', () => {
  it('leaves alone what fits', () => {
    expect(fitScale(600, 600)).toBe(1);
    expect(fitScale(600, 400)).toBe(1);
  });

  it('scales down just enough when it goes a little over, never past the room', () => {
    const scale = fitScale(600, 640);
    expect(scale).toBeLessThan(1);
    expect(scale).toBeGreaterThan(0.93);
    expect(640 * scale).toBeLessThanOrEqual(600);
  });

  it('keeps the normal size when it goes too far over, to let it scroll', () => {
    expect(fitScale(600, 800)).toBe(1);
  });

  it('keeps the normal size before the column has any height', () => {
    expect(fitScale(0, 400)).toBe(1);
  });
});
