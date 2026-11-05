<script lang="ts">
  import type { Mount } from './foreign';

  let { mount, props }: { mount: Mount; props: Record<string, unknown> } = $props();
  let target: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (!target) return;
    try {
      return mount(target, { ...props });
    } catch (error) {
      console.error('A module failed to show', error);
    }
  });
</script>

<div class="foreign" bind:this={target}></div>

<style>
  .foreign {
    display: contents;
  }
</style>
