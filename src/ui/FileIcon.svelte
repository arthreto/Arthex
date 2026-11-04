<script lang="ts">
  import Icon from './Icon.svelte';

  /** The sober file and folder pictograms of search results and FileSize. */
  let { folder, size }: { folder: boolean; size: number } = $props();
</script>

{#if folder}
  <svg aria-hidden="true" width={size} height={size} viewBox="0 0 24 24">
    <path
      d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
      fill="var(--folder)"
    />
  </svg>
{:else}
  <Icon name="file" {size} stroke={1.7} />
{/if}
