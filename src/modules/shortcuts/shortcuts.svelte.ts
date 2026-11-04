import { shortcutsApi, type Shortcut } from './api';

class ShortcutsState {
  items = $state<Shortcut[]>([]);
  #loading: Promise<void> | null = null;

  /** Loads the list once and follows later changes (icons arriving in the background). */
  load(): Promise<void> {
    this.#loading ??= (async () => {
      this.items = await shortcutsApi.list();
      await shortcutsApi.onChanged((list) => (this.items = list));
    })();
    return this.#loading;
  }

  async addSite(url: string, name: string | null): Promise<void> {
    this.items = await shortcutsApi.addSite(url, name);
  }
}

export const shortcuts = new ShortcutsState();
