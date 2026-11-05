<script lang="ts">
  import InstalledModules from './InstalledModules.svelte';
  import ModuleCatalog from './ModuleCatalog.svelte';
  import { modulesView } from './modulesView.svelte';
</script>

{#if modulesView.tab === 'installed'}
  <InstalledModules />
{:else}
  <ModuleCatalog />
{/if}
