<script lang="ts">
  import { errorMessage } from '../core/api';
  import { moduleText, t } from '../core/i18n';
  import { getModule, isEnabled, launcherIds, toggleFavorite } from '../core/modules';
  import { settings } from '../core/settings/settings.svelte';
  import { toast } from '../core/toast.svelte';
  import Button from '../ui/Button.svelte';
  import Icon from '../ui/Icon.svelte';
  import HubZones from './HubZones.svelte';
  import { navigation } from './navigation.svelte';
  import { modulesView } from './settings/modulesView.svelte';

  let customizing = $state(false);

  const pages = $derived(
    launcherIds(settings.value).flatMap((id) => {
      const module = getModule(id);
      return module
        ? [
            {
              module,
              enabled: isEnabled(settings.value, id, 'page'),
              favorite: settings.value.favorites.includes(id),
            },
          ]
        : [];
    }),
  );

  function open(id: string, enabled: boolean) {
    if (enabled) navigation.open({ kind: 'page', id });
    else toast.show(t('hub.off', { name: moduleText(id, 'name') }));
  }

  function addModules() {
    modulesView.tab = 'catalog';
    navigation.open({ kind: 'settings', section: 'modules' });
  }

  async function pin(id: string, favorite: boolean) {
    const name = moduleText(id, 'name');
    try {
      await settings.update({ favorites: toggleFavorite(settings.value.favorites, id) });
      toast.show(t(favorite ? 'hub.unpinned' : 'hub.pinned', { name }));
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

<div class="hub fade-in">
  <div class="header">
    <h2>{t('hub.title')}</h2>
    <Button
      variant={customizing ? 'primary' : 'default'}
      aria-pressed={customizing}
      onclick={() => (customizing = !customizing)}
    >
      <Icon name="edit" size={15} stroke={2} />
      {t(customizing ? 'hub.done' : 'hub.customize')}
    </Button>
  </div>
  {#if customizing}
    <HubZones />
  {:else}
    <div class="center">
      <ul class="grid">
        {#each pages as { module, enabled, favorite } (module.id)}
          {@const name = moduleText(module.id, 'name')}
          <li class="card" class:off={!enabled}>
            <button
              class="open"
              aria-label={t('hub.open', { name })}
              onclick={() => open(module.id, enabled)}
            >
              <span class="icon" style:color="var(--tint-{module.tint})">
                <Icon path={module.icon} size={46} stroke={1.6} />
              </span>
              <span class="text">
                <span class="name">{name}</span>
                <span class="description">{moduleText(module.id, 'description')}</span>
              </span>
            </button>
            {#if enabled}
              <button
                class="pin"
                class:on={favorite}
                aria-pressed={favorite}
                aria-label={t(favorite ? 'hub.unpin' : 'hub.pin', { name })}
                title={t(favorite ? 'hub.unpin' : 'hub.pin', { name })}
                onclick={() => pin(module.id, favorite)}
              >
                <Icon name="pin" size={18} stroke={2.1} filled={favorite} />
              </button>
            {/if}
          </li>
        {/each}
        <li class="card">
          <button class="open" onclick={addModules}>
            <span class="icon add"><Icon name="plus" size={40} stroke={1.6} /></span>
            <span class="name">{t('hub.addModules')}</span>
          </button>
        </li>
      </ul>
    </div>
  {/if}
</div>

<style>
  .hub {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  h2 {
    flex: 1;
    font-size: 20px;
    font-weight: 600;
  }

  .center {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .grid {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 22px;
  }

  .card {
    position: relative;
    width: 212px;
    min-height: 220px;
    display: flex;
  }

  .open {
    width: 100%;
    flex: 1;
    padding: 22px 14px 18px;
    border-radius: var(--radius-xl);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 22px;
    text-align: center;
    transition: background-color var(--motion-fast);
  }

  .open:hover {
    background: var(--surface);
  }

  .icon {
    width: 96px;
    height: 96px;
    border-radius: 26px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color var(--motion-fast);
  }

  .open:hover .icon {
    border-color: var(--text-subtle);
  }

  .text {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .name {
    font-size: 16px;
    font-weight: 600;
  }

  .description {
    font-size: 12.5px;
    line-height: 1.45;
    color: var(--text-muted);
  }

  .add {
    background: none;
    border: 1.5px dashed var(--border-strong);
    color: var(--text-muted);
  }

  .off .icon,
  .off .text {
    opacity: 0.45;
    filter: grayscale(1);
  }

  .pin {
    position: absolute;
    top: 9px;
    left: 127px;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-dialog);
    border: 1px solid var(--border-strong);
    color: var(--text-muted);
    opacity: 0;
    transition:
      opacity var(--motion-fast),
      background-color var(--motion-fast);
  }

  .card:hover .pin,
  .pin:focus-visible,
  .pin.on {
    opacity: 1;
  }

  .pin.on {
    background: var(--accent);
    border: 3px solid var(--surface-dialog);
    color: var(--on-accent);
  }
</style>
