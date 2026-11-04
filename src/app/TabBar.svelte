<script lang="ts">
  import { moduleText, t } from '../core/i18n';
  import { tabBarIds } from '../core/modules';
  import { settings } from '../core/settings/settings.svelte';
  import Icon from '../ui/Icon.svelte';
  import type { IconName } from '../ui/icons';
  import { navigation, type View } from './navigation.svelte';
  import { search } from './search.svelte';

  const pinned = $derived(tabBarIds(settings.value));
  const current = $derived(navigation.view.kind === 'page' ? navigation.view.id : null);
  const tabs = $derived(current && !pinned.includes(current) ? [...pinned, current] : pinned);

  function go(view: View) {
    search.clear();
    navigation.open(view);
  }

  const isActive = (kind: 'hub' | 'settings') => !search.active && navigation.view.kind === kind;
</script>

{#snippet iconTab(kind: 'hub' | 'settings', icon: IconName, label: string)}
  <button
    class="tab icon"
    class:on={isActive(kind)}
    aria-pressed={isActive(kind)}
    aria-label={label}
    title={label}
    onclick={() => go({ kind })}
  >
    <Icon name={icon} size={17} />
  </button>
{/snippet}

<nav class="bar">
  <div class="tabs" role="tablist" aria-label={t('tabs.label')}>
    {#each tabs as id (id)}
      {@const selected = !search.active && navigation.isPage(id)}
      <button
        class="tab"
        class:on={selected}
        class:temporary={!pinned.includes(id)}
        role="tab"
        aria-selected={selected}
        onclick={() => go({ kind: 'page', id })}>{moduleText(id, 'name')}</button
      >
    {/each}
  </div>
  <span class="divider" aria-hidden="true"></span>
  {@render iconTab('hub', 'grid', t('tabs.hub'))}
  {@render iconTab('settings', 'settings', t('tabs.settings'))}
</nav>

<style>
  .bar {
    align-self: center;
    margin-top: 16px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px;
    border-radius: 22px;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
  }

  .tabs {
    display: flex;
    gap: 4px;
  }

  .tab {
    height: 34px;
    padding: 0 18px;
    border-radius: 17px;
    border: 1px solid transparent;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    white-space: nowrap;
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text-muted);
    transition:
      background-color var(--motion-fast),
      color var(--motion-fast);
  }

  .tab:hover {
    color: var(--text);
    background: var(--surface-hover);
  }

  .tab.on {
    background: var(--surface-hover);
    color: var(--text);
    font-weight: 600;
  }

  .temporary {
    border-style: dashed;
    border-color: var(--border-strong);
  }

  .icon {
    width: 34px;
    padding: 0;
  }

  .divider {
    width: 1px;
    height: 20px;
    margin: 0 4px;
    background: var(--border-strong);
  }
</style>
