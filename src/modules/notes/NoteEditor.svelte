<script lang="ts">
  import { tick } from 'svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import { t } from './i18n';
  import { continueList, togglePrefix, wrap, type Edit } from './markdown';
  import { notebook } from './notes.svelte';

  let { text }: { text: string } = $props();

  let area = $state<HTMLTextAreaElement>();

  async function apply(edit: (text: string, start: number, end: number) => Edit) {
    const start = area?.selectionStart ?? text.length;
    const end = area?.selectionEnd ?? text.length;
    const result = edit(text, start, end);
    notebook.update({ text: result.text });
    await tick();
    area?.focus();
    area?.setSelectionRange(result.start, result.end);
  }

  const bold = () => apply((text, start, end) => wrap(text, start, end, '**', t('boldText')));
  const italic = () => apply((text, start, end) => wrap(text, start, end, '*', t('italicText')));
  const prefix = (mark: string) => apply((text, start, end) => togglePrefix(text, start, end, mark));

  function onkeydown(event: KeyboardEvent) {
    const key = event.key.toLowerCase();
    if ((event.ctrlKey || event.metaKey) && (key === 'b' || key === 'i')) {
      event.preventDefault();
      void (key === 'b' ? bold() : italic());
    } else if (event.key === 'Enter' && !event.shiftKey && area) {
      const result = continueList(text, area.selectionStart);
      if (!result) return;
      event.preventDefault();
      void apply(() => result);
    }
  }

  /** Keeps the caret in the text while a toolbar button is pressed. */
  const keepFocus = (event: MouseEvent) => event.preventDefault();
</script>

<div class="toolbar" role="toolbar" aria-label={t('toolbar')}>
  <IconButton icon="bold" label={t('bold')} onmousedown={keepFocus} onclick={bold} />
  <IconButton icon="italic" label={t('italic')} onmousedown={keepFocus} onclick={italic} />
  <IconButton icon="heading" label={t('heading')} onmousedown={keepFocus} onclick={() => prefix('## ')} />
  <span class="separator" aria-hidden="true"></span>
  <IconButton icon="list" label={t('list')} onmousedown={keepFocus} onclick={() => prefix('- ')} />
  <IconButton icon="task" label={t('task')} onmousedown={keepFocus} onclick={() => prefix('- [ ] ')} />
</div>
<!-- svelte-ignore a11y_autofocus -->
<textarea
  bind:this={area}
  aria-label={t('content')}
  placeholder={t('placeholder')}
  value={text}
  autofocus
  oninput={(event) => notebook.update({ text: event.currentTarget.value })}
  {onkeydown}></textarea>

<style>
  .toolbar {
    padding: 4px;
    border-radius: 10px;
    align-self: flex-start;
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--surface-sunken);
  }

  .separator {
    width: 1px;
    height: 18px;
    margin: 0 4px;
    background: var(--border-strong);
  }

  textarea {
    flex: 1;
    min-height: 0;
    padding: 14px 16px;
    border-radius: 12px;
    resize: none;
    font: inherit;
    font-family: var(--font-mono);
    font-size: 13.5px;
    line-height: 1.6;
    color: var(--text);
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    outline: none;
  }

  textarea:focus-visible {
    border-color: var(--accent);
  }
</style>
