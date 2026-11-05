import { invoke } from '@tauri-apps/api/core';

export const webSearchApi = {
  /** Opens an `http` or `https` address in the default browser, then hides the overlay. */
  open: (url: string) => invoke<null>('web_search_open', { url }),
};
