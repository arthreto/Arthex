<script lang="ts">
  import { onMount } from 'svelte';
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { toast } from '../../core/toast.svelte';
  import Icon from '../../ui/Icon.svelte';
  import Keycap from '../../ui/Keycap.svelte';
  import AddShortcutDialog from './AddShortcutDialog.svelte';
  import { MAX_SHORTCUTS, shortcutsApi, type Shortcut } from './api';
  import { t } from './i18n';
  import { shortcuts } from './shortcuts.svelte';

  let adding = $state(false);

  const full = $derived(shortcuts.items.length >= MAX_SHORTCUTS);
  const columns = $derived(Math.min(10, Math.max(8, shortcuts.items.length + 1)));

  onMount(() => {
    void shortcuts.load();
  });

  async function open(shortcut: Shortcut) {
    try {
      await shortcutsApi.open(shortcut.id);
    } catch (error) {
      toast.show(shellT('errors.generic', { message: errorMessage(error) }));
    }
  }

  function onkeydown(event: KeyboardEvent) {
    if (!event.altKey || event.ctrlKey || !/^Digit[1-9]$/.test(event.code)) return;
    const shortcut = shortcuts.items[Number(event.code.slice(5)) - 1];
    if (!shortcut) return;
    event.preventDefault();
    void open(shortcut);
  }
</script>

<svelte:window {onkeydown} />

<section>
  <header>
    <h2 class="section-title">{t('title')}</h2>
    <span class="hint">
      <span class="keys"><Keycap>Alt</Keycap>+<Keycap>1…9</Keycap></span>
      {t('keyboard')}
    </span>
  </header>
  <div class="grid" style:grid-template-columns="repeat({columns}, minmax(0, 1fr))">
    {#each shortcuts.items as shortcut (shortcut.id)}
      <button
        class="tile"
        title={shortcut.kind === 'site' ? shortcut.url : shortcut.path}
        onclick={() => open(shortcut)}
      >
        {#if shortcut.icon}
          <img src={shortcut.icon} alt="" width="46" height="46" />
        {:else}
          <span class="letter">{shortcut.name.charAt(0).toUpperCase()}</span>
        {/if}
        <span class="name">{shortcut.name}</span>
      </button>
    {/each}
    {#if !full}
      <button class="tile add" onclick={() => (adding = true)}>
        <Icon name="plus" size={22} stroke={2} />
        <span class="name">{t('add')}</span>
      </button>
    {/if}
  </div>
</section>

{#if adding}
  <AddShortcutDialog onclose={() => (adding = false)} />
{/if}

<style>
  header {
    margin-bottom: 10px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .hint {
    position: relative;
    top: -4px;
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 12px;
    color: var(--text-subtle);
  }

  .keys {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .grid {
    display: grid;
    gap: 10px;
  }

  .tile {
    height: 90px;
    border-radius: 14px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 9px;
    background: var(--surface);
    border: 1px solid var(--border);
    transition:
      background-color var(--motion-fast),
      border-color var(--motion-fast);
  }

  .tile:hover {
    background: var(--surface-hover);
    border-color: var(--border-strong);
  }

  img {
    width: 46px;
    height: 46px;
    padding: 3px;
    object-fit: contain;
  }

  .letter {
    width: 46px;
    height: 46px;
    border-radius: 13px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    font-weight: 700;
    color: var(--text-muted);
  }

  .name {
    max-width: 100%;
    padding: 0 8px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
  }

  .add {
    border-style: dashed;
    border-color: var(--border-strong);
    color: var(--text-muted);
  }
</style>
