<script lang="ts">
  /** A pill that is on or off, for filters such as the catalog categories. */
  let { label, pressed, onclick }: { label: string; pressed: boolean; onclick: () => void } = $props();
</script>

<button type="button" class="chip" class:on={pressed} aria-pressed={pressed} {onclick}>{label}</button>

<style>
  .chip {
    height: 28px;
    padding: 0 12px;
    border-radius: var(--radius-pill);
    background: var(--surface);
    border: 1px solid var(--border-strong);
    font-size: 12.5px;
    white-space: nowrap;
    color: var(--text-muted);
  }

  .chip:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .on,
  .on:hover {
    background: var(--surface-selected);
    border-color: var(--accent);
    color: var(--text);
  }
</style>
