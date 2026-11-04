import { invoke } from '@tauri-apps/api/core';
import { on } from '../../core/api';

/** Mirrors `Category` in `src-tauri/src/services/file_index/usage.rs`. */
export type Category = 'games' | 'personal' | 'videos' | 'apps' | 'system' | 'images' | 'other';

export interface Drive {
  path: string;
  /** The name given to the drive, empty when it has none. */
  label: string;
  /** Bytes, 0 when the system does not say. */
  total: number;
  free: number;
}

export interface Item {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  category: Category;
  hasChildren: boolean;
  protected: boolean;
}

export interface Folder {
  path: string;
  size: number;
  /** Largest first. */
  shares: { category: Category; size: number }[];
  /** Largest first. */
  items: Item[];
}

export interface LargeFile {
  name: string;
  path: string;
  /** The parent folder, ending with a separator. */
  location: string;
  size: number;
  protected: boolean;
}

export const filesizeApi = {
  drives: () => invoke<Drive[]>('filesize_drives'),
  /** `null` until the folder is indexed. */
  folder: (path: string) => invoke<Folder | null>('filesize_folder', { path }),
  largest: (path: string) => invoke<LargeFile[]>('filesize_largest', { path }),
  rescan: () => invoke<null>('filesize_rescan'),
  reveal: (path: string) => invoke<null>('filesize_reveal', { path }),
  pickFolder: () => invoke<string | null>('filesize_pick_folder'),
  trash: (path: string) => invoke<null>('filesize_trash', { path }),
  /** A scan finished: sizes may have changed. */
  onUpdated: (handler: () => void) => on<null>('file-index://updated', handler),
};
