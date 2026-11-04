<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '../core/i18n';

  interface Props {
    /** Where the menu opens, in window coordinates. */
    x: number;
    y: number;
    label: string;
    onclose: () => void;
    children: Snippet;
  }

  let { x, y, label, onclose, children }: Props = $props();

  let menu: HTMLDivElement | undefined = $state();
  const left = $derived(Math.max(8, Math.min(x, window.innerWidth - 208)));
  const top = $derived(Math.max(8, Math.min(y, window.innerHeight - 160)));

  const items = () => [...(menu?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])];

  $effect(() => {
    items()[0]?.focus();
  });

  function onkeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.stopPropagation();
      onclose();
    } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      const list = items();
      const index = list.indexOf(document.activeElement as HTMLElement);
      const step = event.key === 'ArrowDown' ? 1 : -1;
      list[(index + step + list.length) % list.length]?.focus();
    }
  }

  function closeOnRightClick(event: MouseEvent) {
    event.preventDefault();
    onclose();
  }
</script>

<button
  class="backdrop"
  tabindex="-1"
  aria-label={t('common.close')}
  onclick={onclose}
  oncontextmenu={closeOnRightClick}
></button>
<div
  class="menu fade-in"
  role="menu"
  tabindex="-1"
  aria-label={label}
  bind:this={menu}
  style:left="{left}px"
  style:top="{top}px"
  {onkeydown}
>
  {@render children()}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 25;
    cursor: default;
  }

  .menu {
    position: fixed;
    z-index: 26;
    width: 200px;
    padding: 6px;
    border-radius: 10px;
    background: var(--surface-dialog);
    border: 1px solid var(--border-strong);
    box-shadow: var(--shadow-dialog);
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
</style>
