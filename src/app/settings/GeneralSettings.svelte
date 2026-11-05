<script lang="ts">
  import { onMount } from 'svelte';
  import logo from '../../assets/logo.png';
  import { commands, errorMessage } from '../../core/api';
  import { availableLocales, getLocale, t } from '../../core/i18n';
  import { settings } from '../../core/settings/settings.svelte';
  import type { SettingsPatch, Theme } from '../../core/settings/types';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import Switch from '../../ui/Switch.svelte';
  import { dialogs } from '../dialogs.svelte';
  import SettingRow from './SettingRow.svelte';

  let version = $state('');
  onMount(async () => {
    version = (await commands.appInfo()).version;
  });

  const languages = $derived(
    availableLocales().map((meta) => ({ value: meta.code, label: meta.nativeName })),
  );
  const themes = $derived(
    (['system', 'dark', 'light'] as const).map((value) => ({
      value,
      label: t(`settings.general.themes.${value}`),
    })),
  );

  async function save(patch: SettingsPatch) {
    try {
      await settings.update(patch);
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }

  async function open(folder: () => Promise<null>) {
    try {
      await folder();
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

<Panel list>
  <SettingRow label={t('settings.general.language')}>
    <Segmented
      label={t('settings.general.language')}
      options={languages}
      value={getLocale()}
      onchange={(language) => save({ language })}
    />
  </SettingRow>
  <SettingRow label={t('settings.general.theme')}>
    <Segmented
      label={t('settings.general.theme')}
      options={themes}
      value={settings.value.theme}
      onchange={(theme: Theme) => save({ theme })}
    />
  </SettingRow>
  <SettingRow label={t('settings.general.autostart')}>
    <Switch
      label={t('settings.general.autostart')}
      checked={settings.value.autostart}
      onchange={(autostart) => save({ autostart })}
    />
  </SettingRow>
  <SettingRow label={t('settings.general.reduceMotion')}>
    <Switch
      label={t('settings.general.reduceMotion')}
      checked={settings.value.reduceMotion}
      onchange={(reduceMotion) => save({ reduceMotion })}
    />
  </SettingRow>
  <SettingRow label={t('settings.general.debugLog')}>
    <Button size="small" onclick={() => open(commands.appOpenLogDir)}>{t('settings.general.openLogs')}</Button
    >
    <Switch
      label={t('settings.general.debugLog')}
      checked={settings.value.debugLog}
      onchange={(debugLog) => save({ debugLog })}
    />
  </SettingRow>
</Panel>

<Panel>
  <div class="about">
    <img src={logo} alt="" width="34" height="34" />
    <span class="version">{t('settings.general.version', { version })}</span>
    <Button onclick={() => open(commands.appOpenDataDir)}>{t('settings.general.data')}</Button>
    <Button onclick={() => (dialogs.uninstall = true)}>{t('settings.general.uninstall')}</Button>
  </div>
</Panel>

<style>
  .about {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .version {
    flex: 1;
    font-size: 14px;
    font-weight: 600;
  }
</style>
