import type { Settings } from '../settings/types';
import * as layout from './layout';
import type { ModuleManifest, WidgetSlot } from './manifest';

export { defineModule } from './manifest';
export type { ModuleManifest, SettingsSection, Tint, ToolProps, WidgetSlot } from './manifest';
export { ELEMENTS, HOME_ID, MAX_TAB_BAR, ZONES, elementsOf, switchElement, toggleFavorite } from './layout';
export type { Element, Placement, Zone } from './layout';

const found = import.meta.glob<{ default: ModuleManifest }>('../../modules/*/manifest.ts', { eager: true });

/** Every module of `src/modules/*`, discovered at build time. */
export const modules: readonly ModuleManifest[] = Object.values(found)
  .map((entry) => entry.default)
  .sort((a, b) => a.order - b.order);

export function getModule(id: string): ModuleManifest | undefined {
  return modules.find((m) => m.id === id);
}

export const isEnabled = (settings: Settings, id: string, element: layout.Element) =>
  layout.isEnabled(modules, settings, id, element);
export const isActive = (settings: Settings, id: string) => layout.isActive(modules, settings, id);
export const tabBarIds = (settings: Settings) => layout.tabBarIds(modules, settings);
export const launcherIds = (settings: Settings) => layout.launcherIds(modules, settings);
export const zoneIds = (settings: Settings) => layout.zoneIds(modules, settings);
export const placePage = (settings: Settings, id: string, zone: layout.Zone, beforeId?: string) =>
  layout.placePage(modules, settings, id, zone, beforeId);
export const moveInBar = (settings: Settings, id: string, step: -1 | 1) =>
  layout.moveInBar(modules, settings, id, step);
export const homeWidgets = (settings: Settings, slot: WidgetSlot) =>
  modules.filter((m) => m.widget?.slot === slot && layout.isEnabled(modules, settings, m.id, 'widget'));
export const searchProviders = (settings: Settings) =>
  modules.flatMap((m) => (m.search && layout.isEnabled(modules, settings, m.id, 'search') ? [m.search] : []));
export const toolModules = (settings: Settings) =>
  modules.filter((m) => layout.isEnabled(modules, settings, m.id, 'tool'));
export const settingsModules = (settings: Settings) =>
  modules.filter((m) => m.settingsSection && layout.isActive(modules, settings, m.id));
export const badgeModules = (settings: Settings) =>
  modules.filter((m) => m.badge && layout.isActive(modules, settings, m.id));
