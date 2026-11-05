export type Theme = 'system' | 'dark' | 'light';

/** Mirrors `Settings` in `src-tauri/src/app/settings.rs`. */
export interface Settings {
  version: number;
  onboarded: boolean;
  language: string | null;
  theme: Theme;
  /** Shows the blurred desktop behind the overlay; otherwise its background is opaque. */
  translucent: boolean;
  /** Shows the logo of Arthex on the Windows taskbar, to open the overlay. */
  taskbarSearch: boolean;
  reduceMotion: boolean;
  autostart: boolean;
  hotkey: string[];
  tabBar: string[];
  favorites: string[];
  /** Turned-off elements, as `<module id>:<element>`. */
  disabledElements: string[];
  /** Installs updates found at launch without asking. */
  autoUpdate: boolean;
  /** Writes the detail of what Arthex does to its logs. */
  debugLog: boolean;
}

export type SettingsPatch = Partial<Omit<Settings, 'version'>>;
