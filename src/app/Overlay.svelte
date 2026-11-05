<script lang="ts">
  import { onMount, tick } from 'svelte';
  import logo from '../assets/logo.png';
  import { commands, events, on } from '../core/api';
  import { t } from '../core/i18n';
  import { badgeModules, getModule, isEnabled, tabBarIds } from '../core/modules';
  import { settings } from '../core/settings/settings.svelte';
  import Hub from './Hub.svelte';
  import KeyHints from './KeyHints.svelte';
  import { navigation } from './navigation.svelte';
  import PageView from './PageView.svelte';
  import { loadBadge } from './pages';
  import SearchBar from './SearchBar.svelte';
  import SearchResults from './SearchResults.svelte';
  import { search } from './search.svelte';
  import Settings from './settings/Settings.svelte';
  import TabBar from './TabBar.svelte';
  import ToolSheet from './ToolSheet.svelte';
  import { tools } from '../core/tools.svelte';
  import UninstallDialog from './UninstallDialog.svelte';
  import { updates } from './updates.svelte';
  import WhatsNew from './WhatsNew.svelte';
  import { dialogs } from './dialogs.svelte';
  import { moduleHost } from './moduleHost.svelte';
  import OpenedModuleDialog from './OpenedModuleDialog.svelte';

  let shown = $state(0);
  let searchBar: SearchBar | undefined = $state();

  const tool = $derived(
    tools.current && isEnabled(settings.value, tools.current.id, 'tool')
      ? { module: getModule(tools.current.id), input: tools.current.input }
      : null,
  );

  function closeTool() {
    tools.close();
    searchBar?.focus();
  }

  onMount(() => {
    void updates.loadWhatsNew();
    const listeners = [
      on(events.overlayShown, async () => {
        shown += 1;
        await tick();
        searchBar?.focus();
      }),
      on<string>(events.navigate, (target) => {
        search.clear();
        navigation.openTarget(target);
      }),
      on(events.confirmUninstall, () => (dialogs.uninstall = true)),
    ];
    searchBar?.focus();
    return () => listeners.forEach((listener) => listener.then((unlisten) => unlisten()));
  });

  function hide() {
    void commands.overlayHide();
  }

  function cycleTabs(step: number) {
    const tabs = tabBarIds(settings.value);
    const index = navigation.view.kind === 'page' ? tabs.indexOf(navigation.view.id) : -1;
    const next = tabs[(index + step + tabs.length) % tabs.length];
    if (!next) return;
    search.clear();
    navigation.open({ kind: 'page', id: next });
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.defaultPrevented) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      hide();
    } else if (event.key === 'Tab' && event.ctrlKey) {
      event.preventDefault();
      cycleTabs(event.shiftKey ? -1 : 1);
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="veil" aria-hidden="true"></div>
<button class="backdrop" tabindex="-1" aria-label={t('common.close')} onclick={hide}></button>

{#key shown}
  <section class="overlay enter" aria-label="Arthex">
    <div class="brand">
      <img src={logo} alt="" width="36" height="36" />
      <span>Arthex</span>
    </div>
    <SearchBar bind:this={searchBar} />
    <TabBar />
    <div class="badges">
      {#each badgeModules(settings.value) as module (module.id)}
        {#await loadBadge(module.id) then loaded}
          {#if loaded}
            {@const Badge = loaded.default}
            <Badge />
          {/if}
        {/await}
      {/each}
    </div>
    <main class="content">
      {#if search.active}
        <SearchResults />
      {:else if navigation.view.kind === 'hub'}
        <Hub />
      {:else if navigation.view.kind === 'settings'}
        <Settings />
      {:else}
        <PageView id={navigation.view.id} />
      {/if}
    </main>
    <KeyHints />
  </section>
{/key}

{#if tool?.module}
  <ToolSheet module={tool.module} input={tool.input} onclose={closeTool} />
{/if}

{#if moduleHost.opened}
  <OpenedModuleDialog request={moduleHost.opened} />
{/if}

{#if dialogs.uninstall}
  <UninstallDialog onclose={() => (dialogs.uninstall = false)} />
{/if}

{#if updates.whatsNew}
  <WhatsNew notes={updates.whatsNew} onclose={() => (updates.whatsNew = null)} />
{/if}

<style>
  .veil {
    position: fixed;
    inset: 0;
    background: var(--veil);
  }

  .backdrop {
    position: fixed;
    inset: 0;
    cursor: default;
  }

  .overlay {
    position: absolute;
    inset: 34px 0 12px;
    width: min(1240px, calc(100% - 120px));
    margin: 0 auto;
    display: flex;
    flex-direction: column;
  }

  .brand {
    position: absolute;
    left: 0;
    top: 10px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 16px;
    font-weight: 650;
  }

  .brand img {
    display: block;
  }

  .badges {
    position: absolute;
    right: 0;
    top: 77px;
    display: flex;
    gap: 8px;
  }

  .content {
    flex: 1;
    min-height: 0;
    position: relative;
    margin-top: 22px;
  }
</style>
