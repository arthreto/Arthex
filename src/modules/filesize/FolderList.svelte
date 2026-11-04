<script lang="ts">
  import { formatBytes, formatPercent } from '../../core/format';
  import FileIcon from '../../ui/FileIcon.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import type { Folder, Item } from './api';
  import { t } from './i18n';

  interface Props {
    folder: Folder;
    selected: string | null;
    onselect: (item: Item) => void;
    onenter: (item: Item) => void;
  }

  let { folder, selected, onselect, onenter }: Props = $props();

  const largest = $derived(folder.items[0]?.size || 1);
</script>

<div class="list scroll">
  {#each folder.items as item (item.path)}
    <div class="row" class:on={selected === item.path}>
      <button
        class="item"
        aria-pressed={selected === item.path}
        onclick={() => onselect(item)}
        ondblclick={() => item.hasChildren && onenter(item)}
      >
        <FileIcon folder={item.isDir} size={16} />
        <span class="name">{item.name}</span>
        <span class="bar"
          ><span
            style:width="{Math.max(1.5, (item.size / largest) * 100)}%"
            style:background="var(--category-{item.category})"
          ></span></span
        >
        <span class="size">{formatBytes(item.size)}</span>
        <span class="share">{formatPercent(folder.size ? item.size / folder.size : 0)}</span>
      </button>
      {#if item.hasChildren}
        <IconButton
          icon="chevronRight"
          size={28}
          label={t('openFolder', { name: item.name })}
          onclick={() => onenter(item)}
        />
      {:else}
        <span class="gap"></span>
      {/if}
    </div>
  {/each}
</div>

<style>
  .list {
    flex: 1;
    min-height: 0;
    padding-right: 8px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .row {
    display: flex;
    align-items: center;
    border-radius: 8px;
    transition: background-color var(--motion-fast);
  }

  .row:hover {
    background: var(--surface-hover);
  }

  .row.on {
    background: var(--surface-selected);
  }

  .item {
    flex: 1;
    min-width: 0;
    height: 32px;
    padding: 0 4px 0 9px;
    display: flex;
    align-items: center;
    gap: 9px;
    text-align: left;
  }

  .name {
    width: 170px;
    flex: none;
    font-size: 13.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .bar {
    flex: 1;
    min-width: 0;
    height: 7px;
    border-radius: 4px;
    background: var(--surface-raised);
    overflow: hidden;
  }

  .bar span {
    display: block;
    height: 100%;
    border-radius: 4px;
  }

  .size,
  .share {
    flex: none;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .size {
    width: 66px;
    font-size: 13px;
  }

  .share {
    width: 52px;
    font-size: 12px;
    color: var(--text-subtle);
  }

  .gap {
    width: 28px;
    flex: none;
  }
</style>
