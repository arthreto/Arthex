import { getLocale } from '../../core/i18n';

export function clockTime(time: Date | number): string {
  return new Intl.DateTimeFormat(getLocale(), { hour: '2-digit', minute: '2-digit' }).format(time);
}

export function isTomorrow(time: Date | number, now = new Date()): boolean {
  return new Date(time).getDate() !== now.getDate();
}

const MINUTE = 60_000;

/** The moment a choice points to: in N minutes, or at HH:MM (tomorrow if already past). */
export function targetTime(mode: 'in' | 'at', minutes: number, clock: string, now = new Date()): Date {
  if (mode === 'in') return new Date(now.getTime() + Math.max(1, minutes) * MINUTE);
  const [hours = 0, mins = 0] = clock.split(':').map(Number);
  const target = new Date(now);
  target.setHours(hours, mins, 0, 0);
  if (target <= now) target.setDate(target.getDate() + 1);
  return target;
}

/** The next half hour, at least an hour away: a sensible default for "at a set time". */
export function defaultClock(now = new Date()): string {
  const target = new Date(now.getTime() + 60 * MINUTE);
  target.setMinutes(target.getMinutes() < 30 ? 30 : 60, 0, 0);
  return `${String(target.getHours()).padStart(2, '0')}:${String(target.getMinutes()).padStart(2, '0')}`;
}

/** "45 min", "1 h", "2 h 15". */
export function formatDuration(minutes: number): string {
  if (minutes < 60) return `${minutes} min`;
  const rest = minutes % 60;
  return `${Math.floor(minutes / 60)} h${rest ? ` ${String(rest).padStart(2, '0')}` : ''}`;
}

export function minutesUntil(time: number, now = Date.now()): number {
  return Math.max(1, Math.round((time - now) / MINUTE));
}
