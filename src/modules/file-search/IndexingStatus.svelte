<script lang="ts">
  import { onMount } from 'svelte';
  import { formatCount, formatPercent } from '../../core/format';
  import FileIcon from '../../ui/FileIcon.svelte';
  import { t } from './i18n';
  import { indexing } from './indexing.svelte';

  onMount(() => indexing.watch());
</script>

{#if indexing.current}
  {@const building = indexing.current}
  <div class="status fade-in" role="status">
    <div class="line">
      <FileIcon folder size={16} />
      <span class="title">{t('indexing.title')}</span>
      <span class="count">{t('indexing.items', { count: formatCount(building.items) })}</span>
      {#if building.percent !== null}
        <span class="percent">{formatPercent(building.percent / 100, 0)}</span>
      {/if}
    </div>
    {#if building.percent !== null}
      <div class="track"><div class="bar" style:width="{building.percent}%"></div></div>
    {/if}
  </div>
{/if}

<style>
  .status {
    margin: 4px 0 6px;
    padding: 12px 14px;
    border-radius: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 9px;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
  }

  .title {
    font-weight: 600;
  }

  .count {
    color: var(--text-subtle);
  }

  .percent {
    margin-left: auto;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .track {
    height: 4px;
    border-radius: 2px;
    background: var(--border);
    overflow: hidden;
  }

  .bar {
    height: 100%;
    border-radius: 2px;
    background: var(--accent);
    transition: width 400ms var(--ease-out);
  }
</style>
