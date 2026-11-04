<script lang="ts">
  import { t } from '../../core/i18n';
  import Button from '../../ui/Button.svelte';
  import Icon from '../../ui/Icon.svelte';
  import Keycap from '../../ui/Keycap.svelte';
  import { keyLabel } from '../hotkey';

  interface Props {
    hotkey: string[];
    recap: { label: string; value: string }[];
    onopen: () => void;
  }

  let { hotkey, recap, onopen }: Props = $props();
</script>

<div class="done">
  <svg class="check" aria-hidden="true" width="96" height="96" viewBox="0 0 52 52">
    <circle cx="26" cy="26" r="24" />
    <path d="M15 27l7 7 15-16" />
  </svg>
  <div class="heading">
    <h1>{t('onboarding.done.title')}</h1>
    <p>{t('onboarding.done.subtitle')}</p>
  </div>
  <div class="keys">
    {#each hotkey as key, index (key)}
      {#if index > 0}+{/if}
      <Keycap size="l">{keyLabel(key)}</Keycap>
    {/each}
  </div>
  <ul class="recap">
    {#each recap as item (item.label)}
      <li>{item.label}{t('onboarding.recap.separator')}<strong>{item.value}</strong></li>
    {/each}
  </ul>
  <span class="open">
    <Button variant="primary" size="large" onclick={onopen}>
      {t('onboarding.done.open')}
      <Icon name="arrowRight" stroke={2.2} />
    </Button>
  </span>
</div>

<style>
  .done {
    width: 720px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 26px;
    animation: step 380ms var(--ease-out) both;
  }

  @keyframes step {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.99);
    }
  }

  .check {
    fill: none;
    stroke: var(--accent);
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .check circle {
    stroke-width: 2.5;
    stroke-dasharray: 160;
    animation: draw 700ms ease-out both;
  }

  .check path {
    stroke-width: 3;
    stroke-dasharray: 40;
    animation: draw 450ms 550ms ease-out both;
  }

  @keyframes draw {
    from {
      stroke-dashoffset: var(--length, 160);
    }
  }

  .check path {
    --length: 40;
  }

  .heading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    text-align: center;
  }

  h1 {
    font-size: 46px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  p {
    font-size: 16px;
    color: var(--text-muted);
  }

  .keys {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 22px;
    color: var(--text-muted);
  }

  .recap {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
    list-style: none;
  }

  .recap li {
    padding: 7px 13px;
    border-radius: 15px;
    background: var(--surface-raised);
    font-size: 13px;
    color: var(--text-muted);
  }

  strong {
    font-weight: 500;
    color: var(--text);
  }

  .open {
    margin-top: 6px;
  }
</style>
