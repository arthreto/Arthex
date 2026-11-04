import { commands, events, on } from '../api';
import type { Settings, SettingsPatch } from './types';

class SettingsState {
  #value = $state<Settings | null>(null);

  get value(): Settings {
    if (!this.#value) throw new Error('Settings are not loaded yet');
    return this.#value;
  }

  get loaded(): boolean {
    return this.#value !== null;
  }

  async load(): Promise<void> {
    this.#value = await commands.settingsGet();
    await on<Settings>(events.settingsChanged, (value) => (this.#value = value));
  }

  /** Applies the patch immediately, then keeps what Rust validated and saved. */
  async update(patch: SettingsPatch): Promise<void> {
    const previous = this.#value;
    if (previous) this.#value = { ...previous, ...patch };
    try {
      this.#value = await commands.settingsUpdate(patch);
    } catch (error) {
      this.#value = previous;
      throw error;
    }
  }
}

export const settings = new SettingsState();
