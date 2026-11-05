import { invoke } from '@tauri-apps/api/core';
import { on } from '../../core/api';
import type { Tint } from '../../core/modules';
import { shortAddress } from './site';

export type Target =
  | { kind: 'site'; url: string }
  | { kind: 'app'; path: string }
  | { kind: 'path'; path: string }
  | { kind: 'command'; line: string };

export type Shortcut = {
  id: string;
  name: string;
  icon?: string;
  symbol?: { name: string; tint: Tint };
  /** Typed alone in the search, it puts the shortcut first. */
  keyword?: string;
  onHome: boolean;
  inSearch: boolean;
} & Target;

/** How a shortcut looks: its usual icon, a pictogram or an image the user picked. */
export type Look =
  { type: 'auto' } | { type: 'symbol'; name: string; tint: Tint } | { type: 'image'; icon: string };

/** What the dialog sends; Rust checks it and fills in the rest. */
export type Draft = Target & {
  name: string | null;
  look: Look;
  keyword: string | null;
  onHome: boolean;
  inSearch: boolean;
};

export interface Program {
  name: string;
  path: string;
  /** File name of the program itself (`Code.exe`). */
  file: string;
  icon: string | null;
}

/** Mirrors `MAX_ON_HOME` in `src-tauri/src/modules/shortcuts/mod.rs`: one per Alt + 1…9. */
export const MAX_ON_HOME = 9;

export const shortcutsApi = {
  list: () => invoke<Shortcut[]>('shortcuts_list'),
  /** Adds a shortcut, or edits the one with `id`. */
  save: (id: string | null, draft: Draft) => invoke<Shortcut[]>('shortcuts_save', { id, draft }),
  remove: (id: string) => invoke<Shortcut[]>('shortcuts_remove', { id }),
  restore: (shortcut: Shortcut, index: number) =>
    invoke<Shortcut[]>('shortcuts_restore', { shortcut, index }),
  reorder: (ids: string[]) => invoke<Shortcut[]>('shortcuts_reorder', { ids }),
  programs: () => invoke<Program[]>('shortcuts_programs'),
  browseProgram: () => invoke<Program | null>('shortcuts_browse_program'),
  browseFolder: () => invoke<string | null>('shortcuts_browse_folder'),
  pickIcon: () => invoke<string | null>('shortcuts_pick_icon'),
  open: (id: string) => invoke<null>('shortcuts_open', { id }),
  onChanged: (handler: (list: Shortcut[]) => void) => on<Shortcut[]>('shortcuts://changed', handler),
};

/** What a shortcut opens, as shown under its name. */
export function targetText(shortcut: Target): string {
  switch (shortcut.kind) {
    case 'site':
      return shortAddress(shortcut.url);
    case 'command':
      return shortcut.line;
    default:
      return shortcut.path;
  }
}
