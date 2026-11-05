<script lang="ts">
  import { t } from '../core/i18n';
  import { searchProviders } from '../core/modules';
  import { settings } from '../core/settings/settings.svelte';
  import Button from '../ui/Button.svelte';
  import Keycap from '../ui/Keycap.svelte';
  import ResultIcon from './ResultIcon.svelte';
  import SearchPreview from './SearchPreview.svelte';
  import { MIN_LENGTH, search } from './search.svelte';

  const DELAY_MS = 40;

  const statuses = $derived(searchProviders(settings.value).flatMap((provider) => provider.status ?? []));

  $effect(() => {
    void search.query;
    const timer = setTimeout(() => void search.update(), DELAY_MS);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    document.querySelector('.result.on')?.scrollIntoView({ block: 'nearest' });
  });
</script>

<div class="search-view fade-in">
  <div class="list scroll" role="listbox" aria-label={t('search.label')}>
    {#each statuses as Status, index (index)}
      <Status />
    {/each}
    {#each search.results as result, index (result.key)}
      {#if index === 0 || search.results[index - 1]?.group !== result.group}
        <div class="group" role="presentation">{result.group}</div>
      {/if}
      <button
        class="result"
        class:on={index === search.selected}
        role="option"
        aria-selected={index === search.selected}
        onclick={() => search.activate(result)}
        onmouseenter={() => (search.selected = index)}
      >
        <ResultIcon icon={result.icon} size={34} />
        <span class="text">
          <span class="title">{result.title}</span>
          {#if result.subtitle}<span class="subtitle">{result.subtitle}</span>{/if}
        </span>
        {#if result.meta}<span class="meta">{result.meta}</span>{/if}
        {#if index === search.selected}<Keycap>{t('keys.enter')}</Keycap>{/if}
      </button>
    {:else}
      {#if search.tooShort}
        <p class="empty">{t('search.tooShort', { count: MIN_LENGTH })}</p>
      {:else if search.pending === 0}
        <p class="empty">{t('search.noResults', { query: search.query.trim() })}</p>
      {/if}
    {/each}
  </div>
  {#if search.current}
    <SearchPreview result={search.current} />
  {/if}
  {#if search.confirming?.confirm}
    {@const pending = search.confirming}
    <div class="confirm fade-in" role="alertdialog" aria-label={pending.title}>
      <span>{pending.confirm?.text}</span>
      <Button onclick={() => (search.confirming = null)}>{t('common.cancel')}</Button>
      <Button variant="danger" onclick={() => search.activate(pending)}>{pending.confirm?.label}</Button>
    </div>
  {/if}
</div>

<style>
  .search-view {
    position: absolute;
    inset: 0;
    display: flex;
    gap: 20px;
  }

  .list {
    flex: 1;
    min-width: 0;
    /* Room for the 4 px focus outline of a result, which the scrolling list would otherwise cut. */
    padding: 4px 16px 12px 4px;
    scroll-padding: 4px;
  }

  .group {
    padding: 12px 12px 6px;
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  .result {
    width: 100%;
    height: 50px;
    padding: 0 12px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    gap: 14px;
    text-align: left;
  }

  .result.on {
    background: var(--surface-selected);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .title,
  .subtitle {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .title {
    font-size: 14.5px;
    font-weight: 500;
  }

  .subtitle,
  .meta {
    font-size: 12px;
    color: var(--text-subtle);
  }

  .meta {
    white-space: nowrap;
  }

  .empty {
    padding: 70px 0;
    text-align: center;
    font-size: 14px;
    color: var(--text-muted);
  }

  .confirm {
    position: absolute;
    left: 0;
    right: 400px;
    bottom: 8px;
    padding: 12px 12px 12px 18px;
    border-radius: 12px;
    background: var(--surface-dialog);
    border: 1px solid var(--border-strong);
    box-shadow: var(--shadow-dialog);
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13.5px;
  }

  .confirm span {
    flex: 1;
  }
</style>
