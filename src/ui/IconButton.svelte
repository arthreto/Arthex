<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';
  import type { IconName } from './icons';

  interface Props extends HTMLButtonAttributes {
    /** A shell icon name, or raw path data with `path` (module icons). */
    icon?: IconName;
    path?: string;
    label: string;
    size?: number;
    active?: boolean;
  }

  let { icon, path, label, size = 32, active = false, type = 'button', ...rest }: Props = $props();
</script>

<button
  class="icon-button"
  class:active
  {type}
  aria-label={label}
  title={label}
  style:width="{size}px"
  style:height="{size}px"
  {...rest}
>
  <Icon name={icon} {path} size={Math.round(size * 0.52)} />
</button>

<style>
  .icon-button {
    flex: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-pill);
    color: var(--text-muted);
    transition:
      background-color var(--motion-fast),
      color var(--motion-fast);
  }

  .icon-button:hover,
  .active {
    background: var(--surface-hover);
    color: var(--text);
  }
</style>
