<script lang="ts">
  import { t } from '../core/i18n';
  import { settings } from '../core/settings/settings.svelte';
  import Icon from '../ui/Icon.svelte';
  import IconButton from '../ui/IconButton.svelte';
  import Keycap from '../ui/Keycap.svelte';
  import { keyLabel } from './hotkey';
  import { search } from './search.svelte';

  let input: HTMLInputElement | undefined = $state();

  export function focus(): void {
    input?.focus();
    input?.select();
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && search.active) {
      event.preventDefault();
      search.clear();
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
</div>

<style>
  .search {
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

  .hotkey {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-subtle);
  }
</style>
