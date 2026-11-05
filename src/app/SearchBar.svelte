<script lang="ts">
  import { t } from '../core/i18n';
  import { settings } from '../core/settings/settings.svelte';
  import Icon from '../ui/Icon.svelte';
  import IconButton from '../ui/IconButton.svelte';
  import Keycap from '../ui/Keycap.svelte';
  import { keyLabel } from './hotkey';
  import { search } from './search.svelte';

  let input: HTMLInputElement | undefined = $state();
  /** Shown once a provider takes a moment, so instant answers never flash it. */
  let loading = $state(false);
  const LOADING_DELAY_MS = 150;

  $effect(() => {
    if (search.pending === 0 || !search.active) {
      loading = false;
      return;
    }
    const timer = setTimeout(() => (loading = true), LOADING_DELAY_MS);
    return () => clearTimeout(timer);
  });

  export function focus(): void {
    input?.focus();
    input?.select();
  }

  function onkeydown(event: KeyboardEvent) {
    if (!search.active) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      if (search.confirming) search.confirming = null;
      else search.clear();
    } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      search.move(event.key === 'ArrowDown' ? 1 : -1);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      void search.activate(search.current, event.ctrlKey);
    }
  }
</script>

<div class="search">
  <Icon name="search" size={20} stroke={2} />
  <label for="search-input" class="visually-hidden">{t('search.label')}</label>
  <input
    id="search-input"
    bind:this={input}
    bind:value={search.query}
    placeholder={t('search.placeholder')}
    autocomplete="off"
    spellcheck="false"
    {onkeydown}
  />
  {#if search.active}
    <IconButton icon="close" label={t('search.clear')} onclick={() => search.clear()} />
  {/if}
  <span class="hotkey" aria-hidden="true">
    {#each settings.value.hotkey as key, index (key)}
      {#if index > 0}<span>+</span>{/if}
      <Keycap>{keyLabel(key)}</Keycap>
    {/each}
  </span>
  {#if loading}
    <div class="loading" role="progressbar" aria-label={t('search.loading')}><span></span></div>
  {/if}
</div>

<style>
  .search {
    position: relative;
    align-self: center;
    width: min(760px, 100%);
    height: 56px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 14px 0 20px;
    border-radius: 28px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    box-shadow: var(--shadow-search);
    color: var(--text-muted);
  }

  input {
    flex: 1;
    min-width: 0;
    height: 40px;
    background: transparent;
    border: 0;
    outline: none;
    font-size: 18px;
    color: var(--text);
  }

  .loading {
    position: absolute;
    left: 26px;
    right: 26px;
    bottom: -1px;
    height: 2px;
    border-radius: 1px;
    overflow: hidden;
  }

  .loading span {
    position: absolute;
    top: 0;
    bottom: 0;
    left: -30%;
    width: 30%;
    border-radius: 1px;
    background: var(--accent);
    animation: loading 1.1s ease-in-out infinite;
  }

  :global(:root[data-motion='reduced']) .loading span {
    left: 0;
    width: 100%;
    opacity: 0.5;
  }

  @keyframes loading {
    to {
      left: 100%;
    }
  }

  .hotkey {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-subtle);
  }
</style>
