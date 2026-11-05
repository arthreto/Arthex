<script lang="ts">
  import { moduleText, t } from '../core/i18n';
  import type { ModuleManifest } from '../core/modules';
  import Icon from '../ui/Icon.svelte';
  import IconButton from '../ui/IconButton.svelte';
  import { focusTrap } from '../ui/focusTrap';
  import { loadTool } from './pages';

  let { module, input, onclose }: { module: ModuleManifest; input?: string; onclose: () => void } = $props();

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
    use:focusTrap
    aria-modal="true"
    aria-labelledby={titleId}
    tabindex="-1"
    {onkeydown}
  >
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
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 30;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--scrim);
    backdrop-filter: blur(22px) saturate(120%);
  }

  .backdrop {
    position: absolute;
    inset: 0;
    cursor: default;
  }

  .dialog {
    position: relative;
    width: min(920px, calc(100vw - 48px));
    height: min(640px, calc(100vh - 48px));
    padding: 30px 36px 34px;
    border-radius: 26px;
    display: flex;
    flex-direction: column;
    gap: 26px;
    background: var(--surface-dialog);
    border: 1px solid var(--border-strong);
    box-shadow: var(--shadow-dialog);
  }

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
