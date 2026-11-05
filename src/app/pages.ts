import type { Component } from 'svelte';
import { getModule, type ToolProps } from '../core/modules';

type Loaded<Props extends Record<string, unknown> = Record<string, never>> = {
  default: Component<Props>;
};

/** Loads each component of a module the first time it is needed, then reuses it. */
function cache<T>() {
  const promises = new Map<string, Promise<T>>();
  return (id: string, load: (() => Promise<T>) | undefined): Promise<T> | null => {
    if (!load) return null;
    let promise = promises.get(id);
    if (!promise) {
      promise = load();
      promises.set(id, promise);
    }
    return promise;
  };
}

const pages = cache<Loaded>();
const actions = cache<Loaded>();
const sections = cache<Loaded>();
const badges = cache<Loaded>();
const widgets = cache<Loaded>();
const tools = cache<Loaded<ToolProps>>();

export const loadPage = (id: string) => pages(id, getModule(id)?.page);

/** The control a module shows next to its first switch in Settings → Modules. */
export const loadSettingsAction = (id: string) => actions(id, getModule(id)?.settingsAction);

export const loadSettingsSection = (id: string) => sections(id, getModule(id)?.settingsSection?.component);

/** The status a module shows at the top right of the overlay. */
export const loadBadge = (id: string) => badges(id, getModule(id)?.badge);

export const loadWidget = (id: string) => widgets(id, getModule(id)?.widget?.component);

export const loadTool = (id: string) => tools(id, getModule(id)?.tool);
