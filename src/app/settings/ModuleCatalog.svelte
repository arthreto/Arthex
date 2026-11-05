<script lang="ts">
  import { onMount } from 'svelte';
  import { commands, errorMessage, type ModuleOffer } from '../../core/api';
  import { moduleText, t } from '../../core/i18n';
  import { ELEMENTS, elementsOf, getModule, type Element } from '../../core/modules';
  import type { Tint } from '../../core/modules/manifest';
  import { matchesAll } from '../../core/text';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import Chip from '../../ui/Chip.svelte';
  import Icon from '../../ui/Icon.svelte';
  import { icons } from '../../ui/icons';
  import Panel from '../../ui/Panel.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import TextField from '../../ui/TextField.svelte';
  import { moduleHost } from '../moduleHost.svelte';
  import { knownElements, pickTexts, summary, tintOf } from './moduleSummary';
  import UninstallModuleDialog from './UninstallModuleDialog.svelte';

  interface Row {
    id: string;
    name: string;
    description: string;
    icon: string;
    tint: Tint;
    elements: Element[];
    size?: number;
    state: ModuleOffer['state'] | 'dev';
  }

  /** `null` while the catalog loads. */
  let offers = $state<ModuleOffer[] | null>(null);
  let failed = $state(false);
  let query = $state('');
  let filter = $state<Element | 'all'>('all');
  let busy = $state<string | null>(null);
  let uninstalling = $state<Row | null>(null);

  const filters = $derived([
    { value: 'all' as const, label: t('settings.modules.all') },
    ...ELEMENTS.map((element) => ({ value: element, label: t(`settings.modules.elements.${element}`) })),
  ]);

  function offerRow(offer: ModuleOffer): Row {
    const texts = pickTexts(offer.texts);
    const installed = moduleHost.installed.find((module) => module.id === offer.id);
    return {
      id: offer.id,
      name: texts?.name ?? offer.id,
      description: texts?.description ?? '',
      icon: offer.icon,
      tint: tintOf(offer.tint),
      elements: knownElements(offer.elements),
      size: offer.size,
      state: installed?.dev ? 'dev' : offer.state,
    };
  }

  /** Installed from a file, or missing from the catalog: still listed, so it can be uninstalled. */
  function installedRow(id: string, dev: boolean): Row {
    const module = getModule(id);
    return {
      id,
      name: module ? moduleText(id, 'name') : id,
      description: module ? moduleText(id, 'description') : '',
      icon: module?.icon ?? icons.modules,
      tint: module?.tint ?? 'blue',
      elements: module ? elementsOf(module) : [],
      state: dev ? 'dev' : 'installed',
    };
  }

  const rows = $derived.by(() => {
    const listed = (offers ?? []).map(offerRow);
    const others = moduleHost.installed
      .filter((module) => !listed.some((row) => row.id === module.id))
      .map((module) => installedRow(module.id, module.dev));
    return [...listed, ...others].filter(
      (row) =>
        (filter === 'all' || row.elements.includes(filter)) &&
        matchesAll(`${row.name} ${row.description}`, query),
    );
  });

  async function refresh() {
    failed = false;
    try {
      offers = await commands.modulesCatalog();
    } catch {
      failed = true;
      offers = [];
    }
  }

  function markOffer(id: string, state: ModuleOffer['state']) {
    offers = (offers ?? []).map((offer) => (offer.id === id ? { ...offer, state } : offer));
  }

  function report(error: unknown) {
    toast.show(t('errors.generic', { message: errorMessage(error) }));
  }

  async function install(row: Row) {
    busy = row.id;
    try {
      await commands.modulesInstall(row.id);
      await moduleHost.load();
      markOffer(row.id, 'installed');
      toast.show(
        t(row.state === 'update' ? 'settings.modules.updated' : 'settings.modules.installed', {
          name: row.name,
        }),
      );
    } catch (error) {
      report(error);
    } finally {
      busy = null;
    }
  }

  async function installFile() {
    try {
      const installed = await commands.modulesInstallFile();
      if (!installed) return;
      await moduleHost.load();
      const offer = offers?.find((candidate) => candidate.id === installed.id);
      if (offer?.version === installed.version) markOffer(installed.id, 'installed');
      toast.show(t('settings.modules.installed', { name: installedRow(installed.id, false).name }));
    } catch (error) {
      report(error);
    }
  }

  async function uninstall(row: Row, deleteData: boolean) {
    uninstalling = null;
    try {
      await commands.modulesUninstall(row.id, deleteData);
      await moduleHost.load();
      markOffer(row.id, 'available');
      toast.show(t('settings.modules.uninstalled', { name: row.name }));
    } catch (error) {
      report(error);
    }
  }

  onMount(() => void refresh());
</script>

<div class="bar">
  <div class="search">
    <TextField
      label={t('settings.modules.search')}
      hideLabel
      placeholder={t('settings.modules.search')}
      bind:value={query}
    />
  </div>
  <Button onclick={installFile}>{t('settings.modules.fromFile')}</Button>
</div>
<Segmented
  label={t('settings.modules.filter')}
  options={filters}
  value={filter}
  onchange={(value) => (filter = value)}
/>

{#if failed}
  <div class="notice">
    {t('settings.modules.failed')}
    <Button size="small" onclick={refresh}>{t('settings.modules.retry')}</Button>
  </div>
{/if}

{#if offers === null}
  <p class="notice">{t('settings.modules.loading')}</p>
{:else if rows.length === 0}
  <p class="notice">
    {t(query.trim() || filter !== 'all' ? 'settings.modules.noMatch' : 'settings.modules.empty')}
  </p>
{:else}
  <Panel list>
    {#each rows as row (row.id)}
      <div class="row">
        <span class="icon" style:color="var(--tint-{row.tint})">
          <Icon path={row.icon} size={20} />
        </span>
        <span class="text">
          <span class="name">{row.name}</span>
          {#if row.description}<span class="description">{row.description}</span>{/if}
          <span class="meta">{summary(row.elements, row.size)}</span>
        </span>
        {#if row.state === 'dev'}
          <Chip label={t('settings.modules.dev')} />
        {:else if row.state === 'outdated'}
          <span class="meta">{t('settings.modules.outdated')}</span>
        {:else}
          {#if row.state === 'available' || row.state === 'update'}
            <Button size="small" variant="primary" disabled={busy !== null} onclick={() => install(row)}>
              {busy === row.id
                ? t('settings.modules.installing')
                : t(row.state === 'update' ? 'settings.modules.update' : 'settings.modules.install')}
            </Button>
          {/if}
          {#if row.state === 'installed' || row.state === 'update'}
            <Button size="small" disabled={busy !== null} onclick={() => (uninstalling = row)}>
              {t('settings.modules.uninstall')}
            </Button>
          {/if}
        {/if}
      </div>
    {/each}
  </Panel>
{/if}

{#if uninstalling}
  {@const row = uninstalling}
  <UninstallModuleDialog
    name={row.name}
    onconfirm={(deleteData) => uninstall(row, deleteData)}
    onclose={() => (uninstalling = null)}
  />
{/if}

<style>
  .bar {
    display: flex;
    align-items: flex-end;
    gap: 10px;
  }

  .search {
    flex: 1;
    min-width: 0;
  }

  .notice {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 0;
    font-size: 13.5px;
    color: var(--text-muted);
  }

  .row {
    padding: 12px 0;
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .row + .row {
    border-top: 1px solid var(--border);
  }

  .icon {
    width: 40px;
    height: 40px;
    flex: none;
    border-radius: 12px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .name {
    font-size: 14px;
    font-weight: 600;
  }

  .description {
    font-size: 12.5px;
    color: var(--text-muted);
  }

  .meta {
    font-size: 12px;
    color: var(--text-subtle);
  }
</style>
