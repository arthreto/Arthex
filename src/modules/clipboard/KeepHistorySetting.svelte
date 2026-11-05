<script lang="ts">
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { settings } from '../../core/settings/settings.svelte';
  import { toast } from '../../core/toast.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Switch from '../../ui/Switch.svelte';
  import { t } from './i18n';

  async function setKeep(keepClipboard: boolean) {
    try {
      await settings.update({ keepClipboard });
    } catch (error) {
      toast.show(shellT('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

<Panel list>
  <div class="row">
    <span>{t('keep')}</span>
    <Switch label={t('keep')} checked={settings.value.keepClipboard} onchange={setKeep} />
  </div>
</Panel>

<style>
  .row {
    padding: 12px 0;
    display: flex;
    align-items: center;
    gap: 14px;
  }

  span {
    flex: 1;
    min-width: 0;
    font-size: 14px;
    font-weight: 500;
  }
</style>
