<script lang="ts">
  import { t } from '../../core/i18n';
  import Segmented from '../../ui/Segmented.svelte';
  import { modulesView } from './modulesView.svelte';

  const tabs = $derived([
    { value: 'installed' as const, label: t('settings.modules.tabs.installed') },
    { value: 'catalog' as const, label: t('settings.modules.tabs.catalog') },
  ]);
</script>

<Segmented
  label={t('settings.sections.modules')}
  options={tabs}
  value={modulesView.tab}
  onchange={(tab) => (modulesView.tab = tab)}
/>
