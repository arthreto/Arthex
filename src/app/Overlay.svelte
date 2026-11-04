<script lang="ts">
  import { onMount, tick } from 'svelte';
  import logo from '../assets/logo.png';
  import { commands, events, on } from '../core/api';
  import { t } from '../core/i18n';
  import { tabBarIds } from '../core/modules';
  import { settings } from '../core/settings/settings.svelte';
  import Hub from './Hub.svelte';
  import KeyHints from './KeyHints.svelte';
  import { navigation } from './navigation.svelte';
  import PageView from './PageView.svelte';
  import SearchBar from './SearchBar.svelte';
  import SearchResults from './SearchResults.svelte';
  import { search } from './search.svelte';
  import Settings from './settings/Settings.svelte';
  import TabBar from './TabBar.svelte';
  import UninstallDialog from './UninstallDialog.svelte';

  let shown = $state(0);
  let confirmingUninstall = $state(false);
  let searchBar: SearchBar | undefined = $state();

  onMount(() => {
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
      on(events.confirmUninstall, () => (confirmingUninstall = true)),
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
    <main class="content">
      {#if search.active}
        <SearchResults />
      {:else if navigation.view.kind === 'hub'}
        <Hub />
      {:else if navigation.view.kind === 'settings'}
        <Settings onuninstall={() => (confirmingUninstall = true)} />
      {:else}
        <PageView id={navigation.view.id} />
      {/if}
    </main>
    <KeyHints />
  </section>
{/key}

{#if confirmingUninstall}
  <UninstallDialog onclose={() => (confirmingUninstall = false)} />
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

  .content {
    flex: 1;
    min-height: 0;
    position: relative;
    margin-top: 22px;
  }
</style>
