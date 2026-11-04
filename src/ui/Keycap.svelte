<script lang="ts">
  import type { Snippet } from 'svelte';

  let { large = false, children }: { large?: boolean; children: Snippet } = $props();
</script>

<kbd class:large>{@render children()}</kbd>

<style>
  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 22px;
    height: 20px;
    padding: 0 6px;
    border-radius: 5px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    border-bottom-width: 2px;
    font-family: inherit;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .large {
    height: 28px;
    padding: 0 10px;
    border-radius: 7px;
    font-size: 13px;
    color: var(--text);
  }
</style>
