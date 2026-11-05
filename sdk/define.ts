import { mount, unmount, type Component } from 'svelte';
import type { MessageTree } from '../src/core/i18n/types';
import type { ForeignManifest, Mount } from '../src/core/modules/foreign';
import type { ContextMenuAction, Tint, ToolProps, WidgetSlot } from '../src/core/modules/manifest';
import { TAGS, type Tag } from '../src/core/modules/tags';
import type { SearchProvider } from '../src/core/search/types';

type Lazy<Props extends Record<string, unknown> = Record<string, never>> = () => Promise<{
  default: Component<Props>;
}>;

/** What a module of the catalog declares in its `manifest.ts`. */
export interface InstallableModule {
  id: string;
  /** Semantic version, raised for each release of the module. */
  version: string;
  /**
   * The oldest version of what Arthex offers modules that this one works with: raise it when it
   * uses something newer, and users of an older Arthex are asked to update only then.
   */
  api?: number;
  /** SVG path data of a 24×24 stroked icon. */
  icon: string;
  tint: Tint;
  order: number;
  /** Its categories in the catalog, among the fixed list `TAGS`. */
  tags?: readonly Tag[];
  /** `import.meta.glob('./locales/*.json', { eager: true, import: 'default' })`. */
  locales: Record<string, MessageTree>;
  /** Its program runs as long as the module is active, not only while it is called. */
  background?: boolean;
  page?: Lazy;
  widget?: { slot: WidgetSlot; component: Lazy };
  /** Results in the search bar. `panel` is not available to installed modules. */
  search?: SearchProvider;
  tool?: Lazy<ToolProps>;
  /** Its actions in the right-click menu of Explorer: needs `api: 4` and a program in `native/`. */
  contextMenu?: readonly ContextMenuAction[];
  badge?: Lazy;
  settingsAction?: Lazy;
  settingsSection?: {
    component: Lazy;
    action?: { label: () => string; run: () => unknown; hidden?: () => boolean };
  };
}

export type DefinedModule = ForeignManifest & {
  version: string;
  api: number;
  background: boolean;
  tags: Tag[];
};

/** The module brings its own Svelte: the shell receives functions that mount each view. */
function mounted<Props extends Record<string, unknown>>(load: Lazy<Props>): () => Promise<Mount> {
  return async () => {
    const view = (await load()).default;
    return (target, props) => {
      const instance = mount(view, { target, props: props as Props });
      return () => void unmount(instance);
    };
  };
}

const lazy = <Props extends Record<string, unknown>>(load: Lazy<Props> | undefined) => load && mounted(load);

/** Turns `./locales/fr.json` keys into `fr`. */
function byLanguage(locales: Record<string, MessageTree>): Record<string, MessageTree> {
  return Object.fromEntries(
    Object.entries(locales).map(([path, messages]) => [path.replace(/^.*\/|\.json$/g, ''), messages]),
  );
}

export function defineModule(module: InstallableModule): DefinedModule {
  // Plain JavaScript modules skip the type check: `arthex pack` fails here instead.
  const unknown = (module.tags ?? []).filter((tag) => !TAGS.includes(tag));
  if (unknown.length > 0) throw new Error(`unknown tags: ${unknown.join(', ')}`);
  return {
    id: module.id,
    version: module.version,
    api: module.api ?? 1,
    icon: module.icon,
    tint: module.tint,
    order: module.order,
    tags: [...(module.tags ?? [])],
    background: module.background ?? false,
    locales: byLanguage(module.locales),
    page: lazy(module.page),
    widget: module.widget && { slot: module.widget.slot, component: mounted(module.widget.component) },
    search: module.search,
    tool: lazy(module.tool),
    contextMenu: module.contextMenu,
    badge: lazy(module.badge),
    settingsAction: lazy(module.settingsAction),
    settingsSection: module.settingsSection && {
      component: mounted(module.settingsSection.component),
      action: module.settingsSection.action,
    },
  };
}
