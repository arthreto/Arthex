<script lang="ts">
  import { onMount } from 'svelte';
  import { formatBytes } from '../../core/format';
  import Button from '../../ui/Button.svelte';
  import Icon from '../../ui/Icon.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import ActionBar from './ActionBar.svelte';
  import { filesizeApi } from './api';
  import { crumbs } from './crumbs';
  import { filesize } from './filesize.svelte';
  import FolderList from './FolderList.svelte';
  import { t } from './i18n';
  import LargestList from './LargestList.svelte';
  import ScanningView from './ScanningView.svelte';
  import Shares from './Shares.svelte';
  import Treemap from './Treemap.svelte';

  onMount(() => {
    void filesize.start();
    const listening = filesizeApi.onUpdated(() => void filesize.refresh());
    const stopWatching = filesize.watchScan();
    return () => {
      void listening.then((stop) => stop());
      stopWatching();
    };
  });

  const drive = $derived(filesize.drives.find((drive) => drive.path === filesize.root));
  const driveOptions = $derived(
    filesize.drives.map((drive) => ({
      value: drive.path,
      label: drive.total ? `${drive.path.replace(/[\\/]$/, '')}  ${formatBytes(drive.total)}` : drive.path,
    })),
  );
  const modes = $derived([
    { value: 'folders' as const, label: t('folders') },
    { value: 'largest' as const, label: t('largest') },
  ]);
  const trail = $derived(filesize.root && filesize.folder ? crumbs(filesize.root, filesize.folder.path) : []);
</script>

<div class="filesize fade-in">
  <div class="top">
    <Segmented
      label={t('drives')}
      options={driveOptions}
      value={filesize.root ?? ''}
      onchange={(path) => filesize.show(path)}
    />
    <Button size="small" onclick={() => filesize.pickFolder()}>{t('pickFolder')}</Button>
    <span class="spacer"></span>
    {#if drive?.total}
      <span class="usage">
        {t('used', { used: formatBytes(drive.total - drive.free), total: formatBytes(drive.total) })}
      </span>
    {/if}
    <IconButton icon="refresh" label={t('rescan')} onclick={() => filesize.rescan()} />
  </div>

  {#if filesize.folder}
    {@const folder = filesize.folder}
    <div class="body">
      <div class="side">
        <Segmented
          label={t('view')}
          options={modes}
          value={filesize.mode}
          onchange={(mode) => filesize.setMode(mode)}
        />
        {#if filesize.mode === 'folders'}
          <nav class="trail" aria-label={t('path')}>
            {#each trail as crumb, position (crumb.path)}
              {#if position > 0}<Icon name="chevronRight" size={12} stroke={2} />{/if}
              <button
                class:here={position === trail.length - 1}
                aria-current={position === trail.length - 1 ? 'location' : undefined}
                onclick={() => filesize.enter(crumb.path)}>{crumb.label}</button
              >
            {/each}
          </nav>
          <FolderList
            {folder}
            selected={filesize.selection?.path ?? null}
            onselect={(item) => filesize.select(item)}
            onenter={(item) => filesize.enter(item.path)}
          />
        {:else}
          <LargestList
            files={filesize.largest}
            selected={filesize.selection?.path ?? null}
            onselect={(file) => filesize.select(file)}
          />
        {/if}
      </div>
      <div class="map">
        <Shares {folder} />
        <Treemap
          items={folder.items}
          selected={filesize.selection?.path ?? null}
          onselect={(item) => filesize.select(item)}
          onenter={(item) => filesize.enter(item.path)}
        />
      </div>
    </div>
    <ActionBar {folder} />
  {:else if filesize.answered}
    <ScanningView />
  {/if}
</div>

<style>
  .filesize {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .top {
    height: 36px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .spacer {
    flex: 1;
  }

  .usage {
    font-size: 12.5px;
    color: var(--text-subtle);
  }

  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 20px;
  }

  .side {
    width: 470px;
    flex: none;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .trail {
    min-height: 24px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 2px;
    color: var(--text-subtle);
  }

  .trail button {
    padding: 2px 5px;
    border-radius: 5px;
    font-size: 12.5px;
    color: var(--accent);
  }

  .trail button:hover {
    background: var(--surface-hover);
  }

  .trail .here {
    color: var(--text);
  }

  .map {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
</style>
