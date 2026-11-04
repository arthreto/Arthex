import { describe, expect, it } from 'vitest';
import { formatBytes, formatPercent, formatWhen } from './format';

const now = new Date(2026, 10, 4, 15, 0).getTime();
const minutesAgo = (minutes: number) => now - minutes * 60_000;
/** Intl uses narrow no-break spaces in some languages. */
const plain = (text: string) => text.replace(/\s/g, ' ');

describe('formatWhen', () => {
  it('counts minutes, then hours, within today', () => {
    expect(formatWhen(minutesAgo(2), now, 'en')).toBe('2 min. ago');
    expect(formatWhen(minutesAgo(180), now, 'en')).toBe('3 hr. ago');
    expect(plain(formatWhen(minutesAgo(2), now, 'fr'))).toBe('il y a 2 min');
  });

  it('names yesterday with its time, then gives the date', () => {
    expect(plain(formatWhen(new Date(2026, 10, 3, 22, 14).getTime(), now, 'en'))).toBe('yesterday, 10:14 PM');
    expect(plain(formatWhen(new Date(2026, 10, 3, 22, 14).getTime(), now, 'fr'))).toBe('hier, 22:14');
    expect(formatWhen(new Date(2026, 9, 12, 9, 0).getTime(), now, 'en')).toBe('Oct 12');
  });
});

describe('formatBytes', () => {
  it('picks the unit in 1024 steps, in the given language', () => {
    expect(plain(formatBytes(512, 'en'))).toBe('512 byte');
    expect(plain(formatBytes(1.2 * 1024 ** 3, 'en'))).toBe('1.2 GB');
    expect(plain(formatBytes(1.2 * 1024 ** 3, 'fr'))).toBe('1,2 Go');
  });
});

describe('formatPercent', () => {
  it('follows the language', () => {
    expect(formatPercent(0.1234, 1, 'en')).toBe('12.3%');
    expect(formatPercent(0.1234, 0, 'fr').replace(/\s/g, ' ')).toBe('12 %');
  });
});
