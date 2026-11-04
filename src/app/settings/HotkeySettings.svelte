<script lang="ts">
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { onDestroy } from 'svelte';
  import { commands, errorMessage, events, on } from '../../core/api';
  import { t } from '../../core/i18n';
  import { settings } from '../../core/settings/settings.svelte';
  import Button from '../../ui/Button.svelte';
  import Keycap from '../../ui/Keycap.svelte';
  import Panel from '../../ui/Panel.svelte';
  import {
    MODIFIERS,
    formatKeys,
    heldModifiers,
    hotkeyProblem,
    keyFromEvent,
    keyLabel,
    type HotkeyProblem,
  } from '../hotkey';

  const DEFAULT = ['Shift', 'Win'];

  let recording = $state(false);
  let held = $state<string[]>([]);
  let status = $state<{ text: string; warn: boolean } | null>(null);
  /** On Windows the keyboard hook records, because the webview never sees the Win key. */
  let native = false;
  let unlisteners: UnlistenFn[] = [];

  const keys = $derived(settings.value.hotkey);

  async function start() {
    if (recording) return;
    recording = true;
    held = [];
    status = null;
    native = await commands.hotkeyRecord(true);
    if (native && recording) {
      unlisteners = await Promise.all([
        on<string[]>(events.hotkeyHeld, (names) => (held = names)),
        on<string[]>(events.hotkeyRecorded, finish),
      ]);
    }
  }

  function stop() {
    if (!recording) return;
    recording = false;
    held = [];
    for (const unlisten of unlisteners) unlisten();
    unlisteners = [];
    void commands.hotkeyRecord(false);
  }

  function finish(names: string[]) {
    stop();
    if (names.length > 0) void save(names, 'saved');
  }

  function problemText(problem: HotkeyProblem, names: string[]): string {
    if (problem.kind !== 'reserved') return t(`settings.hotkey.problems.${problem.kind}`);
    return t('settings.hotkey.problems.reserved', {
      keys: formatKeys(names),
      use: t(`settings.hotkey.uses.${problem.use}`),
    });
  }

  async function save(names: string[], done: 'saved' | 'restored') {
    const problem = hotkeyProblem(names);
    if (problem) {
      status = { text: problemText(problem, names), warn: true };
      return;
    }
    try {
      await settings.update({ hotkey: names });
      status = { text: t(`settings.hotkey.${done}`, { keys: formatKeys(names) }), warn: false };
    } catch (error) {
      status = { text: t('errors.generic', { message: errorMessage(error) }), warn: true };
    }
  }

  function onkeydown(event: KeyboardEvent) {
    if (!recording || native) return;
    event.preventDefault();
    event.stopPropagation();
    const modifiers = heldModifiers(event);
    if (event.key === 'Escape' && modifiers.length === 0) return stop();
    const key = keyFromEvent(event);
    if (key) return finish([...modifiers, key]);
    held = MODIFIERS.filter((name) => held.includes(name) || modifiers.includes(name));
  }

  function onkeyup(event: KeyboardEvent) {
    if (!recording || native) return;
    event.preventDefault();
    if (heldModifiers(event).length === 0 && held.length > 0) finish(held);
  }

  onDestroy(stop);
</script>

<Panel>
  <p class="intro">{t('settings.hotkey.intro')}</p>
  <div class="row">
    <button
      class="recorder"
      class:recording
      aria-label={t('settings.hotkey.changeLabel', { keys: formatKeys(keys) })}
      onclick={start}
      onblur={stop}
      {onkeydown}
      {onkeyup}
    >
      {#if recording}
        <span class="waiting">
          {held.length > 0 ? `${formatKeys(held)} + …` : t('settings.hotkey.waiting')}
        </span>
      {:else}
        {#each keys as key, index (key)}
          {#if index > 0}<span class="plus">+</span>{/if}
          <Keycap size="m">{keyLabel(key)}</Keycap>
        {/each}
        <span class="change">{t('settings.hotkey.change')}</span>
      {/if}
    </button>
    <Button onclick={() => save(DEFAULT, 'restored')}>
      {t('settings.hotkey.reset', { keys: formatKeys(DEFAULT) })}
    </Button>
  </div>
  {#if status}
    <p class="status" class:warn={status.warn} role="status">{status.text}</p>
  {/if}
</Panel>

<style>
  .intro {
    margin: 2px 0 14px;
    font-size: 13px;
    line-height: 1.55;
    color: var(--text-muted);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }

  .recorder {
    flex: 1;
    min-width: 260px;
    height: 48px;
    padding: 0 14px;
    border-radius: var(--radius-m);
    border: 1px solid var(--border-strong);
    background: var(--surface-sunken);
    display: flex;
    align-items: center;
    gap: 6px;
    transition: border-color var(--motion-fast);
  }

  .recorder:hover,
  .recording {
    border-color: var(--accent);
  }

  .plus {
    color: var(--text-muted);
  }

  .change {
    margin-left: auto;
    font-size: 12.5px;
    color: var(--accent);
  }

  .waiting {
    font-size: 13px;
    color: var(--accent);
  }

  .status {
    margin-top: 12px;
    font-size: 12.5px;
    color: var(--success);
  }

  .status.warn {
    color: var(--warning);
  }
</style>
