<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { formatWhen } from '../../core/format';
  import Button from '../../ui/Button.svelte';
  import Icon from '../../ui/Icon.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import type { Note } from './api';
  import { t } from './i18n';
  import { plain, tasks } from './markdown';
  import NoteEditor from './NoteEditor.svelte';
  import { notebook, type NoteMode } from './notes.svelte';
  import NoteView from './NoteView.svelte';

  onMount(() => {
    void notebook.load();
  });
  onDestroy(() => void notebook.flush());

  const current = $derived(notebook.current);
  const modes = $derived([
    { value: 'view' as NoteMode, label: t('view') },
    { value: 'edit' as NoteMode, label: t('edit') },
  ]);

  function preview(note: Note): string {
    const list = tasks(note.text);
    if (list.length > 0) {
      return t('tasksDone', { done: list.filter((task) => task.done).length, total: list.length });
    }
    const first = note.text.split('\n').find((line) => line.trim());
    return (first && plain(first)) || t('emptyPreview');
  }
</script>

<div class="notes fade-in">
  <div class="list scroll">
    <Button onclick={() => notebook.create()}><Icon name="plus" size={14} stroke={2.2} />{t('new')}</Button>
    {#each notebook.notes as note (note.id)}
      <button
        class="entry"
        class:on={note.id === current?.id}
        aria-current={note.id === current?.id ? 'true' : undefined}
        onclick={() => notebook.open(note.id)}
      >
        <span class="line">
          <span class="title">{note.title || t('untitled')}</span>
          <span class="when">{formatWhen(note.updated)}</span>
        </span>
        <span class="preview">{preview(note)}</span>
      </button>
    {/each}
  </div>
  {#if current}
    <section class="editor">
      <div class="head">
        <input
          class="name"
          aria-label={t('titleLabel')}
          placeholder={t('untitled')}
          value={current.title}
          oninput={(event) => notebook.update({ title: event.currentTarget.value })}
        />
        <span class="status">
          {#if !notebook.saving}<Icon name="check" size={12} stroke={2.4} />{/if}
          {t(notebook.saving ? 'saving' : 'saved')}
        </span>
        <Segmented
          label={t('display')}
          options={modes}
          value={notebook.mode}
          onchange={(mode) => (notebook.mode = mode)}
        />
        <IconButton icon="trash" label={t('delete')} onclick={() => notebook.remove(current)} />
      </div>
      {#if notebook.mode === 'view'}
        <NoteView text={current.text} />
      {:else}
        {#key current.id}<NoteEditor text={current.text} />{/key}
      {/if}
    </section>
  {/if}
</div>

<style>
  .notes {
    position: absolute;
    inset: 0;
    display: flex;
    gap: 20px;
  }

  .list {
    width: 290px;
    flex: none;
    padding-right: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .list > :global(.button) {
    width: 100%;
    margin-bottom: 8px;
  }

  .entry {
    width: 100%;
    padding: 11px 12px;
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    text-align: left;
    transition: background-color var(--motion-fast);
  }

  .entry:hover {
    background: var(--surface-hover);
  }

  .entry.on {
    background: var(--surface-selected);
  }

  .line {
    width: 100%;
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }

  .title,
  .preview {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .title {
    font-size: 14px;
    font-weight: 500;
  }

  .when {
    flex: none;
    font-size: 11.5px;
    color: var(--text-subtle);
  }

  .preview {
    font-size: 12.5px;
    color: var(--text-subtle);
  }

  .editor {
    flex: 1;
    min-width: 0;
    padding: 16px 22px 18px;
    border-radius: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .name {
    flex: 1;
    min-width: 0;
    padding: 4px 0;
    font: inherit;
    font-size: 22px;
    font-weight: 600;
    color: var(--text);
    background: transparent;
    border: 0;
    outline: none;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: var(--text-subtle);
  }
</style>
