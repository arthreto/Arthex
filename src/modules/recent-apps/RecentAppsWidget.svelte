<script lang="ts">
  import { onMount } from 'svelte';
  import { errorMessage, events, on } from '../../core/api';
  import { formatWhen } from '../../core/format';
  import { t as shellT } from '../../core/i18n';
  import { toast } from '../../core/toast.svelte';
  import { recentAppsApi, type RecentApp } from './api';
  import { t } from './i18n';

  let apps = $state<RecentApp[]>([]);
  let now = $state(Date.now());

  async function refresh() {
    now = Date.now();
    try {
      apps = await recentAppsApi.list();
    } catch (error) {
      toast.show(shellT('errors.generic', { message: errorMessage(error) }));
    }
  }

  onMount(() => {
    void refresh();
    const listening = on(events.overlayShown, refresh);
    return () => void listening.then((unlisten) => unlisten());
  });

  async function open(app: RecentApp) {
    try {
      await recentAppsApi.open(app.path);
    } catch (error) {
      toast.show(shellT('errors.generic', { message: errorMessage(error) }));
    }
  }
</script>

{#if apps.length > 0}
  <h2 class="section-title">{t('title')}</h2>
  <div class="list scroll">
    {#each apps as app (app.path)}
      <button class="row" title={app.path} onclick={() => open(app)}>
        {#if app.icon}
          <img src={app.icon} alt="" width="30" height="30" />
        {:else}
          <span class="letter">{app.name.charAt(0).toUpperCase()}</span>
        {/if}
        <span class="name">{app.name}</span>
        {#if app.running}
          <span class="running">{t('running')}</span>
        {/if}
        <span class="when">{formatWhen(app.lastUsed, now)}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  h2 {
    margin-bottom: 2px;
  }

  .list {
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .row {
    width: 100%;
    height: 44px;
    flex: none;
    padding: 0 12px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    gap: 14px;
    text-align: left;
    transition: background-color var(--motion-fast);
  }

  .row:hover {
    background: var(--surface-hover);
  }

  img,
  .letter {
    width: 30px;
    height: 30px;
    flex: none;
  }

  img {
    object-fit: contain;
  }

  .letter {
    border-radius: 8px;
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

  .running {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--success);
  }

  .running::before {
    content: '';
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }

  .when {
    width: 100px;
    text-align: right;
    white-space: nowrap;
    font-size: 12px;
    color: var(--text-subtle);
  }
</style>
