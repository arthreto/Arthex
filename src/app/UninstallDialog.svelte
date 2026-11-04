<script lang="ts">
  import { commands, errorMessage } from '../core/api';
  import { t } from '../core/i18n';
  import { toast } from '../core/toast.svelte';
  import Button from '../ui/Button.svelte';
  import Modal from '../ui/Modal.svelte';

  let { onclose }: { onclose: () => void } = $props();

  async function uninstall() {
    try {
      await commands.appUninstall();
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
      onclose();
    }
  }
</script>

<Modal title={t('uninstall.title')} width={460} {onclose}>
  <p>{t('uninstall.body')}</p>
  {#snippet actions()}
    <Button onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="danger" onclick={uninstall}>{t('uninstall.confirm')}</Button>
  {/snippet}
</Modal>

<style>
  p {
    font-size: 14px;
    line-height: 1.55;
    color: var(--text-muted);
  }
</style>
