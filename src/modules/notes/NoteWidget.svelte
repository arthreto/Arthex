<script lang="ts">
  import { onMount } from 'svelte';
  import { navigation } from '../../app/navigation.svelte';
  import CardButton from '../../ui/CardButton.svelte';
  import Check from '../../ui/Check.svelte';
  import { t } from './i18n';
  import { tasks } from './markdown';
  import { notebook } from './notes.svelte';

  const SHOWN = 3;

  onMount(() => {
    void notebook.load();
  });

  /** The latest note with tasks, as a small checklist. */
  const note = $derived(notebook.notes.find((note) => tasks(note.text).length > 0) ?? null);
  const items = $derived(note ? tasks(note.text).slice(0, SHOWN) : []);
</script>

<CardButton
  title={note ? note.title || t('untitled') : t('name')}
  link={t('name')}
  onclick={() => {
    if (note) notebook.open(note.id);
    navigation.open({ kind: 'page', id: 'notes' });
  }}
>
  {#each items as item, position (position)}
    <span class="task" class:done={item.done}>
      <Check checked={item.done} size={15} />
      <span class="text">{item.text}</span>
    </span>
  {/each}
</CardButton>

<style>
  .task {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 13px;
  }

  .text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .done .text {
    text-decoration: line-through;
    color: var(--text-subtle);
  }
</style>
