import type { ResultIcon, SearchProvider, SearchResult } from '../../core/search/types';
import { matchesAll, normalize } from '../../core/text';
import { shortcutsApi, targetText, type Shortcut, type Target } from './api';
import { editor } from './editor.svelte';
import { t } from './i18n';
import { shortcuts } from './shortcuts.svelte';
import { siteProblem } from './site';
import { symbolPath } from './symbols';

const LIMIT = 4;
const PLUS = 'M12 5v14M5 12h14';

function icon(shortcut: Shortcut): ResultIcon {
  if (shortcut.symbol)
    return { kind: 'symbol', path: symbolPath(shortcut.symbol.name), tint: shortcut.symbol.tint };
  if (shortcut.icon) return { kind: 'image', src: shortcut.icon };
  return { kind: 'letter', text: shortcut.name.charAt(0).toUpperCase() };
}

function result(shortcut: Shortcut): SearchResult {
  return {
    key: `shortcuts:${shortcut.id}`,
    group: t('search.group'),
    kind: t(`search.kinds.${shortcut.kind}`),
    title: shortcut.name,
    subtitle: targetText(shortcut),
    meta: shortcut.keyword ? t('search.keyword', { keyword: shortcut.keyword }) : t('search.meta'),
    icon: icon(shortcut),
    details: [{ label: t(`search.details.${shortcut.kind}`), value: targetText(shortcut) }],
    primary: {
      label: t(shortcut.kind === 'command' ? 'search.run' : 'search.open'),
      run: () => shortcutsApi.open(shortcut.id),
    },
  };
}

/** The user's shortcuts: the one whose keyword was typed first, then those that match. */
export const shortcutsSearch: SearchProvider = {
  order: 3,
  async search(query) {
    if (query.commandsOnly) return [];
    await shortcuts.load();
    const listed = shortcuts.items.filter((item) => item.inSearch);
    const chosen = listed.filter((item) => item.keyword && normalize(item.keyword) === query.normalized);
    const others = listed.filter(
      (item) =>
        !chosen.includes(item) &&
        matchesAll(`${item.name} ${targetText(item)} ${item.keyword ?? ''}`, query.text),
    );
    return [...chosen, ...others.slice(0, LIMIT)].map(result);
  },
};

/** What was typed, as the start of a new shortcut: an address gives a site, anything else a command. */
function prefill(text: string): Target & { name?: string } {
  const isAddress = !/\s/.test(text) && text.includes('.') && siteProblem(text) === null;
  return isAddress ? { kind: 'site', url: text } : { kind: 'command', line: '', name: text };
}

/** Last of all results, "Create a shortcut" opens the dialog filled with what was typed. */
export const createShortcutSearch: SearchProvider = {
  order: 90,
  search(query) {
    if (query.commandsOnly || query.text.length < 2) return [];
    return [
      {
        key: 'shortcuts:create',
        group: t('search.createGroup'),
        kind: t('search.createKind'),
        title: t('search.create', { text: query.text }),
        subtitle: t('search.createHint'),
        icon: { kind: 'symbol', path: PLUS, tint: 'blue' },
        primary: {
          label: t('search.createAction'),
          run: async () => {
            // Imported here: the shell's state imports the module registry, which imports this file.
            const [{ search }, { navigation }] = await Promise.all([
              import('../../app/search.svelte'),
              import('../../app/navigation.svelte'),
            ]);
            editor.add(prefill(query.text));
            search.clear();
            navigation.open({ kind: 'settings', section: 'shortcuts' });
          },
        },
      },
    ];
  },
};
