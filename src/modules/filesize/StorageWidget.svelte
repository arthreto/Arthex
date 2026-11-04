<script lang="ts">
  import { onMount } from 'svelte';
  import { navigation } from '../../app/navigation.svelte';
  import { formatBytes } from '../../core/format';
  import CardButton from '../../ui/CardButton.svelte';
  import { filesizeApi, type Drive } from './api';
  import { moduleText } from '../../core/i18n';
  import { t } from './i18n';

  /** A drive this full shows its bar in the warning colour. */
  const NEARLY_FULL = 0.85;

  let drives = $state<Drive[]>([]);

  onMount(() => {
    filesizeApi
      .drives()
      .then((list) => (drives = list.filter((drive) => drive.total > 0)))
      .catch(() => (drives = []));
  });
</script>

{#if drives.length > 0}
  <CardButton
    title={t('card.title')}
    link={moduleText('filesize', 'name')}
    onclick={() => navigation.open({ kind: 'page', id: 'filesize' })}
  >
    {#each drives as drive (drive.path)}
      {@const used = (drive.total - drive.free) / drive.total}
      <span class="drive">
        <span class="line">
          <span>{drive.path.replace(/[\\/]$/, '')}{drive.label ? `  ${drive.label}` : ''}</span>
          <span>{t('card.free', { free: formatBytes(drive.free), total: formatBytes(drive.total) })}</span>
        </span>
        <span class="bar"><span style:width="{used * 100}%" class:full={used >= NEARLY_FULL}></span></span>
      </span>
    {/each}
  </CardButton>
{/if}

<style>
  .drive {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .line {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 12.5px;
    color: var(--text-muted);
    white-space: pre;
  }

  .bar {
    height: 7px;
    border-radius: 4px;
    background: var(--surface-hover);
    overflow: hidden;
  }

  .bar span {
    display: block;
    height: 100%;
    border-radius: 4px;
    background: var(--accent);
  }

  .bar .full {
    background: var(--tint-amber);
  }
</style>
