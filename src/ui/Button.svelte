<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface Props extends HTMLButtonAttributes {
    variant?: 'default' | 'primary' | 'danger';
    children: Snippet;
  }

  let { variant = 'default', type = 'button', children, ...rest }: Props = $props();
</script>

<button class="button {variant}" {type} {...rest}>{@render children()}</button>

<style>
  .button {
    height: 36px;
    padding: 0 16px;
    border-radius: 10px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    flex: none;
    white-space: nowrap;
    font-size: 13px;
    font-weight: 500;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    transition: background-color var(--motion-fast);
  }

  .button:hover:not(:disabled) {
    background: var(--surface-hover);
  }

  .primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
  }

  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .danger {
    background: var(--danger);
    border-color: var(--danger);
    color: var(--on-danger);
  }

  .danger:hover:not(:disabled) {
    background: var(--danger-hover);
  }

  .button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
