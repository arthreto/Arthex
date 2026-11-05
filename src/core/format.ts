import { getLocale, t } from './i18n';

const MINUTE = 60_000;
const DAY = 24 * 60 * MINUTE;

/** "2 min ago", "3 hr ago", "yesterday, 22:14", then "Oct 12", in the active language. */
export function formatWhen(time: number, now = Date.now(), locale = getLocale()): string {
  const relative = new Intl.RelativeTimeFormat(locale, { numeric: 'auto', style: 'short' });
  const minutes = Math.max(0, Math.round((now - time) / MINUTE));
  if (minutes < 60) return relative.format(-minutes, 'minute');

  const startOfToday = new Date(now).setHours(0, 0, 0, 0);
  if (time >= startOfToday) return relative.format(-Math.round(minutes / 60), 'hour');

  const date = new Date(time);
  if (time >= startOfToday - DAY) {
    const clock = new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit' }).format(date);
    return t('time.dayAt', { day: relative.format(-1, 'day'), time: clock });
  }
  return new Intl.DateTimeFormat(locale, { day: 'numeric', month: 'short' }).format(date);
}

const UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte'] as const;

/** "1.2 GB" / "1,2 Go", counted in 1024 steps like File Explorer. */
export function formatBytes(bytes: number, locale = getLocale()): string {
  let value = Math.max(0, bytes);
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return new Intl.NumberFormat(locale, {
    style: 'unit',
    unit: UNITS[unit],
    unitDisplay: 'short',
    maximumFractionDigits: unit < 2 ? 0 : 1,
  }).format(value);
}

/** "4,383,486" / "4 383 486". */
export function formatCount(count: number, locale = getLocale()): string {
  return new Intl.NumberFormat(locale).format(count);
}

/** "12.3%" / "12,3 %", for a share between 0 and 1. */
export function formatPercent(share: number, digits = 1, locale = getLocale()): string {
  return new Intl.NumberFormat(locale, {
    style: 'percent',
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  }).format(share);
}
