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

  async saveSite(id: string | null, url: string, name: string | null, icon: string | null): Promise<void> {
    this.items = await shortcutsApi.saveSite(id, url, name, icon);
  }

  async saveApp(id: string | null, path: string, name: string | null): Promise<void> {
    this.items = await shortcutsApi.saveApp(id, path, name);
  }

  /** Removes a shortcut and returns a function that puts it back. */
  async remove(shortcut: Shortcut): Promise<() => Promise<void>> {
    const index = this.items.findIndex((item) => item.id === shortcut.id);
    this.items = await shortcutsApi.remove(shortcut.id);
    return async () => {
      this.items = await shortcutsApi.restore(shortcut, index);
    };
  }

  /** Moves `id` to the place of `targetId`, before it when moving back, after it when moving on. */
  async move(id: string, targetId: string): Promise<void> {
    const ids = this.items.map((item) => item.id);
    const from = ids.indexOf(id);
    const to = ids.indexOf(targetId);
    if (from === -1 || to === -1 || from === to) return;
    ids.splice(from, 1);
    ids.splice(to, 0, id);
    const previous = this.items;
    this.items = ids.flatMap((itemId) => previous.filter((item) => item.id === itemId));
    try {
      this.items = await shortcutsApi.reorder(ids);
    } catch (error) {
      this.items = previous;
      throw error;
    }
  }
}

export const shortcuts = new ShortcutsState();
