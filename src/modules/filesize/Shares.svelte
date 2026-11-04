<script lang="ts">
  import { formatPercent } from '../../core/format';
  import type { Folder } from './api';
  import { t } from './i18n';

  let { folder }: { folder: Folder } = $props();

  const shares = $derived(
    folder.shares.map((share) => ({ ...share, part: folder.size ? share.size / folder.size : 0 })),
  );
</script>

<div class="bar" aria-hidden="true">
  {#each shares as share (share.category)}
    <span style:width="{share.part * 100}%" style:background="var(--category-{share.category})"></span>
  {/each}
</div>
<ul class="legend">
  {#each shares as share (share.category)}
    <li>
      <span class="swatch" style:background="var(--category-{share.category})"></span>
      {t(`categories.${share.category}`)}
      <span class="part">{formatPercent(Math.max(0.01, share.part), 0)}</span>
    </li>
  {/each}
</ul>

<style>
  .bar {
    height: 10px;
    flex: none;
    border-radius: 5px;
    overflow: hidden;
    display: flex;
    gap: 2px;
  }

  .bar span {
    height: 100%;
  }

  .legend {
    min-height: 20px;
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    gap: 4px 16px;
    font-size: 12.5px;
    color: var(--text-muted);
  }

  li {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .swatch {
    width: 9px;
    height: 9px;
    border-radius: 2px;
  }

  .part {
    color: var(--text-subtle);
  }
</style>
