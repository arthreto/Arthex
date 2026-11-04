<script lang="ts">
  import { onMount } from 'svelte';
  import { navigation } from '../../app/navigation.svelte';
  import { formatWhen } from '../../core/format';
  import CardButton from '../../ui/CardButton.svelte';
  import { clipboard } from './clipboard.svelte';
  import { t } from './i18n';

  onMount(() => {
    void clipboard.load();
  });

  const latest = $derived(clipboard.latest);
</script>

<CardButton
  title={t('card.title')}
  note={latest ? formatWhen(latest.copiedAt) : undefined}
  link={t('card.link')}
  onclick={() => navigation.open({ kind: 'page', id: 'clipboard' })}
>
  <span class="text">{latest?.text ?? t('card.empty')}</span>
</CardButton>

<style>
  .text {
    width: 100%;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
