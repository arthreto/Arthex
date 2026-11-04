<script lang="ts">
  import Icon from '../../ui/Icon.svelte';

  let { on, size = 24 }: { on: boolean; size?: number } = $props();
</script>

<span class="tick" class:on style:width="{size}px" style:height="{size}px">
  {#if on}<Icon name="check" size={Math.round(size * 0.55)} stroke={3.2} />{/if}
</span>

<style>
  .tick {
    flex: none;
    border-radius: 50%;
    border: 2px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--on-accent);
  }

  .on {
    background: var(--accent);
    border-color: var(--accent);
  }
</style>
