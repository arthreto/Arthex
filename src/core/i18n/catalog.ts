import { IntlMessageFormat } from 'intl-messageformat';
import type { LocaleMeta, MessageTree } from './types';

const catalogs = new Map<string, Map<string, string>>();
const metas = new Map<string, LocaleMeta>();
const formatters = new Map<string, IntlMessageFormat>();

function flatten(tree: MessageTree, prefix: string, into: Map<string, string>): void {
  for (const [key, value] of Object.entries(tree)) {
    if (key.startsWith('$')) continue;
    const path = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'string') into.set(path, value);
    else flatten(value, path, into);
  }
}

/** Adds messages for a locale under a namespace (`''` for the shell, `modules.<id>` for a module). */
export function register(locale: string, namespace: string, messages: MessageTree): void {
  let catalog = catalogs.get(locale);
  if (!catalog) {
    catalog = new Map();
    catalogs.set(locale, catalog);
  }
  flatten(messages, namespace, catalog);
  const meta = messages['$meta'];
  if (meta && typeof meta !== 'string') metas.set(locale, meta as unknown as LocaleMeta);
}

export function hasLocale(locale: string): boolean {
  return catalogs.has(locale);
}

export function availableLocales(): LocaleMeta[] {
  return [...metas.values()].sort((a, b) => a.nativeName.localeCompare(b.nativeName));
}

export function message(locale: string, key: string): string | undefined {
  return catalogs.get(locale)?.get(key);
}

export function formatter(locale: string, key: string, source: string): IntlMessageFormat {
  const cacheKey = `${locale}\u0000${key}`;
  let cached = formatters.get(cacheKey);
  if (!cached) {
    cached = new IntlMessageFormat(source, locale);
    formatters.set(cacheKey, cached);
  }
  return cached;
}
