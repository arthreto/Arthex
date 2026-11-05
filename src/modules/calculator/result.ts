import { getLocale } from '../../core/i18n';

/** A result as shown: grouped digits, at most 8 decimals, in the active language. */
export function shown(value: number): string {
  return new Intl.NumberFormat(getLocale(), { maximumFractionDigits: 8 }).format(value);
}

/** A result as copied or typed again: no grouping, so it pastes and evaluates cleanly. */
export function plain(value: number): string {
  return new Intl.NumberFormat(getLocale(), { maximumFractionDigits: 8, useGrouping: false }).format(value);
}
