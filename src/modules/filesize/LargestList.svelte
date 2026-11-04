<script lang="ts">
  import { formatBytes } from '../../core/format';
  import FileIcon from '../../ui/FileIcon.svelte';
  import type { LargeFile } from './api';
  import { t } from './i18n';

  interface Props {
    files: LargeFile[];
    selected: string | null;
    onselect: (file: LargeFile) => void;
  }

  let { files, selected, onselect }: Props = $props();

  const notes: Record<string, 'notes.hiberfil' | 'notes.pagefile'> = {
    'hiberfil.sys': 'notes.hiberfil',
    'pagefile.sys': 'notes.pagefile',
  };

  function detail(file: LargeFile): string {
    const note = file.protected ? notes[file.name.toLowerCase()] : undefined;
    return note ? `${file.location} · ${t(note)}` : file.location;
  }
</script>

<div class="list scroll">
  {#each files as file (file.path)}
    <button
      class="row"
      class:on={selected === file.path}
      aria-pressed={selected === file.path}
      onclick={() => onselect(file)}
    >
      <FileIcon folder={false} size={16} />
      <span class="text">
        <span class="name">{file.name}</span>
        <span class="location">{detail(file)}</span>
      </span>
      <span class="size">{formatBytes(file.size)}</span>
    </button>
  {/each}
</div>

<style>
  .list {
    flex: 1;
    min-height: 0;
    padding-right: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .row {
    width: 100%;
    padding: 7px 9px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 10px;
    text-align: left;
    transition: background-color var(--motion-fast);
  }

  .row:hover {
    background: var(--surface-hover);
  }

  .row.on {
    background: var(--surface-selected);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .name,
  .location {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .name {
    font-size: 13.5px;
  }

  .location {
    font-size: 12px;
    color: var(--text-subtle);
  }

  .size {
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }
</style>
