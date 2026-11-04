import { formatBytes } from '../../core/format';
import type { SearchProvider, SearchResult } from '../../core/search/types';
import { matchesAll } from '../../core/text';
import { toast } from '../../core/toast.svelte';
import { systemApi, type Command } from './api';
import { t } from './i18n';
import { planner } from './planner.svelte';
import SchedulePanel from './SchedulePanel.svelte';
import { clockTime, isTomorrow } from './times';

/** In the order of the mockup; `schedule` is the shutdown planner. */
const ENTRIES: { id: Command | 'schedule'; danger?: boolean }[] = [
  { id: 'lock' },
  { id: 'sleep' },
  { id: 'restart', danger: true },
  { id: 'shutdown', danger: true },
  { id: 'schedule' },
  { id: 'signOut', danger: true },
  { id: 'emptyRecycleBin', danger: true },
  { id: 'taskManager' },
  { id: 'windowsSettings' },
];
/** Outside the `>` prefix, commands only show for a precise enough word, and only a couple. */
const MIN_LENGTH = 3;
const LIMIT = 2;

function scheduleResult(): SearchResult {
  const current = planner.current;
  return {
    key: 'system:schedule',
    group: t('group'),
    kind: t('kind'),
    title: t('schedule.name'),
    subtitle: current
      ? t('schedule.planned', { action: current.action, time: clockTime(current.at) })
      : t('schedule.idle'),
    meta: current ? t('schedule.meta') : undefined,
    icon: { kind: 'command' },
    panel: SchedulePanel,
    primary: {
      label: t('schedule.submit'),
      run: async () => {
        if (planner.current) return;
        const at = await planner.plan();
        const action = planner.draft.action;
        toast.show(
          t('schedule.done', { action, time: clockTime(at), tomorrow: isTomorrow(at) ? 'yes' : 'no' }),
        );
      },
    },
  };
}

async function commandResult(id: Command, danger: boolean): Promise<SearchResult> {
  const name = t(`commands.${id}.name`);
  const size = id === 'emptyRecycleBin' ? formatBytes(await systemApi.recycleBinSize()) : '';
  const hint = id === 'emptyRecycleBin' ? size : t(`commands.${id}.hint`);
  return {
    key: `system:${id}`,
    group: t('group'),
    kind: t('kind'),
    title: name,
    subtitle: danger ? t('subtitleConfirm') : t('subtitle'),
    meta: hint || undefined,
    icon: { kind: 'command', danger },
    details: [
      { label: t('windowsShortcut'), value: id === 'emptyRecycleBin' ? t('none') : hint || t('none') },
      { label: t('confirmation'), value: danger ? t('confirmYes') : t('confirmNo') },
    ],
    confirm: danger ? { text: confirmText(id, size), label: name } : undefined,
    primary: { label: t('run'), run: () => systemApi.run(id) },
  };
}

function confirmText(id: Command, size: string): string {
  switch (id) {
    case 'restart':
      return t('commands.restart.confirm');
    case 'shutdown':
      return t('commands.shutdown.confirm');
    case 'signOut':
      return t('commands.signOut.confirm');
    default:
      return t('commands.emptyRecycleBin.confirm', { size });
  }
}

function searchable(id: Command | 'schedule'): string {
  return id === 'schedule'
    ? `${t('schedule.name')} ${t('schedule.keywords')}`
    : `${t(`commands.${id}.name`)} ${t(`commands.${id}.keywords`)}`;
}

export const systemSearch: SearchProvider = {
  order: 30,
  async search(query) {
    if (!query.commandsOnly && query.normalized.length < MIN_LENGTH) return [];
    await planner.load();
    const matching = ENTRIES.filter((entry) => matchesAll(searchable(entry.id), query.text));
    const shown = query.commandsOnly ? matching : matching.slice(0, LIMIT);
    return Promise.all(
      shown.map((entry) =>
        entry.id === 'schedule' ? scheduleResult() : commandResult(entry.id, entry.danger ?? false),
      ),
    );
  },
};
