<script lang="ts">
  import Button from '../../ui/Button.svelte';
  import Check from '../../ui/Check.svelte';
  import { t } from './i18n';
  import { parse, toggleTask, type Span } from './markdown';
  import { notebook } from './notes.svelte';

  let { text }: { text: string } = $props();

  const blocks = $derived(parse(text));
</script>

{#snippet spans(list: Span[])}
  {#each list as span, position (position)}
    <span class:bold={span.bold} class:italic={span.italic} class:strike={span.strike} class:code={span.code}
      >{span.text}</span
    >
  {/each}
{/snippet}

<div class="view scroll">
  {#each blocks as block (block.line)}
    {#if block.kind === 'gap'}
      <div class="gap"></div>
    {:else if block.kind === 'task'}
      <button
        class="task"
        class:done={block.done}
        role="checkbox"
        aria-checked={block.done}
        onclick={() => notebook.update({ text: toggleTask(text, block.line) })}
      >
        <Check checked={block.done} />
        <span>{@render spans(block.spans)}</span>
      </button>
    {:else if block.kind === 'li' || block.kind === 'ol'}
      <div class="item">
        <span class="marker" aria-hidden="true">{block.kind === 'ol' ? `${block.number}.` : '•'}</span>
        <span>{@render spans(block.spans)}</span>
      </div>
    {:else}
      <div class={block.kind}>{@render spans(block.spans)}</div>
    {/if}
  {:else}
    <div class="empty">
      {t('empty')}
      <Button onclick={() => (notebook.mode = 'edit')}>{t('start')}</Button>
    </div>
  {/each}
</div>

<style>
  .view {
    flex: 1;
    min-height: 0;
    padding: 2px 10px 6px 2px;
    overflow-wrap: anywhere;
  }

  .h1 {
    margin: 6px 0 8px;
    font-size: 22px;
    font-weight: 700;
    line-height: 1.3;
    letter-spacing: -0.01em;
  }

  .h2 {
    margin: 12px 0 6px;
    font-size: 17.5px;
    font-weight: 650;
    line-height: 1.35;
  }

  .h3 {
    margin: 12px 0 4px;
    font-size: 12.5px;
    font-weight: 700;
    line-height: 1.4;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  .p,
  .item {
    font-size: 14.5px;
    line-height: 1.6;
  }

  .p {
    margin: 0 0 6px;
  }

  .item {
    margin: 0 0 3px;
    padding-left: 4px;
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  .marker {
    min-width: 14px;
    flex: none;
    font-weight: 600;
    color: var(--accent);
  }

  .task {
    width: calc(100% + 8px);
    margin-left: -8px;
    padding: 6px 8px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 14.5px;
    line-height: 1.5;
    text-align: left;
    transition: background-color var(--motion-fast);
  }

  .task:hover {
    background: var(--surface-hover);
  }

  .task.done > span:last-child {
    text-decoration: line-through;
    color: var(--text-subtle);
  }

  .gap {
    height: 10px;
  }

  .bold {
    font-weight: 700;
  }

  .italic {
    font-style: italic;
  }

  .strike {
    text-decoration: line-through;
  }

  .code {
    padding: 1px 5px;
    border-radius: 4px;
    font-family: var(--font-mono);
    background: var(--surface-hover);
  }

  .empty {
    padding: 48px 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    font-size: 14px;
    color: var(--text-muted);
  }
</style>
