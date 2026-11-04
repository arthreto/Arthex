<script lang="ts">
  import { toast } from '../core/toast.svelte';

  function act() {
    const action = toast.action;
    toast.hide();
    action?.run();
  }
</script>

<div class="region" role="status" aria-live="polite">
  {#if toast.message}
    <div class="toast fade-in">
      {toast.message}
      {#if toast.action}
        <button onclick={act}>{toast.action.label}</button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .region {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 56px;
    z-index: 30;
    display: flex;
    justify-content: center;
    pointer-events: none;
  }

  .toast {
    max-width: 640px;
    padding: 11px 18px;
    border-radius: 12px;
    background: var(--surface-dialog);
    border: 1px solid var(--border-strong);
    box-shadow: var(--shadow-dialog);
    font-size: 13.5px;
    display: flex;
    align-items: center;
    gap: 16px;
    pointer-events: auto;
  }

  button {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--accent);
  }
</style>
