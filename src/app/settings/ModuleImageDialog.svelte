<script lang="ts">
  import { t } from '../../core/i18n';
  import Dialog from '../../ui/Dialog.svelte';
  import IconButton from '../../ui/IconButton.svelte';

  let { name, image, onclose }: { name: string; image: string; onclose: () => void } = $props();

  const titleId = $props.id();
</script>

<Dialog labelledby={titleId} width={1000} {onclose}>
  <header>
    <h2 id={titleId}>{name}</h2>
    <IconButton icon="close" label={t('common.close')} onclick={onclose} />
  </header>
  <img src={image} alt={t('settings.modules.details.image', { name })} />
</Dialog>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  h2 {
    flex: 1;
    min-width: 0;
    font-size: 20px;
    font-weight: 600;
  }

  img {
    display: block;
    width: 100%;
    max-height: calc(100vh - 170px);
    object-fit: contain;
    border-radius: 14px;
  }
</style>
