import type { Settings } from '../settings/types';
import type { ModuleManifest } from './manifest';

export const HOME_ID = 'home';
export const MAX_TAB_BAR = 6;

type Layout = Pick<Settings, 'modules' | 'tabBar' | 'favorites'>;

export function isEnabled(catalog: readonly ModuleManifest[], layout: Layout, id: string): boolean {
  const module = catalog.find((m) => m.id === id);
  return module !== undefined && (module.locked === true || layout.modules[id] !== false);
}

function enabledPages(catalog: readonly ModuleManifest[], layout: Layout): string[] {
  return catalog.filter((m) => m.kind === 'page' && isEnabled(catalog, layout, m.id)).map((m) => m.id);
}

/** Tabs shown in the bar: home first, then the saved order, without unknown or disabled pages. */
export function tabBarIds(catalog: readonly ModuleManifest[], layout: Layout): string[] {
  const pages = new Set(enabledPages(catalog, layout));
  const rest = layout.tabBar.filter((id) => id !== HOME_ID && pages.has(id));
  return [HOME_ID, ...new Set(rest)].slice(0, MAX_TAB_BAR);
}

/** Pages of “All pages”: favourites first (in pin order), then the others, then disabled ones. */
export function launcherIds(catalog: readonly ModuleManifest[], layout: Layout): string[] {
  const enabled = enabledPages(catalog, layout);
  const favorites = layout.favorites.filter((id) => enabled.includes(id));
  const disabled = catalog.filter((m) => m.kind === 'page' && !enabled.includes(m.id)).map((m) => m.id);
  return [...new Set(favorites), ...enabled.filter((id) => !favorites.includes(id)), ...disabled];
}

export function toggleFavorite(favorites: readonly string[], id: string): string[] {
  return favorites.includes(id) ? favorites.filter((f) => f !== id) : [...favorites, id];
}
