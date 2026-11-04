<script lang="ts">
  import Icon from './Icon.svelte';

  /** The look of a checkbox; the control around it carries the role and the state. */
  let { checked, size = 18 }: { checked: boolean; size?: number } = $props();
</script>

<span class="check" class:checked style:width="{size}px" style:height="{size}px" aria-hidden="true">
  {#if checked}<Icon name="check" size={Math.round(size * 0.67)} stroke={3.5} />{/if}
</span>

<style>
  .check {
    flex: none;
    border-radius: 5px;
    border: 1.5px solid var(--border-strong);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--on-accent);
    transition:
      background-color var(--motion-fast),
      border-color var(--motion-fast);
  }

  .checked {
    background: var(--accent);
    border-color: var(--accent);
  }
</style>
