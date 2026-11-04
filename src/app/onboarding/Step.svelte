<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '../../core/i18n';
  import Button from '../../ui/Button.svelte';
  import Icon from '../../ui/Icon.svelte';

  const STEPS = 3;

  interface Props {
    number: number;
    title: string;
    subtitle: string;
    onback: () => void;
    onnext: () => void;
    children: Snippet;
  }

  let { number, title, subtitle, onback, onnext, children }: Props = $props();
</script>

<div class="step">
  <div class="dots" role="img" aria-label={t('onboarding.step', { n: number, total: STEPS })}>
    {#each { length: STEPS }, index (index)}
      <span class:done={index < number} class:current={index + 1 === number}></span>
    {/each}
  </div>
  <div class="heading">
    <h1>{title}</h1>
    <p>{subtitle}</p>
  </div>
  {@render children()}
  <div class="nav">
    <button class="back" onclick={onback}>
      <Icon name="chevronLeft" size={17} stroke={2.2} />
      {t('onboarding.back')}
    </button>
    <Button variant="primary" size="large" onclick={onnext}>
      {number === STEPS ? t('onboarding.finish') : t('onboarding.next')}
      <Icon name="arrowRight" stroke={2.2} />
    </Button>
  </div>
</div>

<style>
  .step {
    width: 720px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 34px;
    animation: step 380ms var(--ease-out) both;
  }

  @keyframes step {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.99);
    }
  }

  .dots {
    display: flex;
    gap: 6px;
  }

  .dots span {
    width: 8px;
    height: 8px;
    border-radius: 4px;
    background: var(--border-strong);
    transition:
      width 240ms,
      background-color 240ms;
  }

  .dots .done {
    background: var(--accent);
  }

  .dots .current {
    width: 26px;
  }

  .heading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    text-align: center;
  }

  h1 {
    font-size: 40px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  p {
    font-size: 16px;
    color: var(--text-muted);
  }

  .nav {
    margin-top: 4px;
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .back {
    height: 46px;
    padding: 0 16px;
    border-radius: 23px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 15px;
    font-weight: 500;
    color: var(--text-muted);
    transition: background-color var(--motion-fast);
  }

  .back:hover {
    background: var(--surface-hover);
  }
</style>
