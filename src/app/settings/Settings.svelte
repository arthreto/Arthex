<script lang="ts">
  import type { Component } from 'svelte';
  import { moduleText, t } from '../../core/i18n';
  import { settingsModules } from '../../core/modules';
  import { settings } from '../../core/settings/settings.svelte';
  import Button from '../../ui/Button.svelte';
  import Icon from '../../ui/Icon.svelte';
  import type { IconName } from '../../ui/icons';
  import { navigation } from '../navigation.svelte';
  import { loadSettingsSection } from '../pages';
  import FeaturesSettings from './FeaturesSettings.svelte';
  import GeneralSettings from './GeneralSettings.svelte';
  import HotkeySettings from './HotkeySettings.svelte';
  import UpdatesSettings from './UpdatesSettings.svelte';
  import { updates } from '../updates.svelte';

  type Loaded = { default: Component };

  interface Section {
    id: string;
    title: string;
    icon: { name: IconName } | { path: string };
    /** Shell sections are bundled; a module section loads the first time it opens. */
    body: { component: Component } | { load: () => Promise<Loaded> | null };
    /** A button next to the section title. */
    action?: { label: string; run: () => unknown; disabled?: boolean };
  }

  const sections = $derived<Section[]>([
    {
      id: 'general',
      title: t('settings.sections.general'),
      icon: { name: 'sliders' },
      body: { component: GeneralSettings },
    },
    {
      id: 'hotkey',
      title: t('settings.sections.hotkey'),
      icon: { name: 'keyboard' },
      body: { component: HotkeySettings },
    },
    {
      id: 'features',
      title: t('settings.sections.features'),
      icon: { name: 'toggle' },
      body: { component: FeaturesSettings },
      action: { label: t('settings.features.managePages'), run: () => navigation.open({ kind: 'hub' }) },
    },
    ...settingsModules(settings.value).map((module) => {
      const action = module.settingsSection?.action;
      return {
        id: module.id,
        title: moduleText(module.id, 'name'),
        icon: { path: module.icon },
        body: { load: () => loadSettingsSection(module.id) },
        action: action && { label: action.label(), run: action.run },
      };
    }),
    {
      id: 'updates',
      title: t('settings.sections.updates'),
      icon: { name: 'refresh' },
      body: { component: UpdatesSettings },
      action: { label: t('settings.updates.check'), run: () => updates.check(), disabled: updates.busy },
    },
  ]);

  let currentId = $state('general');
  const current = $derived(sections.find((s) => s.id === currentId) ?? sections[0]);
</script>

<div class="settings fade-in">
  <nav aria-label={t('settings.label')}>
    {#each sections as section (section.id)}
      <button
        class:on={section.id === current?.id}
        aria-current={section.id === current?.id ? 'page' : undefined}
        onclick={() => (currentId = section.id)}
      >
        <Icon {...section.icon} />
        {section.title}
      </button>
    {/each}
  </nav>
  {#if current}
    {@const body = current.body}
    <div class="content scroll">
      <div class="title">
        <h3>{current.title}</h3>
        {#if current.action}
          {@const action = current.action}
          <Button size="small" disabled={action.disabled} onclick={action.run}>{action.label}</Button>
        {/if}
      </div>
      {#if 'component' in body}
        <body.component />
      {:else}
        {#await body.load() then loaded}
          {#if loaded}
            <loaded.default />
          {/if}
        {/await}
      {/if}
    </div>
  {/if}
</div>

<style>
  .settings {
    position: absolute;
    inset: 0;
    display: flex;
    gap: 24px;
  }

  nav {
    width: 214px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  nav button {
    height: 40px;
    padding: 0 12px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text-muted);
    transition: background-color var(--motion-fast);
  }

  nav button:hover {
    background: var(--surface-hover);
  }

  nav button.on {
    background: var(--surface-selected);
    color: var(--text);
    font-weight: 600;
  }

  .content {
    flex: 1;
    min-width: 0;
    padding-right: 6px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  h3 {
    font-size: 18px;
    font-weight: 600;
  }
</style>
