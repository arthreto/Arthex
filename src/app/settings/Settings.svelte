<script lang="ts">
  import type { Component } from 'svelte';
  import { t, type ShellKey } from '../../core/i18n';
  import Icon from '../../ui/Icon.svelte';
  import type { IconName } from '../../ui/icons';
  import GeneralSettings from './GeneralSettings.svelte';
  import HotkeySettings from './HotkeySettings.svelte';

  interface Section {
    id: string;
    title: ShellKey;
    icon: IconName;
    component: Component;
  }

  const sections: Section[] = [
    { id: 'general', title: 'settings.sections.general', icon: 'sliders', component: GeneralSettings },
    { id: 'hotkey', title: 'settings.sections.hotkey', icon: 'keyboard', component: HotkeySettings },
  ];

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
        <Icon name={section.icon} />
        {t(section.title)}
      </button>
    {/each}
  </nav>
  {#if current}
    {@const Body = current.component}
    <div class="content scroll">
      <h3>{t(current.title)}</h3>
      <Body />
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

  h3 {
    font-size: 18px;
    font-weight: 600;
  }
</style>
