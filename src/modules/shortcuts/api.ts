import { invoke } from '@tauri-apps/api/core';
import { on } from '../../core/api';

export type Shortcut = { id: string; name: string; icon?: string } & (
  { kind: 'site'; url: string } | { kind: 'app'; path: string }
);

/** Mirrors `MAX` in `src-tauri/src/modules/shortcuts/mod.rs`: one per Alt + 1…9. */
export const MAX_SHORTCUTS = 9;

export const shortcutsApi = {
  list: () => invoke<Shortcut[]>('shortcuts_list'),
  addSite: (url: string, name: string | null) => invoke<Shortcut[]>('shortcuts_add_site', { url, name }),
  open: (id: string) => invoke<null>('shortcuts_open', { id }),
  onChanged: (handler: (list: Shortcut[]) => void) => on<Shortcut[]>('shortcuts://changed', handler),
};
