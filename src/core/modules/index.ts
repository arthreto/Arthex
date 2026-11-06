import type { SearchProvider } from '../search/types';
import type { Settings } from '../settings/types';
import * as layout from './layout';
import type { ModuleManifest, WidgetSlot } from './manifest';
import { registry } from './registry.svelte';

export { defineModule } from './manifest';
export type { KeyHint, ModuleManifest, SettingsSection, Tint, ToolProps, WidgetSlot } from './manifest';
export { ELEMENTS, HOME_ID, MAX_TAB_BAR, ZONES, elementsOf, switchElement, toggleFavorite } from './layout';
export type { Element, Placement, Zone } from './layout';
export { registry } from './registry.svelte';
export { TAGS, knownTags, type Tag } from './tags';

/** Built-in modules and the installed ones, by order. */
export const allModules = () => registry.all;

export function getModule(id: string): ModuleManifest | undefined {
  return registry.all.find((m) => m.id === id);
}

export const isEnabled = (settings: Settings, id: string, element: layout.Element) =>
  layout.isEnabled(registry.all, settings, id, element);
export const isActive = (settings: Settings, id: string) => layout.isActive(registry.all, settings, id);
export const tabBarIds = (settings: Settings) => layout.tabBarIds(registry.all, settings);
export const launcherIds = (settings: Settings) => layout.launcherIds(registry.all, settings);
export const zoneIds = (settings: Settings) => layout.zoneIds(registry.all, settings);
export const placePage = (settings: Settings, id: string, zone: layout.Zone, beforeId?: string) =>
  layout.placePage(registry.all, settings, id, zone, beforeId);
export const moveInBar = (settings: Settings, id: string, step: -1 | 1) =>
  layout.moveInBar(registry.all, settings, id, step);
export const homeWidgets = (settings: Settings, slot: WidgetSlot) =>
  registry.all.filter(
    (m) => m.widget?.slot === slot && layout.isEnabled(registry.all, settings, m.id, 'widget'),
  );
export const searchProviders = (settings: Settings): SearchProvider[] =>
  registry.all.flatMap((m) =>
    m.search && layout.isEnabled(registry.all, settings, m.id, 'search')
      ? ([] as SearchProvider[]).concat(m.search)
      : [],
  );
export const toolModules = (settings: Settings) =>
  registry.all.filter((m) => layout.isEnabled(registry.all, settings, m.id, 'tool'));
export const settingsModules = (settings: Settings) =>
  registry.all.filter((m) => m.settingsSection && layout.isActive(registry.all, settings, m.id));
export const badgeModules = (settings: Settings) =>
  registry.all.filter((m) => m.badge && layout.isActive(registry.all, settings, m.id));
