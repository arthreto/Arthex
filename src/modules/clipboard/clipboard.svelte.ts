import { errorMessage } from '../../core/api';
import { t as shellT } from '../../core/i18n';
import { toast } from '../../core/toast.svelte';
import { clipboardApi, type Clip, type ClipboardState } from './api';
import { t } from './i18n';

function fail(error: unknown) {
  toast.show(shellT('errors.generic', { message: errorMessage(error) }));
}

/** The history as Rust holds it, shared by the page and the Home card. */
class ClipboardHistory {
  clips = $state<Clip[]>([]);
  paused = $state(false);
  #listening = false;

  /** The latest copy that is not pinned, for the Home card. */
  get latest(): Clip | null {
    return this.clips.find((clip) => !clip.pinned) ?? this.clips[0] ?? null;
  }

  async load(): Promise<void> {
    if (!this.#listening) {
      this.#listening = true;
      void clipboardApi.onChanged(() => void this.load());
    }
    await this.#apply(clipboardApi.state());
  }

  async copy(clip: Clip): Promise<void> {
    try {
      await navigator.clipboard.writeText(clip.text);
      toast.show(t('copied', { text: clip.text.length > 40 ? `${clip.text.slice(0, 40)}…` : clip.text }));
    } catch (error) {
      fail(error);
    }
  }

  togglePause = () => this.#apply(clipboardApi.pause(!this.paused));
  pin = (clip: Clip) => this.#apply(clipboardApi.pin(clip.id, !clip.pinned));
  remove = (clip: Clip) => this.#apply(clipboardApi.remove(clip.id));

  async clear(): Promise<void> {
    await this.#apply(clipboardApi.clear());
    toast.show(t('cleared'));
  }

  async #apply(next: Promise<ClipboardState>): Promise<void> {
    try {
      const state = await next;
      this.clips = state.clips;
      this.paused = state.paused;
    } catch (error) {
      fail(error);
    }
  }
}

export const clipboard = new ClipboardHistory();
