import { invoke } from '@tauri-apps/api/core';

export interface App {
  id: string;
  name: string;
  /** The program file (`Code.exe`), absent for Store apps. */
  file: string | null;
  icon: string | null;
}

export const appsApi = {
  list: () => invoke<App[]>('apps_list'),
  open: (id: string) => invoke<null>('apps_open', { id }),
};
