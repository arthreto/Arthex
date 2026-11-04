import type { SearchProvider } from '../../core/search/types';
import { matchesAll } from '../../core/text';
import { shortcutsApi } from './api';
import { t } from './i18n';
import { shortcuts } from './shortcuts.svelte';
import { shortAddress } from './site';

const LIMIT = 3;

export const shortcutsSearch: SearchProvider = {
  order: 20,
  async search(query) {
    if (query.commandsOnly) return [];
    await shortcuts.load();
    return shortcuts.items
      .flatMap((item) =>
        item.kind === 'site' && matchesAll(`${item.name} ${item.url}`, query.text) ? [item] : [],
      )
      .slice(0, LIMIT)
      .map((site) => ({
        key: `shortcuts:${site.id}`,
        group: t('search.group'),
        kind: t('search.kind'),
        title: site.name,
        subtitle: shortAddress(site.url),
        meta: t('search.meta'),
        icon: site.icon
          ? { kind: 'image', src: site.icon }
          : { kind: 'letter', text: site.name.charAt(0).toUpperCase() },
        details: [
          { label: t('search.address'), value: site.url },
          { label: t('search.opening'), value: t('search.browser') },
        ],
        primary: { label: t('search.open'), run: () => shortcutsApi.open(site.id) },
      }));
  },
};
