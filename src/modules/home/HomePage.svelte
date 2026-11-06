<script lang="ts">
  import logo from '../../assets/logo.png';
  import { homeWidgets, type ModuleManifest } from '../../core/modules';
  import { settings } from '../../core/settings/settings.svelte';
  import { loadWidget } from '../../app/pages';
  import { fitScale } from './fit';
  import { t } from './i18n';

  const top = $derived(homeWidgets(settings.value, 'top'));
  const main = $derived(homeWidgets(settings.value, 'main'));
  const side = $derived(homeWidgets(settings.value, 'side'));
  const empty = $derived(top.length + main.length + side.length === 0);

  let sideScale = $state(1);

  function fitSide(aside: HTMLElement) {
    const stack = aside.firstElementChild;
    if (!stack) return;
    const measure = () => {
      // Rounded: a zoomed height comes back a fraction off, which must not start another measure.
      const content = Math.round(stack.getBoundingClientRect().height / sideScale);
      sideScale = fitScale(aside.clientHeight, content);
    };
    const observer = new ResizeObserver(measure);
    observer.observe(aside);
    observer.observe(stack);
    return { destroy: () => observer.disconnect() };
  }
</script>

{#snippet widgets(list: ModuleManifest[])}
  {#each list as module (module.id)}
    {#await loadWidget(module.id) then loaded}
      {#if loaded}
        {@const Widget = loaded.default}
        <Widget />
      {/if}
    {/await}
  {/each}
{/snippet}

{#if empty}
  <div class="welcome">
    <img src={logo} alt="" width="88" height="88" />
    <h2>{t('welcome')}</h2>
  </div>
{:else}
  <div class="home">
    {#if top.length > 0}
      <div class="top">{@render widgets(top)}</div>
    {/if}
    <div class="columns">
      <div class="main">{@render widgets(main)}</div>
      {#if side.length > 0}
        <aside class="scroll" use:fitSide>
          <div class="stack" style:zoom={sideScale}>{@render widgets(side)}</div>
        </aside>
      {/if}
    </div>
  </div>
{/if}

<style>
  .home {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 22px;
    overflow: hidden;
  }

  .top {
    flex: none;
  }

  .columns {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 24px;
  }

  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  aside {
    width: 420px;
    flex: none;
    min-height: 0;
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .welcome {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 24px;
  }

  h2 {
    font-size: 26px;
    font-weight: 600;
  }
</style>
