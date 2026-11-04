import { invoke } from '@tauri-apps/api/core';

export interface FileHit {
  path: string;
  name: string;
  /** The parent folder, ending with a separator. */
  location: string;
  isDir: boolean;
  size: number;
  /** Seconds since the Unix epoch. */
  modified: number;
}

export interface IndexStatus {
  /** The fixed drives, always indexed. */
  drives: string[];
  /** Folders added by the user. */
  folders: string[];
  exclusions: string[];
}

export const fileSearchApi = {
  query: (query: string) => invoke<FileHit[]>('file_search_query', { query }),
  open: (path: string) => invoke<null>('file_search_open', { path }),
  reveal: (path: string) => invoke<null>('file_search_reveal', { path }),
  status: () => invoke<IndexStatus>('file_search_status'),
  rescan: () => invoke<null>('file_search_rescan'),
  addFolder: () => invoke<IndexStatus>('file_search_add_folder'),
  removeFolder: (folder: string) => invoke<IndexStatus>('file_search_remove_folder', { folder }),
  removeExclusion: (exclusion: string) => invoke<IndexStatus>('file_search_remove_exclusion', { exclusion }),
};
