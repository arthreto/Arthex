import type en from '../locales/en.json';
import { message, register } from './catalog';
import { FALLBACK_LOCALE, getLocale, translate } from './i18n.svelte';
import type { MessageKey, MessageTree, MessageValues } from './types';

export { availableLocales } from './catalog';
export { getLocale, resolveLocale, setLocale } from './i18n.svelte';
export type { LocaleMeta, MessageKey, MessageValues } from './types';

type Catalog = { default: MessageTree };

const shellCatalogs = import.meta.glob<Catalog>('../locales/*.json', { eager: true });
const moduleCatalogs = import.meta.glob<Catalog>('../../modules/*/locales/*.json', { eager: true });

const fileName = (path: string) => path.slice(path.lastIndexOf('/') + 1, -'.json'.length);
const moduleId = (path: string) => path.split('/').at(-3) ?? '';

for (const [path, catalog] of Object.entries(shellCatalogs)) {
  register(fileName(path), '', catalog.default);
}
for (const [path, catalog] of Object.entries(moduleCatalogs)) {
  register(fileName(path), `modules.${moduleId(path)}`, catalog.default);
}

export type ShellKey = MessageKey<typeof en>;

export function t(key: ShellKey, values?: MessageValues): string {
  return translate(key, values);
}

/** A shell message in a given language, whatever the active one (the greetings of the first launch). */
export function textIn(locale: string, key: ShellKey): string {
  return message(locale, key) ?? translate(key);
}

/** Typed translator scoped to a module's own `locales/<lang>.json`. */
export function moduleTranslator<Messages>(id: string) {
  return (key: MessageKey<Messages>, values?: MessageValues): string =>
    translate(`modules.${id}.${key}`, values);
}

/** Name or description of any module, used by the shell (tab bar, hub, settings). */
export function moduleText(id: string, field: 'name' | 'description'): string {
  return translate(`modules.${id}.${field}`);
}

/** The optional `keywords` of a module, which help the search find its tool. */
export function moduleKeywords(id: string): string {
  const key = `modules.${id}.keywords`;
  return message(getLocale(), key) ?? message(FALLBACK_LOCALE, key) ?? '';
}
