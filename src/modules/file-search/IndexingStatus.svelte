<script lang="ts">
  import { onMount } from 'svelte';
  import { formatCount } from '../../core/format';
  import ProgressCard from '../../ui/ProgressCard.svelte';
  import { t } from './i18n';
  import { indexing } from './indexing.svelte';

  onMount(() => indexing.watch());
</script>

{#if indexing.current}
  {@const building = indexing.current}
  <div class="status fade-in">
    <ProgressCard
      title={t('indexing.title')}
      detail={t('indexing.items', { count: formatCount(building.items) })}
      percent={building.percent}
    />
  </div>
{/if}

<style>
  .status {
    margin: 4px 0 6px;
  }
</style>
