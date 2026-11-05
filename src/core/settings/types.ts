export type Theme = 'system' | 'dark' | 'light';

/** Mirrors `Settings` in `src-tauri/src/app/settings.rs`. */
export interface Settings {
  version: number;
  onboarded: boolean;
  language: string | null;
  theme: Theme;
  reduceMotion: boolean;
  autostart: boolean;
  hotkey: string[];
  tabBar: string[];
  favorites: string[];
  /** Turned-off elements, as `<module id>:<element>`. */
  disabledElements: string[];
  /** Installs updates found at launch without asking. */
  autoUpdate: boolean;
}

export type SettingsPatch = Partial<Omit<Settings, 'version'>>;
