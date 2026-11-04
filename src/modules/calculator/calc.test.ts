import { describe, expect, it } from 'vitest';
import { evaluate } from './calc';

describe('evaluate', () => {
  it('follows the usual priorities', () => {
    expect(evaluate('12*8+3')).toBe(99);
    expect(evaluate('2+3*4')).toBe(14);
    expect(evaluate('(2+3)*4')).toBe(20);
    expect(evaluate('2^3^2')).toBe(512);
    expect(evaluate('-2^2')).toBe(-4);
    expect(evaluate('10 % 4')).toBe(2);
  });

  it('accepts commas, symbols, sqrt and pi', () => {
    expect(evaluate('1,5 × 2')).toBe(3);
    expect(evaluate('9 ÷ 2')).toBe(4.5);
    expect(evaluate('sqrt(16) + 1')).toBe(5);
    expect(evaluate('= pi')).toBeCloseTo(Math.PI);
    expect(evaluate('= 42')).toBe(42);
  });

  it('leaves searches and broken input alone', () => {
    expect(evaluate('2024')).toBeNull();
    expect(evaluate('2024 report')).toBeNull();
    expect(evaluate('(1+2')).toBeNull();
    expect(evaluate('1/0')).toBeNull();
    expect(evaluate('alert(1)')).toBeNull();
  });
});
