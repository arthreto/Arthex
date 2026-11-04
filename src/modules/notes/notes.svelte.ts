import { errorMessage } from '../../core/api';
import { t as shellT } from '../../core/i18n';
import { toast } from '../../core/toast.svelte';
import { notesApi, type Note } from './api';
import { t } from './i18n';

/** Typing is saved this long after the last key, and right away when leaving the note. */
const SAVE_DELAY_MS = 500;

export type NoteMode = 'view' | 'edit';

function fail(error: unknown) {
  toast.show(shellT('errors.generic', { message: errorMessage(error) }));
}

class Notebook {
  notes = $state<Note[]>([]);
  currentId = $state<string | null>(null);
  mode = $state<NoteMode>('view');
  saving = $state(false);
  #loaded: Promise<void> | null = null;
  #timer: ReturnType<typeof setTimeout> | undefined;
  #pending: Note | null = null;

  get current(): Note | null {
    return this.notes.find((note) => note.id === this.currentId) ?? this.notes[0] ?? null;
  }

  load(): Promise<void> {
    this.#loaded ??= notesApi
      .list()
      .then((notes) => {
        this.notes = notes;
        this.open(notes[0]?.id ?? null);
      })
      .catch((error: unknown) => {
        this.#loaded = null;
        fail(error);
      });
    return this.#loaded;
  }

  open(id: string | null): void {
    void this.flush();
    this.currentId = id;
    this.mode = this.current?.text.trim() ? 'view' : 'edit';
  }

  async create(): Promise<void> {
    await this.flush();
    try {
      const note = await notesApi.save({ id: crypto.randomUUID(), title: '', text: '', updated: 0 });
      this.notes = [note, ...this.notes];
      this.currentId = note.id;
      this.mode = 'edit';
    } catch (error) {
      fail(error);
    }
  }

  /** Changes the open note at once on screen, and saves it shortly after. */
  update(patch: Partial<Pick<Note, 'title' | 'text'>>): void {
    const current = this.current;
    if (!current) return;
    const next = { ...current, ...patch };
    this.notes = this.notes.map((note) => (note.id === next.id ? next : note));
    this.#pending = next;
    this.saving = true;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.flush(), SAVE_DELAY_MS);
  }

  async flush(): Promise<void> {
    clearTimeout(this.#timer);
    const pending = this.#pending;
    this.#pending = null;
    if (!pending) return;
    try {
      const saved = await notesApi.save(pending);
      this.notes = this.notes.map((note) =>
        note.id === saved.id ? { ...note, updated: saved.updated } : note,
      );
    } catch (error) {
      fail(error);
    } finally {
      this.saving = this.#pending !== null;
    }
  }

  async remove(note: Note): Promise<void> {
    if (this.#pending?.id === note.id) {
      clearTimeout(this.#timer);
      this.#pending = null;
      this.saving = false;
    }
    try {
      await notesApi.remove(note.id);
      this.notes = this.notes.filter((other) => other.id !== note.id);
      this.open(this.notes[0]?.id ?? null);
      toast.show(t('deleted', { title: note.title || t('untitled') }), {
        label: shellT('common.undo'),
        run: () => void this.#restore(note),
      });
    } catch (error) {
      fail(error);
    }
  }

  async #restore(note: Note): Promise<void> {
    try {
      const saved = await notesApi.save(note);
      this.notes = [saved, ...this.notes];
      this.open(saved.id);
    } catch (error) {
      fail(error);
    }
  }
}

export const notebook = new Notebook();
