<script lang="ts">
  import { onMount } from 'svelte';
  import { formatWhen } from '../../core/format';
  import Button from '../../ui/Button.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import { clipboard } from './clipboard.svelte';
  import { t } from './i18n';
  import { clipKind } from './kind';

  onMount(() => {
    void clipboard.load();
  });

  const clips = $derived(clipboard.clips.map((clip) => ({ clip, kind: clipKind(clip.text) })));
</script>

<div class="clipboard fade-in">
  <div class="bar">
    <span class="count">{t('count', { count: clips.length })}</span>
    <span class="spacer"></span>
    <Button aria-pressed={clipboard.paused} onclick={clipboard.togglePause}>
      {t(clipboard.paused ? 'resume' : 'pause')}
    </Button>
    <Button onclick={() => clipboard.clear()}>{t('clear')}</Button>
  </div>
  {#if clipboard.paused}
    <p class="banner">{t('paused')}</p>
  {/if}
  {#if clips.length > 0}
    <div class="grid scroll">
      {#each clips as { clip, kind } (clip.id)}
        <div class="clip">
          <button
            class="copy"
            aria-label={t('copy', { text: clip.text.slice(0, 60) })}
            onclick={() => clipboard.copy(clip)}
          >
            <span class="text" class:mono={kind === 'code' || kind === 'path'}>{clip.text}</span>
            <span class="meta">
              {t(`kinds.${kind}`)}{clip.pinned ? ` · ${t('pinned')}` : ''} · {formatWhen(clip.copiedAt)}
            </span>
          </button>
          <IconButton
            icon="pin"
            label={t(clip.pinned ? 'unpin' : 'pin')}
            aria-pressed={clip.pinned}
            style={clip.pinned ? 'color: var(--accent)' : undefined}
            onclick={() => clipboard.pin(clip)}
          />
          <IconButton icon="close" label={t('remove')} onclick={() => clipboard.remove(clip)} />
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty">{t('empty')}</p>
  {/if}
</div>

<style>
  .clipboard {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
  }

  .bar {
    height: 36px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .count {
    font-size: 14px;
    font-weight: 500;
  }

  .spacer {
    flex: 1;
  }

  .banner {
    margin-top: 10px;
    padding: 9px 14px;
    border-radius: 10px;
    font-size: 13px;
    color: var(--warning);
    background: color-mix(in srgb, var(--warning) 13%, transparent);
  }

  .grid {
    flex: 1;
    min-height: 0;
    margin-top: 12px;
    padding-right: 6px;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    align-content: start;
    gap: 10px;
  }

  .clip {
    padding: 4px 6px 4px 4px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--surface);
    border: 1px solid var(--border);
    transition: background-color var(--motion-fast);
  }

  .clip:hover {
    background: var(--surface-hover);
  }

  .copy {
    flex: 1;
    min-width: 0;
    padding: 10px;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    text-align: left;
  }

  .text {
    width: 100%;
    font-size: 13.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mono {
    font-family: var(--font-mono);
  }

  .meta {
    font-size: 12px;
    color: var(--text-subtle);
  }

  .empty {
    margin: auto auto 140px;
    font-size: 13.5px;
    color: var(--text-muted);
  }
</style>
