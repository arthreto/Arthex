<script lang="ts">
  import logo from '../../assets/logo.png';
</script>

<span class="preview" aria-hidden="true">
  <span class="box">
    <span class="lens"></span>
    <span class="text"></span>
    <img src={logo} alt="" width="24" height="24" />
  </span>
  {#each [1, 0.7, 0.45] as opacity (opacity)}
    <span class="app" style:opacity></span>
  {/each}
</span>

<style>
  .preview {
    width: 560px;
    height: 72px;
    border-radius: 18px;
    background: var(--surface);
    border: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
  }

  .box {
    width: 280px;
    height: 40px;
    padding: 0 11px 0 14px;
    border-radius: 20px;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .lens {
    width: 13px;
    height: 13px;
    flex: none;
    border-radius: 50%;
    border: 2px solid var(--border-strong);
  }

  .text {
    width: 92px;
    height: 7px;
    margin-right: auto;
    border-radius: 4px;
    background: var(--border-strong);
  }

  img {
    display: block;
    flex: none;
  }

  .app {
    width: 26px;
    height: 26px;
    flex: none;
    border-radius: 8px;
    background: var(--surface-raised);
  }
</style>
