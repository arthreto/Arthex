<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    selected: boolean;
    /** `card`: wide option, `tile`: preview above its label, `row`: one line. */
    layout: 'card' | 'tile' | 'row';
    onselect: () => void;
    children: Snippet;
  }

  let { selected, layout, onselect, children }: Props = $props();
</script>

<button role="radio" aria-checked={selected} class="choice {layout}" class:selected onclick={onselect}>
  {@render children()}
</button>

<style>
  .choice {
    display: flex;
    align-items: center;
    background: var(--surface);
    border: 2px solid var(--border);
    transition:
      transform 200ms var(--ease-out),
      background-color 200ms,
      border-color 200ms,
      box-shadow 200ms;
  }

  .choice:hover {
    transform: translateY(-3px);
  }

  .selected {
    background: var(--surface-selected);
    border-color: var(--accent);
    box-shadow: var(--shadow-chosen);
  }

  .card {
    height: 104px;
    padding: 0 22px;
    border-radius: 22px;
    gap: 18px;
  }

  .tile {
    padding: 10px;
    border-radius: 20px;
    flex-direction: column;
    gap: 12px;
  }

  .row {
    height: 72px;
    padding: 0 22px;
    border-radius: 18px;
    gap: 16px;
  }
</style>
