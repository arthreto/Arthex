<script lang="ts">
  import { onMount } from 'svelte';
  import { toast } from '../../core/toast.svelte';
  import Icon from '../../ui/Icon.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import { t } from './i18n';
  import { planner } from './planner.svelte';
  import { clockTime } from './times';

  onMount(() => {
    void planner.load();
  });

  async function cancel() {
    await planner.cancel();
    toast.show(t('schedule.cancelled'));
  }
</script>

{#if planner.current}
  <div class="pill fade-in" role="status">
    <Icon name="power" size={14} stroke={2.2} />
    {t('schedule.pill', { action: planner.current.action, time: clockTime(planner.current.at) })}
    <IconButton icon="close" size={24} label={t('schedule.cancel')} onclick={cancel} />
  </div>
{/if}

<style>
  .pill {
    height: 34px;
    padding: 0 5px 0 12px;
    border-radius: 17px;
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent) 32%, transparent);
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
  }
</style>
