<script lang="ts">
  import { errorMessage } from '../core/api';
  import { moduleText, t } from '../core/i18n';
  import { MAX_TAB_BAR, ZONES, getModule, moveInBar, placePage, zoneIds, type Zone } from '../core/modules';
  import { settings } from '../core/settings/settings.svelte';
  import type { SettingsPatch } from '../core/settings/types';
  import { toast } from '../core/toast.svelte';
  import Icon from '../ui/Icon.svelte';

  const zones = $derived(zoneIds(settings.value));

  let dragged: string | null = null;
  let hovered = $state<Zone | null>(null);

  async function save(patch: SettingsPatch) {
    try {
      await settings.update(patch);
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }

  function place(id: string, zone: Zone, beforeId?: string) {
    const name = moduleText(id, 'name');
    const placement = placePage(settings.value, id, zone, beforeId);
    if (!placement.ok) {
      toast.show(
        placement.reason === 'locked' ? t('hub.locked', { name }) : t('hub.barFull', { max: MAX_TAB_BAR }),
      );
      return;
    }
    void save(placement.patch);
    if (zone === 'off' && !zones.off.includes(id)) toast.show(t('hub.turnedOff', { name }));
  }

  function onkeydown(event: KeyboardEvent, id: string, zone: Zone) {
    if (getModule(id)?.locked) return;
    if (zone === 'bar' && (event.key === 'ArrowLeft' || event.key === 'ArrowRight')) {
      event.preventDefault();
      const tabBar = moveInBar(settings.value, id, event.key === 'ArrowLeft' ? -1 : 1);
      if (tabBar) void save({ tabBar });
    } else if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
      event.preventDefault();
      const next = ZONES[ZONES.indexOf(zone) + (event.key === 'ArrowUp' ? -1 : 1)];
      if (next) place(id, next);
    }
  }

  function dragStart(event: DragEvent, id: string) {
    dragged = id;
    event.dataTransfer?.setData('text/plain', id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move';
  }

  function dragEnd() {
    dragged = null;
    hovered = null;
  }

  function dropOnZone(event: DragEvent, zone: Zone) {
    event.preventDefault();
    const id = dragged;
    dragEnd();
    if (id) place(id, zone);
  }

  /** Dropping on a tab of the bar inserts before it. */
  function dropOnItem(event: DragEvent, zone: Zone, targetId: string) {
    if (zone !== 'bar') return;
    event.preventDefault();
    event.stopPropagation();
    const id = dragged;
    dragEnd();
    if (id && id !== targetId) place(id, 'bar', targetId);
  }
</script>

<div class="zones fade-in">
  {#each ZONES as zone (zone)}
    {@const label = t(`hub.zones.${zone}`)}
    <section aria-label={label}>
      <div class="header">
        <h3 class:bar={zone === 'bar'}>{label}</h3>
        {#if zone === 'bar'}
          <span class="hint">{t('hub.barHint', { count: zones.bar.length, max: MAX_TAB_BAR })}</span>
        {/if}
      </div>
      <div
        class="list {zone}"
        class:hovered={hovered === zone}
        role="group"
        aria-label={label}
        ondragover={(event) => event.preventDefault()}
        ondragenter={() => (hovered = zone)}
        ondrop={(event) => dropOnZone(event, zone)}
      >
        {#each zones[zone] as id (id)}
          {@const module = getModule(id)}
          {@const name = moduleText(id, 'name')}
          {#if module}
            <button
              class="item"
              class:locked={module.locked}
              draggable={!module.locked}
              aria-label={module.locked
                ? t('hub.lockedItem', { name, zone: label })
                : t('hub.item', { name, zone: label })}
              ondragstart={(event) => dragStart(event, id)}
              ondragend={dragEnd}
              ondragover={(event) => zone === 'bar' && event.preventDefault()}
              ondrop={(event) => dropOnItem(event, zone, id)}
              onkeydown={(event) => onkeydown(event, id, zone)}
            >
              <span class="icon" style:color="var(--tint-{module.tint})">
                <Icon path={module.icon} size={27} stroke={1.7} />
                {#if module.locked}
                  <span class="lock" title={t('hub.startPage')}
                    ><Icon name="lock" size={11} stroke={2.4} /></span
                  >
                {/if}
              </span>
              <span class="name">{name}</span>
            </button>
          {/if}
        {:else}
          <span class="empty">{t('hub.dropHere')}</span>
        {/each}
      </div>
    </section>
  {/each}
</div>

<style>
  .zones {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .header {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  h3 {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  h3.bar {
    color: var(--accent);
  }

  .hint {
    font-size: 12px;
    color: var(--text-subtle);
  }

  .list {
    min-height: 113px;
    padding: 10px;
    border-radius: 18px;
    border: 1.5px dashed var(--border-strong);
    background: var(--surface);
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    transition:
      border-color var(--motion-fast),
      background-color var(--motion-fast);
  }

  .list.bar {
    border-color: color-mix(in srgb, var(--accent) 45%, transparent);
    background: color-mix(in srgb, var(--accent) 6%, transparent);
  }

  .list.off {
    background: var(--surface-sunken);
  }

  .list.hovered {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }

  .item {
    width: 108px;
    padding: 6px 4px;
    border-radius: 16px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    cursor: grab;
    transition: background-color var(--motion-fast);
  }

  .item:hover {
    background: var(--surface-hover);
  }

  .item.locked {
    cursor: default;
  }

  .icon {
    position: relative;
    width: 56px;
    height: 56px;
    border-radius: 16px;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .off .icon,
  .off .name {
    opacity: 0.5;
    filter: grayscale(1);
  }

  .lock {
    position: absolute;
    right: -6px;
    bottom: -6px;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--surface-dialog);
    border: 2px solid var(--border-strong);
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .name {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    font-weight: 500;
  }

  .empty {
    flex: 1;
    text-align: center;
    font-size: 13px;
    color: var(--text-subtle);
  }
</style>
