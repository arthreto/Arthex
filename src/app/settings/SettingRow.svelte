<script lang="ts">
  import type { Snippet } from 'svelte';

  let { label, children }: { label: string; children: Snippet } = $props();
</script>

<div class="row">
  <span class="label">{label}</span>
  {@render children()}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 16px;
    min-height: 50px;
    padding: 8px 0;
  }

  .row + :global(.row) {
    border-top: 1px solid var(--border);
  }

  .label {
    flex: 1;
    min-width: 0;
    font-size: 13.5px;
  }
</style>
