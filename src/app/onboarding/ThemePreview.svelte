<script lang="ts">
  import type { Theme } from '../../core/settings/types';

  let { theme }: { theme: Theme } = $props();
</script>

<!-- A drawing of each theme, so its colours do not follow the active theme. -->
<span class="preview {theme}" aria-hidden="true">
  <span class="bar"></span>
  <span class="tiles">
    {#each [1, 0.75, 0.55, 0.35] as opacity (opacity)}
      <span style:opacity></span>
    {/each}
  </span>
  <span class="line" style:width="62%" style:top="76px"></span>
  <span class="line" style:width="42%" style:top="92px"></span>
</span>

<style>
  .preview {
    position: relative;
    display: block;
    width: 100%;
    height: 116px;
    border-radius: 12px;
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .dark {
    --bg: #1c1f26;
    --ink: rgba(255, 255, 255, 0.14);
    --tile: #4cc2ff;
  }

  .light {
    --bg: #eef1f5;
    --ink: rgba(17, 24, 39, 0.11);
    --tile: #0067c0;
  }

  .system {
    --bg: linear-gradient(90deg, #1c1f26 50%, #eef1f5 50%);
    --ink: rgba(128, 128, 128, 0.38);
    --tile: #3d8fd6;
  }

  .preview {
    background: var(--bg);
  }

  .bar {
    position: absolute;
    left: 16px;
    right: 16px;
    top: 14px;
    height: 14px;
    border-radius: 7px;
    background: var(--ink);
  }

  .tiles {
    position: absolute;
    left: 16px;
    top: 40px;
    display: flex;
    gap: 7px;
  }

  .tiles span {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: var(--tile);
  }

  .line {
    position: absolute;
    left: 16px;
    height: 7px;
    border-radius: 4px;
    background: var(--ink);
  }
</style>
