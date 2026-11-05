import { moduleKeywords, moduleText, t } from '../core/i18n';
import { toolModules } from '../core/modules';
import type { SearchProvider, SearchResult } from '../core/search/types';
import { settings } from '../core/settings/settings.svelte';
import { matchesAll } from '../core/text';
import { tools } from './tools.svelte';

/** Finds the tools of every module by name or keyword. Tools rank before applications. */
export const toolSearch: SearchProvider = {
  order: 5,
  search(query) {
    if (query.commandsOnly) return [];
    return toolModules(settings.value).flatMap((module): SearchResult[] => {
      const name = moduleText(module.id, 'name');
      if (!matchesAll(`${name} ${moduleKeywords(module.id)}`, query.text)) return [];
      return [
        {
          key: `tool:${module.id}`,
          group: t('search.tools'),
          kind: t('search.tool'),
          title: name,
          subtitle: moduleText(module.id, 'description'),
          icon: { kind: 'symbol', path: module.icon, tint: module.tint },
          primary: { label: t('search.openTool'), run: () => tools.open(module.id) },
        },
      ];
    });
  },
};
