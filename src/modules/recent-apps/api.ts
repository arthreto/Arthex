import { invoke } from '@tauri-apps/api/core';

export interface RecentApp {
  path: string;
  name: string;
  /** Milliseconds since the Unix epoch. */
  lastUsed: number;
  running: boolean;
  icon: string | null;
}

export const recentAppsApi = {
  list: () => invoke<RecentApp[]>('recent_apps_list'),
  open: (path: string) => invoke<null>('recent_apps_open', { path }),
  clear: () => invoke<null>('recent_apps_clear'),
};
