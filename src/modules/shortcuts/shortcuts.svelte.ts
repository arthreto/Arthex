import { shortcutsApi, type Draft, type Shortcut } from './api';

class ShortcutsState {
  items = $state<Shortcut[]>([]);
  /** Home shows these, in order; Alt + 1…9 opens them. */
  home = $derived(this.items.filter((item) => item.onHome));
  #loading: Promise<void> | null = null;

  /** Loads the list once and follows later changes (icons arriving in the background). */
  load(): Promise<void> {
    this.#loading ??= (async () => {
      this.items = await shortcutsApi.list();
      await shortcutsApi.onChanged((list) => (this.items = list));
    })();
    return this.#loading;
  }

  async save(id: string | null, draft: Draft): Promise<void> {
    this.items = await shortcutsApi.save(id, draft);
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
