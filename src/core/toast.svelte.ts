const DURATION_MS = 2600;
const WITH_ACTION_MS = 5000;

export interface ToastAction {
  label: string;
  run: () => void;
}

class ToastState {
  message = $state<string | null>(null);
  action = $state<ToastAction | null>(null);
  #timer: ReturnType<typeof setTimeout> | undefined;

  /** Shows a short message, optionally with one action such as Undo. */
  show(message: string, action: ToastAction | null = null): void {
    this.message = message;
    this.action = action;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => this.hide(), action ? WITH_ACTION_MS : DURATION_MS);
  }

  hide(): void {
    clearTimeout(this.#timer);
    this.message = null;
    this.action = null;
  }
}

export const toast = new ToastState();
