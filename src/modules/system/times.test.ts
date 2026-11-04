import { describe, expect, it } from 'vitest';
import { defaultClock, formatDuration, targetTime } from './times';

const now = new Date(2026, 10, 4, 22, 40);

describe('planner times', () => {
  it('counts minutes from now or aims at a clock time, tomorrow when already past', () => {
    expect(targetTime('in', 30, '', now).getTime()).toBe(now.getTime() + 30 * 60_000);
    expect(targetTime('at', 0, '23:30', now).getHours()).toBe(23);
    expect(targetTime('at', 0, '07:00', now).getDate()).toBe(5);
  });

  it('suggests the next half hour at least an hour away', () => {
    expect(defaultClock(now)).toBe('00:00');
    expect(defaultClock(new Date(2026, 10, 4, 9, 10))).toBe('10:30');
  });

  it('writes durations shortly', () => {
    expect(formatDuration(45)).toBe('45 min');
    expect(formatDuration(60)).toBe('1 h');
    expect(formatDuration(135)).toBe('2 h 15');
  });
});
