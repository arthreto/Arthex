import { invoke } from '@tauri-apps/api/core';
import { on } from '../../core/api';

/** Mirrors `Command` in `src-tauri/src/modules/system/mod.rs`. */
export type Command =
  | 'lock'
  | 'sleep'
  | 'restart'
  | 'shutdown'
  | 'signOut'
  | 'emptyRecycleBin'
  | 'taskManager'
  | 'windowsSettings';

export type PlannedAction = 'shutdown' | 'restart' | 'sleep';

export interface Schedule {
  action: PlannedAction;
  /** Milliseconds since the Unix epoch. */
  at: number;
}

export const systemApi = {
  run: (command: Command) => invoke<null>('system_run', { command }),
  recycleBinSize: () => invoke<number>('system_recycle_bin_size'),
  schedule: (schedule: Schedule) => invoke<Schedule>('system_schedule', { schedule }),
  cancelSchedule: () => invoke<null>('system_schedule_cancel'),
  currentSchedule: () => invoke<Schedule | null>('system_schedule_get'),
  onSchedule: (handler: (schedule: Schedule | null) => void) =>
    on<Schedule | null>('system://schedule', handler),
};
