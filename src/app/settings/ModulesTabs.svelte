<script lang="ts">
  import { t } from '../../core/i18n';
  import IconButton from '../../ui/IconButton.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import { modulesView } from './modulesView.svelte';

  const tabs = $derived([
    { value: 'configuration' as const, label: t('settings.modules.tabs.configuration') },
    { value: 'catalog' as const, label: t('settings.modules.tabs.catalog') },
  ]);
</script>

<div class="controls">
  <span class="refresh" class:spinning={modulesView.refreshing}>
    <IconButton
      icon="refresh"
      label={t('settings.modules.refresh')}
      disabled={modulesView.refreshing || modulesView.resting}
      onclick={() => modulesView.refresh()}
    />
  </span>
  <Segmented
    label={t('settings.sections.modules')}
    options={tabs}
    value={modulesView.tab}
    onchange={(tab) => (modulesView.tab = tab)}
  />
</div>

<style>
  .controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .refresh {
    display: flex;
  }

  .refresh :global(button:disabled) {
    opacity: 0.35;
    cursor: default;
  }

  .spinning :global(svg) {
    animation: spin 800ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
