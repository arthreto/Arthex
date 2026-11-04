const DURATION_MS = 2600;

class ToastState {
  message = $state<string | null>(null);
  #timer: ReturnType<typeof setTimeout> | undefined;

  show(message: string): void {
    this.message = message;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => (this.message = null), DURATION_MS);
  }
}

export const toast = new ToastState();
