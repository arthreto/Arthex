<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '../core/i18n';
  import IconButton from './IconButton.svelte';

  interface Props {
    title: string;
    width?: number;
    onclose: () => void;
    children: Snippet;
    actions?: Snippet;
  }

  let { title, width = 560, onclose, children, actions }: Props = $props();
  const titleId = $props.id();

  function onkeydown(event: KeyboardEvent) {
    if (event.key !== 'Escape') return;
    event.stopPropagation();
    onclose();
  }
</script>

<div class="scrim fade-in">
  <button class="backdrop" aria-label={t('common.close')} tabindex="-1" onclick={onclose}></button>
  <div
    class="dialog enter"
    role="dialog"
    aria-modal="true"
    aria-labelledby={titleId}
    tabindex="-1"
    style:width="{width}px"
    {onkeydown}
  >
    <header>
      <h2 id={titleId}>{title}</h2>
      <IconButton icon="close" label={t('common.close')} onclick={onclose} />
    </header>
    {@render children()}
    {#if actions}
      <footer>{@render actions()}</footer>
    {/if}
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--scrim);
    backdrop-filter: blur(6px);
  }

  .backdrop {
    position: absolute;
    inset: 0;
    cursor: default;
  }

  .dialog {
    position: relative;
    max-width: calc(100vw - 48px);
    padding: 26px 28px 24px;
    border-radius: 20px;
    background: var(--surface-dialog);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-dialog);
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  h2 {
    flex: 1;
    font-size: 20px;
    font-weight: 600;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding-top: 4px;
  }
</style>
