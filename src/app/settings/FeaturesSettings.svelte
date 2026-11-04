<script lang="ts">
  import { errorMessage } from '../../core/api';
  import { moduleText, t } from '../../core/i18n';
  import { isEnabled, modules } from '../../core/modules';
  import { settings } from '../../core/settings/settings.svelte';
  import { toast } from '../../core/toast.svelte';
  import Icon from '../../ui/Icon.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Switch from '../../ui/Switch.svelte';
  import { loadSettingsAction } from '../pages';

  const features = $derived(modules.filter((module) => module.kind === 'feature'));

  async function toggle(id: string, enabled: boolean) {
    try {
      await settings.update({ modules: { ...settings.value.modules, [id]: enabled } });
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

<Panel list>
  {#each features as feature (feature.id)}
    {@const name = moduleText(feature.id, 'name')}
    {@const enabled = isEnabled(settings.value, feature.id)}
    <div class="row" class:off={!enabled}>
      <span class="icon" style:--tint="var(--tint-{feature.tint})">
        <Icon path={feature.icon} size={19} />
      </span>
      <span class="text">
        <span class="name">{name}</span>
        <span class="description">{moduleText(feature.id, 'description')}</span>
      </span>
      {#if enabled}
        {#await loadSettingsAction(feature.id) then loaded}
          {#if loaded}
            {@const Action = loaded.default}
            <Action />
          {/if}
        {/await}
      {/if}
      <Switch
        label={t(enabled ? 'settings.features.turnOff' : 'settings.features.turnOn', { name })}
        checked={enabled}
        onchange={(value) => toggle(feature.id, value)}
      />
    </div>
  {/each}
</Panel>

<style>
  .row {
    padding: 11px 0;
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .row + .row {
    border-top: 1px solid var(--border);
  }

  .icon {
    width: 36px;
    height: 36px;
    flex: none;
    border-radius: 11px;
    background: color-mix(in srgb, var(--tint) 16%, transparent);
    color: var(--tint);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .off .icon,
  .off .text {
    opacity: 0.42;
  }

  .name {
    font-size: 14px;
    font-weight: 600;
  }

  .description {
    font-size: 12.5px;
    color: var(--text-muted);
  }
</style>
