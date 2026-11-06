<script lang="ts">
  import { formatCount } from '../../core/format';
  import ProgressCard from '../../ui/ProgressCard.svelte';
  import Skeleton from '../../ui/Skeleton.svelte';
  import { filesize } from './filesize.svelte';
  import { t } from './i18n';

  const ROWS = [
    { name: 96, bar: 100, size: 50 },
    { name: 124, bar: 72, size: 46 },
    { name: 80, bar: 54, size: 50 },
    { name: 140, bar: 38, size: 42 },
    { name: 104, bar: 26, size: 46 },
    { name: 70, bar: 18, size: 40 },
    { name: 116, bar: 12, size: 44 },
    { name: 92, bar: 8, size: 38 },
    { name: 132, bar: 5, size: 40 },
  ];
  const SHARES = [84, 66, 92, 58, 74];
  const BLOCKS = [
    { column: '1 / 6', row: '1 / 7' },
    { column: '6 / 10', row: '1 / 5' },
    { column: '10 / 13', row: '1 / 5' },
    { column: '6 / 9', row: '5 / 7' },
    { column: '9 / 11', row: '5 / 7' },
    { column: '11 / 13', row: '5 / 7' },
  ];

  const building = $derived(filesize.building);
</script>

<div class="scanning fade-in" aria-busy="true">
  <div class="side">
    <ProgressCard
      title={t('scanning')}
      detail={building ? t('items', { count: formatCount(building.items) }) : undefined}
      percent={building?.percent ?? null}
      shine
    />
    {#each ROWS as row (row.name)}
      <div class="row">
        <Skeleton width="16px" height="16px" radius="4px" />
        <span class="name"><Skeleton width="{row.name}px" height="12px" /></span>
        <span class="bar"><Skeleton width="{row.bar}%" height="6px" radius="3px" /></span>
        <Skeleton width="{row.size}px" height="12px" />
      </div>
    {/each}
  </div>
  <div class="map">
    <div class="shares">
      {#each SHARES as width (width)}
        <Skeleton width="{width}px" height="14px" radius="7px" />
      {/each}
    </div>
    <div class="blocks">
      {#each BLOCKS as block (block.column + block.row)}
        <div style:grid-column={block.column} style:grid-row={block.row}>
          <Skeleton width="100%" height="100%" radius="8px" />
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .scanning {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 20px;
  }

  .side {
    width: 470px;
    flex: none;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .row {
    height: 32px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 0 9px;
  }

  .name {
    width: 170px;
    flex: none;
  }

  .bar {
    flex: 1;
    min-width: 0;
  }

  .map {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .shares {
    height: 24px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .blocks {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: repeat(12, 1fr);
    grid-template-rows: repeat(6, 1fr);
    gap: 3px;
  }
</style>
