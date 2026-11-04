import type en from '../locales/en.json';
import { register } from './catalog';
import { translate } from './i18n.svelte';
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

/** Typed translator scoped to a module's own `locales/<lang>.json`. */
export function moduleTranslator<Messages>(id: string) {
  return (key: MessageKey<Messages>, values?: MessageValues): string =>
    translate(`modules.${id}.${key}`, values);
}

/** Name or description of any module, used by the shell (tab bar, hub, settings). */
export function moduleText(id: string, field: 'name' | 'description'): string {
  return translate(`modules.${id}.${field}`);
}
