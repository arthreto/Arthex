<script lang="ts">
  import { onMount } from 'svelte';
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import Chip from '../../ui/Chip.svelte';
  import Panel from '../../ui/Panel.svelte';
  import { fileSearchApi, type IndexStatus } from './api';
  import { t } from './i18n';

  let status = $state<IndexStatus | null>(null);

  onMount(() => {
    void apply(fileSearchApi.status());
  });

  async function apply(next: Promise<IndexStatus>) {
    try {
      status = await next;
    } catch (error) {
      toast.show(shellT('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

{#if status}
  <Panel>
    <div class="row">
      <span class="label">{t('settings.folders')}</span>
      <div class="chips">
        {#each status.drives as drive (drive)}
          <Chip label={drive} />
        {/each}
        {#each status.folders as folder (folder)}
          <Chip
            label={folder}
            removeLabel={t('settings.removeFolder', { folder })}
            onremove={() => apply(fileSearchApi.removeFolder(folder))}
          />
        {/each}
        <Button size="small" onclick={() => apply(fileSearchApi.addFolder())}
          >{t('settings.addFolder')}</Button
        >
      </div>
    </div>
    <div class="row">
      <span class="label">{t('settings.exclusions')}</span>
      <div class="chips">
        {#each status.exclusions as exclusion (exclusion)}
          <Chip
            label={exclusion}
            removeLabel={t('settings.removeExclusion', { exclusion })}
            onremove={() => apply(fileSearchApi.removeExclusion(exclusion))}
          />
        {/each}
      </div>
    </div>
  </Panel>
{/if}

<style>
  .row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 8px 0;
  }

  .row:first-child {
    padding-top: 0;
  }

  .row:last-child {
    padding-bottom: 0;
  }

  .row + .row {
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }

  .label {
    width: 140px;
    flex: none;
    font-size: 13.5px;
    line-height: 28px;
  }

  .chips {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
</style>
