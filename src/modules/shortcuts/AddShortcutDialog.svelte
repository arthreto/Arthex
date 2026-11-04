<script lang="ts">
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import Modal from '../../ui/Modal.svelte';
  import TextField from '../../ui/TextField.svelte';
  import { t } from './i18n';
  import { shortcuts } from './shortcuts.svelte';
  import { guessName, siteProblem } from './site';

  let { onclose }: { onclose: () => void } = $props();

  const formId = $props.id();
  let address = $state('');
  let name = $state('');
  let error = $state<string | null>(null);
  let saving = $state(false);

  const placeholder = $derived(guessName(address) || 'YouTube');

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    const problem = siteProblem(address);
    if (problem) {
      error = t(`errors.${problem}`);
      return;
    }
    saving = true;
    try {
      await shortcuts.addSite(address, name.trim() || null);
      toast.show(t('added', { name: name.trim() || guessName(address) }));
      onclose();
    } catch (failure) {
      error = shellT('errors.generic', { message: errorMessage(failure) });
    } finally {
      saving = false;
    }
  }
</script>

<Modal title={t('dialog.title')} {onclose}>
  <form id={formId} class="fields" onsubmit={submit} oninput={() => (error = null)}>
    <TextField label={t('dialog.address')} bind:value={address} placeholder="youtube.com" autofocus />
    <TextField label={t('dialog.name')} bind:value={name} {placeholder} />
  </form>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
  {#snippet actions()}
    <Button onclick={onclose}>{shellT('common.cancel')}</Button>
    <Button variant="primary" type="submit" form={formId} disabled={saving}>{t('dialog.submit')}</Button>
  {/snippet}
</Modal>

<style>
  .fields {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .error {
    margin-top: -6px;
    font-size: 13px;
    color: var(--danger-text);
  }
</style>
