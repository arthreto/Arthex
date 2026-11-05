<script lang="ts">
  import { errorMessage } from '../../core/api';
  import { moduleText, t } from '../../core/i18n';
  import { ELEMENTS, allModules, isActive, isEnabled, switchElement, type Element } from '../../core/modules';
  import { settings } from '../../core/settings/settings.svelte';
  import { toast } from '../../core/toast.svelte';
  import Icon from '../../ui/Icon.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Switch from '../../ui/Switch.svelte';
  import { loadSettingsAction } from '../pages';

  const groups = $derived(
    ELEMENTS.map((element) => ({
      element,
      modules: allModules().filter((module) => !module.locked && module[element] !== undefined),
    })).filter((group) => group.modules.length > 0),
  );

  /** A module's control ("Clear history") sits next to its first row. */
  const firstElement = (id: string) =>
    groups.find((group) => group.modules.some((m) => m.id === id))?.element;

  async function toggle(id: string, element: Element, enabled: boolean) {
    try {
      await settings.update({
        disabledElements: switchElement(settings.value.disabledElements, id, element, enabled),
      });
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

{#each groups as group (group.element)}
  {@const elementName = t(`settings.modules.elements.${group.element}`)}
  <section aria-label={elementName}>
    <h4>{elementName}</h4>
    <Panel list>
      {#each group.modules as module (module.id)}
        {@const name = moduleText(module.id, 'name')}
        {@const enabled = isEnabled(settings.value, module.id, group.element)}
        <div class="row" class:off={!enabled}>
          <span class="icon" style:--tint="var(--tint-{module.tint})">
            <Icon path={module.icon} size={17} />
          </span>
          <span class="name">{name}</span>
          {#if firstElement(module.id) === group.element && isActive(settings.value, module.id)}
            {#await loadSettingsAction(module.id) then loaded}
              {#if loaded}
                <loaded.default />
              {/if}
            {/await}
          {/if}
          <Switch
            label={t('settings.modules.switch', { name, element: elementName })}
            checked={enabled}
            onchange={(value) => toggle(module.id, group.element, value)}
          />
        </div>
      {/each}
    </Panel>
  </section>
{/each}

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  h4 {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  .row {
    padding: 9px 0;
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .row + .row {
    border-top: 1px solid var(--border);
  }

  .icon {
    width: 32px;
    height: 32px;
    flex: none;
    border-radius: 10px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    color: var(--tint);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .name {
    flex: 1;
    min-width: 0;
    font-size: 14px;
    font-weight: 500;
  }

  .off .icon,
  .off .name {
    opacity: 0.42;
  }
</style>
