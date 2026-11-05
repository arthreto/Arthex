import { formatBytes, formatWhen } from '../../core/format';
import type { SearchProvider, SearchResult } from '../../core/search/types';
import { fileSearchApi, type FileHit } from './api';
import { t } from './i18n';
import IndexingStatus from './IndexingStatus.svelte';

/** "C:\Users\Léa" rather than "C:\Users\Léa\", but a drive keeps its "C:\". */
function folderOf(hit: FileHit): string {
  return hit.location.length > 3 ? hit.location.replace(/[\\/]$/, '') : hit.location;
}

function toResult(hit: FileHit): SearchResult {
  const size = hit.isDir ? null : formatBytes(hit.size);
  const modified = formatWhen(hit.modified * 1000);
  return {
    key: `file-search:${hit.path}`,
    group: t('search.group'),
    kind: t(hit.isDir ? 'search.folder' : 'search.file'),
    title: hit.name,
    subtitle: size ? `${folderOf(hit)} · ${size}` : folderOf(hit),
    meta: modified,
    icon: { kind: hit.isDir ? 'folder' : 'file' },
    details: [
      { label: t('search.location'), value: hit.location },
      ...(size ? [{ label: t('search.size'), value: size }] : []),
      { label: t('search.modified'), value: modified },
    ],
    primary: { label: t('search.open'), run: () => fileSearchApi.open(hit.path) },
    secondary: { label: t('search.openFolder'), run: () => fileSearchApi.reveal(hit.path) },
    path: hit.path,
  };
}

export const fileSearch: SearchProvider = {
  order: 40,
  status: IndexingStatus,
  async search(query) {
    if (query.commandsOnly) return [];
    return (await fileSearchApi.query(query.text)).map(toResult);
  },
};
