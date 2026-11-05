<script lang="ts">
  import { errorMessage } from '../../core/api';
  import { moduleText, t } from '../../core/i18n';
  import { elementsOf, isActive, isEnabled, switchElement, type Element } from '../../core/modules';
  import type { ModuleManifest } from '../../core/modules/manifest';
  import { settings } from '../../core/settings/settings.svelte';
  import { toast } from '../../core/toast.svelte';
  import Dialog from '../../ui/Dialog.svelte';
  import Icon from '../../ui/Icon.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Switch from '../../ui/Switch.svelte';
  import { loadSettingsAction } from '../pages';

  let { module, onclose }: { module: ModuleManifest; onclose: () => void } = $props();

  const titleId = $props.id();
  const name = $derived(moduleText(module.id, 'name'));

  async function toggle(element: Element, enabled: boolean) {
    try {
      await settings.update({
        disabledElements: switchElement(settings.value.disabledElements, module.id, element, enabled),
      });
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

<Dialog labelledby={titleId} width={560} {onclose}>
  <header>
    <span class="icon" style:color="var(--tint-{module.tint})">
      <Icon path={module.icon} size={22} />
    </span>
    <span class="title">
      <h2 id={titleId}>{name}</h2>
      <span class="description">{moduleText(module.id, 'description')}</span>
    </span>
    <IconButton icon="close" label={t('common.close')} onclick={onclose} />
  </header>
  <Panel list>
    {#each elementsOf(module) as element (element)}
      {@const elementName = t(`settings.modules.element.${element}`)}
      {@const enabled = isEnabled(settings.value, module.id, element)}
      <div class="row">
        <span class="element" class:off={!enabled}>{elementName}</span>
        <Switch
          label={t('settings.modules.switch', { name, element: elementName })}
          checked={enabled}
          onchange={(value) => toggle(element, value)}
        />
      </div>
    {/each}
  </Panel>
  {#if isActive(settings.value, module.id)}
    {#await loadSettingsAction(module.id) then loaded}
      {#if loaded}
        <div class="action"><loaded.default /></div>
      {/if}
    {/await}
  {/if}
</Dialog>

<style>
  header {
    display: flex;
    align-items: flex-start;
    gap: 14px;
  }

  .icon {
    width: 44px;
    height: 44px;
    flex: none;
    box-sizing: border-box;
    border-radius: 13px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .title {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  h2 {
    font-size: 20px;
    font-weight: 600;
  }

  .description {
    font-size: 12.5px;
    color: var(--text-subtle);
  }

  .row {
    padding: 12px 0;
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .row + .row {
    border-top: 1px solid var(--border);
  }

  .element {
    flex: 1;
    min-width: 0;
    font-size: 14px;
    font-weight: 500;
  }

  .element.off {
    opacity: 0.5;
  }

  .action {
    display: flex;
  }
</style>
