import type { Component } from 'svelte';
import type { SearchProvider } from '../search/types';

export type ModuleKind = 'page' | 'feature';

/** Where a card goes on Home: the full-width row on top, the main column or the side column. */
export type WidgetSlot = 'top' | 'main' | 'side';

type Lazy = () => Promise<{ default: Component }>;

/** A section of its own in Settings, after Features, shown while the module is enabled. */
export interface SettingsSection {
  component: Lazy;
  /** A button next to the section title ("Reindex"). */
  action?: { label: () => string; run: () => unknown };
}

/** Accent colours defined in `src/ui/tokens.css` (`--tint-<name>`), tuned for both themes. */
export type Tint = 'blue' | 'amber' | 'green' | 'violet' | 'sky' | 'pink' | 'mint' | 'orange' | 'red';

export interface ModuleManifest {
  id: string;
  kind: ModuleKind;
  /** SVG path data of a 24×24 stroked icon. */
  icon: string;
  tint: Tint;
  /** Position among modules when nothing else decides the order. */
  order: number;
  /** A locked module can never be turned off (the home page). */
  locked?: boolean;
  /** Pages only: the component is loaded the first time the page opens. */
  page?: Lazy;
  /** A card on Home, shown while the module is enabled. */
  widget?: { slot: WidgetSlot; component: Lazy };
  /** A small control shown next to the module's switch in Settings ("Clear history"). */
  settingsAction?: Lazy;
  settingsSection?: SettingsSection;
  /** A small status at the top right of the overlay, such as the planned shutdown. */
  badge?: Lazy;
  /** Results this module adds to the search, while it is enabled. */
  search?: SearchProvider;
}

export function defineModule(manifest: ModuleManifest): ModuleManifest {
  return manifest;
}
