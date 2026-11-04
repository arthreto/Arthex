<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface Props extends HTMLButtonAttributes {
    children: Snippet;
  }

  let { type = 'button', children, ...rest }: Props = $props();
</script>

<button class="link" {type} {...rest}>{@render children()}</button>

<style>
  .link {
    align-self: flex-start;
    padding: 2px 0;
    font-size: 13px;
    color: var(--accent);
  }

  .link:hover {
    text-decoration: underline;
  }
</style>
