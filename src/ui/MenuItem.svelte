<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  import type { IconName } from './icons';

  interface Props {
    icon: IconName;
    danger?: boolean;
    onclick: () => void;
    children: Snippet;
  }

  let { icon, danger = false, onclick, children }: Props = $props();
</script>

<button role="menuitem" class="item" class:danger {onclick}>
  <Icon name={icon} size={15} stroke={1.9} />
  <span>{@render children()}</span>
</button>

<style>
  .item {
    width: 100%;
    height: 34px;
    padding: 0 10px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    text-align: left;
    outline: none;
  }

  .item:hover,
  .item:focus-visible {
    background: var(--surface-hover);
  }

  .danger {
    color: var(--danger-text);
  }
</style>
