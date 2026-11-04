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
  appOpenDataDir: () => invoke<null>('app_open_data_dir'),
  appUninstall: () => invoke<null>('app_uninstall'),
  /** Fails with `unsupported` in builds without an update key. */
  updatesCheck: () => invoke<UpdateAvailable | null>('updates_check'),
  /** Downloads, checks the signature, installs and restarts. */
  updatesInstall: () => invoke<null>('updates_install'),
  updatesWhatsNew: () => invoke<WhatsNew | null>('updates_whats_new'),
};

/** Event names shared with `src-tauri/src/app/events.rs`. */
export const events = {
  overlayShown: 'overlay://shown',
  settingsChanged: 'settings://changed',
  navigate: 'app://navigate',
  confirmUninstall: 'app://confirm-uninstall',
  hotkeyHeld: 'hotkey://held',
  hotkeyRecorded: 'hotkey://recorded',
  updateAvailable: 'updates://available',
  updateProgress: 'updates://progress',
} as const;

export function on<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (e) => handler(e.payload));
}

/** Rust errors arrive as `{ kind, message }`. */
export function errorMessage(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error) return String(error.message);
  return String(error);
}
