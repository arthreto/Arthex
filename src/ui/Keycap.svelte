<script lang="ts">
  import type { Snippet } from 'svelte';

  let { size = 's', children }: { size?: 's' | 'm' | 'l'; children: Snippet } = $props();
</script>

<kbd class={size}>{@render children()}</kbd>

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

  .m {
    height: 28px;
    padding: 0 10px;
    border-radius: 7px;
    font-size: 13px;
    color: var(--text);
  }

  .l {
    min-width: 88px;
    height: 52px;
    padding: 0 18px;
    border-radius: 12px;
    border-bottom-width: 4px;
    font-size: 18px;
    color: var(--text);
  }
</style>
