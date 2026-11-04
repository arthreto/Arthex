import { errorMessage } from '../../core/api';
import { t as shellT } from '../../core/i18n';
import { toast } from '../../core/toast.svelte';
import { filesizeApi, type Drive, type Folder, type LargeFile } from './api';
import { t } from './i18n';

export type Mode = 'folders' | 'largest';

/** What the action bar works on: an item of the folder or one of the largest files. */
export interface Selection {
  name: string;
  path: string;
  size: number;
  protected: boolean;
}

function fail(error: unknown) {
  toast.show(shellT('errors.generic', { message: errorMessage(error) }));
}

class FileSizeView {
  drives = $state<Drive[]>([]);
  /** Where the view starts: a drive or a chosen folder. */
  root = $state<string | null>(null);
  /** The folder shown. */
  path = $state<string | null>(null);
  /** `null` until the folder is indexed. */
  folder = $state<Folder | null>(null);
  largest = $state<LargeFile[]>([]);
  mode = $state<Mode>('folders');
  selection = $state<Selection | null>(null);
  confirming = $state(false);
  /** Set after the first answer, so the scanning notice never flashes. */
  answered = $state(false);
  #run = 0;

  async start(): Promise<void> {
    try {
      this.drives = await filesizeApi.drives();
    } catch (error) {
      fail(error);
    }
    const first = this.drives[0];
    if (this.root) await this.refresh();
    else if (first) this.show(first.path);
  }

  show(root: string): void {
    this.root = root;
    this.path = root;
    this.largest = [];
    this.select(null);
    void this.refresh();
  }

  enter(path: string): void {
    this.path = path;
    this.select(null);
    void this.refresh();
  }

  setMode(mode: Mode): void {
    this.mode = mode;
    this.select(null);
    void this.refresh();
  }

  select(selection: Selection | null): void {
    this.selection = selection;
    this.confirming = false;
  }

  /** Loads the folder, and the largest files when they are shown; older answers are dropped. */
  async refresh(): Promise<void> {
    const { root, path } = this;
    if (!root || !path) return;
    const run = ++this.#run;
    try {
      const [folder, largest] = await Promise.all([
        filesizeApi.folder(path),
        this.mode === 'largest' ? filesizeApi.largest(root) : Promise.resolve(this.largest),
      ]);
      if (run !== this.#run) return;
      this.folder = folder;
      this.largest = largest;
      this.answered = true;
    } catch (error) {
      fail(error);
    }
  }

  async pickFolder(): Promise<void> {
    try {
      const folder = await filesizeApi.pickFolder();
      if (folder) this.show(folder);
    } catch (error) {
      fail(error);
    }
  }

  async rescan(): Promise<void> {
    try {
      await filesizeApi.rescan();
      toast.show(t('rescanStarted'));
    } catch (error) {
      fail(error);
    }
  }

  async reveal(path: string): Promise<void> {
    try {
      await filesizeApi.reveal(path);
    } catch (error) {
      fail(error);
    }
  }

  async trash(selection: Selection): Promise<void> {
    this.confirming = false;
    try {
      await filesizeApi.trash(selection.path);
      toast.show(t('trashed', { name: selection.name }));
      this.select(null);
      await this.refresh();
    } catch (error) {
      fail(error);
    }
  }
}

export const filesize = new FileSizeView();
