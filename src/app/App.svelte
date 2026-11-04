<script lang="ts">
  import { onMount } from 'svelte';
  import { commands } from '../core/api';
  import { applyAppearance, onSystemThemeChange } from '../core/appearance';
  import { resolveLocale, setLocale } from '../core/i18n';
  import { settings } from '../core/settings/settings.svelte';
  import Toast from '../ui/Toast.svelte';
  import Onboarding from './onboarding/Onboarding.svelte';
  import Overlay from './Overlay.svelte';

  let systemLocale = $state<string | null>(null);
  let systemThemeTick = $state(0);

  onMount(() => {
    void commands.appInfo().then((info) => (systemLocale = info.systemLocale));
    void settings.load();
    return onSystemThemeChange(() => (systemThemeTick += 1));
  });

  $effect(() => {
    if (!settings.loaded) return;
    const { language, theme, reduceMotion } = settings.value;
    setLocale(resolveLocale(language, systemLocale, navigator.language));
    void systemThemeTick;
    applyAppearance(theme, reduceMotion);
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
