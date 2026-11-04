<script lang="ts">
  import { formatBytes } from '../../core/format';
  import type { Item } from './api';
  import { squarify } from './treemap';

  interface Props {
    items: Item[];
    selected: string | null;
    onselect: (item: Item) => void;
    onenter: (item: Item) => void;
  }

  let { items, selected, onselect, onenter }: Props = $props();

  let width = $state(0);
  let height = $state(0);
  /** Tiles are inset so neighbours never touch. */
  const GAP = 3;
  const tiles = $derived(squarify(items, (item) => item.size, width, height));
</script>

<div class="treemap" bind:clientWidth={width} bind:clientHeight={height}>
  {#each tiles as tile (tile.item.path)}
    {@const w = Math.max(0, tile.width - GAP)}
    {@const h = Math.max(0, tile.height - GAP)}
    <button
      class:on={selected === tile.item.path}
      aria-label="{tile.item.name}, {formatBytes(tile.item.size)}"
      style:left="{tile.x + GAP / 2}px"
      style:top="{tile.y + GAP / 2}px"
      style:width="{w}px"
      style:height="{h}px"
      style:background="var(--category-{tile.item.category})"
      style:color="var(--category-{tile.item.category}-ink)"
      onclick={() => onselect(tile.item)}
      ondblclick={() => tile.item.hasChildren && onenter(tile.item)}
    >
      {#if w > 56 && h > 26}<span class="label">{tile.item.name}</span>{/if}
      {#if w > 66 && h > 46}<span class="size">{formatBytes(tile.item.size)}</span>{/if}
    </button>
  {/each}
</div>

<style>
  .treemap {
    position: relative;
    flex: 1;
    min-height: 0;
    border-radius: 10px;
    overflow: hidden;
    background: var(--surface-sunken);
  }

  button {
    position: absolute;
    padding: 7px 9px;
    border-radius: 5px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    overflow: hidden;
    text-align: left;
    transition: filter var(--motion-fast);
  }

  button:hover {
    filter: brightness(1.12);
  }

  button.on {
    box-shadow:
      inset 0 0 0 2px var(--text),
      inset 0 0 0 4px var(--scrim);
  }

  .label {
    max-width: 100%;
    font-size: 12.5px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .size {
    font-size: 12px;
    opacity: 0.85;
  }
</style>
