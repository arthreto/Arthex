import type { Component } from 'svelte';
import { getModule } from '../core/modules';

type Loaded = { default: Component };

const loaded = new Map<string, Promise<Loaded>>();

function once(key: string, load: (() => Promise<Loaded>) | undefined): Promise<Loaded> | null {
  if (!load) return null;
  let promise = loaded.get(key);
  if (!promise) {
    promise = load();
    loaded.set(key, promise);
  }
  return promise;
}

/** Loads a page component once and reuses it on later visits. */
export function loadPage(id: string): Promise<Loaded> | null {
  return once(`page:${id}`, getModule(id)?.page);
}

/** Loads the control a module shows next to its switch in Settings, once. */
export function loadSettingsAction(id: string): Promise<Loaded> | null {
  return once(`action:${id}`, getModule(id)?.settingsAction);
}

/** Loads the status a module shows at the top right of the overlay, once. */
export function loadBadge(id: string): Promise<Loaded> | null {
  return once(`badge:${id}`, getModule(id)?.badge);
}

/** Loads the Home card of a module once. */
export function loadWidget(id: string): Promise<Loaded> | null {
  return once(`widget:${id}`, getModule(id)?.widget?.component);
}
