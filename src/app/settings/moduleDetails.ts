import type { ModuleOffer } from '../../core/api';

/** A language code as people read it, in the language of the interface. */
export function languageName(code: string, locale: string): string {
  const name = new Intl.DisplayNames([locale], { type: 'language' }).of(code) ?? code;
  return name.charAt(0).toLocaleUpperCase(locale) + name.slice(1);
}

const SYSTEMS: Record<string, string> = { windows: 'Windows', macos: 'macOS', linux: 'Linux' };

/** `null` for a module that runs everywhere. */
export function systemNames(platforms: readonly string[]): string | null {
  if (platforms.includes('any')) return null;
  const names = platforms.map((platform) => {
    const system = platform.split('-')[0] ?? platform;
    return SYSTEMS[system] ?? system;
  });
  return [...new Set(names)].join(', ');
}

/** `2026-11-05` written out, or the text as it came when it is not a date. */
export function dayName(day: string, locale: string): string {
  const date = new Date(`${day}T12:00:00`);
  return Number.isNaN(date.getTime()) ? day : date.toLocaleDateString(locale, { dateStyle: 'long' });
}

/** Each search keyword once: a module may repeat a word, and the list is keyed by it. */
export const keywordList = (keywords: string): string[] => [
  ...new Set(keywords.split(/\s+/).filter(Boolean)),
];

const labelOf = (labels: Record<string, string>, locale: string) =>
  labels[locale] ?? labels['en'] ?? Object.values(labels)[0] ?? '';

/** The right-click entries, one per label: a label used for several file types shows once. */
export function rightClickEntries(
  actions: ModuleOffer['contextMenu'],
  locale: string,
): { label: string; choices: string[] }[] {
  const entries = new Map<string, Set<string>>();
  for (const action of actions) {
    const label = labelOf(action.labels, locale);
    const choices = entries.get(label) ?? new Set<string>();
    for (const choice of action.submenu) choices.add(choice.label);
    entries.set(label, choices);
  }
  return [...entries].map(([label, choices]) => ({ label, choices: [...choices] }));
}
