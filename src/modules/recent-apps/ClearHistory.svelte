<script lang="ts">
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import { recentAppsApi } from './api';
  import { t } from './i18n';

  async function clear() {
    try {
      await recentAppsApi.clear();
      toast.show(t('cleared'));
    } catch (error) {
      toast.show(shellT('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

<Button size="small" onclick={clear}>{t('clear')}</Button>
