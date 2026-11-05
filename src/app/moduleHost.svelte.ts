import { commands, errorMessage, on, type InstallRequest, type InstalledModule } from '../core/api';
import { getLocale, registerModule, translate } from '../core/i18n';
import { registry, type ModuleManifest } from '../core/modules';
import { adopt, type HostApi } from '../core/modules/foreign';
import { toast } from '../core/toast.svelte';
import { navigation } from './navigation.svelte';
import { forget } from './pages';
import { search } from './search.svelte';
import { tools } from '../core/tools.svelte';

async function importModule(module: InstalledModule): Promise<ModuleManifest | null> {
  // A new URL for each version: the webview keeps every module it imported.
  const version = module.dev ? `${module.version}-${Date.now()}` : module.version;
  try {
    const exported: unknown = (await import(/* @vite-ignore */ `${module.url}index.js?v=${version}`)).default;
    const manifest = adopt(exported, module.id);
    if (!manifest) throw new Error('unusable manifest');
    registerModule(module.id, (exported as { locales: Parameters<typeof registerModule>[1] }).locales);
    return manifest;
  } catch (error) {
    console.error(`Module ${module.id} did not load`, error);
    return null;
  }
}

function syncStyles(installed: readonly InstalledModule[]): void {
  for (const link of document.querySelectorAll<HTMLLinkElement>('link[data-module]')) link.remove();
  for (const module of installed.filter((m) => m.style)) {
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = `${module.url}style.css?v=${module.version}`;
    link.dataset.module = module.id;
    document.head.append(link);
  }
}

class ModuleHost {
  /** What Rust reports as installed, including a module that failed to load. */
  installed = $state<InstalledModule[]>([]);
  /** A `.arthex` file opened from Explorer, waiting for the user's answer. */
  opened = $state<InstallRequest | null>(null);
  #run = 0;

  async checkOpened(): Promise<void> {
    try {
      this.opened = await commands.modulesOpened();
    } catch (error) {
      toast.show(translate('errors.generic', { message: errorMessage(error) }));
    }
  }

  /** Opens the page a right-click action asked for, once its module is loaded. */
  async openPendingPage(): Promise<void> {
    const page = await commands.modulesPendingPage().catch(() => null);
    if (!page) return;
    search.clear();
    navigation.openTarget(page);
  }

  /** Loads every installed module again. A newer call wins over a slower, older one. */
  async load(): Promise<void> {
    const run = ++this.#run;
    const list = await commands.modulesList();
    const loaded = await Promise.all(list.map(importModule));
    if (run !== this.#run) return;
    const next = new Map(list.map((module) => [module.id, module]));
    for (const module of this.installed) {
      const after = next.get(module.id);
      if (!after || after.dev || after.version !== module.version) forget(module.id);
    }
    this.installed = list;
    syncStyles(list);
    registry.setInstalled(loaded.filter((manifest) => manifest !== null));
  }
}

export const moduleHost = new ModuleHost();

/** Gives installed modules `window.__ARTHEX__`, the only part of the shell they can reach. */
export function installHostApi(): void {
  let listeners: ((locale: string) => void)[] = [];
  $effect.root(() => {
    $effect(() => {
      const locale = getLocale();
      for (const listener of listeners) listener(locale);
    });
  });
  const api: HostApi = {
    api: 5,
    locale: getLocale,
    onLocaleChange(listener) {
      listeners = [...listeners, listener];
      return () => {
        listeners = listeners.filter((other) => other !== listener);
      };
    },
    translate,
    toast: (message) => toast.show(message),
    openTool: (id, input) => tools.open(id, input),
    closeTool: () => tools.close(),
    navigate(target) {
      search.clear();
      navigation.openTarget(target);
    },
    call: (id, method, params) => commands.modulesCall(id, method, params),
    listen(id, event, handler) {
      const listening = on<unknown>(`${id}://${event}`, handler);
      return () => void listening.then((unlisten) => unlisten());
    },
    callHidden: (id, method, params) => commands.modulesCallHidden(id, method, params),
    hideOverlay: () => void commands.modulesHideOverlay(),
    showOverlay: () => void commands.modulesShowOverlay(),
  };
  Object.defineProperty(window, '__ARTHEX__', { value: Object.freeze(api) });
}
