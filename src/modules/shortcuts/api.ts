import { invoke } from '@tauri-apps/api/core';
import { on } from '../../core/api';

export type Shortcut = { id: string; name: string; icon?: string } & (
  { kind: 'site'; url: string } | { kind: 'app'; path: string }
);

export interface Program {
  name: string;
  path: string;
  /** File name of the program itself (`Code.exe`). */
  file: string;
  icon: string | null;
}

/** Mirrors `MAX` in `src-tauri/src/modules/shortcuts/mod.rs`: one per Alt + 1…9. */
export const MAX_SHORTCUTS = 9;

export const shortcutsApi = {
  list: () => invoke<Shortcut[]>('shortcuts_list'),
  addSite: (url: string, name: string | null, icon: string | null) =>
    invoke<Shortcut[]>('shortcuts_add_site', { url, name, icon }),
  addApp: (path: string, name: string | null) => invoke<Shortcut[]>('shortcuts_add_app', { path, name }),
  programs: () => invoke<Program[]>('shortcuts_programs'),
  browseProgram: () => invoke<Program | null>('shortcuts_browse_program'),
  pickIcon: () => invoke<string | null>('shortcuts_pick_icon'),
  open: (id: string) => invoke<null>('shortcuts_open', { id }),
  onChanged: (handler: (list: Shortcut[]) => void) => on<Shortcut[]>('shortcuts://changed', handler),
};
