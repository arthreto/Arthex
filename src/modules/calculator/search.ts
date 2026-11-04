import { getLocale } from '../../core/i18n';
import type { SearchProvider } from '../../core/search/types';
import { toast } from '../../core/toast.svelte';
import { evaluate } from './calc';
import { t } from './i18n';

export const calculatorSearch: SearchProvider = {
  order: 0,
  exclusive: true,
  search(query) {
    if (query.commandsOnly) return [];
    const value = evaluate(query.text);
    if (value === null) return [];
    const locale = getLocale();
    const shown = new Intl.NumberFormat(locale, { maximumFractionDigits: 8 }).format(value);
    const copied = new Intl.NumberFormat(locale, { maximumFractionDigits: 8, useGrouping: false }).format(
      value,
    );
    return [
      {
        key: 'calculator',
        group: t('group'),
        kind: t('group'),
        title: `= ${shown}`,
        subtitle: t('subtitle', { expression: query.text }),
        icon: { kind: 'calc' },
        details: [
          { label: t('expression'), value: query.text },
          { label: t('tip'), value: t('tipText') },
        ],
        primary: {
          label: t('copy'),
          run: async () => {
            await navigator.clipboard.writeText(copied);
            toast.show(t('copied', { value: shown }));
          },
        },
      },
    ];
  },
};
