<script lang="ts">
  import { dragWithGhost } from '../../ui/dragGhost';
  import { onMount } from 'svelte';
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { toast } from '../../core/toast.svelte';
  import Icon from '../../ui/Icon.svelte';
  import Keycap from '../../ui/Keycap.svelte';
  import Menu from '../../ui/Menu.svelte';
  import MenuItem from '../../ui/MenuItem.svelte';
  import { MAX_ON_HOME, shortcutsApi, targetText, type Shortcut } from './api';
  import { editor } from './editor.svelte';
  import { t } from './i18n';
  import ShortcutDialog from './ShortcutDialog.svelte';
  import ShortcutIcon from './ShortcutIcon.svelte';
  import { shortcuts } from './shortcuts.svelte';

  let menu = $state<{ shortcut: Shortcut; x: number; y: number } | null>(null);
  let dragged = $state<string | null>(null);

  const full = $derived(shortcuts.home.length >= MAX_ON_HOME);
  const columns = $derived(Math.min(10, Math.max(8, shortcuts.home.length + 1)));

  onMount(() => {
    void shortcuts.load();
  });

  function fail(error: unknown) {
    toast.show(shellT('errors.generic', { message: errorMessage(error) }));
  }

  async function open(shortcut: Shortcut) {
    try {
      await shortcutsApi.open(shortcut.id);
    } catch (error) {
      fail(error);
    }
  }

  function openMenu(event: MouseEvent, shortcut: Shortcut) {
    event.preventDefault();
    const fromKeyboard = event.clientX === 0 && event.clientY === 0;
    const tile = (event.currentTarget as HTMLElement).getBoundingClientRect();
    menu = {
      shortcut,
      x: fromKeyboard ? tile.left : event.clientX,
      y: fromKeyboard ? tile.bottom : event.clientY,
    };
  }

  function edit(shortcut: Shortcut) {
    menu = null;
    editor.edit(shortcut);
  }

  async function remove(shortcut: Shortcut) {
    menu = null;
    try {
      const undo = await shortcuts.remove(shortcut);
      toast.show(t('deleted', { name: shortcut.name }), {
        label: shellT('common.undo'),
        run: () => void undo().catch(fail),
      });
    } catch (error) {
      fail(error);
    }
  }

  function drop(event: DragEvent, target: Shortcut) {
    event.preventDefault();
    const id = dragged;
    dragged = null;
    if (id) shortcuts.move(id, target.id).catch(fail);
  }

  function onkeydown(event: KeyboardEvent) {
    if (!event.altKey || event.ctrlKey || !/^Digit[1-9]$/.test(event.code)) return;
    const shortcut = shortcuts.home[Number(event.code.slice(5)) - 1];
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
    {#each shortcuts.home as shortcut (shortcut.id)}
      <button
        class="tile"
        class:dragged={dragged === shortcut.id}
        title={targetText(shortcut)}
        draggable="true"
        onclick={() => open(shortcut)}
        oncontextmenu={(event) => openMenu(event, shortcut)}
        ondragstart={(event) => {
          dragged = shortcut.id;
          if (event.currentTarget instanceof HTMLElement) dragWithGhost(event, event.currentTarget);
        }}
        ondragend={() => (dragged = null)}
        ondragover={(event) => event.preventDefault()}
        ondrop={(event) => drop(event, shortcut)}
      >
        <ShortcutIcon {shortcut} size={46} />
        <span class="name">{shortcut.name}</span>
      </button>
    {/each}
    {#if !full}
      <button class="tile add" onclick={() => editor.add()}>
        <Icon name="plus" size={22} stroke={2} />
        <span class="name">{t('add')}</span>
      </button>
    {/if}
  </div>
</section>

{#if menu}
  {@const target = menu.shortcut}
  <Menu x={menu.x} y={menu.y} label={t('menu', { name: target.name })} onclose={() => (menu = null)}>
    <MenuItem icon="edit" onclick={() => edit(target)}>{t('edit')}</MenuItem>
    <MenuItem icon="trash" danger onclick={() => remove(target)}>{t('delete')}</MenuItem>
  </Menu>
{/if}

{#if editor.current}
  <ShortcutDialog
    editing={editor.current.editing}
    prefill={editor.current.prefill}
    onclose={() => editor.close()}
  />
{/if}

<style>
  header {
    margin-bottom: 10px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .hint {
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

  .name {
    max-width: 100%;
    padding: 0 8px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
  }

  .dragged {
    opacity: 0.4;
  }

  .add {
    border-style: dashed;
    border-color: var(--border-strong);
    color: var(--text-muted);
  }
</style>
