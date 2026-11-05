import { t as shellT } from '../../core/i18n';
import { isEnabled } from '../../core/modules';
import type { SearchProvider, SearchResult } from '../../core/search/types';
import { settings } from '../../core/settings/settings.svelte';
import { toast } from '../../core/toast.svelte';
import { tools } from '../../core/tools.svelte';
import { evaluate } from './calc';
import { t } from './i18n';
import { ICON, ID, TINT } from './identity';
import { plain, shown } from './result';

function openInCalculator(expression: string): SearchResult {
  return {
    key: 'calculator:tool',
    group: shellT('search.tools'),
    kind: shellT('search.tool'),
    title: t('name'),
    subtitle: t('openWith', { expression }),
    icon: { kind: 'symbol', path: ICON, tint: TINT },
    primary: { label: shellT('search.openTool'), run: () => tools.open(ID, expression) },
  };
}

export const calculatorSearch: SearchProvider = {
  order: 0,
  exclusive: true,
  search(query) {
    if (query.commandsOnly) return [];
    const value = evaluate(query.text);
    if (value === null) return [];
    const result: SearchResult = {
      key: 'calculator',
      group: t('group'),
      kind: t('group'),
      title: `= ${shown(value)}`,
      subtitle: t('subtitle', { expression: query.text }),
      icon: { kind: 'calc' },
      details: [
        { label: t('expression'), value: query.text },
        { label: t('tip'), value: t('tipText') },
      ],
      primary: {
        label: t('copy'),
        run: async () => {
          await navigator.clipboard.writeText(plain(value));
          toast.show(t('copied', { value: shown(value) }));
        },
      },
    };
    return isEnabled(settings.value, ID, 'tool') ? [result, openInCalculator(query.text)] : [result];
  },
};
