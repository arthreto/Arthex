<script lang="ts">
  import Icon from '../../ui/Icon.svelte';
  import type { Shortcut } from './api';
  import { symbolPath } from './symbols';

  let { shortcut, size }: { shortcut: Pick<Shortcut, 'name' | 'icon' | 'symbol'>; size: number } = $props();
</script>

{#if shortcut.symbol}
  <span
    class="tile"
    style:width="{size}px"
    style:height="{size}px"
    style:color="var(--tint-{shortcut.symbol.tint})"
  >
    <Icon path={symbolPath(shortcut.symbol.name)} size={Math.round(size * 0.52)} />
  </span>
{:else if shortcut.icon}
  <img src={shortcut.icon} alt="" width={size} height={size} />
{:else}
  <span
    class="tile letter"
    style:width="{size}px"
    style:height="{size}px"
    style:font-size="{Math.round(size * 0.36)}px">{shortcut.name.charAt(0).toUpperCase()}</span
  >
{/if}

<style>
  .tile {
    flex: none;
    border-radius: 28%;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .letter {
    font-weight: 700;
    color: var(--text-muted);
  }

  img {
    flex: none;
    object-fit: contain;
    padding: 6%;
  }
</style>
