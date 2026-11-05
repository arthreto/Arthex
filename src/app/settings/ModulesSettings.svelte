<script lang="ts">
  import ModuleCatalog from './ModuleCatalog.svelte';
  import ModulesConfiguration from './ModulesConfiguration.svelte';
  import { modulesView } from './modulesView.svelte';
</script>

{#if modulesView.tab === 'configuration'}
  <ModulesConfiguration />
{:else}
  <ModuleCatalog />
{/if}
