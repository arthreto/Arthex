<script lang="ts">
  import { onMount } from 'svelte';
  import { commands, events, on } from '../core/api';
  import { applyAppearance, onSystemThemeChange } from '../core/appearance';
  import { resolveLocale, setLocale } from '../core/i18n';
  import { settings } from '../core/settings/settings.svelte';
  import Toast from '../ui/Toast.svelte';
  import { installHostApi, moduleHost } from './moduleHost.svelte';
  import Onboarding from './onboarding/Onboarding.svelte';
  import Overlay from './Overlay.svelte';

  let systemLocale = $state<string | null>(null);
  let systemThemeTick = $state(0);

  onMount(() => {
    void commands.appInfo().then((info) => (systemLocale = info.systemLocale));
    void settings.load();
    installHostApi();
    void moduleHost.load();
    void moduleHost.checkOpened();
    const modulesChanged = on(events.modulesChanged, () => void moduleHost.load());
    const moduleOpened = on(events.modulesOpened, () => void moduleHost.checkOpened());
    const stopThemeWatch = onSystemThemeChange(() => (systemThemeTick += 1));
    return () => {
      stopThemeWatch();
      for (const listening of [modulesChanged, moduleOpened]) void listening.then((unlisten) => unlisten());
    };
  });

  $effect(() => {
    if (!settings.loaded) return;
    const { language, theme, translucent, reduceMotion } = settings.value;
    setLocale(resolveLocale(language, systemLocale, navigator.language));
    void systemThemeTick;
    applyAppearance(theme, translucent, reduceMotion);
  });
</script>

{#if settings.loaded}
  {#if settings.value.onboarded}
    <Overlay />
  {:else}
    <Onboarding {systemLocale} />
  {/if}
{/if}
<Toast />
