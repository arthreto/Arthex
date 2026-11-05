<script lang="ts">
  import type { ToolProps } from '../../core/modules';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import Panel from '../../ui/Panel.svelte';
  import { evaluate } from './calc';
  import { history, remember } from './history.svelte';
  import { t } from './i18n';
  import { plain, shown } from './result';

  let { input }: ToolProps = $props();

  // The calculation a search result passed on is only where the calculator starts.
  // svelte-ignore state_referenced_locally
  let expression = $state(input ?? '');
  let field: HTMLInputElement | undefined = $state();

  // A lone number is a valid result here, unlike in the search bar.
  const value = $derived(expression.trim() ? evaluate(`=${expression}`) : null);

  type Key = { label: string; insert?: string; action?: 'clear' | 'back' | 'equals'; wide?: boolean };
  const KEYS: Key[] = [
    { label: 'C', action: 'clear' },
    { label: '(', insert: '(' },
    { label: ')', insert: ')' },
    { label: '%', insert: '%' },
    { label: '÷', insert: '÷' },
    { label: '√', insert: 'sqrt(' },
    { label: '7', insert: '7' },
    { label: '8', insert: '8' },
    { label: '9', insert: '9' },
    { label: '×', insert: '×' },
    { label: '^', insert: '^' },
    { label: '4', insert: '4' },
    { label: '5', insert: '5' },
    { label: '6', insert: '6' },
    { label: '−', insert: '-' },
    { label: 'π', insert: 'pi' },
    { label: '1', insert: '1' },
    { label: '2', insert: '2' },
    { label: '3', insert: '3' },
    { label: '+', insert: '+' },
    { label: '⌫', action: 'back' },
    { label: '0', insert: '0' },
    { label: ',', insert: ',' },
    { label: '=', action: 'equals', wide: true },
  ];

  function finish() {
    if (value === null) return;
    remember(expression, shown(value));
    expression = plain(value);
  }

  function press(key: Key) {
    if (key.action === 'clear') expression = '';
    else if (key.action === 'back') expression = expression.slice(0, -1);
    else if (key.action === 'equals') finish();
    else expression += key.insert ?? '';
    field?.focus();
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key !== 'Enter') return;
    event.preventDefault();
    finish();
  }

  async function copy() {
    if (value === null) return;
    await navigator.clipboard.writeText(plain(value));
    toast.show(t('copied', { value: shown(value) }));
  }

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }
</script>

<div class="calculator">
  <div class="pad">
    <div class="screen">
      <input
        bind:this={field}
        bind:value={expression}
        use:focusOnMount
        aria-label={t('expression')}
        spellcheck="false"
        autocomplete="off"
        {onkeydown}
      />
      <output>{value === null ? '' : `= ${shown(value)}`}</output>
    </div>
    <div class="keys">
      {#each KEYS as key (key.label)}
        <span class:wide={key.wide}>
          <Button
            size="large"
            variant={key.action === 'equals' ? 'primary' : 'default'}
            aria-label={key.action === 'clear' ? t('clear') : key.action === 'back' ? t('back') : undefined}
            onclick={() => press(key)}>{key.label}</Button
          >
        </span>
      {/each}
    </div>
  </div>
  <div class="side">
    <div class="side-title">
      <h3>{t('history')}</h3>
      <Button size="small" disabled={value === null} onclick={copy}>{t('copy')}</Button>
    </div>
    {#if history.length > 0}
      <div class="history scroll">
        <Panel list>
          {#each history as entry, position (position)}
            <button class="entry" onclick={() => (expression = entry.expression)}>
              <span class="expression">{entry.expression}</span>
              <span class="result">= {entry.result}</span>
            </button>
          {/each}
        </Panel>
      </div>
    {/if}
  </div>
</div>

<style>
  .calculator {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 28px;
  }

  .pad {
    width: 440px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .screen {
    padding: 14px 20px 16px;
    border-radius: var(--radius-l);
    background: var(--surface);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }

  input {
    width: 100%;
    background: none;
    border: none;
    outline: none;
    text-align: right;
    font-size: 18px;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  output {
    min-height: 52px;
    font-size: 40px;
    font-weight: 600;
    letter-spacing: -0.01em;
    font-variant-numeric: tabular-nums;
  }

  .keys {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 8px;
  }

  .keys :global(.button) {
    width: 100%;
    font-size: 18px;
  }

  .wide {
    grid-column: span 2;
  }

  .side {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .side-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h3 {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  .history {
    min-height: 0;
  }

  .entry {
    width: calc(100% + 16px);
    margin-left: -8px;
    padding: 10px 8px;
    border-radius: var(--radius-s);
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }

  .entry + .entry {
    border-top: 1px solid var(--border);
  }

  .entry:hover {
    background: var(--surface-hover);
  }

  .expression {
    font-size: 13px;
    color: var(--text-subtle);
  }

  .result {
    font-size: 17px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
</style>
