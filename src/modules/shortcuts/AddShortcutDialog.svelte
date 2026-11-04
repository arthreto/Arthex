<script lang="ts">
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { matchesAll } from '../../core/text';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import LinkButton from '../../ui/LinkButton.svelte';
  import Modal from '../../ui/Modal.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import TextField from '../../ui/TextField.svelte';
  import { shortcutsApi, type Program } from './api';
  import { t } from './i18n';
  import { shortcuts } from './shortcuts.svelte';
  import { guessName, siteProblem } from './site';

  let { onclose }: { onclose: () => void } = $props();

  type Kind = 'site' | 'app';

  const formId = $props.id();
  let kind = $state<Kind>('site');
  let address = $state('');
  let name = $state('');
  let icon = $state<string | null>(null);
  let query = $state('');
  let programs = $state<Program[] | null>(null);
  let picked = $state<Program | null>(null);
  let error = $state<string | null>(null);
  let saving = $state(false);

  const kinds = $derived([
    { value: 'site' as const, label: t('dialog.site') },
    { value: 'app' as const, label: t('dialog.app') },
  ]);
  const placeholder = $derived(guessName(address) || 'YouTube');
  const taken = $derived(
    new Set(shortcuts.items.flatMap((item) => (item.kind === 'app' ? [item.path] : []))),
  );
  const choices = $derived(
    (programs ?? []).filter(
      (program) => !taken.has(program.path) && matchesAll(`${program.name} ${program.file}`, query),
    ),
  );

  function fail(failure: unknown) {
    error = shellT('errors.generic', { message: errorMessage(failure) });
  }

  async function choose(next: Kind) {
    kind = next;
    error = null;
    if (next === 'app' && programs === null) {
      try {
        programs = await shortcutsApi.programs();
      } catch (failure) {
        programs = [];
        fail(failure);
      }
    }
  }

  async function pickIcon() {
    try {
      const chosen = await shortcutsApi.pickIcon();
      if (chosen) {
        icon = chosen;
        toast.show(t('iconChosen'));
      }
    } catch (failure) {
      fail(failure);
    }
  }

  async function browse() {
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

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (kind === 'site') {
      const problem = siteProblem(address);
      if (problem) return void (error = t(`errors.${problem}`));
    } else if (!picked) {
      return void (error = t('errors.pickApp'));
    }
    saving = true;
    try {
      if (kind === 'site') {
        await shortcuts.addSite(address, name.trim() || null, icon);
        toast.show(t('added', { name: name.trim() || guessName(address) }));
      } else if (picked) {
        await shortcuts.addApp(picked.path, null);
        toast.show(t('addedApp', { name: picked.name }));
      }
      onclose();
    } catch (failure) {
      fail(failure);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title={t('dialog.title')} {onclose}>
  <Segmented label={t('dialog.type')} options={kinds} value={kind} onchange={choose} />
  <form id={formId} class="fields" onsubmit={submit} oninput={() => (error = null)}>
    {#if kind === 'site'}
      <TextField label={t('dialog.address')} bind:value={address} placeholder="youtube.com" autofocus />
      <TextField label={t('dialog.name')} bind:value={name} {placeholder} />
      <LinkButton onclick={pickIcon}>{t('dialog.customIcon')}</LinkButton>
    {:else}
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
            {#if program.icon}
              <img src={program.icon} alt="" width="28" height="28" />
            {:else}
              <span class="letter">{program.name.charAt(0).toUpperCase()}</span>
            {/if}
            <span class="name">{program.name}</span>
            <span class="file">{program.file}</span>
          </button>
        {:else}
          {#if programs !== null}
            <p class="none">{t('dialog.none')}</p>
          {/if}
        {/each}
      </div>
      <LinkButton onclick={browse}>{t('dialog.browse')}</LinkButton>
    {/if}
  </form>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
  {#snippet actions()}
    <Button onclick={onclose}>{shellT('common.cancel')}</Button>
    <Button variant="primary" type="submit" form={formId} disabled={saving}>{t('dialog.submit')}</Button>
  {/snippet}
</Modal>

<style>
  .fields {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .programs {
    height: 228px;
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

  img,
  .letter {
    width: 28px;
    height: 28px;
    flex: none;
  }

  img {
    object-fit: contain;
  }

  .letter {
    border-radius: 7px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 700;
    color: var(--text-muted);
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
