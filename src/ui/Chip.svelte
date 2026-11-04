<script lang="ts">
  import Icon from './Icon.svelte';

  /** A small labelled pill; with `onremove`, it ends with a ✕ button named by `removeLabel`. */
  let { label, removeLabel, onremove }: { label: string; removeLabel?: string; onremove?: () => void } =
    $props();
</script>

<span class="chip" class:removable={onremove}>
  {label}
  {#if onremove}
    <button type="button" aria-label={removeLabel} title={removeLabel} onclick={onremove}>
      <Icon name="close" size={11} stroke={2.6} />
    </button>
  {/if}
</span>

<style>
  .chip {
    height: 28px;
    padding: 0 10px;
    border-radius: var(--radius-pill);
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12.5px;
    color: var(--text);
  }

  .removable {
    padding-right: 4px;
  }

  button {
    width: 20px;
    height: 20px;
    border-radius: var(--radius-pill);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    transition:
      background-color var(--motion-fast),
      color var(--motion-fast);
  }

  button:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
</style>
