<script lang="ts">
  import { errorMessage } from '../../core/api';
  import { availableLocales, getLocale, resolveLocale, t } from '../../core/i18n';
  import { isWindows } from '../../core/platform';
  import { settings } from '../../core/settings/settings.svelte';
  import type { SettingsPatch, Theme } from '../../core/settings/types';
  import { toast } from '../../core/toast.svelte';
  import Check from '../../ui/Check.svelte';
  import { formatKeys } from '../hotkey';
  import Choice from './Choice.svelte';
  import Done from './Done.svelte';
  import Flag from './Flag.svelte';
  import Step from './Step.svelte';
  import TaskbarPreview from './TaskbarPreview.svelte';
  import ThemePreview from './ThemePreview.svelte';
  import Tick from './Tick.svelte';
  import Welcome from './Welcome.svelte';

  let { systemLocale }: { systemLocale: string | null } = $props();

  const THEMES: Theme[] = ['dark', 'light', 'system'];
  const THEME_STEP = 2;
  const SEARCH_STEP = 3;
  const STARTUP_STEP = 4;
  const TOTAL = isWindows ? STARTUP_STEP : STARTUP_STEP - 1;

  let step = $state(0);
  const language = $derived(getLocale());
  const systemLanguage = $derived(resolveLocale(systemLocale, navigator.language));
  const languages = $derived(availableLocales());

  const recap = $derived([
    {
      label: t('onboarding.recap.language'),
      value: languages.find((meta) => meta.code === language)?.nativeName ?? language,
    },
    { label: t('onboarding.recap.theme'), value: t(`settings.general.themes.${settings.value.theme}`) },
    ...(isWindows
      ? [
          {
            label: t('onboarding.recap.search'),
            value: t(settings.value.taskbarSearch ? 'onboarding.recap.on' : 'onboarding.recap.off'),
          },
        ]
      : []),
    {
      label: t('onboarding.recap.startup'),
      value: t(settings.value.autostart ? 'onboarding.recap.on' : 'onboarding.recap.off'),
    },
  ]);

  async function save(patch: SettingsPatch) {
    try {
      await settings.update(patch);
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }

  const next = () => (step += !isWindows && step === THEME_STEP ? 2 : 1);
  const back = () => (step -= !isWindows && step === STARTUP_STEP ? 2 : 1);
</script>

<div class="onboarding">
  <div class="backdrop" aria-hidden="true">
    <span class="glow one"></span>
    <span class="glow two"></span>
  </div>

  <main>
    {#if step === 0}
      <Welcome onstart={next} />
    {:else if step === 1}
      <Step
        number={1}
        total={TOTAL}
        title={t('onboarding.language.title')}
        subtitle={t('onboarding.language.subtitle')}
        onback={back}
        onnext={next}
      >
        <div class="options languages" role="radiogroup" aria-label={t('onboarding.language.title')}>
          {#each languages as meta (meta.code)}
            <Choice
              layout="card"
              selected={meta.code === language}
              onselect={() => save({ language: meta.code })}
            >
              <Flag code={meta.code} />
              <span class="label">
                <span class="name">{meta.nativeName}</span>
                <span class="note">{meta.code === systemLanguage ? t('onboarding.language.system') : ''}</span
                >
              </span>
              <Tick on={meta.code === language} size={26} />
            </Choice>
          {/each}
        </div>
      </Step>
    {:else if step === THEME_STEP}
      <Step
        number={THEME_STEP}
        total={TOTAL}
        title={t('onboarding.theme.title')}
        subtitle={t('onboarding.theme.subtitle')}
        onback={back}
        onnext={next}
      >
        <div class="options themes" role="radiogroup" aria-label={t('onboarding.theme.title')}>
          {#each THEMES as theme (theme)}
            <Choice layout="tile" selected={settings.value.theme === theme} onselect={() => save({ theme })}>
              <ThemePreview {theme} />
              <span class="tile-label">
                <span class="theme-name">{t(`settings.general.themes.${theme}`)}</span>
                <Tick on={settings.value.theme === theme} size={22} />
              </span>
            </Choice>
          {/each}
        </div>
        <button
          class="translucent"
          role="checkbox"
          aria-checked={settings.value.translucent}
          onclick={() => save({ translucent: !settings.value.translucent })}
        >
          <Check checked={settings.value.translucent} size={22} />
          {t('settings.general.translucent')}
        </button>
      </Step>
    {:else if step === SEARCH_STEP}
      <Step
        number={SEARCH_STEP}
        total={TOTAL}
        title={t('onboarding.search.title')}
        subtitle={t('onboarding.search.subtitle')}
        onback={back}
        onnext={next}
      >
        <TaskbarPreview />
        <div class="options startup" role="radiogroup" aria-label={t('onboarding.search.title')}>
          {#each [true, false] as taskbarSearch (taskbarSearch)}
            <Choice
              layout="row"
              selected={settings.value.taskbarSearch === taskbarSearch}
              onselect={() => save({ taskbarSearch })}
            >
              <Tick on={settings.value.taskbarSearch === taskbarSearch} />
              <span class="row-label"
                >{t(taskbarSearch ? 'onboarding.search.yes' : 'onboarding.search.no')}</span
              >
            </Choice>
          {/each}
        </div>
      </Step>
    {:else if step === STARTUP_STEP}
      <Step
        number={TOTAL}
        total={TOTAL}
        title={t('onboarding.startup.title')}
        subtitle={t('onboarding.startup.subtitle', { keys: formatKeys(settings.value.hotkey) })}
        onback={back}
        onnext={next}
      >
        <div class="options startup" role="radiogroup" aria-label={t('onboarding.startup.title')}>
          {#each [true, false] as autostart (autostart)}
            <Choice
              layout="row"
              selected={settings.value.autostart === autostart}
              onselect={() => save({ autostart })}
            >
              <Tick on={settings.value.autostart === autostart} />
              <span class="row-label"
                >{t(autostart ? 'onboarding.startup.yes' : 'onboarding.startup.no')}</span
              >
              {#if autostart}<span class="badge">{t('onboarding.startup.recommended')}</span>{/if}
            </Choice>
          {/each}
        </div>
      </Step>
    {:else}
      <Done hotkey={settings.value.hotkey} {recap} onopen={() => save({ onboarded: true })} />
    {/if}
  </main>
</div>

<style>
  .onboarding {
    position: fixed;
    inset: 0;
    overflow: hidden;
  }

  .backdrop {
    position: absolute;
    inset: 0;
    background: var(--veil);
    animation: fade 600ms ease-out both;
  }

  .glow {
    position: absolute;
    border-radius: 50%;
  }

  .one {
    left: 20%;
    top: 15%;
    width: 520px;
    height: 520px;
    background: var(--glow-1);
    filter: blur(90px);
    animation: drift-one 18s ease-in-out infinite alternate;
  }

  .two {
    left: 44%;
    top: 25%;
    width: 480px;
    height: 480px;
    background: var(--glow-2);
    filter: blur(100px);
    animation: drift-two 22s ease-in-out infinite alternate;
  }

  @keyframes fade {
    from {
      opacity: 0;
    }
  }

  @keyframes drift-one {
    from {
      transform: translate(-60px, -30px);
    }
    to {
      transform: translate(80px, 40px) scale(1.15);
    }
  }

  @keyframes drift-two {
    from {
      transform: translate(50px, 30px) scale(1.1);
    }
    to {
      transform: translate(-70px, -40px) scale(0.95);
    }
  }

  main {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .options {
    display: grid;
    gap: 18px;
  }

  .languages {
    width: 660px;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .themes {
    width: 660px;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
  }

  .startup {
    width: 560px;
    gap: 12px;
  }

  .translucent {
    height: 44px;
    margin-top: -14px;
    padding: 0 14px;
    border-radius: 22px;
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 15px;
    font-weight: 500;
  }

  .label {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    text-align: left;
  }

  .name {
    font-size: 21px;
    font-weight: 600;
  }

  .note {
    min-height: 17px;
    white-space: nowrap;
    font-size: 13px;
    color: var(--text-muted);
  }

  .tile-label {
    width: 100%;
    padding: 0 6px 4px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .theme-name {
    font-size: 16px;
    font-weight: 600;
  }

  .row-label {
    flex: 1;
    text-align: left;
    font-size: 17px;
    font-weight: 500;
  }

  .badge {
    padding: 5px 11px;
    border-radius: 13px;
    background: var(--surface-selected);
    color: var(--accent);
    font-size: 12px;
    font-weight: 600;
  }
</style>
