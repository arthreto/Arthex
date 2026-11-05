<script lang="ts">
  import { onMount } from 'svelte';
  import { commands, errorMessage, type InstalledModule, type ModuleOffer } from '../../core/api';
  import { moduleText, t } from '../../core/i18n';
  import {
    ELEMENTS,
    TAGS,
    elementsOf,
    getModule,
    knownTags,
    type Element,
    type Tag,
  } from '../../core/modules';
  import type { Tint } from '../../core/modules/manifest';
  import { matchesAll } from '../../core/text';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import Chip from '../../ui/Chip.svelte';
  import EmptyState from '../../ui/EmptyState.svelte';
  import Icon from '../../ui/Icon.svelte';
  import { icons } from '../../ui/icons';
  import Panel from '../../ui/Panel.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import TextField from '../../ui/TextField.svelte';
  import ToggleChip from '../../ui/ToggleChip.svelte';
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
    tags: Tag[];
    size?: number;
    /** What is installed, or else what the catalog offers. */
    version: string;
    /** What an update brings. */
    nextVersion?: string;
    state: ModuleOffer['state'] | 'dev';
  }

  /** `null` while the catalog loads. */
  let offers = $state<ModuleOffer[] | null>(null);
  let failed = $state(false);
  let query = $state('');
  let filter = $state<Element | 'all'>('all');
  let tag = $state<Tag | null>(null);
  let busy = $state<string | null>(null);
  let checking = $state(false);
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
      tags: knownTags(offer.tags),
      size: offer.size,
      version: installed?.version ?? offer.version,
      nextVersion: offer.state === 'update' ? offer.version : undefined,
      state: installed?.dev ? 'dev' : offer.state,
    };
  }

  /** Installed from a file, or missing from the catalog: still listed, so it can be uninstalled. */
  function installedRow({ id, version, dev }: InstalledModule): Row {
    const module = getModule(id);
    return {
      id,
      name: module ? moduleText(id, 'name') : id,
      description: module ? moduleText(id, 'description') : '',
      icon: module?.icon ?? icons.modules,
      tint: module?.tint ?? 'blue',
      elements: module ? elementsOf(module) : [],
      tags: [...(module?.tags ?? [])],
      version,
      state: dev ? 'dev' : 'installed',
    };
  }

  const tagLabel = (name: Tag) => t(`settings.modules.tags.${name}`);

  const versionLabel = ({ version, nextVersion }: Row) =>
    nextVersion
      ? t('settings.modules.versionUpdate', { from: version, to: nextVersion })
      : t('settings.modules.version', { version });

  const allRows = $derived.by(() => {
    const listed = (offers ?? []).map(offerRow);
    const others = moduleHost.installed
      .filter((module) => !listed.some((row) => row.id === module.id))
      .map(installedRow);
    return [...listed, ...others];
  });

  /** Only the categories that hold a module, so the row stays short. */
  const tags = $derived(TAGS.filter((name) => allRows.some((row) => row.tags.includes(name))));

  const rows = $derived(
    allRows.filter(
      (row) =>
        (filter === 'all' || row.elements.includes(filter)) &&
        (tag === null || row.tags.includes(tag)) &&
        matchesAll(`${row.name} ${row.description} ${row.tags.map(tagLabel).join(' ')}`, query),
    ),
  );

  async function refresh() {
    failed = false;
    try {
      offers = await commands.modulesCatalog();
    } catch {
      failed = true;
      offers = [];
    }
  }

  async function checkUpdates() {
    checking = true;
    await refresh();
    checking = false;
    if (failed) return;
    const count = allRows.filter((row) => row.state === 'update').length;
    if (count === 0) toast.show(t('settings.modules.upToDate'));
    else if (count === 1) toast.show(t('settings.modules.updateFound'));
    else toast.show(t('settings.modules.updatesFound', { count }));
  }

  const narrowed = $derived(query.trim() !== '' || filter !== 'all' || tag !== null);

  function showAll() {
    query = '';
    filter = 'all';
    tag = null;
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
      toast.show(t('settings.modules.installed', { name: installedRow(installed).name }));
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
  <Button disabled={checking} onclick={checkUpdates}>
    {t(checking ? 'settings.modules.checking' : 'settings.modules.check')}
  </Button>
  <Button onclick={installFile}>{t('settings.modules.fromFile')}</Button>
</div>
<Segmented
  label={t('settings.modules.filter')}
  options={filters}
  value={filter}
  onchange={(value) => (filter = value)}
/>
{#if tags.length > 0}
  <div class="tags" role="group" aria-label={t('settings.modules.categories')}>
    <ToggleChip label={t('settings.modules.allTags')} pressed={tag === null} onclick={() => (tag = null)} />
    {#each tags as name (name)}
      <ToggleChip
        label={tagLabel(name)}
        pressed={tag === name}
        onclick={() => (tag = tag === name ? null : name)}
      />
    {/each}
  </div>
{/if}

{#if offers === null}
  <EmptyState icon={icons.modules} title={t('settings.modules.loading')} />
{:else if rows.length === 0 && narrowed}
  <EmptyState
    icon={icons.search}
    title={query.trim()
      ? t('settings.modules.noMatch', { query: query.trim() })
      : t('settings.modules.noneOfKind')}
    text={t('settings.modules.tryOther')}
  >
    <Button onclick={showAll}>{t('settings.modules.showAll')}</Button>
  </EmptyState>
{:else if rows.length === 0 && failed}
  <EmptyState icon={icons.refresh} title={t('settings.modules.failed')}>
    <Button onclick={refresh}>{t('settings.modules.retry')}</Button>
  </EmptyState>
{:else if rows.length === 0}
  <EmptyState icon={icons.modules} title={t('settings.modules.empty')} />
{:else}
  {#if failed}
    <div class="notice">
      {t('settings.modules.failed')}
      <Button size="small" onclick={refresh}>{t('settings.modules.retry')}</Button>
    </div>
  {/if}
  <Panel list>
    {#each rows as row (row.id)}
      <div class="row">
        <span class="icon" style:color="var(--tint-{row.tint})">
          <Icon path={row.icon} size={20} />
        </span>
        <span class="text">
          <span class="name">{row.name}</span>
          {#if row.description}<span class="description">{row.description}</span>{/if}
          <span class="meta">
            {[...row.tags.map(tagLabel), summary(row.elements, row.size), versionLabel(row)].join(' · ')}
          </span>
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

  .bar > :global(.button) {
    height: 44px;
  }

  .search {
    flex: 1;
    min-width: 0;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
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
