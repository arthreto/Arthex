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
  /** Adds a site, or edits the shortcut with `id`. */
  saveSite: (id: string | null, url: string, name: string | null, icon: string | null) =>
    invoke<Shortcut[]>('shortcuts_save_site', { id, url, name, icon }),
  /** Adds a program, or edits the shortcut with `id`. */
  saveApp: (id: string | null, path: string, name: string | null) =>
    invoke<Shortcut[]>('shortcuts_save_app', { id, path, name }),
  remove: (id: string) => invoke<Shortcut[]>('shortcuts_remove', { id }),
  restore: (shortcut: Shortcut, index: number) =>
    invoke<Shortcut[]>('shortcuts_restore', { shortcut, index }),
  reorder: (ids: string[]) => invoke<Shortcut[]>('shortcuts_reorder', { ids }),
  programs: () => invoke<Program[]>('shortcuts_programs'),
  browseProgram: () => invoke<Program | null>('shortcuts_browse_program'),
  pickIcon: () => invoke<string | null>('shortcuts_pick_icon'),
  open: (id: string) => invoke<null>('shortcuts_open', { id }),
  onChanged: (handler: (list: Shortcut[]) => void) => on<Shortcut[]>('shortcuts://changed', handler),
};
