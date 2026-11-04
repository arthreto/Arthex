<script lang="ts">
  import { onMount } from 'svelte';
  import logo from '../../assets/logo.png';
  import { availableLocales, getLocale, t, textIn } from '../../core/i18n';
  import Button from '../../ui/Button.svelte';
  import Icon from '../../ui/Icon.svelte';

  const WORD_MS = 2900;
  const LETTER_MS = 55;

  let { onstart }: { onstart: () => void } = $props();

  /** "Hello" then "Welcome" in every language, the current one first. */
  const words = $derived.by(() => {
    const current = getLocale();
    const locales = availableLocales()
      .map((meta) => meta.code)
      .sort((a, b) => Number(b === current) - Number(a === current));
    return [
      ...locales.map((code) => textIn(code, 'onboarding.hello')),
      ...locales.map((code) => textIn(code, 'onboarding.welcome')),
    ];
  });

  let index = $state(0);
  const word = $derived(words[index % words.length] ?? '');

  onMount(() => {
    const timer = setInterval(() => (index += 1), WORD_MS);
    return () => clearInterval(timer);
  });
</script>

<div class="welcome">
  <img class="logo" src={logo} alt="Arthex" width="112" height="112" />
  <h1 class="visually-hidden">{t('onboarding.welcome')}</h1>
  <div class="words" aria-hidden="true">
    {#key index}
      <span class="word">
        {#each [...word] as letter, position (position)}
          <span class="letter" style:animation-delay="{position * LETTER_MS}ms">{letter}</span>
        {/each}
      </span>
    {/key}
  </div>
  <span class="start">
    <Button variant="primary" size="large" onclick={onstart}>
      {t('onboarding.start')}
      <Icon name="arrowRight" stroke={2.2} />
    </Button>
  </span>
</div>

<style>
  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .logo {
    display: block;
    margin-bottom: 34px;
    filter: drop-shadow(var(--shadow-logo));
    animation: logo 900ms var(--ease-out) both;
  }

  .words {
    width: 900px;
    height: 130px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 104px;
    font-weight: 600;
    letter-spacing: -0.035em;
    line-height: 1;
  }

  .word {
    display: flex;
    justify-content: center;
    white-space: pre;
    animation: word-out 2900ms ease-in forwards;
  }

  .letter {
    display: inline-block;
    animation: letter 760ms var(--ease-out) both;
  }

  .start {
    margin-top: 56px;
    animation: rise 700ms 1.4s var(--ease-out) both;
  }

  @keyframes logo {
    from {
      opacity: 0;
      transform: scale(0.86);
    }
  }

  @keyframes letter {
    from {
      opacity: 0;
      filter: blur(14px);
      transform: translateY(26px) scale(0.96);
    }
  }

  @keyframes word-out {
    0%,
    84% {
      opacity: 1;
      filter: none;
      transform: none;
    }
    100% {
      opacity: 0;
      filter: blur(12px);
      transform: translateY(-14px);
    }
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
