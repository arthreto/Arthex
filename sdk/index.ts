import { host } from './host';

export { defineModule, type DefinedModule, type InstallableModule } from './define';
export { getLocale, moduleTranslator } from './i18n.svelte';
export type { ContextMenuAction, KeyHint, ToolProps, Tint, WidgetSlot } from '../src/core/modules/manifest';
export type { Query, ResultIcon, SearchProvider, SearchResult } from '../src/core/search/types';

export { default as Button } from '../src/ui/Button.svelte';
export { default as CardButton } from '../src/ui/CardButton.svelte';
export { default as Check } from '../src/ui/Check.svelte';
export { default as Chip } from '../src/ui/Chip.svelte';
export { default as EmptyState } from '../src/ui/EmptyState.svelte';
export { default as FileIcon } from '../src/ui/FileIcon.svelte';
export { default as Icon } from '../src/ui/Icon.svelte';
export { default as IconButton } from '../src/ui/IconButton.svelte';
export { default as Keycap } from '../src/ui/Keycap.svelte';
export { default as LinkButton } from '../src/ui/LinkButton.svelte';
export { default as Menu } from '../src/ui/Menu.svelte';
export { default as MenuItem } from '../src/ui/MenuItem.svelte';
export { default as Modal } from '../src/ui/Modal.svelte';
export { default as Panel } from '../src/ui/Panel.svelte';
export { default as Segmented } from '../src/ui/Segmented.svelte';
export { default as Skeleton } from '../src/ui/Skeleton.svelte';
export { default as Switch } from '../src/ui/Switch.svelte';
export { default as TextField } from '../src/ui/TextField.svelte';
export { default as ToggleChip } from '../src/ui/ToggleChip.svelte';
export { TAGS, type Tag } from '../src/core/modules/tags';
export { formatBytes, formatPercent, formatWhen } from '../src/core/format';
export { matchesAll, normalize } from '../src/core/text';

export const toast = (message: string) => host().toast(message);
export const openTool = (id: string, input?: string) => host().openTool(id, input);
export const closeTool = () => host().closeTool();
/** `hub`, `settings` or a page id. */
export const navigate = (target: string) => host().navigate(target);

export const hideOverlay = () => host().hideOverlay();
export const showOverlay = () => host().showOverlay();

/** The program of a module, in its `native/` folder. Its events arrive as `<id>://<name>`. */
export function moduleNative(id: string) {
  return {
    call: <T = unknown>(method: string, params?: unknown) => host().call(id, method, params) as Promise<T>,
    /** With the overlay out of the way until it answers, however long it takes: needs `api: 3`. */
    callHidden: <T = unknown>(method: string, params?: unknown) =>
      host().callHidden(id, method, params) as Promise<T>,
    on: (event: string, handler: (payload: unknown) => void) => host().listen(id, event, handler),
  };
}
