import { t } from './i18n';
import { toast } from './toast.svelte';

/** Copies a path and says so. */
export async function copyPath(path: string): Promise<void> {
  await navigator.clipboard.writeText(path);
  toast.show(t('search.pathCopied', { path }));
}
