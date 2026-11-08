<script lang="ts">
  import { onMount } from 'svelte';
  import type { ModuleOffer } from '../../core/api';
  import { formatBytes } from '../../core/format';
  import { getLocale, t } from '../../core/i18n';
  import type { Element, Tag } from '../../core/modules';
  import type { Tint } from '../../core/modules/manifest';
  import Icon from '../../ui/Icon.svelte';
  import Keycap from '../../ui/Keycap.svelte';
  import Skeleton from '../../ui/Skeleton.svelte';
  import ModuleImageDialog from './ModuleImageDialog.svelte';
  import { dayName, keywordList, languageName, rightClickEntries, systemNames } from './moduleDetails';
  import { pickTexts } from './moduleSummary';
  import { modulesView } from './modulesView.svelte';

  interface Props {
    name: string;
    description: string;
    icon: string;
    tint: Tint;
    version: string;
    nextVersion?: string;
    installed: boolean;
    size?: number;
    tags: readonly Tag[];
    elements: readonly Element[];
    /** Missing for a module installed from a file. */
    offer?: ModuleOffer;
  }

  let { name, description, icon, tint, version, nextVersion, installed, size, tags, elements, offer }: Props =
    $props();

  const texts = $derived(offer && pickTexts(offer.texts));
  const locale = $derived(getLocale());

  const versionText = $derived.by(() => {
    if (nextVersion) return t('settings.modules.details.versionNext', { version, next: nextVersion });
    return installed ? t('settings.modules.details.versionInstalled', { version }) : version;
  });

  const facts = $derived(
    [
      { label: t('settings.modules.details.version'), value: versionText },
      offer?.updated && {
        label: t('settings.modules.details.updated'),
        value: dayName(offer.updated, locale),
      },
      offer && {
        label: t('settings.modules.details.languages'),
        value: Object.keys(offer.texts)
          .map((code) => languageName(code, locale))
          .join(', '),
      },
      offer && {
        label: t('settings.modules.details.system'),
        value: systemNames(offer.platforms) ?? t('settings.modules.details.allSystems'),
      },
      size !== undefined && { label: t('settings.modules.details.size'), value: formatBytes(size) },
      offer?.shared.length && {
        label: t('settings.modules.details.shared'),
        value: offer.shared
          .map((part) =>
            part.installed
              ? t('settings.modules.details.sharedInstalled', { name: part.name })
              : t('settings.modules.details.sharedSize', { name: part.name, size: formatBytes(part.size) }),
          )
          .join(', '),
      },
    ].filter((fact) => !!fact),
  );

  const keywords = $derived(keywordList(texts?.keywords ?? ''));
  const examples = $derived([...new Set(texts?.examples)]);
  const rightClick = $derived(offer ? rightClickEntries(offer.contextMenu, locale) : []);
  const preview = $derived(offer?.hasPreview ? modulesView.previews[offer.id] : null);
  let enlarged = $state(false);

  onMount(() => {
    if (offer?.hasPreview) void modulesView.askPreview(offer.id);
  });
</script>

<div class="details fade-in">
  <div class="top">
    {#if preview}
      <button
        class="image enlarge fade-in"
        aria-label={t('settings.modules.details.enlarge', { name })}
        onclick={() => (enlarged = true)}
      >
        <img src={preview} alt="" />
      </button>
    {:else if preview === undefined}
      <Skeleton width="400px" height="250px" radius="12px" />
    {:else}
      <span class="image placeholder" style:color="var(--tint-{tint})" aria-hidden="true">
        <Icon path={icon} size={48} stroke={1.4} />
      </span>
    {/if}
    <dl>
      {#each facts as fact (fact.label)}
        <dt>{fact.label}</dt>
        <dd>{fact.value}</dd>
      {/each}
      {#if tags.length > 0}
        <dt>{t('settings.modules.details.categories')}</dt>
        <dd class="tags">
          {#each tags as tag (tag)}
            <span class="tag">{t(`settings.modules.tags.${tag}`)}</span>
          {/each}
        </dd>
      {/if}
    </dl>
  </div>
  {#if elements.length > 0}
    <section>
      <h4>{t('settings.modules.details.adds')}</h4>
      {#each elements as element (element)}
        <div class="adds">
          <span class="element">{t(`settings.modules.element.${element}`)}</span>
          <div class="what">
            {#if element === 'page'}
              <span class="text">{texts?.page || description}</span>
            {:else if element === 'tool'}
              <span class="text">{texts?.tool || description}</span>
            {:else if element === 'widget'}
              <span class="text">{description}</span>
            {:else if element === 'search'}
              {#if keywords.length > 0}
                <span class="line">
                  <span class="label">{t('settings.modules.details.keywords')}</span>
                  {#each keywords as keyword (keyword)}<Keycap>{keyword}</Keycap>{/each}
                </span>
              {/if}
              {#if examples.length > 0}
                <span class="line">
                  <span class="label">{t('settings.modules.details.examples')}</span>
                  {#each examples as example (example)}<Keycap>{example}</Keycap>{/each}
                </span>
              {/if}
            {:else}
              {#each rightClick as entry (entry.label)}
                <span class="line">
                  <span class="label">{entry.label}</span>
                  {#each entry.choices as choice (choice)}<Keycap>{choice}</Keycap>{/each}
                </span>
              {/each}
            {/if}
          </div>
        </div>
      {/each}
    </section>
  {/if}
</div>

{#if enlarged && preview}
  <ModuleImageDialog {name} image={preview} onclose={() => (enlarged = false)} />
{/if}

<style>
  .details {
    padding: 2px 0 18px 54px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .top {
    display: flex;
    align-items: flex-start;
    gap: 18px;
  }

  .image {
    width: 400px;
    height: 250px;
    flex: none;
    box-sizing: border-box;
    border-radius: 12px;
    border: 1px solid var(--border-strong);
  }

  .enlarge {
    padding: 0;
    overflow: hidden;
    cursor: zoom-in;
  }

  .enlarge img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .placeholder {
    background: var(--surface-sunken);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  dl {
    flex: 1;
    min-width: 0;
    margin: 0;
    display: grid;
    grid-template-columns: 96px 1fr;
    gap: 8px 12px;
    font-size: 13px;
  }

  dt {
    color: var(--text-subtle);
  }

  dd {
    min-width: 0;
    margin: 0;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }

  .tag {
    height: 22px;
    padding: 0 9px;
    border-radius: var(--radius-pill);
    background: var(--surface-raised);
    border: 1px solid var(--border);
    font-size: 12px;
    display: inline-flex;
    align-items: center;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  h4 {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  .adds {
    display: grid;
    grid-template-columns: 96px 1fr;
    gap: 12px;
    font-size: 13px;
  }

  .element {
    font-weight: 500;
  }

  .what {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .text {
    color: var(--text-muted);
  }

  .line {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 5px;
  }

  .label {
    margin-right: 3px;
    color: var(--text-subtle);
  }
</style>
