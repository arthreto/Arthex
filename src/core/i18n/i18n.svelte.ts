import { formatter, hasLocale, message } from './catalog';
import type { MessageValues } from './types';

export const FALLBACK_LOCALE = 'en';

let current = $state(FALLBACK_LOCALE);

/** Picks the first supported locale among the candidates, matching on the language part (`fr-CA` → `fr`). */
export function resolveLocale(...candidates: (string | null | undefined)[]): string {
  for (const candidate of candidates) {
    if (!candidate) continue;
    if (hasLocale(candidate)) return candidate;
    const language = candidate.toLowerCase().split(/[-_]/)[0] ?? '';
    if (hasLocale(language)) return language;
  }
  return FALLBACK_LOCALE;
}

export function setLocale(locale: string): void {
  current = resolveLocale(locale);
  if (typeof document !== 'undefined') document.documentElement.lang = current;
}

export function getLocale(): string {
  return current;
}

export function translate(key: string, values?: MessageValues): string {
  const locale = current;
  const own = message(locale, key);
  const source = own ?? message(FALLBACK_LOCALE, key);
  if (source === undefined) {
    if (import.meta.env.DEV) console.warn(`Missing translation: ${key}`);
    return key;
  }
  if (!values && !source.includes('{')) return source;
  return String(formatter(own === undefined ? FALLBACK_LOCALE : locale, key, source).format(values));
}
