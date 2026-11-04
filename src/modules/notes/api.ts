import { invoke } from '@tauri-apps/api/core';

export interface Note {
  id: string;
  title: string;
  /** Markdown. */
  text: string;
  /** Milliseconds since the Unix epoch, set by Rust on each save. */
  updated: number;
}

export const notesApi = {
  /** The most recently changed first. */
  list: () => invoke<Note[]>('notes_list'),
  /** Creates, updates or restores a note. */
  save: (note: Note) => invoke<Note>('notes_save', { note }),
  remove: (id: string) => invoke<null>('notes_delete', { id }),
};
