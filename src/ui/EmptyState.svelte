<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  interface Props {
    /** Raw path data of the pictogram. */
    icon: string;
    /** A colour token, such as the tint of the module. */
    tint?: string;
    title: string;
    text?: string;
    children?: Snippet;
  }

  let { icon, tint = 'var(--accent)', title, text, children }: Props = $props();
</script>

<div class="empty fade-in" role="status">
  <span class="tile" style:color={tint}><Icon path={icon} size={24} /></span>
  <p class="title">{title}</p>
  {#if text}<p class="text">{text}</p>{/if}
  {#if children}<div class="actions">{@render children()}</div>{/if}
</div>

<style>
  .empty {
    min-height: 340px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    text-align: center;
  }

  .tile {
    width: 56px;
    height: 56px;
    margin-bottom: 6px;
    border-radius: 16px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .title {
    font-size: 16px;
    font-weight: 600;
  }

  .text {
    max-width: 360px;
    font-size: 13px;
    line-height: 1.5;
    color: var(--text-subtle);
  }

  .actions {
    margin-top: 8px;
  }
</style>
