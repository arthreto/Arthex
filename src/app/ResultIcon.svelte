<script lang="ts">
  import type { ResultIcon } from '../core/search/types';
  import FileIcon from '../ui/FileIcon.svelte';
  import Icon from '../ui/Icon.svelte';

  let { icon, size }: { icon: ResultIcon; size: number } = $props();
</script>

<span class="icon" style:width="{size}px" style:height="{size}px">
  {#if icon.kind === 'image'}
    <img src={icon.src} alt="" width={size} height={size} />
  {:else if icon.kind === 'letter'}
    <span class="letter" style:font-size="{Math.round(size * 0.4)}px">{icon.text}</span>
  {:else if icon.kind === 'folder' || icon.kind === 'file'}
    <FileIcon folder={icon.kind === 'folder'} size={Math.round(size * 0.82)} />
  {:else if icon.kind === 'calc'}
    <span class="accent"><Icon name="calculator" size={Math.round(size * 0.8)} stroke={1.8} /></span>
  {:else}
    <span class:danger={icon.danger} class="accent"
      ><Icon name="power" size={Math.round(size * 0.72)} stroke={2} /></span
    >
  {/if}
</span>

<style>
  .icon {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }

  img {
    width: 100%;
    height: 100%;
    padding: 6%;
    object-fit: contain;
  }

  .letter {
    width: 100%;
    height: 100%;
    border-radius: 26%;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
  }

  .accent {
    display: flex;
    color: var(--accent);
  }

  .danger {
    color: var(--danger-text);
  }
</style>
