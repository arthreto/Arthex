<script lang="ts">
  import type { Snippet } from 'svelte';

  /** `list` tightens the padding for a stack of rows that bring their own spacing. */
  let { list = false, children }: { list?: boolean; children: Snippet } = $props();
</script>

<section class="panel" class:list>{@render children()}</section>

<style>
  .panel {
    padding: 16px 18px;
    border-radius: 14px;
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .list {
    padding: 6px 18px;
  }
</style>
