<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '../core/i18n';
  import { focusTrap } from './focusTrap';

  interface Props {
    /** Id of the element that names the dialog, usually its title. */
    labelledby: string;
    width: number;
    /** Fixed height for full sheets such as "What's new" and tools. */
    height?: number;
    onclose: () => void;
    children: Snippet;
  }

  let { labelledby, width, height, onclose, children }: Props = $props();

  function onkeydown(event: KeyboardEvent) {
    if (event.key !== 'Escape') return;
    event.stopPropagation();
    onclose();
  }

  /**
   * Out of the content it covers, which `base.css` blurs: `backdrop-filter` leaves the text sharp in
   * the transparent window. Moving an element loses the focus, so it is given back, and Svelte no
   * longer finds it to remove it.
   */
  function moveToLayer(scrim: HTMLElement) {
    const focused = document.activeElement;
    document.querySelector('.dialog-layer')?.append(scrim);
    if (focused instanceof HTMLElement) focused.focus();
    return { destroy: () => scrim.remove() };
  }
</script>

<div class="scrim fade-in" use:moveToLayer>
  <button class="backdrop" aria-label={t('common.close')} tabindex="-1" onclick={onclose}></button>
  <div
    class="dialog enter"
    class:sheet={height !== undefined}
    role="dialog"
    use:focusTrap
    aria-modal="true"
    aria-labelledby={labelledby}
    tabindex="-1"
    style:width="min({width}px, calc(100vw - 48px))"
    style:height={height === undefined ? undefined : `min(${height}px, calc(100vh - 48px))`}
    {onkeydown}
  >
    {@render children()}
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
    background: var(--dialog-scrim);
    backdrop-filter: blur(34px) saturate(120%);
  }

  .backdrop {
    position: absolute;
    inset: 0;
    cursor: default;
  }

  .dialog {
    position: relative;
    padding: 26px 28px 24px;
    border-radius: 24px;
    background: var(--dialog-surface);
    border: 1px solid var(--dialog-border);
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .dialog:focus-visible {
    outline: none;
  }

  .sheet {
    padding: 30px 36px;
    gap: 26px;
  }
</style>
