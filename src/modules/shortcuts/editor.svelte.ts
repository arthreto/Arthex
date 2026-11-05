import type { Shortcut, Target } from './api';

/** The shortcut dialog: closed, adding (maybe from what was typed in the search) or editing. */
class Editor {
  current = $state<{ editing: Shortcut | null; prefill: (Target & { name?: string }) | null } | null>(null);

  add(prefill: (Target & { name?: string }) | null = null): void {
    this.current = { editing: null, prefill };
  }

  edit(shortcut: Shortcut): void {
    this.current = { editing: shortcut, prefill: null };
  }

  close(): void {
    this.current = null;
  }
}

export const editor = new Editor();
