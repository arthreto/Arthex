<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '../core/i18n';
  import Dialog from './Dialog.svelte';
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
</script>

<Dialog labelledby={titleId} {width} {onclose}>
  <header>
    <h2 id={titleId}>{title}</h2>
    <IconButton icon="close" label={t('common.close')} onclick={onclose} />
  </header>
  {@render children()}
  {#if actions}
    <footer>{@render actions()}</footer>
  {/if}
</Dialog>

<style>
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
