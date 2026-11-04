import type { SearchProvider, SearchResult } from '../../core/search/types';
import { matchesAll } from '../../core/text';
import { appsApi, type App } from './api';
import { t } from './i18n';

const LIMIT = 4;
/** Rust keeps the list fresh; asking again now and then picks up new installs. */
const REFRESH_MS = 60_000;

let cache: { at: number; apps: Promise<App[]> } | null = null;

function installed(): Promise<App[]> {
  if (!cache || Date.now() - cache.at > REFRESH_MS) {
    cache = { at: Date.now(), apps: appsApi.list().catch(() => []) };
  }
  return cache.apps;
}

function toResult(app: App): SearchResult {
  return {
    key: `apps:${app.id}`,
    group: t('group'),
    kind: t('kind'),
    title: app.name,
    subtitle: app.file ? t('subtitle', { file: app.file }) : t('kind'),
    icon: app.icon
      ? { kind: 'image', src: app.icon }
      : { kind: 'letter', text: app.name.charAt(0).toUpperCase() },
    details: [
      ...(app.file ? [{ label: t('executable'), value: app.file }] : []),
      { label: t('tip'), value: t('tipText') },
    ],
    primary: { label: t('launch'), run: () => appsApi.open(app.id) },
  };
}

export const appsSearch: SearchProvider = {
  order: 10,
  async search(query) {
    if (query.commandsOnly) return [];
    const apps = await installed();
    return apps
      .filter((app) => matchesAll(`${app.name} ${app.file ?? ''}`, query.text))
      .slice(0, LIMIT)
      .map(toResult);
  },
};
