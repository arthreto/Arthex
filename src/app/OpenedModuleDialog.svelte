<script lang="ts">
  import { commands, errorMessage, type InstallRequest } from '../core/api';
  import { t } from '../core/i18n';
  import { toast } from '../core/toast.svelte';
  import Button from '../ui/Button.svelte';
  import Icon from '../ui/Icon.svelte';
  import Modal from '../ui/Modal.svelte';
  import { moduleHost } from './moduleHost.svelte';
  import { knownElements, pickTexts, summary, tintOf } from './settings/moduleSummary';

  let { request }: { request: InstallRequest } = $props();

  const name = $derived(pickTexts(request.texts)?.name ?? request.id);
  let busy = $state(false);

  async function answer(install: boolean) {
    busy = true;
    try {
      const installed = await commands.modulesAnswerOpened(install);
      if (installed) {
        await moduleHost.load();
        toast.show(t('settings.modules.installed', { name }));
      }
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    } finally {
      moduleHost.opened = null;
    }
  }
</script>

<Modal title={t('settings.modules.installTitle')} onclose={() => answer(false)}>
  <div class="module">
    <span class="icon" style:color="var(--tint-{tintOf(request.tint)})">
      <Icon path={request.icon} size={22} />
    </span>
    <span class="text">
      <span class="name">{name}</span>
      <span class="meta">{summary(knownElements(request.elements), request.size, request.version)}</span>
    </span>
  </div>
  {#snippet actions()}
    <Button disabled={busy} onclick={() => answer(false)}>{t('common.cancel')}</Button>
    <Button variant="primary" disabled={busy} onclick={() => answer(true)}>
      {t(busy ? 'settings.modules.installing' : 'settings.modules.install')}
    </Button>
  {/snippet}
</Modal>

<style>
  .module {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .icon {
    width: 44px;
    height: 44px;
    flex: none;
    border-radius: 13px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .name {
    font-size: 15px;
    font-weight: 600;
  }

  .meta {
    font-size: 12.5px;
    color: var(--text-subtle);
  }
</style>
