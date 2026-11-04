<script lang="ts">
  import { loadPage } from './pages';

  let { id }: { id: string } = $props();

  const page = $derived(loadPage(id));
</script>

{#if page}
  {#await page then { default: Page }}
    <div class="page fade-in"><Page /></div>
  {/await}
{/if}

<style>
  .page {
    position: absolute;
    inset: 0;
  }
</style>
