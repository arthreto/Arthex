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
  import Skeleton from '../../ui/Skeleton.svelte';
  import TextField from '../../ui/TextField.svelte';
  import ToggleChip from '../../ui/ToggleChip.svelte';
  import { moduleHost } from '../moduleHost.svelte';
  import { navigation } from '../navigation.svelte';
  import ModuleDetails from './ModuleDetails.svelte';
  import { modulesView } from './modulesView.svelte';
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
    installed: boolean;
    offer?: ModuleOffer;
  }

  const CHIP_WIDTHS = ['152px', '86px', '112px', '74px', '98px'];
  const ROW_WIDTHS = [
    { name: '140px', description: '62%', meta: '38%' },
    { name: '108px', description: '48%', meta: '30%' },
    { name: '176px', description: '70%', meta: '42%' },
  ];
  const ARRIVAL_STEP_MS = 40;
  const MAX_ARRIVAL_STEPS = 8;

  let arrived = 0;

  let query = $state('');
  let filter = $state<Element | 'all'>('all');
  let tag = $state<Tag | null>(null);
  let busy = $state<string | null>(null);
  /** The module whose details are open. */
  let openId = $state<string | null>(null);
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
      installed: installed !== undefined,
      offer,
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
      installed: true,
    };
  }

  const tagLabel = (name: Tag) => t(`settings.modules.tags.${name}`);

  const versionLabel = ({ version, nextVersion }: Row) =>
    nextVersion
      ? t('settings.modules.versionUpdate', { from: version, to: nextVersion })
      : t('settings.modules.version', { version });

  const allRows = $derived.by(() => {
    const listed = (modulesView.offers ?? []).map(offerRow);
    const others = moduleHost.installed
      .filter((module) => !listed.some((row) => row.id === module.id))
      .map(installedRow);
    return [...listed, ...others];
  });

  /** Only the categories that hold a module, so the row stays short. */
  const tags = $derived(TAGS.filter((name) => allRows.some((row) => row.tags.includes(name))));

  const loading = $derived(modulesView.offers === null);

  const rows = $derived(
    allRows.filter(
      (row) =>
        (filter === 'all' || row.elements.includes(filter)) &&
        (tag === null || row.tags.includes(tag)) &&
        matchesAll(`${row.name} ${row.description} ${row.tags.map(tagLabel).join(' ')}`, query),
    ),
  );

  const nothing = $derived(!loading && rows.length === 0);
  const placeholders = $derived(loading && modulesView.lookedForKept);

  /** The rows that come with the first answer of the catalog appear one after the other. */
  function arrive(row: HTMLElement) {
    if (!modulesView.arriving) return;
    const delay = Math.min(arrived, MAX_ARRIVAL_STEPS) * ARRIVAL_STEP_MS;
    arrived += 1;
    row.style.animation = `fade-in var(--motion-base) ease-out ${delay}ms both`;
    row.addEventListener('animationend', () => (row.style.animation = ''), { once: true });
  }

  const narrowed = $derived(query.trim() !== '' || filter !== 'all' || tag !== null);

  function showAll() {
    query = '';
    filter = 'all';
    tag = null;
  }

  function report(error: unknown) {
    toast.show(t('errors.generic', { message: errorMessage(error) }));
  }

  async function install(row: Row) {
    busy = row.id;
    try {
      await commands.modulesInstall(row.id);
      await moduleHost.load();
      modulesView.markOffer(row.id, 'installed');
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
      const offer = modulesView.offers?.find((candidate) => candidate.id === installed.id);
      if (offer?.version === installed.version) modulesView.markOffer(installed.id, 'installed');
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
      modulesView.markOffer(row.id, 'available');
      toast.show(t('settings.modules.uninstalled', { name: row.name }));
    } catch (error) {
      report(error);
    }
  }

  const toggleDetails = (id: string) => (openId = openId === id ? null : id);

  onMount(async () => {
    if (loading) await modulesView.showKept();
    await modulesView.load();
  });
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
{#if placeholders}
  <div class="tags">
    {#each CHIP_WIDTHS as width, index (index)}
      <Skeleton {width} height="28px" radius="var(--radius-pill)" />
    {/each}
  </div>
{:else if tags.length > 0}
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

{#if nothing && narrowed}
  <EmptyState
    icon={icons.search}
    title={query.trim()
      ? t('settings.modules.noMatch', { query: query.trim() })
      : t('settings.modules.noneOfKind')}
    text={t('settings.modules.tryOther')}
  >
    <Button onclick={showAll}>{t('settings.modules.showAll')}</Button>
  </EmptyState>
{:else if nothing && modulesView.failed}
  <EmptyState icon={icons.refresh} title={t('settings.modules.failed')}>
    <Button onclick={() => modulesView.load()}>{t('settings.modules.retry')}</Button>
  </EmptyState>
{:else if nothing}
  <EmptyState icon={icons.modules} title={t('settings.modules.empty')} />
{:else}
  {#if modulesView.failed}
    <div class="notice">
      {t('settings.modules.failed')}
      <Button size="small" onclick={() => modulesView.load()}>{t('settings.modules.retry')}</Button>
    </div>
  {/if}
  {#if placeholders}
    <span class="visually-hidden" role="status">{t('settings.modules.loading')}</span>
  {/if}
  <Panel list>
    {#each rows as row (row.id)}
      {@const open = openId === row.id}
      <div class="entry" use:arrive>
        <div class="row">
          <button
            class="open"
            aria-expanded={open}
            aria-label={t(open ? 'settings.modules.hideDetails' : 'settings.modules.showDetails', {
              name: row.name,
            })}
            onclick={() => toggleDetails(row.id)}
          >
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
            <span class="chevron" class:turned={open} aria-hidden="true">
              <Icon path="m7 10 5 5 5-5" size={16} stroke={2} />
            </span>
          </button>
          {#if row.state === 'dev'}
            <Chip label={t('settings.modules.dev')} />
          {:else}
            {#if row.state === 'outdated'}
              <Button
                size="small"
                variant="needs-attention"
                onclick={() => navigation.open({ kind: 'settings', section: 'updates' })}
              >
                {t('settings.modules.updateArthex')}
              </Button>
            {:else if row.state === 'available' || row.state === 'update'}
              <Button size="small" variant="primary" disabled={busy !== null} onclick={() => install(row)}>
                {busy === row.id
                  ? t('settings.modules.installing')
                  : t(row.state === 'update' ? 'settings.modules.update' : 'settings.modules.install')}
              </Button>
            {/if}
            {#if row.installed}
              <Button size="small" disabled={busy !== null} onclick={() => (uninstalling = row)}>
                {t('settings.modules.uninstall')}
              </Button>
            {/if}
          {/if}
        </div>
        {#if open}
          <ModuleDetails {...row} />
        {/if}
      </div>
    {/each}
    {#if placeholders}
      {#each ROW_WIDTHS as widths, index (index)}
        <div class="row">
          <Skeleton width="40px" height="40px" radius="12px" />
          <span class="text placeholder">
            <Skeleton width={widths.name} height="14px" />
            <Skeleton width={widths.description} height="12px" />
            <Skeleton width={widths.meta} height="10px" />
          </span>
          <Skeleton width="92px" height="32px" radius="var(--radius-pill)" />
        </div>
      {/each}
    {/if}
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

  .entry + .entry,
  .entry + .row,
  .row + .row {
    border-top: 1px solid var(--border);
  }

  .open {
    flex: 1;
    min-width: 0;
    margin: -6px 0;
    padding: 6px 8px 6px 6px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    gap: 14px;
    text-align: left;
    transition: background-color var(--motion-fast);
  }

  .open:hover {
    background: var(--surface-hover);
  }

  .chevron {
    flex: none;
    display: flex;
    color: var(--text-subtle);
    transition: transform var(--motion-fast);
  }

  .chevron.turned {
    transform: rotate(180deg);
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

  .placeholder {
    gap: 8px;
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
