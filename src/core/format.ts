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
