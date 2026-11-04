import type { Settings } from '../settings/types';
import * as layout from './layout';
import type { ModuleManifest } from './manifest';

export { defineModule } from './manifest';
export type { ModuleKind, ModuleManifest, Tint } from './manifest';
export { HOME_ID, MAX_TAB_BAR, toggleFavorite } from './layout';

const found = import.meta.glob<{ default: ModuleManifest }>('../../modules/*/manifest.ts', { eager: true });

/** Every module of `src/modules/*`, discovered at build time. */
export const modules: readonly ModuleManifest[] = Object.values(found)
  .map((entry) => entry.default)
  .sort((a, b) => a.order - b.order);

export function getModule(id: string): ModuleManifest | undefined {
  return modules.find((m) => m.id === id);
}

export const isEnabled = (settings: Settings, id: string) => layout.isEnabled(modules, settings, id);
export const tabBarIds = (settings: Settings) => layout.tabBarIds(modules, settings);
export const launcherIds = (settings: Settings) => layout.launcherIds(modules, settings);
