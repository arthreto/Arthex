import { getLocale } from '../../core/i18n';
import type { SearchProvider, SearchResult } from '../../core/search/types';
import { webSearchApi } from './api';
import { t } from './i18n';
import { DEFAULT_ENGINE, ENGINES, address, keyword, knownSites, searchUrl, type Engine } from './sites';

export const ICON =
  'M10.5 4a6.5 6.5 0 1 0 0 13a6.5 6.5 0 1 0 0-13M20 20l-4.5-4.5M4 10.5h13M10.5 4c1.6 1.8 2.4 4 2.4 6.5s-.8 4.7-2.4 6.5';

function result(key: string, kind: string, title: string, subtitle: string, url: string): SearchResult {
  return {
    key: `web-search:${key}`,
    group: t('group'),
    kind,
    title,
    subtitle,
    icon: { kind: 'symbol', path: ICON, tint: 'blue' },
    details: [
      { label: t('address'), value: url },
      { label: t('opening'), value: t('browser') },
    ],
    primary: { label: t('open'), run: () => webSearchApi.open(url) },
  };
}

/** A search on an engine. Without a hint, the subtitle shows the engine's site. */
function search(engine: Engine, terms: string, hint?: string): SearchResult {
  const url = searchUrl(engine, terms, getLocale().split('-')[0] ?? 'en');
  const title = t('searchOn', { terms, engine: engine.name });
  return result(`search:${engine.name}`, t('search'), title, hint ?? new URL(url).host, url);
}

/** A typed address or well-known site, then a search on Google. With "yt", "w"…, that engine only. */
export const webSearch: SearchProvider = {
  order: 25,
  search(query) {
    if (query.commandsOnly) return [];
    const chosen = keyword(query.text);
    if (chosen) return [search(chosen.engine, chosen.terms)];
    const typed = address(query.text);
    const sites = typed
      ? [result('address', t('site'), new URL(typed).host, typed, typed)]
      : knownSites(query.text).map((site) =>
          result(`site:${site.name}`, t('site'), site.name, new URL(site.url).host, site.url),
        );
    const fallback = ENGINES[DEFAULT_ENGINE];
    return fallback ? [...sites, search(fallback, query.text, t('hint'))] : sites;
  },
};
