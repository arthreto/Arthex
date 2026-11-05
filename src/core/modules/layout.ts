import type { Settings } from '../settings/types';
import type { ModuleManifest } from './manifest';

export const HOME_ID = 'home';
export const MAX_TAB_BAR = 6;

/** What a module brings to the overlay, in the order Settings lists them. Each one has its own switch. */
export const ELEMENTS = ['page', 'widget', 'search', 'tool', 'contextMenu'] as const;
export type Element = (typeof ELEMENTS)[number];

type Layout = Pick<Settings, 'disabledElements' | 'tabBar' | 'favorites'>;

export function elementsOf(module: ModuleManifest): Element[] {
  return ELEMENTS.filter((element) => module[element] !== undefined);
}

const elementKey = (id: string, element: Element) => `${id}:${element}`;

export function isEnabled(
  catalog: readonly ModuleManifest[],
  layout: Layout,
  id: string,
  element: Element,
): boolean {
  const module = catalog.find((m) => m.id === id);
  if (module?.[element] === undefined) return false;
  return module.locked === true || !layout.disabledElements.includes(elementKey(id, element));
}

/** A module runs while at least one of its elements is on. */
export function isActive(catalog: readonly ModuleManifest[], layout: Layout, id: string): boolean {
  const module = catalog.find((m) => m.id === id);
  return (
    module !== undefined && elementsOf(module).some((element) => isEnabled(catalog, layout, id, element))
  );
}

/** The `disabledElements` setting once an element is turned on or off. */
export function switchElement(
  disabled: readonly string[],
  id: string,
  element: Element,
  enabled: boolean,
): string[] {
  const key = elementKey(id, element);
  const others = disabled.filter((other) => other !== key);
  return enabled ? others : [...others, key];
}

function enabledPages(catalog: readonly ModuleManifest[], layout: Layout): string[] {
  return catalog.filter((m) => isEnabled(catalog, layout, m.id, 'page')).map((m) => m.id);
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
  const disabled = catalog.filter((m) => m.page && !enabled.includes(m.id)).map((m) => m.id);
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
  const pages = catalog.filter((m) => m.page);
  return {
    bar,
    other: pages
      .filter((m) => isEnabled(catalog, layout, m.id, 'page') && !bar.includes(m.id))
      .map((m) => m.id),
    off: pages.filter((m) => !isEnabled(catalog, layout, m.id, 'page')).map((m) => m.id),
  };
}

export type Placement =
  | { ok: true; patch: Pick<Settings, 'tabBar' | 'disabledElements'> }
  | { ok: false; reason: 'locked' | 'barFull' };

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
      ? { ok: true, patch: { tabBar: bar, disabledElements: [...layout.disabledElements] } }
      : { ok: false, reason: 'locked' };
  }
  if (zone === 'bar' && !bar.includes(id) && bar.length >= MAX_TAB_BAR)
    return { ok: false, reason: 'barFull' };

  const disabledElements = switchElement(layout.disabledElements, id, 'page', zone !== 'off');
  const tabBar = bar.filter((other) => other !== id);
  if (zone === 'bar') {
    const at = beforeId === undefined ? -1 : tabBar.indexOf(beforeId);
    tabBar.splice(at === -1 ? tabBar.length : Math.max(1, at), 0, id);
  }
  return { ok: true, patch: { tabBar, disabledElements } };
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
