import type { Component } from 'svelte';
import type { MessageTree, MessageValues } from '../i18n';
import type { SearchProvider } from '../search/types';
import ForeignView from './ForeignView.svelte';
import {
  TINTS,
  type ContextMenuAction,
  type KeyHint,
  type ModuleManifest,
  type WidgetSlot,
} from './manifest';
import { knownTags } from './tags';

/**
 * Shows a view of an installed module in `target` and returns what removes it. Installed modules
 * bring their own Svelte, so the shell never renders their components itself.
 */
export type Mount = (target: HTMLElement, props: Record<string, unknown>) => () => void;
type LazyMount = () => Promise<Mount>;

/** The default export of an installed module's `index.js`, made by `defineModule` of the SDK. */
export interface ForeignManifest {
  id: string;
  icon: string;
  tint: string;
  order: number;
  tags?: string[];
  /** Its `locales/<lang>.json`, by language code. */
  locales: Record<string, MessageTree>;
  page?: LazyMount;
  widget?: { slot: WidgetSlot; component: LazyMount };
  search?: SearchProvider;
  tool?: LazyMount;
  contextMenu?: readonly ContextMenuAction[];
  badge?: LazyMount;
  settingsAction?: LazyMount;
  hints?: () => readonly KeyHint[];
  settingsSection?: {
    component: LazyMount;
    action?: { label: () => string; run: () => unknown; hidden?: () => boolean };
  };
}

/**
 * What Arthex offers installed modules, as `window.__ARTHEX__`. Its version is `HOST_API` in
 * `src-tauri/src/services/module_host/package.rs`: only add to it, or raise that version.
 */
export interface HostApi {
  api: 7;
  locale(): string;
  onLocaleChange(listener: (locale: string) => void): () => void;
  /** A message by its full key: `modules.<id>.<key>` for a module, `common.close` for the shell. */
  translate(key: string, values?: MessageValues): string;
  toast(message: string): void;
  openTool(id: string, input?: string): void;
  closeTool(): void;
  /** `hub`, `settings` or a page id. */
  navigate(target: string): void;
  /** Calls the module's program, which starts if needed. */
  call(id: string, method: string, params?: unknown): Promise<unknown>;
  /** Hears an event of the module's program; returns what stops listening. */
  listen(id: string, event: string, handler: (payload: unknown) => void): () => void;
  /**
   * Calls the module's program with the overlay hidden, then brings the overlay back, however long
   * it takes. The page is suspended while the overlay is hidden, so only this can wait through it.
   */
  callHidden(id: string, method: string, params?: unknown): Promise<unknown>;
  hideOverlay(): void;
  showOverlay(): void;
}

const SLOTS: readonly WidgetSlot[] = ['top', 'main', 'side'];

/** A Svelte component of the shell that shows what `mount` puts in place. */
export function foreignComponent(mount: Mount): Component<Record<string, unknown>> {
  return (internals, props) => ForeignView(internals, { mount, props });
}

function view(load: unknown): (() => Promise<{ default: Component<Record<string, unknown>> }>) | undefined {
  if (typeof load !== 'function') return undefined;
  return async () => ({ default: foreignComponent(await (load as LazyMount)()) });
}

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

/** Checks what an installed module exported, then turns it into a manifest the shell can use. */
export function adopt(raw: unknown, id: string): ModuleManifest | null {
  if (!isObject(raw) || raw.id !== id || typeof raw.icon !== 'string' || !isObject(raw.locales)) return null;
  const tint = TINTS.find((tint) => tint === raw.tint);
  if (!tint || typeof raw.order !== 'number' || !Number.isFinite(raw.order)) return null;
  const foreign = raw as Partial<ForeignManifest>;
  const widget = isObject(foreign.widget) && SLOTS.includes(foreign.widget.slot) ? foreign.widget : undefined;
  const widgetView = view(widget?.component);
  const section = isObject(foreign.settingsSection) ? foreign.settingsSection : undefined;
  const sectionView = view(section?.component);
  const search =
    isObject(foreign.search) && typeof foreign.search.search === 'function' ? foreign.search : undefined;
  return {
    id,
    icon: raw.icon,
    tint,
    order: raw.order,
    tags: knownTags(foreign.tags),
    page: view(foreign.page),
    widget: widget && widgetView ? { slot: widget.slot, component: widgetView } : undefined,
    search,
    tool: view(foreign.tool),
    contextMenu: Array.isArray(foreign.contextMenu) ? foreign.contextMenu : undefined,
    badge: view(foreign.badge),
    settingsAction: view(foreign.settingsAction),
    hints: typeof foreign.hints === 'function' ? foreign.hints : undefined,
    settingsSection: sectionView ? { component: sectionView, action: section?.action } : undefined,
  };
}
