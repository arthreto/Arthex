import { commands, errorMessage, events, on, type UpdateProgress, type WhatsNew } from '../core/api';
import { t } from '../core/i18n';
import { toast } from '../core/toast.svelte';

export type UpdateStatus = 'idle' | 'checking' | 'available' | 'installing' | 'unavailable';

class Updates {
  status = $state<UpdateStatus>('idle');
  /** The version found by the last check. */
  version = $state<string | null>(null);
  progress = $state<UpdateProgress | null>(null);
  /** Release notes to show once, after an update. */
  whatsNew = $state<WhatsNew | null>(null);

  constructor() {
    void on<{ version: string }>(events.updateAvailable, (found) => {
      this.version = found.version;
      this.status = 'available';
    });
    void on<UpdateProgress>(events.updateProgress, (progress) => (this.progress = progress));
  }

  get busy(): boolean {
    return this.status === 'checking' || this.status === 'installing';
  }

  async loadWhatsNew(): Promise<void> {
    this.whatsNew = await commands.updatesWhatsNew().catch(() => null);
  }

  async check(): Promise<void> {
    if (this.busy) return;
    this.status = 'checking';
    try {
      const found = await commands.updatesCheck();
      this.version = found?.version ?? null;
      this.status = found ? 'available' : 'idle';
    } catch (error) {
      if (typeof error === 'object' && error !== null && 'kind' in error && error.kind === 'unsupported') {
        this.status = 'unavailable';
        return;
      }
      this.status = 'idle';
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }

  async install(): Promise<void> {
    this.status = 'installing';
    this.progress = null;
    try {
      await commands.updatesInstall();
    } catch (error) {
      this.status = 'available';
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }
}

export const updates = new Updates();
