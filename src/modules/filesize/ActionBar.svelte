<script lang="ts">
  import { copyPath } from '../../core/clipboard';
  import { formatBytes } from '../../core/format';
  import { t as shellT } from '../../core/i18n';
  import Button from '../../ui/Button.svelte';
  import Icon from '../../ui/Icon.svelte';
  import type { Folder } from './api';
  import { filesize } from './filesize.svelte';
  import { t } from './i18n';

  let { folder }: { folder: Folder } = $props();

  const selection = $derived(filesize.selection);
  const label = $derived(
    selection
      ? [selection.path, formatBytes(selection.size), ...(selection.protected ? [t('protected')] : [])].join(
          ' · ',
        )
      : `${folder.path} · ${formatBytes(folder.size)} · ${t('hint')}`,
  );
</script>

{#if filesize.confirming && selection}
  <div class="bar confirm fade-in" role="alertdialog" aria-label={t('confirmTrash')}>
    <span class="text">{t('confirm', { name: selection.name, size: formatBytes(selection.size) })}</span>
    <Button onclick={() => (filesize.confirming = false)}>{shellT('common.cancel')}</Button>
    <Button variant="danger" onclick={() => filesize.trash(selection)}>{t('confirmTrash')}</Button>
  </div>
{:else}
  <div class="bar">
    <span class="text">{label}</span>
    <Button disabled={!selection} onclick={() => selection && filesize.reveal(selection.path)}
      ><Icon name="external" size={14} stroke={2} />{t('explorer')}</Button
    >
    <Button disabled={!selection} onclick={() => selection && copyPath(selection.path)}
      ><Icon name="copy" size={14} stroke={2} />{t('copyPath')}</Button
    >
    <Button disabled={!selection || selection.protected} onclick={() => (filesize.confirming = true)}
      ><Icon name="trash" size={14} stroke={2} />{t('trash')}</Button
    >
  </div>
{/if}

<style>
  .bar {
    height: 52px;
    flex: none;
    padding: 0 8px 0 18px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .confirm {
    background: color-mix(in srgb, var(--danger) 22%, var(--surface-dialog));
    border-color: color-mix(in srgb, var(--danger-text) 30%, transparent);
  }

  .text {
    flex: 1;
    min-width: 0;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
