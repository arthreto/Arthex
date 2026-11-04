<script lang="ts">
  import { toast } from '../core/toast.svelte';
</script>

<div class="region" role="status" aria-live="polite">
  {#if toast.message}
    <div class="toast fade-in">{toast.message}</div>
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
  }
</style>
