import type { Component } from 'svelte';
import { getModule } from '../core/modules';

type PageModule = { default: Component };

const loaded = new Map<string, Promise<PageModule>>();

/** Loads a page component once and reuses it on later visits. */
export function loadPage(id: string): Promise<PageModule> | null {
  const load = getModule(id)?.page;
  if (!load) return null;
  let promise = loaded.get(id);
  if (!promise) {
    promise = load();
    loaded.set(id, promise);
  }
  return promise;
}
