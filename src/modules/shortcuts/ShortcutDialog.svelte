<script lang="ts">
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { TINTS, type Tint } from '../../core/modules/manifest';
  import { matchesAll } from '../../core/text';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import Check from '../../ui/Check.svelte';
  import Icon from '../../ui/Icon.svelte';
  import LinkButton from '../../ui/LinkButton.svelte';
  import Modal from '../../ui/Modal.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import TextField from '../../ui/TextField.svelte';
  import { shortcutsApi, type Draft, type Look, type Program, type Shortcut, type Target } from './api';
  import { t } from './i18n';
  import ShortcutIcon from './ShortcutIcon.svelte';
  import { shortcuts } from './shortcuts.svelte';
  import { guessName, shortAddress, siteProblem } from './site';
  import { SYMBOL_NAMES, SYMBOLS, type SymbolName } from './symbols';

  interface Props {
    /** Without `editing`, the dialog adds a new shortcut, maybe filled from the search. */
    editing?: Shortcut | null;
    prefill?: (Target & { name?: string }) | null;
    onclose: () => void;
  }

  let { editing = null, prefill = null, onclose }: Props = $props();

  type Kind = Target['kind'];
  type LookType = Look['type'];

  /** The pictogram and colour a new shortcut of each kind starts with. */
  const DEFAULT_SYMBOL: Record<Kind, { name: SymbolName; tint: Tint }> = {
    site: { name: 'globe', tint: 'blue' },
    app: { name: 'star', tint: 'violet' },
    path: { name: 'folder', tint: 'amber' },
    command: { name: 'terminal', tint: 'green' },
  };

  const formId = $props.id();
  // The dialog is created for one shortcut, so its fields start from it once.
  // svelte-ignore state_referenced_locally
  const start: (Target & { name?: string }) | null = editing ?? prefill;
  let kind = $state<Kind>(start?.kind ?? 'site');
  let address = $state(start?.kind === 'site' ? shortAddress(start.url) : '');
  let path = $state(start?.kind === 'path' ? start.path : '');
  let line = $state(start?.kind === 'command' ? start.line : '');
  let name = $state(start?.name ?? '');
  // svelte-ignore state_referenced_locally
  let keyword = $state(editing?.keyword ?? '');
  // svelte-ignore state_referenced_locally
  let onHome = $state(editing?.onHome ?? !prefill);
  // svelte-ignore state_referenced_locally
  let inSearch = $state(editing?.inSearch ?? true);
  // svelte-ignore state_referenced_locally
  const custom =
    editing && !editing.symbol && editing.icon && (editing.kind === 'path' || editing.kind === 'command');
  // svelte-ignore state_referenced_locally
  let look = $state<LookType>(
    editing?.symbol ? 'symbol' : custom ? 'image' : kind === 'path' || kind === 'command' ? 'symbol' : 'auto',
  );
  // svelte-ignore state_referenced_locally
  let symbol = $state<{ name: string; tint: Tint }>(editing?.symbol ?? DEFAULT_SYMBOL[kind]);
  // svelte-ignore state_referenced_locally
  let image = $state<string | null>(custom ? (editing?.icon ?? null) : null);

  let query = $state('');
  let programs = $state<Program[] | null>(null);
  let picked = $state<Program | null>(null);
  let error = $state<string | null>(null);
  let saving = $state(false);

  const kinds = $derived(
    (['site', 'app', 'path', 'command'] as const).map((value) => ({ value, label: t(`dialog.${value}`) })),
  );
  const looks = $derived(
    (['auto', 'symbol', 'image'] as const).map((value) => ({ value, label: t(`dialog.look.${value}`) })),
  );
  const placeholder = $derived(
    kind === 'site' ? guessName(address) || 'YouTube' : kind === 'app' ? (picked?.name ?? '') : '',
  );
  const taken = $derived(
    new Set(
      shortcuts.items.flatMap((item) => (item.kind === 'app' && item.id !== editing?.id ? [item.path] : [])),
    ),
  );
  const choices = $derived(
    (programs ?? []).filter(
      (program) => !taken.has(program.path) && matchesAll(`${program.name} ${program.file}`, query),
    ),
  );
  const preview = $derived({
    name: name.trim() || placeholder || '?',
    symbol: look === 'symbol' ? symbol : undefined,
    icon:
      look === 'image'
        ? (image ?? undefined)
        : kind === 'app'
          ? (picked?.icon ?? undefined)
          : kind === 'site' && editing?.kind === 'site'
            ? editing.icon
            : undefined,
  });

  function fail(failure: unknown) {
    error = shellT('errors.generic', { message: errorMessage(failure) });
  }

  async function chooseKind(next: Kind) {
    if (look === 'symbol' && symbol.name === DEFAULT_SYMBOL[kind].name) symbol = DEFAULT_SYMBOL[next];
    if (look !== 'image') look = next === 'path' || next === 'command' ? 'symbol' : look;
    kind = next;
    error = null;
    if (next === 'app' && programs === null) {
      try {
        programs = await shortcutsApi.programs();
      } catch (failure) {
        programs = [];
        fail(failure);
      }
      if (editing?.kind === 'app') {
        const current = editing.path;
        picked = programs.find((program) => program.path === current) ?? null;
      }
    }
  }

  // svelte-ignore state_referenced_locally
  if (kind === 'app') void chooseKind('app');

  async function chooseLook(next: LookType) {
    if (next !== 'image') return void (look = next);
    try {
      const chosen = await shortcutsApi.pickIcon();
      if (!chosen) return;
      image = chosen;
      look = 'image';
    } catch (failure) {
      fail(failure);
    }
  }

  async function browseProgram() {
    try {
      const program = await shortcutsApi.browseProgram();
      if (!program) return;
      programs = [program, ...(programs ?? []).filter((other) => other.path !== program.path)];
      picked = program;
      query = '';
    } catch (failure) {
      fail(failure);
    }
  }

  async function browseFolder() {
    try {
      path = (await shortcutsApi.browseFolder()) ?? path;
    } catch (failure) {
      fail(failure);
    }
  }

  /** What the form says, or why it cannot be saved yet. */
  function target(): Target | string {
    switch (kind) {
      case 'site': {
        const problem = siteProblem(address);
        return problem ? t(`errors.${problem}`) : { kind, url: address };
      }
      case 'app':
        return picked ? { kind, path: picked.path } : t('errors.pickApp');
      case 'path':
        return path.trim() ? { kind, path: path.trim() } : t('errors.missingPath');
      case 'command':
        if (!line.trim()) return t('errors.missingCommand');
        return name.trim() ? { kind, line: line.trim() } : t('errors.missingName');
    }
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    const chosen = target();
    if (typeof chosen === 'string') return void (error = chosen);
    if (!onHome && !inSearch) return void (error = t('errors.noPlace'));
    const lookSent: Look =
      look === 'symbol'
        ? { type: 'symbol', ...symbol }
        : look === 'image' && image
          ? { type: 'image', icon: image }
          : { type: 'auto' };
    const draft: Draft = {
      ...chosen,
      name: name.trim() || null,
      look: lookSent,
      keyword: keyword.trim() || null,
      onHome,
      inSearch,
    };
    saving = true;
    try {
      await shortcuts.save(editing?.id ?? null, draft);
      const shown = name.trim() || placeholder;
      const fetching = !editing && kind === 'site' && lookSent.type === 'auto';
      toast.show(t(editing ? 'edited' : fetching ? 'added' : 'addedPlain', { name: shown }));
      onclose();
    } catch (failure) {
      fail(failure);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title={t(editing ? 'dialog.editTitle' : 'dialog.title')} {onclose}>
  <Segmented label={t('dialog.type')} options={kinds} value={kind} onchange={chooseKind} />
  <form id={formId} class="fields" onsubmit={submit} oninput={() => (error = null)}>
    {#if kind === 'site'}
      <TextField label={t('dialog.address')} bind:value={address} placeholder="youtube.com" autofocus />
    {:else if kind === 'app'}
      <TextField
        label={t('dialog.search')}
        hideLabel
        bind:value={query}
        placeholder={t('dialog.search')}
        autofocus
      />
      <div class="programs scroll" role="listbox" aria-label={t('dialog.installed')}>
        {#each choices as program (program.path)}
          <button
            type="button"
            role="option"
            aria-selected={picked?.path === program.path}
            class:on={picked?.path === program.path}
            onclick={() => ((picked = program), (error = null))}
          >
            <ShortcutIcon shortcut={{ name: program.name, icon: program.icon ?? undefined }} size={28} />
            <span class="name">{program.name}</span>
            <span class="file">{program.file}</span>
          </button>
        {:else}
          {#if programs !== null}
            <p class="none">{t('dialog.none')}</p>
          {/if}
        {/each}
      </div>
      <LinkButton onclick={browseProgram}>{t('dialog.browse')}</LinkButton>
    {:else if kind === 'path'}
      <div class="row">
        <div class="grow">
          <TextField
            label={t('dialog.path')}
            bind:value={path}
            placeholder="C:\Users\…\Documents"
            autofocus
          />
        </div>
        <Button onclick={browseFolder}>{t('dialog.browseFolder')}</Button>
      </div>
    {:else}
      <TextField
        label={t('dialog.command')}
        bind:value={line}
        placeholder="code C:\Projets"
        spellcheck="false"
        class="mono"
        autofocus
      />
    {/if}

    <div class="pair">
      <TextField label={t('dialog.name')} bind:value={name} {placeholder} />
      <TextField label={t('dialog.keyword')} bind:value={keyword} placeholder="mail" spellcheck="false" />
    </div>

    <div class="look">
      <ShortcutIcon shortcut={preview} size={44} />
      <Segmented label={t('dialog.icon')} options={looks} value={look} onchange={chooseLook} />
    </div>
    {#if look === 'symbol'}
      <div class="symbols" role="radiogroup" aria-label={t('dialog.look.symbol')}>
        {#each SYMBOL_NAMES as symbolName (symbolName)}
          <button
            type="button"
            role="radio"
            aria-checked={symbol.name === symbolName}
            aria-label={t(`symbols.${symbolName}`)}
            class:on={symbol.name === symbolName}
            style:color="var(--tint-{symbol.tint})"
            onclick={() => (symbol = { ...symbol, name: symbolName })}
          >
            <Icon path={SYMBOLS[symbolName]} size={19} />
          </button>
        {/each}
      </div>
      <div class="tints" role="radiogroup" aria-label={t('dialog.colour')}>
        {#each TINTS as tint, position (tint)}
          <button
            type="button"
            role="radio"
            aria-checked={symbol.tint === tint}
            aria-label={t('dialog.colourNumber', { number: position + 1 })}
            class:on={symbol.tint === tint}
            style:background="var(--tint-{tint})"
            onclick={() => (symbol = { ...symbol, tint })}
          ></button>
        {/each}
      </div>
    {/if}

    <div class="places">
      <button
        type="button"
        class="place"
        role="checkbox"
        aria-checked={onHome}
        onclick={() => (onHome = !onHome)}
      >
        <Check checked={onHome} />
        {t('dialog.onHome')}
      </button>
      <button
        type="button"
        class="place"
        role="checkbox"
        aria-checked={inSearch}
        onclick={() => (inSearch = !inSearch)}
      >
        <Check checked={inSearch} />
        {t('dialog.inSearch')}
      </button>
    </div>
  </form>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
  {#snippet actions()}
    <Button onclick={onclose}>{shellT('common.cancel')}</Button>
    <Button variant="primary" type="submit" form={formId} disabled={saving}
      >{t(editing ? 'dialog.save' : 'dialog.submit')}</Button
    >
  {/snippet}
</Modal>

<style>
  .fields {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .row {
    display: flex;
    align-items: flex-end;
    gap: 8px;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .fields :global(.mono) {
    font-family: var(--font-mono);
    font-size: 14px;
  }

  .pair {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 150px;
    gap: 12px;
  }

  .look {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .symbols {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .symbols button {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-s);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color var(--motion-fast);
  }

  .symbols button:hover {
    background: var(--surface-hover);
  }

  .symbols button.on {
    background: var(--surface-selected);
  }

  .tints {
    display: flex;
    gap: 10px;
    padding-left: 6px;
  }

  .tints button {
    width: 22px;
    height: 22px;
    border-radius: 50%;
  }

  .tints button.on {
    outline: 2px solid var(--text);
    outline-offset: 2px;
  }

  .places {
    display: flex;
    gap: 22px;
  }

  .place {
    margin-left: -8px;
    padding: 6px 8px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13.5px;
  }

  .place:hover {
    background: var(--surface-hover);
  }

  .programs {
    height: 150px;
    padding-right: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .programs button {
    width: 100%;
    min-height: 44px;
    padding: 0 12px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    gap: 12px;
    text-align: left;
    transition: background-color var(--motion-fast);
  }

  .programs button:hover {
    background: var(--surface-hover);
  }

  .programs button.on {
    background: var(--surface-selected);
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14px;
  }

  .file {
    font-size: 12.5px;
    color: var(--text-subtle);
  }

  .none {
    margin: 14px 6px;
    font-size: 13.5px;
    color: var(--text-muted);
  }

  .error {
    margin-top: -6px;
    font-size: 13px;
    color: var(--danger-text);
  }
</style>
