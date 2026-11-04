<script lang="ts">
  import { onMount } from 'svelte';
  import { commands, errorMessage } from '../../core/api';
  import { t } from '../../core/i18n';
  import { settings } from '../../core/settings/settings.svelte';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import Icon from '../../ui/Icon.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Switch from '../../ui/Switch.svelte';
  import { updates } from '../updates.svelte';
  import SettingRow from './SettingRow.svelte';

  let current = $state('');

  onMount(() => {
    commands
      .appInfo()
      .then((info) => (current = info.version))
      .catch(() => {});
  });

  const share = $derived(
    updates.progress?.total ? Math.min(1, updates.progress.downloaded / updates.progress.total) : null,
  );

  async function setAuto(autoUpdate: boolean) {
    try {
      await settings.update({ autoUpdate });
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

<Panel list>
  <div class="status">
    {#if updates.status === 'checking'}
      <span class="spinner" aria-hidden="true"></span>
      <span>{t('settings.updates.checking')}</span>
    {:else if updates.status === 'available'}
      <span class="dot" aria-hidden="true"></span>
      <span class="grow">{t('settings.updates.available', { version: updates.version ?? '' })}</span>
      <Button variant="primary" size="small" onclick={() => updates.install()}
        >{t('settings.updates.install')}</Button
      >
    {:else if updates.status === 'installing'}
      <span class="installing">
        {t('settings.updates.installing', { version: updates.version ?? '' })}
        <span
          class="track"
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={share === null ? undefined : Math.round(share * 100)}
          ><span
            class:indeterminate={share === null}
            style:width={share === null ? undefined : `${share * 100}%`}
          ></span></span
        >
      </span>
    {:else if updates.status === 'unavailable'}
      <span>{t('settings.updates.unavailable')}</span>
    {:else}
      <span class="ok"><Icon name="check" size={16} stroke={2.6} /></span>
      <span>{t('settings.updates.upToDate', { version: current })}</span>
    {/if}
  </div>
  <SettingRow label={t('settings.updates.auto')}>
    <Switch label={t('settings.updates.auto')} checked={settings.value.autoUpdate} onchange={setAuto} />
  </SettingRow>
</Panel>

<style>
  .status {
    min-height: 50px;
    padding: 6px 0;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13.5px;
    border-bottom: 1px solid var(--border);
  }

  .grow {
    flex: 1;
  }

  .ok {
    display: flex;
    color: var(--success);
  }

  .dot {
    width: 8px;
    height: 8px;
    flex: none;
    border-radius: 50%;
    background: var(--accent);
  }

  .spinner {
    width: 16px;
    height: 16px;
    flex: none;
    border-radius: 50%;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
  }

  .installing {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .track {
    height: 6px;
    border-radius: 3px;
    background: var(--surface-hover);
    overflow: hidden;
  }

  .track span {
    display: block;
    height: 100%;
    border-radius: 3px;
    background: var(--accent);
    transition: width var(--motion-base);
  }

  .indeterminate {
    width: 35%;
    animation: slide 1.2s var(--ease-out) infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(300%);
    }
  }
</style>
