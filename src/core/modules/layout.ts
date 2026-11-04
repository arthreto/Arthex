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

export type Zone = 'bar' | 'other' | 'off';
export const ZONES: readonly Zone[] = ['bar', 'other', 'off'];

/** The three groups of the "Customize" view: in the tab bar, enabled elsewhere, turned off. */
export function zoneIds(catalog: readonly ModuleManifest[], layout: Layout): Record<Zone, string[]> {
  const bar = tabBarIds(catalog, layout);
  const pages = catalog.filter((m) => m.kind === 'page');
  return {
    bar,
    other: pages.filter((m) => isEnabled(catalog, layout, m.id) && !bar.includes(m.id)).map((m) => m.id),
    off: pages.filter((m) => !isEnabled(catalog, layout, m.id)).map((m) => m.id),
  };
}

export type Placement =
  { ok: true; patch: Pick<Settings, 'tabBar' | 'modules'> } | { ok: false; reason: 'locked' | 'barFull' };

/** Moves a page into a zone, before `beforeId` in the bar or at its end. Home never leaves the bar. */
export function placePage(
  catalog: readonly ModuleManifest[],
  layout: Layout,
  id: string,
  zone: Zone,
  beforeId?: string,
): Placement {
  const bar = tabBarIds(catalog, layout);
  const module = catalog.find((m) => m.id === id);
  if (module?.locked) {
    return zone === 'bar'
      ? { ok: true, patch: { tabBar: bar, modules: layout.modules } }
      : { ok: false, reason: 'locked' };
  }
  if (zone === 'bar' && !bar.includes(id) && bar.length >= MAX_TAB_BAR)
    return { ok: false, reason: 'barFull' };

  const modules = Object.fromEntries(Object.entries(layout.modules).filter(([other]) => other !== id));
  if (zone === 'off') modules[id] = false;

  const tabBar = bar.filter((other) => other !== id);
  if (zone === 'bar') {
    const at = beforeId === undefined ? -1 : tabBar.indexOf(beforeId);
    tabBar.splice(at === -1 ? tabBar.length : Math.max(1, at), 0, id);
  }
  return { ok: true, patch: { tabBar, modules } };
}

/** Moves a tab one step left or right, never before Home. */
export function moveInBar(
  catalog: readonly ModuleManifest[],
  layout: Layout,
  id: string,
  step: -1 | 1,
): string[] | null {
  const bar = tabBarIds(catalog, layout);
  const from = bar.indexOf(id);
  const to = from + step;
  if (from < 1 || to < 1 || to >= bar.length) return null;
  bar.splice(from, 1);
  bar.splice(to, 0, id);
  return bar;
}
