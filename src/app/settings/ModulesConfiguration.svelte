<script lang="ts">
  import { moduleText, t } from '../../core/i18n';
  import { ELEMENTS, allModules, isEnabled } from '../../core/modules';
  import { settings } from '../../core/settings/settings.svelte';
  import Icon from '../../ui/Icon.svelte';
  import Panel from '../../ui/Panel.svelte';
  import ModuleDetailDialog from './ModuleDetailDialog.svelte';

  let openedId = $state<string | null>(null);

  const groups = $derived(
    ELEMENTS.map((element) => ({
      element,
      modules: allModules().filter((module) => !module.locked && module[element] !== undefined),
    })).filter((group) => group.modules.length > 0),
  );

  const opened = $derived(allModules().find((module) => module.id === openedId));
</script>

{#each groups as group (group.element)}
  {@const elementName = t(`settings.modules.elements.${group.element}`)}
  <section aria-label={elementName}>
    <h4>{elementName}</h4>
    <Panel list>
      <div class="tiles">
        {#each group.modules as module (module.id)}
          {@const name = moduleText(module.id, 'name')}
          {@const enabled = isEnabled(settings.value, module.id, group.element)}
          <button
            class="tile"
            class:off={!enabled}
            aria-label={enabled ? name : t('settings.modules.turnedOff', { name })}
            onclick={() => (openedId = module.id)}
          >
            <span class="icon" style:--tint="var(--tint-{module.tint})">
              <Icon path={module.icon} size={22} />
            </span>
            <span class="name">{name}</span>
          </button>
        {/each}
      </div>
    </Panel>
  </section>
{/each}

{#if opened}
  <ModuleDetailDialog module={opened} onclose={() => (openedId = null)} />
{/if}

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

  .tiles {
    margin: 2px -10px;
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .tile {
    width: 92px;
    padding: 10px 4px 8px;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 7px;
    transition: background-color var(--motion-fast);
  }

  .tile:hover {
    background: var(--surface-hover);
  }

  .icon {
    width: 48px;
    height: 48px;
    flex: none;
    box-sizing: border-box;
    border-radius: 14px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    color: var(--tint);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .name {
    max-width: 100%;
    overflow: hidden;
    font-size: 12.5px;
    font-weight: 500;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .off .icon {
    color: var(--text-subtle);
    opacity: 0.5;
  }

  .off .name {
    color: var(--text-subtle);
    opacity: 0.7;
  }
</style>
