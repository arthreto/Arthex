import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Settings, SettingsPatch } from './settings/types';

export interface UpdateAvailable {
  version: string;
}

export interface WhatsNew {
  version: string;
  /** The release description, with an English and a French section. */
  notes: string;
}

export interface UpdateProgress {
  downloaded: number;
  total: number | null;
}

/** A module installed from the catalog or a file, as `src-tauri/src/services/module_host` lists it. */
export interface InstalledModule {
  id: string;
  version: string;
  /** Where its files are served from, ending with `/`. */
  url: string;
  style: boolean;
  /** Loaded unsigned from `ARTHEX_DEV_MODULES` (development builds). */
  dev: boolean;
}

/** A module of the catalog that runs on this system. */
export interface ModuleOffer {
  id: string;
  version: string;
  icon: string;
  tint: string;
  elements: string[];
  /** Catalog categories; the interface ignores unknown ones. */
  tags: string[];
  /** By language code. */
  texts: Record<string, ModuleTexts>;
  size: number;
  state: 'available' | 'installed' | 'update' | 'outdated';
  /** Where it runs: `any`, or systems such as `windows-x86_64`. */
  platforms: string[];
  contextMenu: {
    id: string;
    labels: Record<string, string>;
    submenu: { id: string; label: string }[];
  }[];
  /** The day it last changed, as `2026-11-05`. */
  updated: string | null;
  hasPreview: boolean;
}

export interface ModuleTexts {
  name: string;
  description: string;
  keywords?: string;
  examples?: string[];
  page?: string;
  tool?: string;
}

/** A `.arthex` file opened from Explorer, checked and waiting for the user's answer. */
export interface InstallRequest {
  id: string;
  version: string;
  icon: string;
  tint: string;
  elements: string[];
  texts: ModuleOffer['texts'];
  size: number;
}

export interface AppInfo {
  version: string;
  systemLocale: string | null;
}

/** Typed wrappers around the Rust commands declared in `src-tauri/build.rs`. */
export const commands = {
  settingsGet: () => invoke<Settings>('settings_get'),
  settingsUpdate: (patch: SettingsPatch) => invoke<Settings>('settings_update', { patch }),
  overlayHide: () => invoke<null>('overlay_hide'),
  /** Resolves to `false` where the interface has to read the keys itself (macOS, Linux). */
  hotkeyRecord: (active: boolean) => invoke<boolean>('hotkey_record', { active }),
  appInfo: () => invoke<AppInfo>('app_info'),
  appOpenInstallDir: () => invoke<null>('app_open_install_dir'),
  appOpenLogDir: () => invoke<null>('app_open_log_dir'),
  appNoteRenderer: (renderer: string) => invoke<null>('app_note_renderer', { renderer }),
  appUninstall: () => invoke<null>('app_uninstall'),
  /** Fails with `unsupported` in builds without an update key. */
  updatesCheck: () => invoke<UpdateAvailable | null>('updates_check'),
  /** Downloads, checks the signature, installs and restarts. */
  updatesInstall: () => invoke<null>('updates_install'),
  updatesWhatsNew: () => invoke<WhatsNew | null>('updates_whats_new'),
  modulesList: () => invoke<InstalledModule[]>('modules_list'),
  /** Reads the signed catalog online. Empty while nothing is published. */
  modulesCatalog: () => invoke<ModuleOffer[]>('modules_catalog'),
  /** The last catalog read, without the network. */
  modulesCatalogKept: () => invoke<ModuleOffer[] | null>('modules_catalog_kept'),
  /** The image of a module of the catalog, as a data address. */
  modulesPreview: (id: string) => invoke<string | null>('modules_preview', { id }),
  /** Installs or updates a module of the catalog. */
  modulesInstall: (id: string) => invoke<InstalledModule>('modules_install', { id }),
  /** Asks for a `.arthex` file, then installs it. `null` when the user cancels. */
  modulesInstallFile: () => invoke<InstalledModule | null>('modules_install_file'),
  modulesUninstall: (id: string, deleteData: boolean) =>
    invoke<null>('modules_uninstall', { id, deleteData }),
  /** Relays a call to the program of an installed module, starting it if needed. */
  modulesCall: (id: string, method: string, params: unknown) =>
    invoke<unknown>('modules_call', { id, method, params: params ?? null }),
  /** Fails once when the opened file cannot be installed. */
  modulesOpened: () => invoke<InstallRequest | null>('modules_opened'),
  modulesCallHidden: (id: string, method: string, params: unknown) =>
    invoke<unknown>('modules_call_hidden', { id, method, params: params ?? null }),
  modulesHideOverlay: () => invoke<null>('modules_hide_overlay'),
  modulesShowOverlay: () => invoke<null>('modules_show_overlay'),
  modulesAnswerOpened: (install: boolean) =>
    invoke<InstalledModule | null>('modules_answer_opened', { install }),
};

/** Event names shared with `src-tauri/src/app/events.rs`. */
export const events = {
  overlayShown: 'overlay://shown',
  overlayHiding: 'overlay://hiding',
  settingsChanged: 'settings://changed',
  navigate: 'app://navigate',
  hotkeyHeld: 'hotkey://held',
  hotkeyRecorded: 'hotkey://recorded',
  updateAvailable: 'updates://available',
  updateProgress: 'updates://progress',
  modulesChanged: 'modules://changed',
  modulesOpened: 'modules://opened',
} as const;

export function on<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (e) => handler(e.payload));
}

/** Rust errors arrive as `{ kind, message }`. */
export function errorMessage(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error) return String(error.message);
  return String(error);
}
