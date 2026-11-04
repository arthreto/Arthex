import { errorMessage } from '../../core/api';
import { t as shellT } from '../../core/i18n';
import { toast } from '../../core/toast.svelte';
import { fileSearchApi } from './api';
import { t } from './i18n';

/** The "Reindex" button next to the title of the section in Settings. */
export const rescan = {
  label: () => t('settings.rescan'),
  async run() {
    try {
      await fileSearchApi.rescan();
      toast.show(t('settings.rescanStarted'));
    } catch (error) {
      toast.show(shellT('errors.generic', { message: errorMessage(error) }));
    }
  },
};
