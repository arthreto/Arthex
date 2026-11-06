<script lang="ts">
  /** A pill that is on or off, for filters such as the catalog categories. */
  interface Props {
    label: string;
    /** Quieter, after the label: the size of a quality. */
    detail?: string;
    pressed: boolean;
    disabled?: boolean;
    onclick: () => void;
  }

  let { label, detail, pressed, disabled = false, onclick }: Props = $props();
</script>

<button type="button" class="chip" class:on={pressed} aria-pressed={pressed} {disabled} {onclick}>
  {label}{#if detail}<span class="detail">{detail}</span>{/if}
</button>

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

  .chip:hover:enabled {
    background: var(--surface-hover);
    color: var(--text);
  }

  .chip:disabled {
    cursor: not-allowed;
  }

  .detail {
    margin-left: 7px;
    color: var(--text-subtle);
  }

  .on,
  .on:hover:enabled {
    background: var(--surface-selected);
    border-color: var(--accent);
    color: var(--text);
  }
</style>
