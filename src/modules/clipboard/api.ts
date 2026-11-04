import { invoke } from '@tauri-apps/api/core';
import { on } from '../../core/api';

export interface Clip {
  id: number;
  text: string;
  pinned: boolean;
  /** Milliseconds since the Unix epoch. */
  copiedAt: number;
}

export interface ClipboardState {
  /** Pinned first, then the latest. */
  clips: Clip[];
  paused: boolean;
}

export const clipboardApi = {
  state: () => invoke<ClipboardState>('clipboard_state'),
  pause: (paused: boolean) => invoke<ClipboardState>('clipboard_pause', { paused }),
  pin: (id: number, pinned: boolean) => invoke<ClipboardState>('clipboard_pin', { id, pinned }),
  remove: (id: number) => invoke<ClipboardState>('clipboard_remove', { id }),
  clear: () => invoke<ClipboardState>('clipboard_clear'),
  onChanged: (handler: () => void) => on<null>('clipboard://changed', handler),
};
