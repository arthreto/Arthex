<script lang="ts">
  import { onMount } from 'svelte';
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import Panel from '../../ui/Panel.svelte';
  import { targetText, type Shortcut } from './api';
  import { editor } from './editor.svelte';
  import { t } from './i18n';
  import ShortcutDialog from './ShortcutDialog.svelte';
  import ShortcutIcon from './ShortcutIcon.svelte';
  import { shortcuts } from './shortcuts.svelte';

  onMount(() => {
    void shortcuts.load();
  });

  function where(shortcut: Shortcut): string {
    const places = [
      shortcut.onHome ? t('settings.home') : '',
      shortcut.inSearch ? t('settings.search') : '',
      shortcut.keyword ? t('search.keyword', { keyword: shortcut.keyword }) : '',
    ];
    return places.filter(Boolean).join(' · ');
  }

  async function remove(shortcut: Shortcut) {
    try {
      const undo = await shortcuts.remove(shortcut);
      toast.show(t('deleted', { name: shortcut.name }), {
        label: shellT('common.undo'),
        run: () =>
          void undo().catch((error) =>
            toast.show(shellT('errors.generic', { message: errorMessage(error) })),
          ),
      });
    } catch (error) {
      toast.show(shellT('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

{#if shortcuts.items.length > 0}
  <Panel list>
    {#each shortcuts.items as shortcut (shortcut.id)}
      <div class="row">
        <ShortcutIcon {shortcut} size={34} />
        <span class="text">
          <span class="name">{shortcut.name}</span>
          <span class="target" class:mono={shortcut.kind === 'command'}>{targetText(shortcut)}</span>
        </span>
        <span class="where">{where(shortcut)}</span>
        <Button
          size="small"
          aria-label={t('settings.editNamed', { name: shortcut.name })}
          onclick={() => editor.edit(shortcut)}>{t('edit')}</Button
        >
        <IconButton
          icon="trash"
          label={t('settings.deleteNamed', { name: shortcut.name })}
          onclick={() => remove(shortcut)}
        />
      </div>
    {/each}
  </Panel>
{/if}

{#if editor.current}
  <ShortcutDialog
    editing={editor.current.editing}
    prefill={editor.current.prefill}
    onclose={() => editor.close()}
  />
{/if}

<style>
  .row {
    padding: 10px 0;
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .row + .row {
    border-top: 1px solid var(--border);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .name {
    font-size: 14px;
    font-weight: 600;
  }

  .target {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12.5px;
    color: var(--text-muted);
  }

  .mono {
    font-family: var(--font-mono);
  }

  .where {
    font-size: 12px;
    white-space: nowrap;
    color: var(--text-subtle);
  }
</style>
