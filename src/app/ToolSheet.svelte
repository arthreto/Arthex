<script lang="ts">
  import { moduleText, t } from '../core/i18n';
  import type { ModuleManifest } from '../core/modules';
  import Icon from '../ui/Icon.svelte';
  import Dialog from '../ui/Dialog.svelte';
  import IconButton from '../ui/IconButton.svelte';
  import { loadTool } from './pages';

  let { module, input, onclose }: { module: ModuleManifest; input?: string; onclose: () => void } = $props();

  const titleId = $props.id();
</script>

<Dialog labelledby={titleId} width={920} height={640} {onclose}>
  <header>
    <span class="icon" style:color="var(--tint-{module.tint})">
      <Icon path={module.icon} size={22} />
    </span>
    <h2 id={titleId}>{moduleText(module.id, 'name')}</h2>
    <IconButton icon="close" label={t('common.close')} onclick={onclose} />
  </header>
  <div class="body">
    {#await loadTool(module.id) then loaded}
      {#if loaded}
        <loaded.default {input} />
      {/if}
    {/await}
  </div>
</Dialog>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .icon {
    width: 44px;
    height: 44px;
    flex: none;
    border-radius: 13px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  h2 {
    flex: 1;
    min-width: 0;
    font-size: 24px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
</style>
