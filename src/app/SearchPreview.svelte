<script lang="ts">
  import { copyPath } from '../core/clipboard';
  import { t } from '../core/i18n';
  import type { SearchResult } from '../core/search/types';
  import Button from '../ui/Button.svelte';
  import ResultIcon from './ResultIcon.svelte';
  import { search } from './search.svelte';

  let { result }: { result: SearchResult } = $props();
</script>

<aside aria-label={t('search.preview')}>
  <ResultIcon icon={result.icon} size={64} />
  <div class="heading">
    <span class="kind">{result.kind}</span>
    <span class="title">{result.title}</span>
  </div>
  {#if result.panel}
    {@const Panel = result.panel}
    <div class="body"><Panel /></div>
  {:else}
    {#if result.details?.length}
      <dl class="body">
        {#each result.details as detail (detail.label)}
          <div>
            <dt>{detail.label}</dt>
            <dd>{detail.value}</dd>
          </div>
        {/each}
      </dl>
    {/if}
    <div class="actions">
      <Button variant="primary" onclick={() => search.activate(result)}>{result.primary.label}</Button>
      {#if result.secondary}
        <Button onclick={() => search.activate(result, true)}>{result.secondary.label}</Button>
      {/if}
      {#if result.path}
        {@const path = result.path}
        <Button onclick={() => copyPath(path)}>{t('search.copyPath')}</Button>
      {/if}
    </div>
  {/if}
</aside>

<style>
  aside {
    width: 380px;
    flex: none;
    align-self: flex-start;
    padding: 22px;
    border-radius: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .heading {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .kind {
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  .title {
    font-size: 19px;
    font-weight: 600;
    line-height: 1.3;
    overflow-wrap: anywhere;
  }

  .body {
    padding-top: 12px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  dl div {
    display: flex;
    gap: 12px;
    font-size: 12.5px;
  }

  dt {
    width: 104px;
    flex: none;
    color: var(--text-subtle);
  }

  dd {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .actions {
    margin-top: 8px;
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }
</style>
