<script lang="ts">
  import { t } from '../../core/i18n';
  import Button from '../../ui/Button.svelte';
  import Check from '../../ui/Check.svelte';
  import Modal from '../../ui/Modal.svelte';

  let {
    name,
    onconfirm,
    onclose,
  }: { name: string; onconfirm: (deleteData: boolean) => void; onclose: () => void } = $props();

  let deleteData = $state(false);
</script>

<Modal title={t('settings.modules.uninstallTitle', { name })} {onclose}>
  <button
    type="button"
    class="option"
    role="checkbox"
    aria-checked={deleteData}
    onclick={() => (deleteData = !deleteData)}
  >
    <Check checked={deleteData} />
    {t('settings.modules.deleteData')}
  </button>
  {#snippet actions()}
    <Button onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="danger" onclick={() => onconfirm(deleteData)}>{t('settings.modules.uninstall')}</Button>
  {/snippet}
</Modal>

<style>
  .option {
    align-self: flex-start;
    margin-left: -8px;
    padding: 6px 8px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 14px;
  }

  .option:hover {
    background: var(--surface-hover);
  }
</style>
