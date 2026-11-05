import type { Component } from 'svelte';
import type { SearchProvider } from '../search/types';

/** Where a card goes on Home: the full-width row on top, the main column or the side column. */
export type WidgetSlot = 'top' | 'main' | 'side';

type Lazy = () => Promise<{ default: Component }>;

/** What a tool receives when it opens, such as the text a search result passed on. */
export type ToolProps = { input?: string };

/** A section of its own in Settings, after Modules, shown while the module is active. */
export interface SettingsSection {
  component: Lazy;
  /** A button next to the section title ("Reindex"). */
  action?: { label: () => string; run: () => unknown };
}

/** Accent colours defined in `src/ui/tokens.css` (`--tint-<name>`), tuned for both themes. */
export type Tint = 'blue' | 'amber' | 'green' | 'violet' | 'sky' | 'pink' | 'mint' | 'orange' | 'red';

/**
 * A module and the elements it provides: `page`, `widget`, `search` and `tool`.
 * Each element has its own switch in Settings → Modules.
 */
export interface ModuleManifest {
  id: string;
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
  /** A mini-app found by its name or keywords in the search, opened over the overlay. */
  tool?: () => Promise<{ default: Component<ToolProps> }>;
}

export function defineModule(manifest: ModuleManifest): ModuleManifest {
  return manifest;
}
